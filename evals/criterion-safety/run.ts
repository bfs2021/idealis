import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import {
  appendFile,
  copyFile,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { cpus, platform, tmpdir, totalmem } from "node:os";
import { basename, dirname, join } from "node:path";
import { fingerprint } from "../rust-safety/provenance.js";
import { digest } from "../usable/provenance.js";

const plan = process.env.IDEALIS_CRITERION_PLAN ?? "0023-criterion-safety";
const target = process.env.IDEALIS_CRITERION_TARGET ?? "rtlc";
assert.ok(target === "rtlc" || target === "rtla-compiler");
const compiler = target === "rtla-compiler";
const revision = process.env.IDEALIS_CRITERION_REVISION;
assert.ok(
  !revision || !compiler,
  "Archived revision control is for the existing rtlc benchmark",
);
assert.match(plan, /^\d{4}-[a-z0-9-]+$/);
const stage = process.argv[2];
const directory = process.argv[3];
assert.ok(stage && /^[a-z][a-z0-9-]*$/.test(stage) && directory);
await mkdir(dirname(directory), { recursive: true });
await mkdir(directory);
const baselineRevision = revision
  ? execFileSync("git", ["rev-parse", "--verify", `${revision}^{commit}`])
      .toString()
      .trim()
  : null;
const archivedRoot = baselineRevision
  ? await mkdtemp(join(tmpdir(), "idealis-criterion-control-"))
  : null;
if (baselineRevision && archivedRoot) {
  const archive = execFileSync("git", [
    "archive",
    baselineRevision,
    "--",
    "crates/rtlc",
  ]);
  await writeFile(join(directory, "source.tar"), archive, { flag: "wx" });
  execFileSync("tar", [
    "-xf",
    join(directory, "source.tar"),
    "-C",
    archivedRoot,
  ]);
}
const crateRoot = archivedRoot
  ? join(archivedRoot, "crates/rtlc")
  : `crates/${target}`;
const paths = [
  `evals/plans/${plan}.md`,
  "evals/plans/0023-adaptive-policy.md",
  ...(await readdir("evals/criterion-safety"))
    .filter((path) => path.endsWith(".ts"))
    .map((path) => `evals/criterion-safety/${path}`),
];
const source = await fingerprint();
const sources = [
  ...source.sources,
  ...(await Promise.all(
    paths.map(async (path) => ({ path, sha256: digest(await readFile(path)) })),
  )),
].sort((left, right) => left.path.localeCompare(right.path));
const sourceVersion = digest(JSON.stringify(sources));
await writeFile(
  join(directory, "manifest.json"),
  JSON.stringify(
    {
      plan,
      stage,
      sourceVersion,
      sources,
      commit: execFileSync("git", ["rev-parse", "HEAD"]).toString().trim(),
      dirtyPaths: execFileSync("git", ["status", "--short"]).toString().trim(),
      startedAt: new Date().toISOString(),
      node: process.version,
      rust: execFileSync("rustc", ["--version"]).toString().trim(),
      arch: process.arch,
      cpu: cpus()[0]?.model,
      memory: totalmem(),
      platform: platform(),
      target,
      baselineRevision,
      benchmarkImplementation: baselineRevision
        ? "source.tar from the pinned Git revision"
        : "working-tree crate",
      timing: compiler
        ? "Criterion: AST-to-IR compilation and prepared IR checking separately; excludes parsing, IPC and process startup"
        : "Criterion: native kernel/API, excludes frontend, IPC, process startup and iter_batched setup/drop",
      sampleSize: 20,
      warmupSeconds: 0.5,
      measurementSeconds: 1,
    },
    null,
    2,
  ),
);
await copyFile(
  compiler
    ? "crates/rtla-compiler/tests/finite-support.json"
    : ".artifacts/criterion-safety/inputs.json",
  join(directory, "inputs.json"),
);
const args = [
  "bench",
  "--locked",
  "--manifest-path",
  join(crateRoot, "Cargo.toml"),
  "--bench",
  compiler ? "compatibility" : "safety",
  "--",
  "--noplot",
  "--save-baseline",
  stage,
];
const log = join(directory, "criterion.log");
const child = spawn("cargo", args, { stdio: ["ignore", "pipe", "pipe"] });
let output = "";
for (const stream of [child.stdout, child.stderr])
  stream.on("data", (chunk) => {
    output += String(chunk);
    process.stdout.write(chunk);
  });
const exit = await new Promise<number | null>((resolve, reject) => {
  child.once("error", reject);
  child.once("close", resolve);
});
await writeFile(log, output, { flag: "wx" });
const files: { path: string; sha256: string }[] = [];
const estimates: {
  benchmark: string;
  mean: number;
  lower: number;
  upper: number;
}[] = [];
const collect = async (path: string, relative: string): Promise<void> => {
  for (const entry of await readdir(path, { withFileTypes: true })) {
    const next = join(relative, entry.name);
    if (entry.isDirectory()) await collect(join(path, entry.name), next);
    else if (
      entry.name.endsWith(".json") &&
      dirname(next).split("/").at(-1) === stage
    ) {
      const target = join(directory, "criterion", next);
      await mkdir(dirname(target), { recursive: true });
      const value = await readFile(join(path, entry.name));
      await writeFile(target, value, { flag: "wx" });
      files.push({ path: `criterion/${next}`, sha256: digest(value) });
      if (entry.name === "estimates.json") {
        const data = JSON.parse(value.toString());
        estimates.push({
          benchmark: dirname(dirname(next)),
          mean: data.mean.point_estimate,
          lower: data.mean.confidence_interval.lower_bound,
          upper: data.mean.confidence_interval.upper_bound,
        });
      }
    }
  }
};
let archiveError: string | null = null;
try {
  await collect(join(crateRoot, "target/criterion"), "");
} catch (error) {
  archiveError = String(error);
}
const sourceUnchanged = (
  await Promise.all(
    sources.map(
      async (entry) => digest(await readFile(entry.path)) === entry.sha256,
    ),
  )
).every(Boolean);
for (const path of [
  "criterion.log",
  "manifest.json",
  "inputs.json",
  ...(archivedRoot ? ["source.tar"] : []),
])
  files.push({ path, sha256: digest(await readFile(join(directory, path))) });
const status =
  exit === 0 &&
  sourceUnchanged &&
  archiveError === null &&
  estimates.length === (compiler ? 12 : 14) &&
  estimates.every(
    (estimate) => Number.isFinite(estimate.mean) && estimate.mean > 0,
  )
    ? "complete"
    : "error";
await writeFile(
  join(directory, "result.json"),
  JSON.stringify(
    {
      status,
      stage,
      sourceVersion,
      exit,
      sourceUnchanged,
      archiveError,
      estimates,
      files,
    },
    null,
    2,
  ),
);
await appendFile(
  "evals/runs/ledger.jsonl",
  `${JSON.stringify({ runId: basename(directory), kind: "criterion-safety", stage, status, sourceVersion })}\n`,
);
assert.equal(status, "complete");
if (archivedRoot) await rm(archivedRoot, { recursive: true, force: true });

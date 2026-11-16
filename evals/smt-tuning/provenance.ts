import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { digest } from "../smt-subset/provenance.js";

export { digest } from "../smt-subset/provenance.js";
export const fingerprint = async () => {
  const paths = execFileSync("git", [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-standard",
    "--",
    "crates/idealis-smt",
    "evals/smt-subset",
    "evals/smt-expansion",
    "evals/smt-scripts",
    "evals/smt-tuning",
    "evals/plans/0040-smt-expansion.md",
    "evals/plans/0040a-smt-expansion-adjustment.md",
    "evals/plans/0041-smt-scripts.md",
    "evals/plans/0042-smt-tuning.md",
    "evals/plans/0042a-smt-measurement-recheck.md",
    "evals/profile-tuning/hotspots.ts",
    "src/trace.ts",
    "src/conformance.ts",
    "src/contracts.ts",
    "evals/harness/ttl.ts",
    "examples/counter-model.ts",
    "examples/durable-counter.ts",
    "scripts/package.ts",
    "scripts/package-test.ts",
    "smt-playwright.config.ts",
    "smt-vitest.config.ts",
    "justfile",
    "package.json",
    "pnpm-lock.yaml",
    "tsconfig.json",
    "tsconfig.build.json",
    "biome.json",
    ".github/workflows/ci.yml",
  ])
    .toString()
    .split("\0")
    .filter((path) => path && !path.endsWith("README.md"));
  const sources = await Promise.all(
    [...new Set(paths)]
      .sort()
      .map(async (path) => ({ path, sha256: digest(await readFile(path)) })),
  );
  return { sources, sourceVersion: digest(JSON.stringify(sources)) };
};

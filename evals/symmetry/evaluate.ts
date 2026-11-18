import assert from "node:assert/strict";
import { appendFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";
import { createPortableWasm } from "../../src/portable-wasm.js";
import { checkSafety, decodeSafetyResult } from "../../src/safety.js";
import { compileTlaSource } from "../../src/tla-source.js";
import { isRecord } from "../../src/trace.js";
import { differential } from "../tlc-differential/differential.js";
import { startNative } from "../usable/native.js";
import { digest } from "../usable/provenance.js";
import { bounds, corpus, plain } from "./cases.js";
import { symmetryFingerprint } from "./provenance.js";

const [directory, java, jar] = process.argv.slice(2);
assert.ok(directory && java && jar);
await mkdir(directory);
const source = await symmetryFingerprint();
const bytes = await readFile(
  "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
);
const core = await createPortableWasm(bytes);
const native = startNative();
const inputs = corpus();
const files: string[] = [];
const save = async (path: string, value: unknown) => {
  await writeFile(
    join(directory, path),
    `${JSON.stringify(value, null, 2)}\n`,
    { flag: "wx" },
  );
  files.push(path);
};
await save("manifest.json", {
  plan: "0060-restricted-symmetry",
  ...source,
  nativeSha256: digest(
    await readFile("crates/idealis-core/target/release/idealis-core"),
  ),
  wasmSha256: digest(bytes),
  jarSha256: digest(await readFile(jar)),
  inputs,
  performanceMeasurement: false,
});
const reports = [];
try {
  for (const input of inputs) {
    const path = join(directory, input.name);
    await mkdir(path);
    const report = await differential(
      { ...input, family: "graph" },
      core,
      native,
      java,
      jar,
      path,
      (input) => compileTlaSource(core, input),
    );
    assert.deepEqual(report.differences, [], input.name);
    const original = checkSafety(
      core,
      compileTlaSource(core, plain(input)),
      bounds,
    );
    assert.equal(report.ours.result?.status, original.status, input.name);
    assert.equal(
      report.ours.result?.violation?.trace.length,
      original.violation?.trace.length,
      input.name,
    );
    const response = await native.request({
      version: 1,
      request: {
        kind: "check-safety-parallel",
        program: compileTlaSource(core, input),
        bounds,
        workers: 8,
      },
    });
    assert.ok(isRecord(response) && response.kind === "safety");
    assert.deepEqual(decodeSafetyResult(response.result), report.ours.result);
    reports.push({ report, original, workers8: response });
    console.log(
      input.name,
      report.ours.result?.status,
      report.reference.states,
    );
  }
  assert.equal(
    (await symmetryFingerprint()).sourceVersion,
    source.sourceVersion,
  );
  await save("reports.json", reports);
  await save("result.json", {
    status: "complete",
    cases: inputs.length,
    mismatches: 0,
    sourceVersion: source.sourceVersion,
    files: await Promise.all(
      files.map(async (path) => ({
        path,
        sha256: digest(await readFile(join(directory, path))),
      })),
    ),
  });
  await appendFile(
    "evals/runs/ledger.jsonl",
    `${JSON.stringify({ runId: basename(directory), kind: "restricted-symmetry", status: "complete", sourceVersion: source.sourceVersion })}\n`,
  );
} catch (error) {
  await save("error.json", { status: "error", error: String(error), reports });
  throw error;
} finally {
  await native.close();
}

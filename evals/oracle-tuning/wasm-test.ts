import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { readOracleWasm } from "./wasm.js";

test("a frozen oracle worker runs its own Wasm binding outside the checkout", async () => {
  const root = await mkdtemp(join(tmpdir(), "roracle-worker-"));
  const frozen = join(root, "snapshot");
  try {
    execFileSync(process.execPath, [
      "dist/evals/oracle-tuning/run.js",
      "freeze",
      frozen,
    ]);
    const output = execFileSync(
      process.execPath,
      [join(frozen, "worker.mjs"), "oracle"],
      {
        cwd: root,
        env: { ...process.env, IDEALIS_ORACLE_BACKEND: "native-json" },
        input: '{"actual":1,"expected":1}\n{"actual":2,"expected":1}\n',
        encoding: "utf8",
      },
    );
    assert.deepEqual(
      output
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line)),
      [{ status: "matched" }, { status: "mismatch" }],
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

test("frozen oracle snapshots accept current and historical Wasm filenames", async () => {
  const root = await mkdtemp(join(tmpdir(), "roracle-frozen-"));
  const directory = join(
    root,
    "crates/oracle/target/wasm32-unknown-unknown/release",
  );
  try {
    await mkdir(directory, { recursive: true });
    await assert.rejects(readOracleWasm(root), { code: "ENOENT" });
    await writeFile(join(directory, "idealis_oracle.wasm"), "historical");
    assert.equal((await readOracleWasm(root)).toString(), "historical");
    await writeFile(join(directory, "roracle.wasm"), "current");
    assert.equal((await readOracleWasm(root)).toString(), "current");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { readOracleWasm } from "./wasm.js";

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

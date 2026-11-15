import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { wasmPath } from "./clients.js";
import { generated, handCases } from "./corpus.js";
import { verify } from "./verify.js";
import { createWasm } from "./wasm.js";

test("native/Wasm subset, Z3 and known fault reproduction agree", async () => {
  await verify(
    [...handCases(), ...generated(0x30a1, 8)],
    await createWasm(await readFile(wasmPath)),
    () => {},
  );
});

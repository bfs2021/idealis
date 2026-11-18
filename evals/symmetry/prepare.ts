import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createPortableWasm } from "../../src/portable-wasm.js";
import { checkSafety } from "../../src/safety.js";
import { compileTlaSource } from "../../src/tla-source.js";
import { bounds, counter, joint, plain } from "./cases.js";

const core = await createPortableWasm(
  await readFile(
    "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
  ),
);
const inputs = [];
for (const [input, counts] of [
  [counter(5, 3), [1024, 56, 5]],
  [counter(7, 5), [279936, 792, 7]],
  [joint(5), [1024, 56, 10]],
] as const) {
  for (const [mode, selected, states] of [
    ["plain", plain(input), counts[0]],
    ["symmetry", input, counts[1]],
  ] as const) {
    const program = compileTlaSource(core, selected);
    const generated = states * counts[2] + 1;
    const result = checkSafety(core, program, bounds);
    assert.equal(result.status, "complete");
    assert.equal(result.states, states);
    assert.equal(result.generated, generated);
    inputs.push({ name: `${input.name}/${mode}`, states, generated, program });
  }
}
await mkdir(".artifacts/symmetry", { recursive: true });
await writeFile(
  ".artifacts/symmetry/inputs.json",
  `${JSON.stringify(inputs, null, 2)}\n`,
);
const fixture = "crates/rtlc/src/symmetry-fixtures.json";
if (process.argv.includes("--write-fixture"))
  await writeFile(fixture, `${JSON.stringify(inputs, null, 2)}\n`);
else assert.deepEqual(JSON.parse(await readFile(fixture, "utf8")), inputs);
console.log(`Verified ${inputs.length} prepared benchmark inputs`);

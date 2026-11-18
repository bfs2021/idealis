import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { PortableCoreError } from "../../src/portable-value.js";
import { createPortableWasm } from "../../src/portable-wasm.js";
import { checkSafety, decodeSafetyResult } from "../../src/safety.js";
import { compileTlaSource } from "../../src/tla-source.js";
import { isRecord } from "../../src/trace.js";
import { startNative } from "../usable/native.js";
import { bounds, corpus, counter, plain } from "./cases.js";

const core = await createPortableWasm(
  await readFile(
    "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
  ),
);

test("restricted symmetry preserves native/Wasm verdicts, concrete traces and serial worker fallback", async () => {
  const native = startNative();
  try {
    for (const input of corpus()) {
      const program = compileTlaSource(core, input);
      const actual = checkSafety(core, program, bounds);
      const original = checkSafety(
        core,
        compileTlaSource(core, plain(input)),
        bounds,
      );
      assert.equal(actual.status, original.status, input.name);
      assert.equal(
        actual.violation?.trace.length,
        original.violation?.trace.length,
        input.name,
      );
      assert.ok(actual.states <= original.states);
      for (const workers of [1, 8]) {
        const response = await native.request({
          version: 1,
          request: { kind: "check-safety-parallel", program, bounds, workers },
        });
        assert.ok(
          isRecord(response) && response.kind === "safety",
          JSON.stringify(response),
        );
        assert.deepEqual(
          decodeSafetyResult(response.result),
          actual,
          input.name,
        );
      }
      if (actual.status === "complete") assert.ok(actual.states > 0);
    }
  } finally {
    await native.close();
  }
});

test("symmetry rejects unsupported groups, runtime shapes, order-sensitive expressions and combinations", () => {
  const input = counter(3, 2);
  const program = compileTlaSource(core, input);
  for (const modelValues of [
    [],
    ["p0"],
    ["p0", "p0"],
    ["n_a", "n_b"],
    ["bad name", "p0"],
    Array.from({ length: 65 }, (_, index) => `p${index}`),
  ]) {
    assert.throws(
      () =>
        checkSafety(core, { ...program, symmetry: { modelValues } }, bounds),
      PortableCoreError,
    );
  }
  for (const altered of [
    { ...input, source: input.source.replace("|-> 0]", "|-> [value |-> 0]]") },
    { ...input, source: input.source.replace("|-> 0]", "|-> node]") },
    { ...input, source: input.source.replace("|-> 0]", '|-> "p0"]') },
    {
      ...input,
      source: input.source.replace(
        "|-> 0]",
        "|-> CHOOSE candidate \\in Nodes: TRUE]",
      ),
    },
  ]) {
    assert.throws(
      () => checkSafety(core, compileTlaSource(core, altered), bounds),
      PortableCoreError,
    );
  }
  assert.throws(
    () =>
      core.request({
        version: 1,
        request: { kind: "check-safety-coverage", program, bounds },
      }),
    PortableCoreError,
  );
  assert.throws(
    () =>
      checkSafety(
        core,
        {
          ...program,
          actionConstraints: [
            {
              name: "C",
              expression: {
                op: "literal",
                value: { type: "bool", value: true },
              },
            },
          ],
        },
        bounds,
      ),
    PortableCoreError,
  );
  assert.throws(
    () =>
      checkSafety(
        core,
        {
          ...program,
          fairness: [
            {
              kind: "atom",
              expression: {
                op: "literal",
                value: { type: "bool", value: true },
              },
            },
          ],
        },
        bounds,
      ),
    PortableCoreError,
  );
});

test("the 64-node boundary uses row reduction without permutation enumeration", () => {
  const result = checkSafety(
    core,
    compileTlaSource(core, counter(64, 1)),
    bounds,
  );
  assert.equal(result.status, "complete");
  assert.equal(result.states, 65);
  assert.equal(result.generated, 4161);
  assert.equal(result.depth, 64);
});

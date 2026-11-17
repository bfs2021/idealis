import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { isDeepStrictEqual } from "node:util";
import { assertOracle, compareOracle } from "./node.js";
import { snapshot } from "./snapshot.js";
import { createOracle } from "./wasm.js";

test("oracle equality retains the previous strict equality on portable snapshots", () => {
  const values = [
    undefined,
    null,
    true,
    false,
    0,
    -0,
    1,
    NaN,
    Infinity,
    -Infinity,
    1n,
    "1",
    [],
    [undefined],
    { optional: undefined },
    {},
    { b: 2, a: [1, "x"] },
    { a: [1, "x"], b: 2 },
  ];
  for (const actual of values)
    for (const expected of values) {
      assert.equal(
        compareOracle(actual, expected).status === "matched",
        isDeepStrictEqual(actual, expected),
      );
    }
  assert.doesNotThrow(() => assertOracle({ a: 1, b: 2 }, { b: 2, a: 1 }));
  assert.throws(
    () => assertOracle({ value: 2 }, { value: 1 }, "count drift"),
    (error) => {
      assert.ok(error instanceof assert.AssertionError);
      assert.match(error.message, /count drift/);
      assert.ok("report" in error);
      return true;
    },
  );
});

test("nonportable values and cycles fail explicitly instead of losing data", () => {
  const cycle: unknown[] = [];
  cycle.push(cycle);
  for (const value of [
    cycle,
    new Date(),
    new Map(),
    () => 1,
    Symbol("x"),
    new Array(2),
  ])
    assert.throws(() => snapshot(value));
  assert.throws(
    () =>
      snapshot(
        Object.defineProperty({}, "value", {
          get: () => {
            throw new Error("getter must not run");
          },
          enumerable: true,
        }),
      ),
    /accessor/,
  );
});

test("batch wire preserves unknown and execution failure without matching them", async () => {
  const core = createOracle(
    await readFile(
      "crates/oracle/target/wasm32-unknown-unknown/release/idealis_oracle.wasm",
    ),
  );
  for (const kind of ["unknown", "invalid", "error"]) {
    const observation =
      kind === "error" ? { kind, message: "fault" } : { kind, reason: "fault" };
    const report = core.request({
      version: 1,
      group: "negative",
      reference: "reference",
      implementations: ["candidate"],
      cases: [
        {
          id: "one",
          input: null,
          reference: observation,
          actual: [observation],
        },
      ],
    });
    assert.equal(report.status, kind);
  }
  assert.throws(() =>
    core.request({
      version: 1,
      group: "empty",
      reference: "reference",
      implementations: [],
      cases: [],
    }),
  );
});

test("declared workspace migration sites all use the shared oracle", async () => {
  const inventory: readonly { path: string; equalitySites: number }[] =
    JSON.parse(
      await readFile("evals/workspace-oracle/migrations.json", "utf8"),
    );
  for (const item of inventory) {
    const source = await readFile(item.path, "utf8");
    assert.equal(
      [...source.matchAll(/\b(?:assertOracle|compareOracle)\(/g)].length,
      item.equalitySites,
      item.path,
    );
  }
});

test("mutating an input is observed on the next call and does not alter a previous report", () => {
  const actual = { nested: [1, 2] };
  const expected = { nested: [1, 2] };
  const previous = compareOracle(actual, expected);
  const saved = JSON.stringify(previous);
  actual.nested.push(3);
  assert.equal(previous.status, "matched");
  assert.equal(compareOracle(actual, expected).status, "mismatch");
  assert.equal(JSON.stringify(previous), saved);
  expected.nested.push(3);
  assert.equal(compareOracle(actual, expected).status, "matched");
});

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { decodeOracleReply } from "../workspace-oracle/protocol.js";
import { snapshot } from "../workspace-oracle/snapshot.js";
import { createOracle } from "../workspace-oracle/wasm.js";
import { createNativeOracle, loadNativeBinding } from "./client.js";
import { fixtures, observationMatrix, requestFor } from "./fixtures.js";

const bytes = await readFile(
  "crates/oracle/target/wasm32-unknown-unknown/release/roracle.wasm",
);
const wasm = createOracle(bytes);
const native = [
  createNativeOracle("native-json"),
  createNativeOracle("native-object"),
];

export const registerNativeOracleContract = (
  test: (name: string, run: () => void | Promise<void>) => unknown,
): void => {
  test("native and Wasm preserve all 64 observation combinations and complete reports", () => {
    for (const reference of observationMatrix)
      for (const actual of observationMatrix) {
        const request = requestFor(reference, actual);
        const expected = wasm.request(request);
        for (const client of native)
          assert.deepEqual(client.request(request), expected);
      }
  });
  test("transport fixtures retain positive, mismatch and unknown controls", () => {
    for (const fixture of fixtures) {
      const expected = wasm.request(fixture.input);
      assert.equal(expected.status, fixture.expected);
      for (const client of native)
        assert.deepEqual(client.request(fixture.input), expected);
    }
  });
  test("special keys and explicit JS snapshots survive both transports", () => {
    const value = snapshot(
      JSON.parse('{"__proto__":{"kept":7},"constructor":null,"雪":"😀"}'),
    );
    const input = requestFor(
      { kind: "value", value },
      { kind: "value", value },
    );
    const expected = wasm.request(input);
    for (const client of native)
      assert.deepEqual(client.request(input), expected);
  });
  test("finite numeric boundaries and shared data retain complete reports", () => {
    for (const value of [
      0,
      0.1,
      -1.5,
      2 ** 32,
      -(2 ** 32),
      Number.MAX_SAFE_INTEGER,
      Number.MAX_VALUE,
      Number.MIN_VALUE,
    ]) {
      const input = requestFor(
        { kind: "value", value },
        { kind: "value", value },
      );
      const expected = wasm.request(input);
      assert.equal(expected.status, "matched");
      for (const client of native)
        assert.deepEqual(client.request(input), expected);
    }
    const shared = { nested: [1, null, "雪"] };
    const value = { first: shared, second: shared };
    const input = requestFor(
      { kind: "value", value },
      { kind: "value", value },
    );
    for (const client of native)
      assert.deepEqual(client.request(input), wasm.request(input));
  });
  test("native raw JSON keeps invalid input, duplicate fields and dimension failures rejected", () => {
    const binding = loadNativeBinding();
    const valid = requestFor(
      { kind: "value", value: 7 },
      { kind: "value", value: 7 },
    );
    for (const json of [
      "invalid",
      "{}",
      JSON.stringify({ ...valid, version: 2 }),
      JSON.stringify({ ...valid, cases: [] }),
      JSON.stringify(valid).replace('"value":7', '"value":7,"value":8'),
    ]) {
      assert.throws(() => decodeOracleReply(binding.dispatchJson(json)));
      assert.throws(() => wasm.requestJson(json));
    }
  });
  test("native object rejects unsafe coercions and accessors before conversion", () => {
    const client = native[1];
    assert.ok(client);
    for (const value of [-0, NaN, Infinity, "\ud800"]) {
      const request = requestFor(
        { kind: "value", value },
        { kind: "value", value },
      );
      assert.throws(() => client.request(request));
    }
    const request = requestFor(
      { kind: "value", value: 7 },
      { kind: "value", value: 7 },
    );
    let called = false;
    Object.defineProperty(request, "group", {
      get: () => {
        called = true;
        return "getter";
      },
      enumerable: true,
    });
    assert.throws(() => client.request(request), /accessors/);
    assert.equal(called, false);
  });
  test("native object rejects cyclic, sparse and inherited data", () => {
    const client = native[1];
    assert.ok(client);
    const cycle: { nested?: unknown } = {};
    cycle.nested = cycle;
    for (const invalid of [
      cycle,
      new Array(2),
      new Date(),
      Object.create({ inherited: 7 }),
    ]) {
      const request = requestFor(
        { kind: "value", value: null },
        { kind: "value", value: null },
      );
      Object.defineProperty(request.cases[0]?.reference, "value", {
        value: invalid,
      });
      assert.throws(() => client.request(request));
    }
    execFileSync(
      process.execPath,
      [
        "--input-type=module",
        "-e",
        `
      import assert from 'node:assert/strict';
      import { createNativeOracle } from './dist/evals/roracle-native/client.js';
      import { fixtures } from './dist/evals/roracle-native/fixtures.js';
      const client = createNativeOracle('native-object');
      Object.defineProperty(Object.prototype, 'inherited', { enumerable: true, get() { throw new Error('getter ran'); } });
      assert.throws(() => client.request(fixtures[0].input), /inherited enumerable/);
    `,
      ],
      { stdio: "pipe" },
    );
  });
  test("missing native binaries never silently fall back to Wasm", () => {
    assert.throws(() =>
      createNativeOracle(
        "native-json",
        ".artifacts/definitely-missing-roracle.node",
      ),
    );
  });
  test("corrupted full reports fail validation instead of passing on status", () => {
    const reply = JSON.parse(
      loadNativeBinding().dispatchJson(JSON.stringify(fixtures[0]?.input)),
    );
    reply.report.cases[0].comparisons[0].actual = { kind: "value" };
    assert.throws(() => decodeOracleReply(JSON.stringify(reply)));
    delete reply.report.reference;
    assert.throws(() => decodeOracleReply(JSON.stringify(reply)));
    for (const cases of [
      [],
      [
        {
          case: { id: "one", input: null },
          reference: { kind: "value", value: 7 },
          comparisons: [],
        },
      ],
    ]) {
      const broken = JSON.parse(
        loadNativeBinding().dispatchJson(JSON.stringify(fixtures[0]?.input)),
      );
      broken.report.cases = cases;
      assert.throws(() => decodeOracleReply(JSON.stringify(broken)));
    }
  });
  test("Rust-generated contract declarations are current", () => {
    execFileSync(
      process.execPath,
      ["--experimental-strip-types", "scripts/oracle-types.ts", "--check"],
      { stdio: "pipe" },
    );
  });
};

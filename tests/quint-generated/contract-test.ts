import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { after, test } from "node:test";
import {
  bounds,
  config,
  names,
} from "../../evals/experiments/quint-generated/cases.js";
import { startNative } from "../../evals/experiments/usable/native.js";
import { createPortableWasm } from "../../packages/idealis/src/portable-wasm.js";
import { decodeSafetyResult } from "../../packages/idealis/src/safety.js";
import { checkTlaSource } from "../../packages/idealis/src/tla-source.js";
import { isRecord } from "../../packages/idealis/src/trace.js";

const directory = "fixtures/quint-generated/generated";
const core = await createPortableWasm(
  await readFile(
    "crates/idealis-core/target/wasm32-unknown-unknown/release/idealis_core.wasm",
  ),
);
const native = startNative();
after(() => native.close());
const oracle: unknown = JSON.parse(
  await readFile("fixtures/quint-generated/tlc-oracle.json", "utf8"),
);
assert.ok(Array.isArray(oracle) && oracle.length === names.length * 2);

test("Quint output is preserved byte for byte with its source provenance", async () => {
  const manifest: unknown = JSON.parse(
    await readFile(`${directory}/manifest.json`, "utf8"),
  );
  assert.ok(isRecord(manifest) && Array.isArray(manifest.records));
  assert.equal(manifest.quint, "0.33.1");
  assert.equal(manifest.records.length, names.length);
  for (const record of manifest.records) {
    assert.ok(isRecord(record) && names.some((name) => name === record.name));
    const source = await readFile(`${directory}/${record.name}.tla`);
    assert.equal(
      createHash("sha256").update(source).digest("hex"),
      record.tlaSha256,
    );
    assert.deepEqual(
      source,
      await readFile(`${directory}/${record.name}.stdout`),
    );
    assert.deepEqual(
      await readFile(`${directory}/${record.name}.qnt`),
      await readFile(`fixtures/quint-generated/${record.name}.qnt`),
    );
  }
});

for (const name of names)
  for (const property of ["safety", "broken"] as const)
    test(`${name}: generated TLA+ ${property} agrees in native and Wasm`, async () => {
      const input = {
        source: await readFile(`${directory}/${name}.tla`, "utf8"),
        config: config(property),
      };
      const result = checkTlaSource(core, input, bounds);
      const reference: unknown = oracle.find(
        (entry: unknown) =>
          isRecord(entry) && entry.name === name && entry.property === property,
      );
      assert.ok(isRecord(reference));
      assert.equal(
        createHash("sha256").update(input.source).digest("hex"),
        reference.tlaSha256,
      );
      assert.equal(
        createHash("sha256").update(input.config).digest("hex"),
        reference.configSha256,
      );
      assert.equal(result.status, reference.status);
      if (reference.states !== null)
        assert.equal(result.states, reference.states);
      if (reference.generated !== null)
        assert.equal(result.generated, reference.generated);
      const response = await native.request({
        version: 1,
        request: { kind: "check-tla", ...input, bounds },
      });
      assert.ok(isRecord(response));
      assert.equal(response.kind, "safety", JSON.stringify(response));
      assert.deepEqual(decodeSafetyResult(response.result), result);
      assert.equal(
        result.status,
        property === "safety" ? "complete" : "invariant-violation",
      );
      assert.ok(result.states > 0);
      assert.equal(result.reason, null);
      if (property === "broken") {
        assert.equal(result.violation?.name, "broken");
        assert.ok(result.violation && result.violation.trace.length > 0);
        if (["Counter", "Choice", "Match"].includes(name))
          assert.ok(result.violation.trace.length > 1);
      }
    });

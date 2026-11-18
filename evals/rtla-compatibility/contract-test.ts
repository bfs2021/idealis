import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { test } from "node:test";
import { isRecord } from "../../src/trace.js";
import { createParserWasm } from "../rtla-parser/wasm.js";

interface Saved {
  readonly records: readonly {
    readonly name: string;
    readonly packet: string;
  }[];
}
interface Observation {
  readonly actual: unknown;
}
interface AuditResult {
  readonly status: string;
  readonly compatibility: string;
  readonly referenceFailures: readonly { readonly name: string }[];
}
interface LabelReference {
  readonly records: readonly {
    readonly results: readonly {
      readonly version: string;
      readonly kind: string;
    }[];
  }[];
}
interface CorpusReference {
  readonly records: readonly {
    readonly name: string;
    readonly sany: string;
    readonly family: string;
    readonly rtla: string;
  }[];
}

test("native/Wasm label junctions match both saved SANY references", async () => {
  const root = "evals/runs/0068-label-junction-reference";
  const saved: LabelReference = JSON.parse(
    await readFile(join(root, "result.json"), "utf8"),
  );
  const wasm = await createParserWasm(
    await readFile(
      "crates/rtla/target/wasm32-unknown-unknown/release/rtla.wasm",
    ),
  );
  assert.equal(saved.records.length, 7);
  for (const [index, record] of saved.records.entries()) {
    const source = await readFile(
      join(root, String(index), "Input.tla"),
      "utf8",
    );
    const request = { kind: "module", source };
    const native: unknown = JSON.parse(
      execFileSync("crates/rtla/target/release/rtla", [], {
        input: `${JSON.stringify(request)}\n`,
      }).toString(),
    );
    assert.deepEqual(wasm.request(request), native, source);
    assert.ok(isRecord(native));
    const actual =
      native.kind === "module"
        ? "accepted"
        : isRecord(native.error)
          ? native.error.kind
          : undefined;
    assert.deepEqual(
      record.results.map((result) => result.version),
      ["1.7.4", "1.8.0"],
    );
    for (const reference of record.results)
      assert.equal(actual, reference.kind, `${reference.version}: ${source}`);
  }
});

test("rtla replays the revised corpus and new symbolic acceptances have a SANY reference", async () => {
  const root = "evals/runs/0069-stable-audit";
  const before = "evals/runs/0068-stable-fixed";
  const inputs: readonly { readonly source: string }[] = JSON.parse(
    await readFile(join(root, "inputs.json"), "utf8"),
  );
  const saved: Saved = JSON.parse(
    await readFile(join(root, "result.json"), "utf8"),
  );
  assert.equal(inputs.length, 368);
  const native = execFileSync("crates/rtla/target/release/rtla", [], {
    input: `${inputs.map((input) => JSON.stringify({ kind: "module", source: input.source })).join("\n")}\n`,
    maxBuffer: 64 * 1024 * 1024,
  })
    .toString()
    .trim()
    .split("\n")
    .map((line): unknown => JSON.parse(line));
  assert.equal(native.length, inputs.length);
  const wasm = await createParserWasm(
    await readFile(
      "crates/rtla/target/wasm32-unknown-unknown/release/rtla.wasm",
    ),
  );
  const changes = [];
  for (const [index, record] of saved.records.entries()) {
    const expected: Observation = JSON.parse(
      await readFile(join(root, record.packet, "observation.json"), "utf8"),
    );
    const old: Observation = JSON.parse(
      await readFile(join(before, record.packet, "observation.json"), "utf8"),
    );
    assert.deepEqual(native[index], expected.actual, record.name);
    assert.deepEqual(
      wasm.request({ kind: "module", source: inputs[index]?.source }),
      expected.actual,
      record.name,
    );
    if (
      isRecord(old.actual) &&
      old.actual.kind === "error" &&
      isRecord(expected.actual) &&
      expected.actual.kind === "module"
    )
      changes.push(record.name);
  }
  assert.equal(changes.length, 17);
  const preview: CorpusReference = JSON.parse(
    await readFile("evals/runs/0069-preview-audit/result.json", "utf8"),
  );
  for (const name of changes)
    assert.equal(
      preview.records.find((record) => record.name === name)?.sany,
      "accepted",
      name,
    );
  assert.deepEqual(
    preview.records.filter(
      (record) =>
        record.family !== "prefix-focus" &&
        record.sany === "syntax" &&
        record.rtla === "accepted",
    ),
    [],
  );
});

test("stable prefix binding is retained and reference exceptions are not syntax matches", async () => {
  const wasm = await createParserWasm(
    await readFile(
      "crates/rtla/target/wasm32-unknown-unknown/release/rtla.wasm",
    ),
  );
  const expression = wasm.request({
    kind: "expression",
    source: "SUBSET 1..2",
  });
  assert.ok(isRecord(expression) && isRecord(expression.expression));
  assert.equal(expression.expression.operator, "..");
  assert.ok(
    Array.isArray(expression.expression.args) &&
      isRecord(expression.expression.args[0]),
  );
  assert.equal(expression.expression.args[0].operator, "SUBSET");
  const stable: AuditResult = JSON.parse(
    await readFile("evals/runs/0068-stable-fixed/result.json", "utf8"),
  );
  const preview: AuditResult = JSON.parse(
    await readFile("evals/runs/0068-preview-fixed/result.json", "utf8"),
  );
  assert.equal(stable.status, "partial-reference");
  assert.equal(stable.referenceFailures.length, 1);
  assert.equal(stable.referenceFailures[0]?.name, "Blank File Parse Error");
  assert.equal(preview.status, "complete");
  assert.equal(preview.compatibility, "partial");
  for (const root of ["0068-stable-fixed", "0068-preview-fixed"]) {
    const response: unknown = JSON.parse(
      await readFile(`evals/runs/${root}/cases/132/sany.json`, "utf8"),
    );
    assert.ok(isRecord(response));
    assert.equal(response.kind, "syntax");
  }
});

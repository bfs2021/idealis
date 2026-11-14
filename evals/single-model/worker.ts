import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createPortableWasm } from "../../src/portable-wasm.js";
import { checkSafety, decodeSafetyResult } from "../../src/safety.js";
import { compileTlaSafety } from "../../src/tla-safety.js";
import { isRecord } from "../../src/trace.js";
import { startNative } from "../usable/native.js";
import { type BenchmarkCase, bounds } from "./cases.js";

const backend = process.argv[2];
const path = process.argv[3];
assert.ok((backend === "native" || backend === "wasm") && path);
const input: BenchmarkCase = JSON.parse(await readFile(path, "utf8"));
const frontendStart = performance.now();
const program = compileTlaSafety(input);
const frontendMs = performance.now() - frontendStart;
const start = performance.now();
const core =
  backend === "wasm"
    ? await createPortableWasm(
        await readFile(
          "crates/modelt-core/target/wasm32-unknown-unknown/release/modelt_core.wasm",
        ),
      )
    : undefined;
const native = backend === "native" ? startNative() : undefined;
if (native)
  await native.request({
    version: 1,
    request: { kind: "shrink-candidates", length: 0 },
  });
const engineStartupMs = performance.now() - start;
try {
  const apiStart = performance.now();
  const response = core
    ? checkSafety(core, program, bounds)
    : await native?.request({
        version: 1,
        request: { kind: "check-safety", program, bounds },
      });
  const result = core
    ? response
    : (() => {
        assert.ok(
          isRecord(response) && response.kind === "safety",
          JSON.stringify(response),
        );
        return decodeSafetyResult(response.result);
      })();
  console.log(
    JSON.stringify({
      frontendMs,
      engineStartupMs,
      apiMs: performance.now() - apiStart,
      result,
    }),
  );
} finally {
  await native?.close();
}

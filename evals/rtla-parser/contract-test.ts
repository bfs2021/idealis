import assert from "node:assert/strict";
import { test } from "node:test";
import { compare } from "./compare.js";
import { normalize } from "./corpus.js";

test("rtla native/Wasm preserve the semantic AST of the fixed finite-fragment corpus", async () => {
  const result = await compare();
  assert.ok(result.moduleComparisons > 150);
  console.log(
    JSON.stringify({
      modules: result.moduleComparisons,
      configs: result.configComparisons,
      nativeWasm: result.nativeWasmComparisons,
    }),
  );
});
test("AST comparison keeps binding order, prime, operator, values and bullet nesting", () => {
  assert.notDeepEqual(
    normalize({ kind: "Prime", expression: { kind: "Identifier", name: "x" } }),
    normalize({ kind: "Identifier", name: "x" }),
  );
  assert.notDeepEqual(
    normalize({ kind: "Apply", operator: "+", args: [1, 2] }),
    normalize({ kind: "Apply", operator: "-", args: [1, 2] }),
  );
  assert.notDeepEqual(
    normalize({ kind: "And", operands: [[1, 2], 3] }),
    normalize({ kind: "And", operands: [1, [2, 3]] }),
  );
});

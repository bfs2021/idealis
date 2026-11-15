import assert from "node:assert/strict";
import { PortableCoreError } from "../src/portable-value.js";
import { decodeSafetyProgram } from "../src/safety-program.js";
import { compileTlaSafety } from "../src/tla-safety.js";

export const registerSourceContract = (
  test: (name: string, run: () => void) => unknown,
): void => {
  test("compiled IR preserves a snapshot and rejects unsupported wire shapes", () => {
    const program = compileTlaSafety({
      source:
        "---- MODULE Wire ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = 0\nNext == x' = x + 1\nSafe == x < 3\n====",
      config: "INIT Init\nNEXT Next\nINVARIANT Safe",
    });
    const parsed = decodeSafetyProgram(program);
    assert.deepEqual(parsed, program);
    assert.notEqual(parsed, program);
    for (const invalid of [
      { ...program, version: 2 },
      { ...program, extra: true },
      { ...program, variables: ["x", "x"] },
      {
        ...program,
        next: [
          {
            name: "Next",
            steps: [{ kind: "guard", expression: { op: "mystery" } }],
          },
        ],
      },
      {
        ...program,
        invariants: [
          { name: "Safe", expression: { op: "read", slot: 0, next: "true" } },
        ],
      },
      {
        ...program,
        invariants: [
          {
            name: "Safe",
            expression: {
              op: "forall",
              bounds: [{ slot: 0, domain: { kind: "unknown" } }],
              body: { op: "old" },
            },
          },
        ],
      },
      {
        ...program,
        properties: [{ name: "Safe", formula: { kind: "unknown" } }],
      },
    ])
      assert.throws(() => decodeSafetyProgram(invalid), PortableCoreError);
  });
};

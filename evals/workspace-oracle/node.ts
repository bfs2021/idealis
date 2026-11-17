import { AssertionError } from "node:assert";
import { readFileSync } from "node:fs";
import { snapshot } from "./snapshot.js";
import { createOracle } from "./wasm.js";

let oracle: ReturnType<typeof createOracle> | undefined;

export const compareOracle = (
  actual: unknown,
  expected: unknown,
  message?: string | Error,
) => {
  oracle ??= createOracle(
    readFileSync(
      "crates/oracle/target/wasm32-unknown-unknown/release/idealis_oracle.wasm",
    ),
  );
  return oracle.request({
    version: 1,
    group: "workspace-equality",
    reference: "expected",
    implementations: ["actual"],
    cases: [
      {
        id: "case",
        input: {
          label: message instanceof Error ? message.message : (message ?? null),
        },
        reference: { kind: "value", value: snapshot(expected) },
        actual: [{ kind: "value", value: snapshot(actual) }],
      },
    ],
  });
};

export const assertOracle = (
  actual: unknown,
  expected: unknown,
  message?: string | Error,
) => {
  const checked = compareOracle(actual, expected, message);
  if (checked.status !== "matched")
    throw Object.assign(
      new AssertionError({
        actual,
        expected,
        operator: "idealis-oracle",
        message: message instanceof Error ? message.message : message,
      }),
      { report: checked.report },
    );
  return checked.report;
};

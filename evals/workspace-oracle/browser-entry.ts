import { snapshot } from "./snapshot.js";
import { createOracle } from "./wasm.js";

export const run = async () => {
  const core = createOracle(
    new Uint8Array(await (await fetch("/wire.wasm")).arrayBuffer()),
  );
  const check = (actual: unknown, expected: unknown) =>
    core.request({
      version: 1,
      group: "browser",
      reference: "model",
      implementations: ["implementation"],
      cases: [
        {
          id: "one",
          input: null,
          reference: { kind: "value", value: snapshot(expected) },
          actual: [{ kind: "value", value: snapshot(actual) }],
        },
      ],
    }).status;
  return {
    matched: check(
      {
        value: 7n,
        optional: undefined,
        nested: [true, undefined, JSON.parse('{"__proto__":7,"日本語":"雪"}')],
      },
      {
        optional: undefined,
        value: 7n,
        nested: [true, undefined, JSON.parse('{"日本語":"雪","__proto__":7}')],
      },
    ),
    mismatch: check(-0, 0),
    isolated: crossOriginIsolated,
  };
};

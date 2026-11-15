import type {
  PortableCore,
  SafetyBounds,
  SafetyProgram,
  SafetyResult,
} from "./contracts.js";
import { PortableCoreError } from "./portable-value.js";

export const compileTlaSource = (
  core: PortableCore,
  input: { readonly source: string; readonly config: string },
): SafetyProgram => {
  const result = core.request({
    version: 1,
    request: {
      kind: "compile-tla",
      source: input.source,
      config: input.config,
    },
  });
  if (result.kind !== "compiled")
    throw new PortableCoreError("protocol", "Expected compiled TLA program");
  return result.program;
};
export const checkTlaSource = (
  core: PortableCore,
  input: { readonly source: string; readonly config: string },
  bounds: SafetyBounds,
): SafetyResult => {
  const result = core.request({
    version: 1,
    request: {
      kind: "check-tla",
      source: input.source,
      config: input.config,
      bounds,
    },
  });
  if (result.kind !== "safety")
    throw new PortableCoreError("protocol", "Expected TLA safety result");
  return result.result;
};

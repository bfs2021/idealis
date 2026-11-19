import { createRequire } from "node:module";
import { resolve } from "node:path";
import type { JsonValue, OracleRequest } from "../../src/generated/roracle.js";
import { isRecord } from "../../src/trace.js";
import {
  decodeOracleReply,
  type OracleClient,
} from "../workspace-oracle/protocol.js";
import type {
  abiVersion,
  dispatchJson,
  dispatchObject,
} from "./generated/index.js";

interface NativeBinding {
  readonly abiVersion: typeof abiVersion;
  readonly dispatchJson: typeof dispatchJson;
  readonly dispatchObject: typeof dispatchObject;
}

export const nativePath = "evals/roracle-native/generated/roracle-native.node";

export const loadNativeBinding = (path = nativePath): NativeBinding => {
  const input: unknown = createRequire(import.meta.url)(resolve(path));
  if (
    !isRecord(input) ||
    typeof input.abiVersion !== "function" ||
    input.abiVersion() !== 1 ||
    typeof input.dispatchJson !== "function" ||
    typeof input.dispatchObject !== "function"
  )
    throw new Error("Invalid native oracle ABI");
  const nativeVersion = input.abiVersion;
  const nativeJson = input.dispatchJson;
  const nativeObject = input.dispatchObject;
  return {
    abiVersion: () => {
      const version: unknown = nativeVersion();
      if (typeof version !== "number")
        throw new Error("Invalid native ABI version");
      return version;
    },
    dispatchJson: (value) => {
      const result: unknown = nativeJson(value);
      if (typeof result !== "string")
        throw new Error("Invalid native JSON reply");
      return result;
    },
    dispatchObject: (value) => {
      const result: unknown = nativeObject(value);
      if (typeof result !== "string")
        throw new Error("Invalid native object reply");
      return result;
    },
  };
};

const canonicalObject = (input: JsonValue) => {
  const ancestors = new Set<object>();
  let remaining = 1_000_000;
  const visit = (value: JsonValue, depth: number): void => {
    if (--remaining < 0 || depth > 128)
      throw new Error("Native JSON conversion budget exceeded");
    if (value === null || typeof value === "boolean") return;
    if (typeof value === "string") {
      if (/[\uD800-\uDFFF]/u.test(value))
        throw new Error("Native JSON requires Unicode scalar strings");
      return;
    }
    if (typeof value === "number") {
      if (!Number.isFinite(value) || Object.is(value, -0))
        throw new Error(
          "Native JSON requires canonical finite numbers; use snapshot for JS values",
        );
      return;
    }
    if (typeof value !== "object" || ancestors.has(value))
      throw new Error("Native JSON requires acyclic JSON data");
    if (Object.getOwnPropertySymbols(value).length)
      throw new Error("Native JSON rejects symbol keys");
    ancestors.add(value);
    if (Array.isArray(value)) {
      if (Object.getOwnPropertyNames(value).length !== value.length + 1)
        throw new Error("Native JSON rejects sparse or extended arrays");
      for (let index = 0; index < value.length; index++) {
        const property = Object.getOwnPropertyDescriptor(value, index);
        if (!property || !("value" in property))
          throw new Error("Native JSON rejects accessors");
        visit(property.value, depth + 1);
      }
    } else {
      if (Object.getPrototypeOf(value) !== Object.prototype)
        throw new Error("Native JSON requires plain objects");
      for (const name in value)
        if (!Object.hasOwn(value, name))
          throw new Error("Native JSON rejects inherited enumerable fields");
      for (const [name, property] of Object.entries(
        Object.getOwnPropertyDescriptors(value),
      )) {
        visit(name, depth + 1);
        if (!property.enumerable || !("value" in property))
          throw new Error("Native JSON rejects accessors and hidden fields");
        visit(property.value, depth + 1);
      }
    }
    ancestors.delete(value);
  };
  visit(input, 0);
};

export const createNativeOracle = (
  mode: "native-json" | "native-object",
  path = nativePath,
): OracleClient => {
  const binding = loadNativeBinding(path);
  return {
    request: (input: OracleRequest) => {
      if (mode === "native-object") {
        canonicalObject(input);
        return decodeOracleReply(binding.dispatchObject(input));
      }
      return decodeOracleReply(binding.dispatchJson(JSON.stringify(input)));
    },
  };
};

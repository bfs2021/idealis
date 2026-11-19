import type {
  ExpansionStats,
  IncrementalSafetyResult,
  PortableCore,
  SafetyBounds,
  SafetyExpansionCache,
  SafetyProgram,
} from "./contracts.js";
import { normalizePortableValue, PortableCoreError } from "./portable-value.js";
import { isRecord } from "./trace.js";

const fields = (value: unknown, names: readonly string[]) => {
  if (
    !isRecord(value) ||
    Object.keys(value).sort().join() !== [...names].sort().join()
  )
    throw new PortableCoreError("protocol", "Invalid expansion fields");
  return value;
};
const count = (value: unknown) => {
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < 0 ||
    value > 0xffff_ffff
  )
    throw new PortableCoreError("protocol", "Invalid expansion count");
  return value;
};
export const createSafetyExpansionCache = (): SafetyExpansionCache => ({
  format: "idealis.safety-expansions",
  version: 1,
  buildDigest: "",
  transitionIdentity: "",
  entries: {},
});

export const decodeSafetyExpansionCache = (
  input: unknown,
): SafetyExpansionCache => {
  const value = fields(input, [
    "format",
    "version",
    "buildDigest",
    "transitionIdentity",
    "entries",
  ]);
  if (
    value.format !== "idealis.safety-expansions" ||
    value.version !== 1 ||
    typeof value.buildDigest !== "string" ||
    (value.buildDigest !== "" && !/^[0-9a-f]{64}$/.test(value.buildDigest)) ||
    typeof value.transitionIdentity !== "string" ||
    !isRecord(value.entries)
  )
    throw new PortableCoreError("protocol", "Invalid expansion cache");
  return {
    format: "idealis.safety-expansions",
    version: 1,
    buildDigest: value.buildDigest,
    transitionIdentity: value.transitionIdentity,
    entries: Object.fromEntries(
      Object.entries(value.entries).map(([key, raw]) => {
        const entry = fields(raw, ["digest", "successors"]);
        if (
          typeof entry.digest !== "string" ||
          !/^[0-9a-f]{64}$/.test(entry.digest) ||
          !Array.isArray(entry.successors)
        )
          throw new PortableCoreError("protocol", "Invalid complete expansion");
        return [
          key,
          {
            digest: entry.digest,
            successors: entry.successors.map((raw: unknown) => {
              const successor = fields(raw, ["action", "values"]);
              if (!Array.isArray(successor.values))
                throw new PortableCoreError(
                  "protocol",
                  "Invalid expansion state",
                );
              return {
                action: count(successor.action),
                values: successor.values.map((value: unknown) =>
                  normalizePortableValue(value),
                ),
              };
            }),
          },
        ];
      }),
    ),
  };
};
export const decodeExpansionStats = (input: unknown): ExpansionStats => {
  const value = fields(input, ["hits", "misses", "writes", "invalidated"]);
  if (typeof value.invalidated !== "boolean")
    throw new PortableCoreError("protocol", "Invalid expansion invalidation");
  return {
    hits: count(value.hits),
    misses: count(value.misses),
    writes: count(value.writes),
    invalidated: value.invalidated,
  };
};

export const checkSafetyIncremental = (
  core: PortableCore,
  program: SafetyProgram,
  bounds: SafetyBounds,
  options: {
    readonly cache: SafetyExpansionCache;
    readonly buildDigest: string;
    readonly maxEntries: number;
  },
): IncrementalSafetyResult => {
  const output = core.request({
    version: 1,
    request: { kind: "check-safety-incremental", program, bounds, ...options },
  });
  if (output.kind !== "incremental-safety")
    throw new PortableCoreError(
      "protocol",
      "Expected incremental safety result",
    );
  return { result: output.result, cache: output.cache, stats: output.stats };
};

export const mergeSafetyExpansions = (
  core: PortableCore,
  left: SafetyExpansionCache,
  right: SafetyExpansionCache,
  maxEntries: number,
): SafetyExpansionCache => {
  const output = core.request({
    version: 1,
    request: { kind: "merge-safety-expansions", left, right, maxEntries },
  });
  if (output.kind !== "safety-expansions")
    throw new PortableCoreError("protocol", "Expected merged expansion cache");
  return output.cache;
};

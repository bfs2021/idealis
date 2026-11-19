import type {
  CheckReport,
  JsonValue,
  Observation,
  OracleRequest,
  Status,
} from "../../src/generated/roracle.js";

export interface OracleReport {
  readonly status: Status;
  readonly report: CheckReport<JsonValue, JsonValue>;
}
export interface OracleClient {
  readonly request: (input: OracleRequest) => OracleReport;
}

const object = (value: JsonValue | undefined, names: readonly string[]) => {
  if (
    !value ||
    typeof value !== "object" ||
    Array.isArray(value) ||
    Object.keys(value).length !== names.length ||
    names.some((name) => !Object.hasOwn(value, name))
  )
    throw new Error("Invalid oracle report fields");
  return value;
};
const text = (value: JsonValue | undefined) => {
  if (typeof value !== "string") throw new Error("Invalid oracle report text");
  return value;
};
const status = (value: JsonValue | undefined): Status => {
  if (
    value !== "matched" &&
    value !== "mismatch" &&
    value !== "unknown" &&
    value !== "invalid" &&
    value !== "error"
  )
    throw new Error("Invalid oracle report status");
  return value;
};
const observation = (input: JsonValue | undefined): Observation<JsonValue> => {
  if (!input || typeof input !== "object" || Array.isArray(input))
    throw new Error("Invalid oracle observation");
  switch (input.kind) {
    case "value": {
      const value = object(input, ["kind", "value"]).value;
      if (value === undefined) throw new Error("Missing oracle value");
      return { kind: "value", value };
    }
    case "allowed": {
      const values = object(input, ["kind", "values"]).values;
      if (!Array.isArray(values))
        throw new Error("Invalid allowed observations");
      return { kind: "allowed", values };
    }
    case "unknown":
    case "invalid":
      object(input, ["kind", "reason"]);
      return { kind: input.kind, reason: text(input.reason) };
    case "error":
      object(input, ["kind", "message"]);
      return { kind: "error", message: text(input.message) };
    default:
      throw new Error("Invalid oracle observation kind");
  }
};

export const decodeOracleReply = (json: string): OracleReport => {
  const input: JsonValue = JSON.parse(json);
  if (
    input &&
    typeof input === "object" &&
    !Array.isArray(input) &&
    input.kind === "error"
  ) {
    const failed = object(input, ["kind", "message"]);
    throw new Error(`Oracle batch failed: ${text(failed.message)}`);
  }
  const response = object(input, ["kind", "report"]);
  if (response.kind !== "checked") throw new Error("Invalid oracle reply kind");
  const report = object(response.report, [
    "version",
    "group",
    "reference",
    "implementations",
    "status",
    "cases",
  ]);
  if (
    report.version !== 1 ||
    !Array.isArray(report.implementations) ||
    !Array.isArray(report.cases)
  )
    throw new Error("Invalid oracle report");
  if (!report.cases.length || !report.implementations.length)
    throw new Error("Empty oracle report");
  const implementations = report.implementations.map(text);
  const decoded: CheckReport<JsonValue, JsonValue> = {
    version: report.version,
    group: text(report.group),
    reference: text(report.reference),
    implementations,
    status: status(report.status),
    cases: report.cases.map((input) => {
      const row = object(input, ["case", "reference", "comparisons"]);
      const saved = object(row.case, ["id", "input"]);
      if (saved.input === undefined || !Array.isArray(row.comparisons))
        throw new Error("Invalid oracle case");
      if (row.comparisons.length !== implementations.length)
        throw new Error("Oracle comparison dimensions differ");
      return {
        case: { id: text(saved.id), input: saved.input },
        reference: observation(row.reference),
        comparisons: row.comparisons.map((input) => {
          const comparison = object(input, ["id", "status", "actual"]);
          return {
            id: text(comparison.id),
            status: status(comparison.status),
            actual: observation(comparison.actual),
          };
        }),
      };
    }),
  };
  return { status: decoded.status, report: decoded };
};

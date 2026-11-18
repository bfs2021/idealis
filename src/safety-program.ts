import type { SafetyProgram } from "./contracts.js";
import { normalizePortableValue, PortableCoreError } from "./portable-value.js";
import { isRecord } from "./trace.js";

const fields = (
  input: unknown,
  required: readonly string[],
  optional: readonly string[] = [],
) => {
  if (
    !isRecord(input) ||
    required.some((key) => !Object.hasOwn(input, key)) ||
    Object.keys(input).some(
      (key) => !required.includes(key) && !optional.includes(key),
    )
  )
    throw new PortableCoreError("protocol", "Invalid compiled program fields");
  return input;
};
const array = (input: unknown): readonly unknown[] => {
  if (!Array.isArray(input))
    throw new PortableCoreError("protocol", "Expected compiled program array");
  return input;
};
const text = (input: unknown): void => {
  if (typeof input !== "string" || !input.length)
    throw new PortableCoreError(
      "protocol",
      "Expected compiled program identifier",
    );
};
const count = (input: unknown): void => {
  if (
    typeof input !== "number" ||
    !Number.isSafeInteger(input) ||
    input < 0 ||
    input > 0xffff_ffff
  )
    throw new PortableCoreError("protocol", "Invalid compiled program count");
};
const domain = (input: unknown, depth: number): void => {
  const value = fields(input, ["kind"], ["values", "lower", "upper", "value"]);
  switch (value.kind) {
    case "set":
      fields(value, ["kind", "values"]);
      array(value.values).forEach((item) => {
        expression(item, depth);
      });
      return;
    case "range":
      fields(value, ["kind", "lower", "upper"]);
      expression(value.lower, depth);
      expression(value.upper, depth);
      return;
    case "value":
      fields(value, ["kind", "value"]);
      expression(value.value, depth);
      return;
    default:
      throw new PortableCoreError("protocol", "Invalid compiled domain");
  }
};
const binding = (input: unknown, depth: number): void => {
  const value = fields(input, ["slot", "domain"], ["tupleSize"]);
  count(value.slot);
  domain(value.domain, depth);
  if (value.tupleSize !== undefined) count(value.tupleSize);
};
const branches = (input: unknown, depth: number): void => {
  for (const item of array(input)) {
    const branch = fields(item, ["name", "steps"]);
    text(branch.name);
    for (const input of array(branch.steps)) {
      if (!isRecord(input))
        throw new PortableCoreError("protocol", "Invalid compiled step");
      switch (input.kind) {
        case "bind": {
          const step = fields(input, ["kind", "slot", "domain"], ["tupleSize"]);
          binding(
            {
              slot: step.slot,
              domain: step.domain,
              ...(step.tupleSize === undefined
                ? {}
                : { tupleSize: step.tupleSize }),
            },
            depth,
          );
          break;
        }
        case "choose": {
          const step = fields(input, ["kind", "slot", "domain"]);
          count(step.slot);
          domain(step.domain, depth);
          break;
        }
        case "assign": {
          const step = fields(input, ["kind", "slot", "value"]);
          count(step.slot);
          expression(step.value, depth);
          break;
        }
        case "guard": {
          const step = fields(input, ["kind", "expression"]);
          expression(step.expression, depth);
          break;
        }
        default:
          throw new PortableCoreError("protocol", "Invalid compiled step kind");
      }
    }
  }
};
const expression = (input: unknown, depth: number): void => {
  if (!isRecord(input) || depth > 128)
    throw new PortableCoreError("protocol", "Invalid compiled expression");
  const child = depth + 1;
  const unary = [
    "prime",
    "not",
    "neg",
    "domain",
    "variant-tag",
    "cardinality",
    "sequence-set",
    "len",
    "head",
    "tail",
    "powerset",
    "union",
  ];
  const binary = [
    "cup",
    "cap",
    "difference",
    "subset-eq",
    "append",
    "concat",
    "function-set",
    "function-in",
    "sequence-in",
    "add",
    "sub",
    "mul",
    "div",
    "mod",
    "pow",
    "eq",
    "ne",
    "lt",
    "le",
    "gt",
    "ge",
    "and",
    "or",
    "implies",
  ];
  if (typeof input.op === "string" && unary.includes(input.op)) {
    const value = fields(input, ["op", "argument"]);
    expression(value.argument, child);
    return;
  }
  if (typeof input.op === "string" && binary.includes(input.op)) {
    const value = fields(input, ["op", "left", "right"]);
    expression(value.left, child);
    expression(value.right, child);
    return;
  }
  switch (input.op) {
    case "unmatched-case":
      fields(input, ["op", "location"]);
      text(input.location);
      return;
    case "old":
      fields(input, ["op"]);
      return;
    case "data":
      fields(input, ["op", "value"]);
      normalizePortableValue(input.value);
      return;
    case "literal": {
      const value = fields(input, ["op", "value"]);
      const scalar = normalizePortableValue(value.value);
      if (scalar.type !== "int" && scalar.type !== "bool")
        throw new PortableCoreError("protocol", "Invalid compiled scalar");
      return;
    }
    case "bound":
      fields(input, ["op", "slot"]);
      count(input.slot);
      return;
    case "read":
      fields(input, ["op", "slot", "next"]);
      count(input.slot);
      if (typeof input.next !== "boolean")
        throw new PortableCoreError("protocol", "Invalid compiled read");
      return;
    case "set":
    case "tuple":
      fields(input, ["op", "values"]);
      array(input.values).forEach((item) => {
        expression(item, child);
      });
      return;
    case "record":
    case "record-set":
      fields(input, ["op", "fields"]);
      for (const item of array(input.fields)) {
        const pair = array(item);
        if (pair.length !== 2 || typeof pair[0] !== "string")
          throw new PortableCoreError("protocol", "Invalid compiled field");
        expression(pair[1], child);
      }
      return;
    case "at":
      fields(input, ["op", "target", "key"]);
      expression(input.target, child);
      expression(input.key, child);
      return;
    case "update":
      fields(input, ["op", "target", "updates"]);
      expression(input.target, child);
      for (const item of array(input.updates)) {
        const update = fields(item, ["path", "value"]);
        array(update.path).forEach((part) => {
          expression(part, child);
        });
        expression(update.value, child);
      }
      return;
    case "forall":
    case "exists":
    case "function":
    case "filter":
    case "map-set":
      fields(input, ["op", "bounds", "body"]);
      array(input.bounds).forEach((item) => {
        binding(item, child);
      });
      expression(input.body, child);
      return;
    case "fold":
      fields(input, [
        "op",
        "accumulator",
        "initial",
        "collection",
        "order",
        "body",
      ]);
      count(input.accumulator);
      if (input.order !== "set-members" && input.order !== "sequence-left")
        throw new PortableCoreError("protocol", "Invalid compiled fold order");
      expression(input.initial, child);
      expression(input.collection, child);
      expression(input.body, child);
      return;
    case "choose":
      fields(input, ["op", "binding", "body"]);
      binding(input.binding, child);
      expression(input.body, child);
      return;
    case "select-seq":
    case "sort-seq":
      fields(input, ["op", "slot", "sequence", "body"]);
      count(input.slot);
      expression(input.sequence, child);
      expression(input.body, child);
      return;
    case "assert":
      fields(input, ["op", "condition", "message"]);
      expression(input.condition, child);
      expression(input.message, child);
      return;
    case "if":
      fields(input, ["op", "condition", "consequent", "alternative"]);
      expression(input.condition, child);
      expression(input.consequent, child);
      expression(input.alternative, child);
      return;
    case "subseq":
      fields(input, ["op", "sequence", "from", "to"]);
      expression(input.sequence, child);
      expression(input.from, child);
      expression(input.to, child);
      return;
    case "in":
      fields(input, ["op", "value", "domain"]);
      expression(input.value, child);
      domain(input.domain, child);
      return;
    case "enabled":
      fields(input, ["op", "branches"], ["subscript"]);
      branches(input.branches, child);
      if (input.subscript === null) delete input.subscript;
      if (input.subscript !== undefined) expression(input.subscript, child);
      return;
    default:
      throw new PortableCoreError("protocol", "Unknown compiled expression");
  }
};
const formula = (input: unknown, depth = 0): void => {
  if (!isRecord(input) || depth > 128)
    throw new PortableCoreError(
      "protocol",
      "Invalid compiled temporal formula",
    );
  switch (input.kind) {
    case "atom":
      fields(input, ["kind", "expression"]);
      expression(input.expression, 0);
      return;
    case "forall":
    case "exists":
      fields(input, ["kind", "bounds", "formula"]);
      array(input.bounds).forEach((item) => {
        binding(item, 0);
      });
      formula(input.formula, depth + 1);
      return;
    case "not":
    case "always":
    case "eventually":
      fields(input, ["kind", "formula"]);
      formula(input.formula, depth + 1);
      return;
    case "and":
    case "or":
      fields(input, ["kind", "left", "right"]);
      formula(input.left, depth + 1);
      formula(input.right, depth + 1);
      return;
    default:
      throw new PortableCoreError(
        "protocol",
        "Unknown compiled temporal formula",
      );
  }
};
export const decodeSafetyProgram = (input: unknown): SafetyProgram => {
  const value = fields(
    structuredClone(input),
    [
      "format",
      "version",
      "name",
      "variables",
      "initial",
      "next",
      "invariants",
      "assumptions",
      "checkDeadlock",
    ],
    [
      "symmetry",
      "initialProperties",
      "actionProperties",
      "properties",
      "fairness",
      "constraints",
      "actionConstraints",
    ],
  );
  if (
    value.format !== "modelt.safety-program" ||
    value.version !== 1 ||
    typeof value.checkDeadlock !== "boolean"
  )
    throw new PortableCoreError("protocol", "Invalid compiled program version");
  text(value.name);
  if (value.symmetry !== undefined) {
    const symmetry = fields(value.symmetry, ["modelValues"]);
    const members = array(symmetry.modelValues);
    if (
      members.length < 2 ||
      members.length > 64 ||
      new Set(members).size !== members.length ||
      members.some(
        (member) =>
          typeof member !== "string" ||
          !/^[A-Za-z_][A-Za-z_0-9]*$/.test(member) ||
          member[1] === "_",
      )
    )
      throw new PortableCoreError(
        "protocol",
        "Invalid compiled symmetry group",
      );
  }
  const variables = array(value.variables);
  variables.forEach(text);
  if (!variables.length || new Set(variables).size !== variables.length)
    throw new PortableCoreError("protocol", "Invalid compiled variables");
  branches(value.initial, 0);
  branches(value.next, 0);
  for (const key of [
    "invariants",
    "assumptions",
    "initialProperties",
    "actionProperties",
    "constraints",
    "actionConstraints",
  ]) {
    if (value[key] === undefined) continue;
    for (const item of array(value[key])) {
      const predicate = fields(item, ["name", "expression"]);
      text(predicate.name);
      expression(predicate.expression, 0);
    }
  }
  if (value.properties !== undefined)
    for (const item of array(value.properties)) {
      const property = fields(item, ["name", "formula"]);
      text(property.name);
      formula(property.formula);
    }
  if (value.fairness !== undefined)
    array(value.fairness).forEach((item) => {
      formula(item);
    });
  return value as unknown as SafetyProgram;
};

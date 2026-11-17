interface Snapshot {
  readonly type: string;
  readonly value?:
    | string
    | boolean
    | readonly Snapshot[]
    | Readonly<Record<string, Snapshot>>;
}

export const snapshot = (value: unknown): Snapshot => {
  const ancestors = new Set<object>();
  const visit = (input: unknown, depth: number): Snapshot => {
    if (depth > 48) throw new Error("Oracle snapshot nesting exceeds 48");
    if (input === null) return { type: "null" };
    switch (typeof input) {
      case "undefined":
        return { type: "undefined" };
      case "boolean":
        return { type: "boolean", value: input };
      case "string":
        return { type: "string", value: input };
      case "bigint":
        return { type: "bigint", value: input.toString() };
      case "number":
        return {
          type: "number",
          value: Object.is(input, -0) ? "-0" : String(input),
        };
      case "object":
        break;
      default:
        throw new Error(`Unsupported oracle snapshot: ${typeof input}`);
    }
    if (ancestors.has(input)) throw new Error("Cyclic oracle snapshot");
    if (Object.getOwnPropertySymbols(input).length > 0)
      throw new Error("Symbol keys are not portable oracle snapshots");
    ancestors.add(input);
    if (Array.isArray(input)) {
      const descriptors = Object.getOwnPropertyDescriptors(input);
      if (Object.keys(descriptors).length !== input.length + 1)
        throw new Error(
          "Sparse or extended arrays are not portable oracle snapshots",
        );
      const items = Array.from({ length: input.length }, (_, index) => {
        const descriptor = descriptors[String(index)];
        if (!descriptor || !("value" in descriptor))
          throw new Error("Sparse array or accessor in oracle snapshot");
        return visit(descriptor.value, depth + 1);
      });
      ancestors.delete(input);
      return { type: "array", value: items };
    }
    if (Object.getPrototypeOf(input) !== Object.prototype)
      throw new Error("Oracle snapshots require plain objects");
    const entries = Object.entries(Object.getOwnPropertyDescriptors(input)).map(
      ([key, descriptor]) => {
        if (!("value" in descriptor))
          throw new Error("Object accessor in oracle snapshot");
        if (!descriptor.enumerable)
          throw new Error("Nonenumerable oracle snapshot property");
        return [key, visit(descriptor.value, depth + 1)];
      },
    );
    ancestors.delete(input);
    return { type: "object", value: Object.fromEntries(entries) };
  };
  return visit(value, 0);
};

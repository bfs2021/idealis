import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import type {
  PortableValue,
  SafetyProgram,
  SafetyResult,
} from "../../src/contracts.js";
import type { Case } from "./cases.js";

const valueExpression = (
  value: PortableValue,
  depth: number,
  models: ReadonlyMap<string, string> = new Map(),
): string => {
  switch (value.type) {
    case "int":
      return value.value;
    case "bool":
      return value.value ? "TRUE" : "FALSE";
    case "string":
      return JSON.stringify(value.value);
    case "model-value":
      return models.get(value.value) ?? value.value;
    case "list":
      return `<<${value.value.map((inner) => valueExpression(inner, depth + 1, models)).join(",")}>>`;
    case "set":
      return `{${value.value.map((inner) => valueExpression(inner, depth + 1, models)).join(",")}}`;
    case "record":
      return valueExpression(
        {
          type: "map",
          value: value.value.map(([key, inner]) => [
            { type: "string", value: key },
            inner,
          ]),
        },
        depth,
        models,
      );
    case "map":
      return `[__key${depth} \\in {${value.value.map(([key]) => valueExpression(key, depth + 1, models)).join(",")}} |-> ${value.value.reduceRight((otherwise, [key, inner]) => `IF __key${depth} = ${valueExpression(key, depth + 1, models)} THEN ${valueExpression(inner, depth + 1, models)} ELSE ${otherwise}`, "TRUE")}]`;
    default:
      throw new Error("Not a TLA value");
  }
};
export const tlaValue = (value: PortableValue): string =>
  valueExpression(value, 0);
export const runTlc = async (
  java: string,
  jar: string,
  directory: string,
  input: Pick<Case, "source" | "config">,
  moduleName: string,
) => {
  await mkdir(directory, { recursive: false });
  await writeFile(join(directory, `${moduleName}.tla`), input.source, {
    flag: "wx",
  });
  await writeFile(join(directory, `${moduleName}.cfg`), input.config, {
    flag: "wx",
  });
  const scratch = await mkdtemp(join(tmpdir(), "idealis-tlc-"));
  const result = spawnSync(
    resolve(java),
    [
      "-XX:+UseParallelGC",
      `-Djava.io.tmpdir=${scratch}`,
      "-jar",
      resolve(jar),
      "-workers",
      "1",
      "-cleanup",
      "-config",
      `${moduleName}.cfg`,
      `${moduleName}.tla`,
    ],
    { cwd: directory, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  await rm(scratch, { recursive: true, force: true });
  if (result.error) throw result.error;
  const log = result.stdout + result.stderr;
  await writeFile(join(directory, "tlc.log"), log, { flag: "wx" });
  const counts = /(\d+) states generated, (\d+) distinct states found/.exec(
    log,
  );
  const status = /Invariant .*is violated/.test(log)
    ? "invariant-violation"
    : /Deadlock reached/.test(log)
      ? "deadlock"
      : /Temporal properties were violated|Action property .*violated|Property .*violated/.test(
            log,
          )
        ? "liveness-violation"
        : result.status === 0
          ? counts?.[2] === "0"
            ? "empty"
            : "complete"
          : "error";
  return {
    status,
    exit: result.status,
    states: counts ? Number(counts[2]) : null,
    generated: counts ? Number(counts[1]) : null,
    log,
  };
};

export const traceInput = (
  input: Pick<Case, "source" | "config">,
  program: SafetyProgram,
  result: SafetyResult,
  negate = true,
): {
  source: string;
  config: string;
  expected: "complete" | "liveness-violation";
} => {
  const violation = result.violation;
  assert.ok(violation && violation.trace.length > 0);
  const length = violation.trace.length;
  const models = new Map<string, string>();
  const collect = (value: PortableValue): void => {
    if (value.type === "model-value" && !models.has(value.value)) {
      const name = `__IdealisModelValue${models.size}`;
      assert.ok(
        !input.source.includes(name),
        "Model value replay name collision",
      );
      models.set(value.value, name);
    } else if (value.type === "list" || value.type === "set")
      value.value.forEach(collect);
    else if (value.type === "record")
      value.value.forEach((entry) => {
        collect(entry[1]);
      });
    else if (value.type === "map")
      value.value.forEach((entry) => {
        collect(entry[0]);
        collect(entry[1]);
      });
  };
  violation.trace.forEach((step) => {
    step.values.forEach(collect);
  });
  const state = (index: number, primed: boolean) =>
    violation.trace[index]?.values
      .map(
        (value, slot) =>
          `${program.variables[slot]}${primed ? "'" : ""} = ${valueExpression(value, 0, models)}`,
      )
      .join(" /\\ ");
  const next = violation.trace
    .map((_, index) => {
      const target =
        index + 1 < length ? index + 1 : (violation.loopStart ?? index);
      return `(__position = ${index} /\\ __position' = ${target} /\\ (${state(target, true)}))`;
    })
    .join(" \\/ ");
  const variables = `<<${[...program.variables, "__position"].join(",")}>>`;
  const originalVariables = `<<${program.variables.join(",")}>>`;
  const isTemporal = result.status === "liveness-violation";
  const isAction =
    result.status === "property-violation" &&
    program.actionProperties?.some(
      (predicate) => predicate.name === violation.name,
    );
  const expected = isAction
    ? violation.name
    : isTemporal
      ? `${program.fairness?.length ? "__OriginalFairness /\\ " : ""}${negate ? "~" : ""}(${violation.name})`
      : `<>(~(${violation.name}))`;
  // Use the source fairness formulas, never the Rust lowering, for witness validation.
  const specBody = /Spec\s*==\s*([\s\S]*?)(?=\n\w+\s*==|\n====)/.exec(
    input.source,
  )?.[1];
  const fairness = specBody?.match(/(?:WF_|SF_)[\s\S]*$/)?.[0];
  const extra = `\n${models.size ? `CONSTANT ${[...models.values()].join(", ")}\n` : ""}VARIABLE __position\n__TraceInit == Init /\\ __position = 0 /\\ (${state(0, false)})\n__TraceNext == (${next}) /\\ (Next \\/ UNCHANGED ${originalVariables})\n__TraceSpec == __TraceInit /\\ [][__TraceNext]_${variables} /\\ WF_${variables}(__TraceNext)\n${fairness ? `__OriginalFairness == ${fairness}\n` : ""}__Expected == ${expected}\n`;
  const source = input.source.replace(/=+\s*$/, () => `${extra}====`);
  const constants = input.config
    .split("\n")
    .filter((line) => line.startsWith("CONSTANT"))
    .join("\n");
  return {
    source,
    expected: isAction ? "liveness-violation" : "complete",
    config: `SPECIFICATION __TraceSpec\n${constants}\n${[...models].map(([value, name]) => `CONSTANT ${name} = ${value}`).join("\n")}\nPROPERTY __Expected\nCHECK_DEADLOCK TRUE`,
  };
};

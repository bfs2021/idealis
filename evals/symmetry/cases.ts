import type { TlaSourceInput } from "../../src/contracts.js";

export interface Input extends TlaSourceInput {
  readonly name: string;
}

export const bounds = {
  maxDepth: 1000,
  maxStates: 1_000_000,
  maxWork: 1_000_000_000,
};
const configuration = (count: number) =>
  `CONSTANT Nodes={${Array.from({ length: count }, (_, index) => `p${index}`).join(",")}}\nINIT Init\nNEXT Next\nINVARIANT Safe\nSYMMETRY Symm`;

export const counter = (
  count: number,
  maximum: number,
  fault = false,
): Input => ({
  name: `counter-${count}-${maximum}-${fault ? "fault" : "safe"}`,
  source: `---- MODULE Differential ----\nEXTENDS Naturals, TLC\nCONSTANT Nodes\nVARIABLE counters\nSymm == Permutations(Nodes)\nInit == counters = [node \\in Nodes |-> 0]\nNext == \\E node \\in Nodes: counters' = [counters EXCEPT ![node] = IF @ < ${maximum} THEN @ + 1 ELSE 0]\nSafe == ${fault ? `\\A node \\in Nodes: counters[node] < ${maximum}` : `counters \\in [Nodes -> 0..${maximum}]`}\n====`,
  config: configuration(count),
});

export const joint = (count: number): Input => ({
  name: `joint-${count}`,
  source: `---- MODULE Differential ----\nEXTENDS TLC\nCONSTANT Nodes\nVARIABLES first, second\nSymm == Permutations(Nodes)\nInit == /\\ first = [n \\in Nodes |-> FALSE] /\\ second = [n \\in Nodes |-> FALSE]\nNext == \\/ (\\E n \\in Nodes: /\\ first' = [first EXCEPT ![n] = ~@] /\\ UNCHANGED second)\n        \\/ (\\E n \\in Nodes: /\\ second' = [second EXCEPT ![n] = ~@] /\\ UNCHANGED first)\nSafe == /\\ first \\in [Nodes -> BOOLEAN] /\\ second \\in [Nodes -> BOOLEAN]\n====`,
  config: configuration(count),
});

export const plain = (input: Input): Input => ({
  ...input,
  name: `${input.name}-plain`,
  config: input.config.replace("\nSYMMETRY Symm", ""),
});

export const corpus = (): readonly Input[] => {
  const input = counter(3, 2);
  const pair = joint(3);
  const global = {
    ...pair,
    name: "joint-global",
    source: pair.source
      .replace("VARIABLES first, second", "VARIABLES first, second, phase")
      .replace("|-> FALSE]\nNext", "|-> FALSE] /\\ phase = FALSE\nNext")
      .replace("UNCHANGED second", "UNCHANGED <<second, phase>>")
      .replace(
        "UNCHANGED first)",
        "UNCHANGED <<first, phase>>)\n        \\/ (/\\ phase' = ~phase /\\ UNCHANGED <<first, second>>)",
      ),
  };
  return [
    ...[2, 3, 4].flatMap((count) =>
      [1, 2].flatMap((maximum) => [
        counter(count, maximum),
        counter(count, maximum, true),
      ]),
    ),
    joint(2),
    pair,
    global,
    {
      ...input,
      name: "initial-fault",
      source: input.source.replace(
        "Safe == counters \\in [Nodes -> 0..2]",
        "Safe == \\A node \\in Nodes: counters[node] > 0",
      ),
    },
    {
      ...input,
      name: "empty",
      source: input.source.replace(
        "Init == counters",
        "Init == FALSE /\\ counters",
      ),
    },
    {
      ...input,
      name: "deadlock",
      source: input.source
        .replace(
          "Next == \\E node \\in Nodes: counters'",
          "Next == FALSE /\\ counters'",
        )
        .replace(
          "[counters EXCEPT ![node] = IF @ < 2 THEN @ + 1 ELSE 0]",
          "counters",
        ),
    },
    {
      ...input,
      name: "boolean-functions",
      source: input.source
        .replace("|-> 0]", "|-> FALSE]")
        .replace("IF @ < 2 THEN @ + 1 ELSE 0", "~@")
        .replace("Nodes -> 0..2", "Nodes -> BOOLEAN"),
    },
  ];
};

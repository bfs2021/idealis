export interface Input {
  readonly name: string;
  readonly family: "value" | "graph" | "control";
  readonly source: string;
  readonly config: string;
  readonly seed?: number;
  readonly expectedRejection?: true;
  readonly goldenFrontendParity?: true;
}
export const seeds = { development: 0x15a1, confirmation: 0x15c3 } as const;
const value = (name: string, expression: string): Input => ({
  name,
  family: "value",
  ...(["string-len-supplementary", "string-tail-prefix"].includes(name)
    ? { expectedRejection: true }
    : {}),
  source: `---- MODULE Differential ----\nEXTENDS Integers, Sequences, FiniteSets\nVARIABLE x\nInit == x = (${expression})\nNext == UNCHANGED x\nProp == x # x\n====`,
  config: "INIT Init\nNEXT Next\nINVARIANT Prop\nCHECK_DEADLOCK FALSE",
});
const foldValue = (
  name: string,
  expression: string,
  definitions = "Op(a, b) == Append(a, b)",
): Input => ({
  name: `fold-${name}`,
  family: "value",
  goldenFrontendParity: true,
  source: `---- MODULE Differential ----\nEXTENDS Integers, Sequences, FiniteSets, Apalache\nVARIABLE x\n${definitions}\nInit == x = (${expression})\nNext == UNCHANGED x\nProp == x # x\n====`,
  config: "INIT Init\nNEXT Next\nINVARIANT Prop\nCHECK_DEADLOCK FALSE",
});
export const foldCases = (): readonly Input[] => [
  foldValue(
    "sequence-values",
    'ApaFoldSeqLeft(Op, <<>>, <<"z", [a |-> 1], "A">>)',
  ),

  foldValue("set-integers", "ApaFoldSet(Op, <<>>, {3, -2, 0, -7, 1})"),
  foldValue("set-booleans", "ApaFoldSet(Op, <<>>, {TRUE, FALSE})"),
  foldValue(
    "set-tuples",
    "ApaFoldSet(Op, <<>>, {<<2>>, <<1, 0>>, <<>>, <<1>>, <<0, 9>>, <<0, 1>>})",
  ),
  foldValue(
    "set-sets",
    "ApaFoldSet(Op, <<>>, {{2}, {1, 0}, {}, {-1}, {0}, {0, 2}})",
  ),
  foldValue(
    "sequence-left",
    "ApaFoldSeqLeft(Op, 20, <<3, 1, 4>>)",
    "Op(a, b) == a * 10 + b",
  ),
  foldValue("empty-set", "ApaFoldSet(Op, 7, {})", "Op(a, b) == Head(<<>>)"),
  foldValue(
    "empty-sequence",
    "ApaFoldSeqLeft(Op, 9, <<>>)",
    "Op(a, b) == Head(<<>>)",
  ),
  foldValue(
    "let-operator",
    "LET offset == 10 Op(a, b) == Append(a, b + offset) IN ApaFoldSet(Op, <<>>, {2, 1})",
    "",
  ),
  foldValue(
    "nested",
    "LET Op(a, b) == a + ApaFoldSeqLeft(Sub, b, <<1>>) IN ApaFoldSet(Op, 0, {3, 2})",
    "Sub(a, b) == a - b",
  ),
  foldValue(
    "bound-capture",
    "[n \\in {2, 3} |-> LET Op(a, b) == a + b + n IN ApaFoldSeqLeft(Op, 0, <<1>>)]",
    "",
  ),
  foldValue(
    "set-recursive-values",
    "ApaFoldSet(Op, <<>>, {<<{2, 1}, <<FALSE>>>>, <<{0}, <<TRUE>>>>})",
  ),
  foldValue(
    "sequence-functions",
    "ApaFoldSeqLeft(Op, <<>>, <<[i \\in {0} |-> 1], <<2>>, {FALSE}, TRUE, 3>>)",
  ),
  ...(["ApaFoldSet", "ApaFoldSeqLeft"] as const).map(
    (operator): Input => ({
      name: `fold-own-${operator}`,
      family: "value",
      goldenFrontendParity: true,
      source: `---- MODULE Differential ----\nVARIABLE x\n${operator}(a, b, c) == b\nInit == x = ${operator}(0, 7, {})\nNext == UNCHANGED x\nProp == x # x\n====`,
      config: "INIT Init\nNEXT Next\nINVARIANT Prop\nCHECK_DEADLOCK FALSE",
    }),
  ),
  {
    name: "fold-next-action",
    family: "graph",
    goldenFrontendParity: true,
    source:
      "---- MODULE Differential ----\nEXTENDS Integers, Apalache\nVARIABLE x\nOp(a, b) == a - b\nInit == x = 5\nNext == IF x = 5 THEN x' = ApaFoldSeqLeft(Op, x, <<2, 1>>) ELSE UNCHANGED x\nSafe == x > 0\n====",
    config: "INIT Init\nNEXT Next\nINVARIANT Safe",
  },
];
export const foldOrderControls = (): readonly Input[] => [
  ...(
    [
      ["string", '"only"'],
      ["record", "[field |-> 0]"],
      ["function", "[i \\in {0} |-> 0]"],
    ] as const
  ).flatMap(([kind, element]) =>
    (
      [
        ["direct", element],
        ["tuple", `<<${element}>>`],
        ["set", `{${element}}`],
      ] as const
    ).map(([container, member]) =>
      foldValue(
        `unsupported-${kind}-${container}`,
        `ApaFoldSet(Op, <<>>, {${member}})`,
      ),
    ),
  ),
  foldValue("set-mixed-kinds", 'ApaFoldSet(Op, <<>>, {TRUE, 0, "a"})'),
];
export const foldRejections = (): readonly Input[] => [
  ...(
    [
      ["lambda", "ApaFoldSet(LAMBDA a, b: a + b, 0, {1})", ""],
      ["arity", "ApaFoldSet(Op, 0, {1})", "Op(a) == a"],
      ["higher-arity", "ApaFoldSet(Op, 0, {1})", "Op(a, b, c) == a"],
      [
        "operator-parameter",
        "Higher(Op)",
        "Op(a, b) == a + b\nHigher(F(_, _)) == ApaFoldSet(F, 0, {1})",
      ],
      ["higher-order-body", "ApaFoldSet(Op, 0, {1})", "Op(F(_), b) == b"],
      [
        "recursive",
        "ApaFoldSet(Op, 0, {1})",
        "RECURSIVE Op(_, _)\nOp(a, b) == IF b = 0 THEN a ELSE Op(a, b - 1)",
      ],
      [
        "mutual-recursion",
        "ApaFoldSet(Op, 0, {1})",
        "Op(a, b) == Other(a, b)\nOther(a, b) == Op(a, b)",
      ],
      ["unknown", "ApaFoldSet(Missing, 0, {1})", ""],
      ["value-argument", "ApaFoldSet(1, 0, {1})", ""],
      ["other-apalache", "Guess({1})", ""],
    ] as const
  ).map(([name, expression, definitions]) =>
    foldValue(`reject-${name}`, expression, definitions),
  ),
  {
    ...foldValue(
      "reject-no-extends",
      "ApaFoldSet(Op, 0, {1})",
      "Op(a, b) == a + b",
    ),
    source:
      "---- MODULE Differential ----\nVARIABLE x\nOp(a, b) == a\nInit == x = ApaFoldSet(Op, 0, {1})\nNext == UNCHANGED x\n====",
    config: "INIT Init\nNEXT Next",
  },
];
export const boundaries = (): readonly Input[] => [
  ...foldCases(),
  ...["<<>>", "<<7>>", "<<7,8,9>>"].flatMap((sequence, size) =>
    [
      [-1, -1],
      [0, -1],
      [0, 0],
      [1, 0],
      [1, 1],
      [1, 3],
      [2, 1],
      [3, 4],
    ].map(([from, to], index) =>
      value(`subseq-${size}-${index}`, `SubSeq(${sequence},${from},${to})`),
    ),
  ),
  ...["TRUE", "FALSE", '"a"', "-1", "0", "1"].flatMap((entry, index) =>
    ["1..0", "0..0", "-1..1", "{}"].map((domain, range) =>
      value(`membership-${index}-${range}`, `${entry} \\in (${domain})`),
    ),
  ),
  ...(
    [
      ["subset-short-circuit", "{{}, {TRUE}} \\subseteq {{0}}"],
      ["subset-short-circuit-negated", "~({{}, {TRUE}} \\subseteq {{0}})"],
      ["subset-empty", "{} \\subseteq 0"],
      ["cap-empty", "{} \\cap 0"],
      ["difference-empty", "{} \\ 0"],
      ["function-empty-range-error", "<<>> \\in [{} -> Head(<<>>)]"],
      ["function-domain-range-error", "<<1>> \\in [2..3 -> Head(<<>>)]"],
      ["record-domain-range-error", "[a |-> 1] \\in [b: Head(<<>>)]"],
      ["record-later-field-error", "[a |-> 1] \\in [a: {}, b: Head(<<>>)]"],
      ["nested-seq-membership", "[a |-> <<0,1>>] \\in [a: Seq(0..1)]"],
      [
        "nested-function-membership",
        "[a |-> <<0,1>>] \\in [a: [1..2 -> 0..1]]",
      ],
      ["empty-seq-range", "<<>> \\in Seq({})"],
      ["seq-range-error", "<<>> \\in Seq(Head(<<>>))"],
      ["record-domain-mismatch", "[a |-> TRUE] \\in [b: {0}]"],
      ["function-domain-mismatch", "<<TRUE>> \\in [2..3 -> 0..1]"],
      ["record-equality-domain", "[a |-> TRUE] = [b |-> 1]"],
      [
        "record-equality-short-circuit",
        "[a |-> 0, b |-> TRUE] = [a |-> 1, b |-> 0]",
      ],
      ["except-missing-field", "[ [a |-> 0] EXCEPT !.b = Head(<<>>) ]"],
      [
        "except-nested-old",
        "[ <<[a |-> 0]>> EXCEPT ![1].a = @+1, ![1].a = @+1 ]",
      ],
      ["string-len-empty", 'Len("")'],
      ["string-len-ascii", 'Len("abc")'],
      ["string-len-supplementary", 'Len("😀")'],
      ["string-tail", 'Tail("abc")'],
      ["string-tail-prefix", 'Tail("a😀")'],
      ["string-tail-empty", 'Tail("")'],
      ["string-head-error", 'Head("abc")'],
      ["string-append-error", 'Append("abc","d")'],
      ["string-concat", '"ab" \\o "cd"'],
      ["string-subseq", 'SubSeq("abc",2,3)'],
      ["string-subseq-empty", 'SubSeq("abc",2,1)'],
      ["empty-quantifier", "\\A n \\in {}: Head(<<>>)"],
      ["empty-existential", "\\E n \\in {}: Head(<<>>)"],
      [
        "quantifier-short-circuit",
        "\\E n \\in 0..1: IF n = 0 THEN TRUE ELSE Head(<<>>)",
      ],
    ] as const
  ).map(([name, expression]) => value(name, expression)),
  {
    name: "case-first-true",
    goldenFrontendParity: true,
    family: "graph",
    source:
      "---- MODULE Differential ----\nEXTENDS Naturals\nVARIABLE x\nInit == x = CASE FALSE -> 5 [] TRUE -> 0 [] TRUE -> 9\nNext == CASE x = 0 -> x' = 1 [] x = 1 -> x' = 2 [] TRUE -> UNCHANGED x [] TRUE -> x' = 9\nSafe == x < 3\n====",
    config: "INIT Init\nNEXT Next\nINVARIANT Safe",
  },
  {
    name: "case-expression-no-match",
    goldenFrontendParity: true,
    family: "control",
    source:
      "---- MODULE Differential ----\nVARIABLE x\nInit == x = CASE FALSE -> 1 [] FALSE -> 2\nNext == UNCHANGED x\n====",
    config: "INIT Init\nNEXT Next",
  },
  {
    name: "case-action-no-match",
    goldenFrontendParity: true,
    family: "control",
    source:
      "---- MODULE Differential ----\nVARIABLE x\nInit == x = 0\nNext == CASE x = 5 -> x' = 1 [] x = 6 -> x' = 2\n====",
    config: "INIT Init\nNEXT Next",
  },
  {
    name: "case-unevaluated-branch",
    goldenFrontendParity: true,
    family: "control",
    source:
      "---- MODULE Differential ----\nVARIABLE x\nInit == x = IF TRUE THEN 0 ELSE CASE FALSE -> 1\nNext == UNCHANGED x\n====",
    config: "INIT Init\nNEXT Next",
  },
  {
    name: "case-invariant-empty-init",
    goldenFrontendParity: true,
    family: "control",
    source:
      "---- MODULE Differential ----\nVARIABLE x\nInit == x \\in {}\nNext == UNCHANGED x\nInv == CASE FALSE -> TRUE\n====",
    config: "INIT Init\nNEXT Next\nINVARIANT Inv",
  },
  ...(
    [
      ["case-action-other", "CASE FALSE -> x' = 9 [] OTHER -> UNCHANGED x"],
      [
        "case-action-short-circuit",
        "CASE TRUE -> UNCHANGED x [] (Head(<<>>) = 0) -> x' = 9",
      ],
      [
        "case-action-unevaluated-branch",
        "IF TRUE THEN UNCHANGED x ELSE CASE FALSE -> x' = 9",
      ],
    ] as const
  ).map(
    ([name, next]): Input => ({
      name,
      goldenFrontendParity: true,
      family: "control",
      source: `---- MODULE Differential ----\nEXTENDS Sequences\nVARIABLE x\nInit == x = 0\nNext == ${next}\n====`,
      config: "INIT Init\nNEXT Next",
    }),
  ),
  {
    name: "positive-control",
    family: "control",
    source:
      "---- MODULE Differential ----\nVARIABLE x\nInit == x = TRUE\nNext == UNCHANGED x\nProp == x\n====",
    config: "INIT Init\nNEXT Next\nINVARIANT Prop",
  },
  {
    name: "error-control",
    family: "control",
    source:
      "---- MODULE Differential ----\nEXTENDS Sequences\nVARIABLE x\nInit == x = Head(<<>>)\nNext == UNCHANGED x\n====",
    config: "INIT Init\nNEXT Next",
  },
];
export const graphs = (seed: number): readonly Input[] => {
  let state = seed;
  const pick = (maximum: number) => {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    return state % maximum;
  };
  return Array.from({ length: 32 }, (_, index) => {
    const nodes = 2 + pick(3);
    const edges = [
      [0, 1],
      ...Array.from({ length: pick(nodes * 2) + 1 }, () => [
        pick(nodes),
        pick(nodes),
      ]),
    ];
    const actions = edges
      .map(([from, to], edge) => `A${edge} == x = ${from} /\\ x' = ${to}`)
      .join("\n");
    const target = pick(nodes);
    const predicates = [
      `[](x < ${nodes})`,
      `<>(x = ${target})`,
      `[]<>(x = ${target})`,
      `<>[](x = ${target})`,
      `(x = 0) ~> (x = ${target})`,
      `<>[](x = 0) \\/ <>[](x = ${target})`,
      `~([]<>(x = ${target}))`,
      `([]<>(x = 0)) /\\ ([]<>(x = ${target}))`,
    ];
    const fairness = [
      "",
      " /\\ WF_x(Next)",
      " /\\ SF_x(A0)",
      " /\\ WF_x(Next) /\\ SF_x(A0)",
    ][Math.floor(index / 8)];
    return {
      name: `graph-${seed.toString(16)}-${index}`,
      family: "graph",
      seed,
      source: `---- MODULE Differential ----\nEXTENDS Naturals\nVARIABLE x\nInit == x \\in ${index % 3 === 0 ? "{0,1}" : "{0}"}\n${actions}\nNext == ${edges.map((_, edge) => `A${edge}`).join(" \\/ ")}\nSpec == Init /\\ [][Next]_x${fairness}\nProp == ${predicates[index % 8]}\n====`,
      config: `SPECIFICATION Spec\nPROPERTY Prop\nCHECK_DEADLOCK ${index % 2 === 0 ? "FALSE" : "TRUE"}`,
    };
  });
};
export const corpus = (
  phase: "development" | "confirmation",
): readonly Input[] => [
  ...(phase === "development" ? boundaries() : []),
  ...graphs(seeds[phase]),
];

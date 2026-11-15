# RustとWasmでPBT・差分テストを駆動する

`idealis-testing`は生成・縮小・再生の状態をRustで管理し、ホストへcaseの評価を要求する。TSでは`@mizchi/idealis/testing`から使う。SUTの初期化・操作・障害注入・観測は各言語のDriverが担当する。Node、テストランナー、形式仕様のbackendはコアの依存に含めない。

## TSから使う

次は、入力列に`bad`が含まれる既知の性質違反を探す例。`just package`で作ったarchiveを導入して実行する。

```ts
import { readFile } from "node:fs/promises";
import { createTestingWasm } from "@mizchi/idealis/testing";

const runtime = await createTestingWasm(
  await readFile(new URL(import.meta.resolve("@mizchi/idealis/testing.wasm"))),
);
const result = await runtime.property(
  {
    generator: {
      id: "example",
      version: "1",
      initial: ["s"],
      states: {
        s: [
          { action: "good", target: "s" },
          { action: "bad", target: "s" },
        ],
      },
      minLength: 0,
      maxLength: 8,
    },
    seed: 0x36a1,
    cases: 32,
    maxAttempts: 64,
    maxShrinkChecks: 64,
    maxWork: 10_000,
  },
  (testCase) =>
    testCase.actions.includes("bad")
      ? { status: "mismatch", signature: "contains-bad" }
      : { status: "matched" },
);
console.log(result.status, result.counterexample?.actions);
```

結果はfailed、縮小したactionsは`["bad"]`になる。Vitestやnode:testではこの結果をassertする。ブラウザでは同梱Wasmを静的ファイルとして配り、fetchしたUint8Arrayを`createTestingWasm`へ渡す。ホストimportとcross-origin isolationは不要。

実装との照合では、callbackから既存の`checkConformance(model, createDriver(), actions)`を呼ぶ。mismatchには原因を表すsignatureを付け、matched/invalidはそのまま返す。各caseで新しいDriverを作り、縮小候補でも初期化から実行する。[状態付きカウンタの接続例](../evals/testing-core/driver.ts)と[生成器](../evals/testing-core/fixture.ts)を参照。

## 結果と縮小

| status       | 意味                                          |
| ------------ | --------------------------------------------- |
| passed       | 要求した数の有効な正常caseを実行した          |
| failed       | 同じsignatureの失敗を最後に再現した           |
| exhausted    | attemptsを使い切り、有効case数が足りない      |
| inconclusive | coreのworkやホストの評価予算が不足した        |
| error        | 実行error、または確認時に失敗が再現しなかった |
| not-failing  | 明示的に縮小した初期caseが失敗していなかった  |

passedは全入力や全実行の証明ではない。縮小のminimalは、有効な単一削除で同じsignatureを保持できない意味。unknownの候補や予算切れがある場合はboundedになる。scalar値の縮小と任意の高階Strategyは未対応。

workは生成と有限状態機械による候補検査を数える。SUTの実行時間やJSON処理を含めない。cases/attempts/shrink checksも明示してホスト呼出回数を制限する。I/Oの中断はホストで管理する。callbackがthrowすると同じ例外を伝播し、内部sessionをcancelする。

## ケースの保存と再生

Caseにはgenerator ID/version、乱数アルゴリズム、seed/index、具体的な全actionsを保存する。再生は保存した列を使い、seedで再生成しない。

```ts
if (result.counterexample && result.signature) {
  const replay = await runtime.replay(
    result.counterexample,
    evaluate,
    result.signature,
  );
}
```

ここでevaluateはpropertyに渡した評価callback。signatureを指定して再現しなければerrorになる。正常版への対照再生はsignatureを指定せず、passedを要求する。モデル・実装・生成器のsource versionはcaseと一緒にmanifestへ保存する。

`runtime.shrink(generator, testCase, evaluate, { maxChecks, maxWork })`で保存caseを単独で縮小できる。caseのgenerator ID/versionと、渡した生成器が一致する必要がある。

## 複数実装の比較

`runtime.compare(oracle, targets)`へreferenceの許可値集合と各実装の観測を渡す。対象を多数決でreferenceにしない。invalid/unknown/errorや空の許可集合をmatchedにしない。対象ごとのoutcomeも残すため、全体のmismatchだけを見て他の対象も評価完了したとは扱わない。

JSONでは配列順を保持し、objectのfield順を無視する。値はplain JSONとsafe integerに限定する。集合、map、任意精度整数等は既存のPortableValueへ正規化してから比較する。

## nativeの接続と並列実行

Rustは`Session::property/shrink/replay`を作り、Evaluateをホストで実行して`resume`へ返す。[単独Rust consumer](../crates/idealis-testing/examples/counter.rs)を参照。native binaryは同じJSONL protocolを提供する。TSの別transportは`createTestingRuntime(dispatch)`へ接続できる。[native接続例](../evals/testing-core/clients.ts)を参照。

1 runtimeは1つのproperty/shrink/replay sessionを所有する。並列workerには別runtimeを作る。比較は進行中のsessionと独立して呼べる。既存の`@mizchi/idealis/property`はfast-checkを使う経路として残す。Strategyやseed/pathは新APIと互換ではない。

```sh
just testing-core-test
just testing-test
just testing-browser
just testing-example
just package
```

採用評価は[0036](../evals/iterations/0036-testing-core.md)。SAT/CNF、並行スケジュール探索、構造fuzzingはこの契約を基準に別の評価で追加する。

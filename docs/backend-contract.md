# 検査backendの契約

`@mizchi/idealis/backend`は、検査方式ごとの保証と対象を結びつける共通契約。`modelt.backend-request` / `modelt.backend-result` v1のJSONとして扱える。Node、test runner、solverをコアへ持ち込まず、実行はadapterへ委ねる。

## 結果が表すもの

| outcome.kind                | 保証・診断                                                                       |
| --------------------------- | -------------------------------------------------------------------------------- |
| `explicit-complete`         | 指定した有限モデルの到達グラフの検査が完了。初期集合は非空                       |
| `bounded-no-counterexample` | 非空の初期集合から、指定したk step以下の範囲で反例なし                           |
| `obligation-holds`          | 帰納性の個別obligationが成立。単独では仕様全体の保証ではない                     |
| `inductive`                 | 同じ対象・意味論・backend版の必要obligationがそろい、非空性も確認済み            |
| `counterexample`            | 元のモデルのbehavior trace、または個別obligationのassignment                     |
| `empty`                     | 初期集合が空。合格として扱わない                                                 |
| `inconclusive`              | 資源切れ、solver unknown、cancel、未対応、初期集合の非空性未確認、obligation不足 |
| `error`                     | 評価やbackendのエラー。code/messageを保持                                        |

rtlcの`bounded`は`inconclusive`へ写す。depthで打ち切られた場合も、genericなBMCの「k以下をすべて検査済み」へ昇格しない。観測したdepth/states/generated/workは`progress`で保持し、coverageとは分ける。BMCのkはNext遷移の回数で、lassoは閉じるedgeも数える。

非空性の`initialization`は、元モデルのInitに対する`nonempty` / `empty` / `unknown`。単なるSMT unsatやApalacheのNoErrorだけから`nonempty`を推定しない。初期集合の充足可能性検査や、初期条件を満たす具体的なwitnessなどが必要。fairnessを満たす無限behaviorの存在まで保証するfieldではない。

## 対象と意味論を固定する

scopeは次を持つ。

- `modelId`: source、設定、パラメータ、依存を特定するID
- `propertyId`: 選択した性質を特定するID。複数の性質をまとめる場合はその集合全体のID
- `semantics`: 整数幅・値の等価性などの意味論profile
- `assumptions`: 環境仮定のID一覧

実際の内容からdigest等を作り、同じIDを異なる内容へ再利用しない。runnerはIDとqueryの一致を確認するが、IDからsourceを復元したり、その対応を暗号学的に認証したりはしない。backendの正当性やSMT encodingの正しさを検証するproof certificateでもない。

`backend.id/version`も結果へ残す。`createRtlcBackend`のprofileは`tlc-1.7.4-int32`。versionは呼び出し側が配布manifestと一致させる。Wasm ABI versionからcrateのversionを推定しない。帰納性の集約ではmodel/property/assumptionsの順序/意味論/invariant/backend版が異なる結果を拒否する。

## 実行する

```ts
import { createRtlcBackend, runBackend } from "@mizchi/idealis/backend";

const backend = createRtlcBackend(core, { version: "0.1.0-alpha.0" });
const result = await runBackend(backend, {
  format: "modelt.backend-request",
  version: 1,
  scope: {
    modelId: modelDigest,
    propertyId: propertyDigest,
    semantics: "tlc-1.7.4-int32",
    assumptions: [],
  },
  query: {
    kind: "explicit",
    bounds: { maxDepth: 100, maxStates: 100_000, maxWork: 10_000_000 },
  },
  input: program,
});
```

`core`は既存のPortableCore、`program`はSafetyProgram。既存のnative/Wasm protocolは変更しない。rtlc adapterは`check-safety`を使う。既存の並列APIもそのまま利用できるが、このfactoryからworker数を指定する機能はない。

`runBackend`はdescriptorのquery/profileを確認し、対応しない要求には実行前に`inconclusive/unsupported`を返す。対象のmetadataと入力dataをsnapshotにし、応答が別のengine・scope・queryを名乗る場合は`BackendContractError`を投げる。入力はstructured-clone可能なdataにする。各adapterがinputのschemaとsolverへの変換を担当する。

`parseBackendResult(unknown)`はwireからtyped resultへ変換する。未知のversion/field、矛盾したqueryと保証、空trace、不正なloop等を拒否する。ID/versionはcallerとproviderの申告であり、この検査は結果の意味の取り違えを防ぐ境界として使う。

評価エラーは`error`、不正なbackend契約は例外として区別する。rtlc adapterは既存のPortableCoreErrorのcode/messageを残し、ASSUME違反をbehaviorの反例へ変換しない。予期しないhost/transport例外はPromiseをrejectする。timeoutやretryは自動で追加しない。

## 帰納性をまとめる

帰納性のqueryは`kind: "induction"`と`invariantId`を持つ。obligationは次の3種類。

1. `init`: `Init => IndInv`。
2. `step`: `IndInv /\ Next => IndInv'`。
3. `implication`: `IndInv => Property`。

```ts
import { completeInduction } from "@mizchi/idealis/backend";

const result = completeInduction([initResult, stepResult, implicationResult]);
```

`invariantId === propertyId`なら含意は同一formulaなのでinit/stepだけでよい。異なるIDならimplicationを要求する。baseの非空性も必要。欠落、unknown、error、反例があれば`inconclusive/obligations`を返し、個々の結果を保持する。modelのsource/configに含まれない任意のパラメータへ一般化しない。

成立した個別queryは`obligation-holds`として渡す。元モデルのInitではなく候補IndInvから始めたstepの反例は、到達しない状態を含む可能性がある。そのため`witness.kind: "obligation"`のassignmentを使い、元モデルから到達する`behavior` traceと区別する。実SUTのoracleへそのまま渡さない。

集約結果のqueryは`obligation: "all"`。`inductive`には検査した個別結果を保持する。根拠となるqueryと個々の診断を捨てて、真偽値だけの成功判定へまとめない。

## adapterの範囲と検証

`VerificationBackend<Input>`はdescriptorとasync checkの境界。Node process、RPC、Wasm等の実装を外側に置ける。`runBackend`を経由してquery/profileと応答を検査する。新しいfrontendがSMTを生成する場合は、初期条件・全step・性質の否定を正しく表現し、k以下の全範囲を検査する責任を持つ。

今回のpublic adapterはrtlcだけ。Apalacheの汎用process/RPC adapter、SMT encoding、ITF importerは含まない。[0027の保存した実応答](../evals/runs/0027-apalache-pilot/result.json)を契約へ載せる検査を[0028](../evals/iterations/0028-backend-contract.md)に置く。保存したpilotでの初期集合の非空性は具体的な列挙で確認したもので、汎用のNoError変換ではない。

公開entryは`@mizchi/idealis/backend`に限定した。共通型、runner、wire decoder、帰納性集約、rtlc bridgeを提供し、既存root APIと結果型を維持する。

```sh
just backend-test
just backend-eval evals/runs/<新規ID>
just check
just browser
just package
```

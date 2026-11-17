# workspaceの差分検査とチューニング

既存evalの差分assertionを、idealis-oracleのRust照合器へ通す。Node adapterはWasmを1 instanceだけ遅延ロードする。[移行一覧](migrations.json)には23ファイル・99箇所を記録した。parserの正規化、Z3のSAT witness検証、TLCの分類と強制trace検証は各adapterに残している。TS/Rust/Goの共通SUTと非同期履歴も同じassertionを使う。metadataの検査や単独の期待値テスト、過去のraw artifactは一括変換していない。

## 差分テストに使う

```ts
import { assertOracle, compareOracle } from "../workspace-oracle/node.js";

assertOracle(actual, reference, "native/Wasm reply");
const checked = compareOracle(actual, reference);
```

assertOracleは不一致でNodeのAssertionErrorを投げ、reportも保持する。入力が表す操作とモデルの観測を渡したい場合は、wasm.tsのbatch requestへcaseのinputを指定する。共通assertionのcase inputは診断用labelで、操作列を自動収集する機能ではない。

snapshotはplain object・dense arrayとscalarを型付きで符号化する。undefined、bigint、NaN、Infinity、負のzeroを区別し、objectのkey順を比較しない。循環、class instance、Map/Set、symbol、accessor、非列挙property、疎なarrayは明示的に拒否する。深さは48まで。JSの任意objectグラフを輸送する契約ではない。

ここでの応答一致はprotocol回帰の意味。意図したエラーやunknownを含む応答snapshotの一致を、モデルの安全性やSATの成功にしない。新しい実装の意味論を検証するには、独立したオラクルとdomainの判定を使う。

## 変更前後を保存して比較する

Rustとwasm32-unknown-unknown、Node 24+、pnpm、justが必要。

```sh
just workspace-oracle-test
just workspace-snapshot .artifacts/tuning-before
# コードを修正し、対象crateをrelease buildして回帰を確認する
just workspace-snapshot .artifacts/tuning-after
just workspace-bench .artifacts/tuning-before .artifacts/tuning-after evals/runs/my-tuning
just workspace-replay evals/runs/my-tuning evals/runs/my-tuning-replay
```

snapshotは6実行境界のbinaryとsourceを別directoryへコピーし、hash・revision・環境・caseを保存する。coreの境界はrtla/rtla-compiler/rtlcを含む。caseのrequest・ID・受理条件が異なるsnapshotは比較しない。baselineとcandidateのbinaryが同じsnapshot同士も、測定器の正常対照として実行できる。

13 casesをcold/reusedで分け、12 groupで実行する。referenceはbaseline専用の別process、速度baselineもbaselineを明示する。coldは毎回起動し、reusedは3つの独立した常駐JSONL processを使う。起動やIPC、JSONの費用を含む。7 samples・1 warmup・3 iterationsで、全呼出結果を保存したreferenceへ照合する。受理条件はunknown/boundedや意図しないエラーを拒否する。

異常終了、応答欠落、不正JSON、途中の結果変化を成功にしない。最後のJSONを返してから異常終了した場合や余分なstdoutが残る場合も、process終了時に検査して全速度比を無効化する。自動timeoutは設けていない。任意の外部programの停止は呼び出し側が管理する。statefulなhandle/sessionを含める場合は、その初期化・解放を独立したcaseへ設計する。

runのinput、preflight、raw ns、順序、統計、source/binary hashを保存する。replayは各targetを保存referenceへ照合し、reference processは呼ばない。上書きを防ぐため、新しいdirectoryを指定する。sourceの異なる変更前後は許可し、snapshot内のsource/binaryの改変は拒否する。

## bridge費用とkernelの最適化を分ける

```sh
just workspace-oracle-cost evals/runs/my-oracle-cost
just workspace-oracle-cost-replay evals/runs/my-oracle-cost evals/runs/my-oracle-cost-replay
```

従来のNode strict equalityと新しいsnapshot/Wasm照合を、同じJSONL runnerで比較する。実parser/coreの応答を使い、7 samples・1 warmup・3 iterationsで測る。Nodeの常駐process、IPCとJSONを含むため、Wasmの純粋な関数時間とは呼ばない。

検査kernelの改善は既存のCriterion/profileを使う。このrunnerは、その変更前後の応答が保たれるかと、呼び出し境界全体の費用を確認する。測定中はbuild、test、別のsolver/profileを並走させない。

既存GoのEOF入力型CLIは、そのまま常駐JSONLとして使えない。実装比較はcross/asyncの既存runnerで行い、汎用process runnerへ接続する場合はcoldまたはJSONL adapterを使う。

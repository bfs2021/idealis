# 0025: TLC互換backendをrtlc crateへ分離する

有限TLA+ fragmentの検査を[`rtlc`](../../crates/rtlc/README.md)へ分離した。Rustの呼び出し側は`modelt-core`に依存せず、型付きIRまたはJSONから検査できる。探索アルゴリズムは変更していない。今回の判断はライブラリ境界と既存挙動の維持であり、速度改善の判定ではない。

## crateの境界

[計画](../plans/0025-rtlc-extraction.md)を先に固定した。`rtlc::ir`に式・遷移・検査条件・結果の契約を置いた。以下の実装とCriterion14本を`modelt-core`から移した。

- 値評価とintern table
- BFS、invariant/deadlock検査、最短safety反例
- temporal property、WF/SF、SCC検査
- nativeの層同期並列探索とwork予算

Arenaと探索・worker内部はprivate。`Value`と`CoreError`は`rtlc`が定義し、`modelt-core`は既存の5型を再exportする。型の変換やcodecの複製は追加していない。`modelt.safety-program` v1、JSONLとWasm ABIは維持する。

TLA+解析はTypeScript frontendに残る。実装照合、有限表、非同期履歴、host、縮小とWasm/JSONL adapterは`modelt-core`に残る。`rtlc`はTLC CLIやTLA+全体の互換実装ではない。[対応範囲](../../docs/rust-safety.md)を参照。

## 分離前後の結果

分離前のnative/Wasm binaryを保存し、同じ241入力で比較した。nativeは1/2/4/8 workerの964件、Wasmは逐次241件を実行した。[入力と全結果](../runs/0025-rtlc-parity/result.json)、[source/binary hash](../runs/0025-rtlc-parity/manifest.json)を保存している。

通常の応答はworkを含めて完全一致した。並列のwork予算切れは実行順で探索済みprefixが変わるため、`bounded`、理由`work`、予算内、違反なしを比較し、両方の応答を保存した。空の初期集合、評価エラー、予算切れを`complete`へ置き換えていない。これは保存した入力での回帰検査であり、全IR入力の同値性証明ではない。

## 単独配布と検証

`just package`は一時workspaceで両crateをpackage検証する。展開した`.crate`だけで`rtlc`単独consumerと`modelt-core`経由のconsumerをoffline実行した。単独consumerの依存graphに`modelt-core`がないことも確認した。registryへのpublishは実行していない。

型付きIR、再exportの型同一性、エラー伝播のテストを追加した。crate作成前のRedと移動後のGreen、native/Wasm build、fmt/clippy、既存テスト、TLC正常/故障対照、ブラウザ、配布consumerのログを[検証記録](../runs/0025-rtlc-verification/result.json)へ保存した。移動後のCriterion14本はsmoke実行だけを行い、新しい速度比較には数えない。

crateの分離を採用する。今後のprovenanceは`crates/rtlc`もhash対象に含め、既存のrunと測定値は変更しない。

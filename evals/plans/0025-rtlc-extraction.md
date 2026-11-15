# 0025: TLC互換backendをrtlc crateへ分離する

TLA+の対応fragmentを実行するIR、値評価、BFS、invariant/deadlock/temporal/fairness検査、native並列化をcrates/rtlcへ移す。アルゴリズムとwire schemaを変えず、modelt-coreはrtlcへ依存し既存のSafetyProgram/Bounds/Result、Value/CoreErrorを再exportする。rtlcからmodelt-coreへ依存しない。TLA+のTS frontend、実装照合、有限表、履歴、host、縮小、Wasm/JSONL adapterはmodelt側に残す。

契約はrtlc::irとして公開し、実行・Arena・worker内部はprivateにする。Value/CoreErrorはrtlcに一本化し、二重のcodecや相互変換を作らない。既存のmodelt.safety-program v1を保持する。TLC CLIやTLA+全体の互換性まで実装したと解釈しない。

## TDDと検証

先にrtlcのみを利用するtyped IRテストとmodelt側の型同一性/委譲テストを追加し、crateの欠落をRedとして保存する。移動後はrtlc単独のnative/Wasm build、全target、fmt/clippy、既存のRust unit/parallel対照、TS既存ABI、241件の逐次/並列対照、TLC正常/故障、ブラウザを確認する。分離前binaryと同じ入力の結果も比較する。cut/empty/errorをcompleteにしない。

Criterion14本とfixtureはrtlcへ移す。justタスクとprovenanceを更新し、今後の測定が新crateをhash対象から落とさないようにする。速度の新しい主張はしない。

## 配布

modelt-core/rtlcは独立manifestとし、既存native/Wasm targetの場所を維持する。package時に一時workspaceで両crateを同時検証する。Cargoの一時registryによる未公開path依存のpackage検証を小さいprobeで確認済み。新しい.crateを同梱し、展開したartifactだけでrtlc単独consumerとmodelt既存consumerをoffline実行する。registryへの初回publishは行わない。

just check/browser、portable/runtime、safety/frontier、TLC、fmt/clippy、evaluate/replay、package consumerを確認する。結果と採否はevalsへ残す。

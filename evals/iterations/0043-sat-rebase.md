# 0043: SAT評価ブランチのmainへの統合確認

PR #45のsat-foundationをmain 9097632へrebaseした。.gitignore、biome.json、justfile、evals/README.md、ledger.jsonlは双方の追加を残した。mainの既存設定・task・CIを削らず、SATの項目を追加した。

[統合確認の計画](../plans/0043-sat-rebase.md)に従い、元の07f2faaと比較した。SATの実装・評価器、0037の計画・結果・保存run・browser設定は同じgit blobを保つ。公開core、adapter、配布設定の変更はない。

## 保存caseの回帰再生

0037の開発64入力・183 roundと確認128入力・375 roundを再照合した。native/Wasm、独立TS全代入列挙の判定、モデル、エラー・予算未対応の対照は保存済みの全204記録と一致した。既知seed 0x37a1/0x37c3の再生で、新しい独立実験には数えない。

nativeのSHA-256は`2f67c2a5d0efb14eaa85559b5512afc6d23ec6c48ffc53f4a05d36d862567639`、Wasmは`5ac9d87c4d71e26fa74c0b345d1a28900fd2fde15609e9008b33cee72554ba15`で、いずれも0037の保存値と一致した。Wasmのhost importは0件。実Chromiumの回帰検査も通過した。

全体設定が変わったため、統合後のsource fingerprintは0037と異なる。過去runのsourceVersionや結果は書き換えていない。0037の厳密なsat-replayは元のソースとbinaryを要求する。今回は同じ評価器を現在の構成で実行し、保存入力と全応答を比較する別の回帰確認として記録した。

## 検証と範囲

`just sat-test`、`sat-browser`、`check`、`browser`、`runtime-build`、`runtime-test`、`evaluate`、`replay-evaluation`が通過した。[統合検証run](../runs/0043-sat-rebase-verification/result.json)にsource/binary hash、保存caseの再生記録とログを保存した。

Varisatのwork-budget未対応と、実行バジェットを保証するbackendへの採用保留は維持する。今回の変更は競合解消と統合確認で、SAT機能や公開APIの拡張ではない。

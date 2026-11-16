# 0043: SAT評価ブランチのmainへの統合確認

sat-foundationの4コミットをmain 9097632へrebaseする。.gitignore、biome.json、justfile、evals/README.md、ledger.jsonlの競合は双方の追加を残す。SATの実装・評価器と0037の保存runは元の07f2faaと同じgit blobであることを確認する。既存mainの設定・task・CIを削らず、SATの項目を追加する。

SAT固有のfmt/clippy/Rust/node tests、実Chromium、just check/browser、runtime-build/test、evaluate/replay-evaluationを実行する。0037の開発・確認runの保存入力と全応答を、現在のnative/Wasmと独立TS全代入列挙で再照合する。既知seed 0x37a1/0x37c3の回帰再生として扱い、新しい独立実験には数えない。過去のrunやsourceVersionを書き換えない。

全体設定が変わるため0037の厳密なsource fingerprintとは区別する。新しいverification runに統合後のsource/binary hash、保存runとの照合、実行ログを残す。既存のwork-budget未対応と公開backendへの採用保留を維持する。PR #45を更新してCIを再実行し、この作業ではmergeしない。

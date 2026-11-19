# 展開cacheを明示的なフラグで有効にする

状態別Next cacheは既定無効にする。小さいモデルで固定費が上回り、Rust全体の機械証明も対象にしていないため、検証済みの重いNextへ明示的に適用する。

TS/JSONLは`enabled: true`、Rustは`ExpansionOptions { enabled: true, build_digest, max_entries }`で有効化する。flag省略・falseと`ExpansionOptions::default()`は通常の`check`へ委譲する。cacheの展開を使用・更新せず、今回のcache計数は0で返す。保存済みentryがあっても自動有効化しない。

Redでは、flagなしの呼び出しが古いcacheを再利用し、通常探索のworkと一致しないことを確認した。Rustのdefault options回帰と、TS/Wasmの保存済み・digest破損cacheを渡す回帰を加え、無指定時の通常探索とcache保持を検査した。利用する既存テスト・評価・benchmark・browser・配布consumerは有効化を明示する。

Leanへ無効時の再利用拒否を1定理追加し、合計27定理を監査・再検査した。有効化の有無を含む32条件を実Rust/Wasmで作り、参照機と一致した。人工故障対照は有効化guardの除去を加えて7件。旧policyの23,606遷移の比較も維持する。

共有10テスト、通常のcheck、Chromium 7件、Rust all-targets/clippyと配布consumerが成功した。Jevにはdefault-offに関する2比較を追加し、全7 crateの39比較を同期した。[入力・証明・ログ](../runs/cache-default-off/evidence.json)を保存する。

性能数値は[直前の計測](cached-expansions.md)のsourceに対応する。今回のflag追加でCriterionの再測定は行わず、過去のrunを更新していない。現在の再利用契約は[APIと証明範囲](../../docs/safety-expansions.md)を参照。

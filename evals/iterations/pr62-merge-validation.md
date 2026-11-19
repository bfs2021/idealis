# PR 62のマージ前検証

対象は`operator-sharing`の35f66b321c68ed348d6ed4fd89f337192fd78f8dと、base d147550c80d814e23f5d76f8ded25e30bf190da4。[先行計画](../plans/pr62-merge-validation.md)に従い、後続のfrontend-semanticsから分けたworktreeで検証した。

## 修正

最新コミットにRustのtarget生成物78,753件が混入していた。indexから除き、全targetをignoreする。CIは追跡されたtargetがあれば失敗する。履歴の書換えは行わない。

既存の解析4,236テストをjustとCIへ接続した。レビュー修正で追加されたframed atom・展開上限のテストも、通常のsafety回帰から実行する。Jevの比較は大きすぎるexcerptを限定し、定数宣言のcontextとSUPPORTがリンクする根拠をsnapshotへ含める。これにより、リンク先を欠いた自己テストの失敗も解消する。

## tuple展開

深さ6・幅16のtuple別名を使い、SPECIFICATIONのsubscriptと空tupleのUNCHANGEDを検査した。修正前はTS・Rustとも、512 MiBの監視上限で停止した。小さい対照は終了した。停止を検査成功と数えない。

target解決と共有IRからの変数読取りを、それぞれ524,288ノードで制限する。深さやleaf数に加えて数えるため、空tupleの複製でも展開を止める。caller frameとactualの意味は保ち、値や結果のmemoizationを追加しない。上限はcompile時のprotocol拒否で、実行時のworkや厳密なRSS上限ではない。

最初の上限1,000,000では12ケース中11件が通り、TSのsubscriptだけが256 MiBの監視を超えた。そのログを残し、上限を524,288へ変更した。監視条件を緩めず、最終12件はnative/Wasm/TSで成功した。既存のtuple chain 16も受理する。

## TLCとの照合

ローカルのtlaplus/tlaplusにあるSpecProcessorのSubscriptCollectorとToolのUNCHANGED処理を読んだ。固定TLCで、stateに依存するtuple subscriptとinlineの空tupleについて、正常版・遷移後の意図したinvariant故障を比較した。4入力と3経路の12比較で、判定・状態数・生成数が一致した。人工故障を未知の実バグ発見とは数えない。

最初のdriverは初期invariant違反にもTLCの計数があると仮定して失敗した。TLCが計数を出さないため、遷移後の故障を別runにした。次のdriverでは定数の空tuple別名をUNCHANGEDへ渡すと、TLCがidentifier undefinedのmodel errorを返した。これは一致と数えない。inlineの空tupleによる別の対照も保存する。大きすぎる入力のprotocol拒否もTLC一致には含めない。

## ローカル検証

| 対象                                   | 結果                                           |
| -------------------------------------- | ---------------------------------------------- |
| 追加した幅の広いtuple                  | 12件成功。修正前は6成功・6失敗                 |
| safety回帰                             | 1,300件成功                                    |
| 両frontendの解析                       | 4,236件成功                                    |
| Rust fmt/clippy・全target・native/Wasm | portable-test成功。direct crateの追加2件も成功 |
| Chromium                               | portable/parser 2件、通常browser 3件成功       |
| mutation                               | 95.4%。未検出も元のreportに保持                |
| 配布consumer                           | npm・Cargo・Wasm・CLIの検証成功                |
| 評価と再生、改善ループと再生           | 成功                                           |
| Jev                                    | live/offlineで全比較に回答。自己テスト8件成功  |

target生成物の削除をcommitする前のunit検査では、git statusの大きすぎる出力でENOBUFSになった。削除後のunit検査は成功した。Jevでは定数宣言と対象行のcontextを追加する前に1 contradictionがあり、比較条件を明記した後の再検査は0となった。失敗ログを保持する。

元PRのWasm性能測定にあるinconclusiveは変更しない。今回の資源上限テストを、一般的な高速化や非劣性の新しい証明には数えない。

[入力・rawログ・判定の保存索引](../runs/pr62-merge-validation/evidence.json)を参照。GitHub上のCIと最終headは[PR 62のchecks](https://github.com/mizchi/idealis/pull/62/checks)で確認する。

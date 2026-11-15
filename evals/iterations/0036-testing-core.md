# 0036: RustでPBT・差分比較・縮小・再生を駆動する

`idealis-testing`を独立crateとして追加し、有限生成器、削除縮小、保存caseの再生、許可値集合との差分比較をRustで管理する。SUTのI/Oは各言語のDriverが担当する。TSには`@mizchi/idealis/testing`とWasmを公開した。[事前計画](../plans/0036-testing-core.md)と[利用ガイド](../../docs/testing.md)を参照。

## 同一corpusでの照合

既存durable counterの正常版と永続化を欠く既知故障版を使った。各seedで正常版には16有効caseを要求し、故障版の失敗は縮小後に両版で再現した。

| run                                                   | seed数 | 正常run通過 | 既知故障の検出・再現 | native request/reply数 |
| ----------------------------------------------------- | -----: | ----------: | -------------------: | ---------------------: |
| [開発](../runs/0036-testing-development/result.json)  |     16 |       16/16 |                16/16 |                    745 |
| [確認](../runs/0036-testing-confirmation/result.json) |     32 |       32/32 |                32/32 |                   1511 |

正常版では開発256 case・確認512 caseが通過した。全runで、native/Wasmの全trialと最終結果、TS参照生成器の操作列、既存TS縮小器の結果とcheck数が一致した。複数実装の観測比較も既存TSのcheckDifferentialと一致した。故障版の反例は全48 seedで3操作へ縮小され、正常版への再生は通過した。

開発seedは0x36a1から16件、確認は0x36c3から32件。sourceVersionは`abf6736b4c094ed0cb4ed698a95dd69a55248793d13cc2d59550f74600aa6393`で共通。manifestと圧縮した全trial・全native輸送記録を保存し、同じソース・binaryで全記録を再現した。既知の人工故障で、TSに対する追加検出や未知バグ発見を示す結果ではない。

## 失敗・予算・protocolの検査

Rust APIを先に失敗させて実装した。12 testsで、有限状態機械の前提、異なるfailure signature、非再現、invalidだけの生成、unknown/error、work切れ、差分の正常対照と壊れたschemaを検査した。

unknownを経由して空列が失敗する場合、候補がない状態を縮小完了へ遷移させず、同じcaseを評価し続ける誤りがあった。決定的な回帰テストで失敗させ、候補がない場合は確認phaseへ移るよう修正した。また、serdeのinternally tagged enumのunit variantが追加fieldを無視したため、空struct variantへ変更し、不正feedbackがpending要求を消費しないことを確認した。

minimalは有効な単一削除について同じsignatureを保持できない意味に限定する。unknown候補と予算切れはbounded、無効な生成だけならexhausted、work切れならinconclusive。合格へ置き換えない。

## ブラウザと配布

同じ3件の契約をnode:testとVitestで実行した。実Chromiumで正常版・故障版、3操作への縮小、両版への再生、work切れを検査した。ホストimportとcross-origin isolationは不要。npm/Cargoの別consumerで公開export、Wasm、型、生成・縮小・再生・比較と依存graphの独立性を確認した。

check/browser、runtime-build/test、usable-test/eval/replay、evaluate/replay、既存mutation、packageとrelease metadata dry runを実行した。既存mutationの95.4%は新しいRust coreのmutation強度を測った結果ではない。[検証run](../runs/0036-testing-verification/result.json)へログと配布物のhashを保存する。

## 採用と次の評価

有限生成器を使うPBT・差分比較・再生の共通契約として採用する。native/Wasm/TSで同じcaseと縮小規則を使えることが今回の成果。fast-checkの任意Strategyやseed/pathの互換、scalar縮小、内部threadの探索は含めない。実行時間は全比較を含む観測値で、言語間の速度比較には使わない。

次はSAT/CNFの候補について、native/Wasm、停止予算、SATの具体値とUNSATの照合、依存と配布サイズを評価する。その後に並行実行の制御、構造fuzzingを同じcase・判定・再生の契約へ接続する。BDDは状態集合の圧縮が必要なモデルを選んでから評価する。

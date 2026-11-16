# 0045: crateの対応チェックリストとJevの同期検査

[計画](../plans/0045-crate-checklists.md)に従い、7 crateへSUPPORT.mdを追加した。既存のSMTチェックリストはidealis-smtへ移し、旧URLは案内として残した。チェック済みは記載条件内の対応、未チェックは未対応・対象外とし、一般の互換性や実装予定を約束しない。

jev-lint 0.8.0のdocuments APIとjev-1.14.0を固定し、README対チェックリストと、宣言した実装境界の28比較を作った。Rustのsource/test、Cargo.toml/lock、README、SUPPORT.md等のdigestを記録へ結ぶ。コミット前は実Git indexのblobからsnapshotを作り、同じ判定をAPIキーなしで再生する。新しいcrate/source、欠落、古い判定、未回答、矛盾を成功へ変換しない。

[最初の試行](../runs/0045-support-development/result.json)ではrtlcのIRとralloyの固定instanceが文脈不足になった。値・探索・instanceの型定義を追加した[次の試行](../runs/0045-support-context/result.json)ではralloyのparser比較に矛盾候補が出た。confidenceは0.31で、構文の受理と後段の型・評価の区別を判断するための文脈が足りないと考えた。記述を合格へ変更するのではなくsyntax/eval/libを加えた。候補はその後の比較では出ず、宣言した範囲内で整合を確認した。この経過自体が、候補を自動で実装の欠陥と断定しない理由になる。

[最終記録](../runs/0045-support-final/result.json)は7 crate・28比較すべてsupports、未回答・未確定・矛盾0。比較のinput、判定、manifest、使用量と検証ログを保存した。最終live一回の使用量は約0.00575 USD。これは今回の引用量の実測で、将来の実行費用の保証ではない。

[事前に固定した対照](../runs/0045-support-controls/result.json)も実行した。SMTのReal/LRAとtestingのscalar縮小を対応済みにする2件はcontradicts、元の正常版2件はsupportsだった。4件のinput/判定は`just support-replay`でオフライン再生できた。小さい人工対照の検出であり、全crateの未知バグ発見や一般的な精度の推定ではない。

決定的な6テストは実Git repoを作る。未stageの編集と修正、追加crate、欠落したチェックリスト、参照切れ、symlink、コードfence内の例、古い・欠落・改変された判定を確認した。end-to-endのgateテストは実際のJev記録を使い、APIキーを空にして実行する。別ツールのpre-commit hookは上書きしない。

配布用archiveにSUPPORT.mdがないRedを確認し、7 Cargo manifestとpackagerへ同梱を追加した。`just package`では7 archiveのチェックリストとnpm/Rust/Wasmの単独consumerが通過した。`just check`、`just browser`（3テスト）、`just support-test`（6テスト）、`support-check`、evaluate/replay-evaluationも通過した。generated manifestのformatter差分はsync recipeで整形し、意味とdigestを変えず再生できる。

採用するのは、選択した比較箇所の意味をJevで確認し、その判定がコミット対象へまだ適用できるかを決定的に検査する仕組み。任意の実装変更の意味を完全に理解する検査ではない。sourceのコメントだけを変えた場合もdigestが変われば再確認を求める。既存のテスト・TLC/Z3差分・モデル検査を代替しない。

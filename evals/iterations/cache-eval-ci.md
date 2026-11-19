# 評価runの初回作成をCIへ合わせる

[PR #70の最初のCI](https://github.com/mizchi/idealis/actions/runs/44567881882)では、cache契約テスト10件が通った後、評価先`.artifacts/cache-expansion-eval`の作成がENOENTで停止した。ローカルには親`.artifacts`が既にあったため、初回の出力先作成を検査できていなかった。

モデル評価とCriterion runnerで、親を再帰的に用意し、runのディレクトリ自体は排他的に新規作成する処理を共有した。既存runの上書きは許可しない。新しい一時ディレクトリ下の未作成parentでCLIを実行する回帰をRedにし、修正後の評価と再生、同名runのEEXIST拒否とmanifestの保持を確認した。

共有11テスト、通常のcheck、12モデル条件・32再利用条件の評価と再生が成功した。cache本体の意味論や性能条件は変更していない。これは評価ハーネスの初回実行の不備で、モデル検査器の意味論的故障の発見には数えない。[失敗ログと修正後の記録](../runs/cache-eval-ci/evidence.json)を保存する。

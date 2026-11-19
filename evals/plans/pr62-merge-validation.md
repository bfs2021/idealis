# PR 62のマージ前検証

対象headは35f66b321c68ed348d6ed4fd89f337192fd78f8d、baseはd147550c80d814e23f5d76f8ded25e30bf190da4。後続開発のfrontend-semanticsとは別worktreeで検証する。

- 追跡されたRust target生成物78,753件を除き、再混入をCIで拒否する。
- source/fixtureを読んで既存レビューの指摘を再確認する。大きなtuple targetの展開には正常対照と資源制限付きの独立プロセスを使い、停止を成功へ読み替えない。
- staleなSUPPORT判定をJev liveで同期し、offline/stagedで再検証する。
- 両frontendの解析をjust/CIへ接続し、unit・native/Wasm・Chromium・mutation・配布consumerを検査する。
- 修正headをpushしてGitHub Actionsの成功を確認し、head SHAを固定してマージする。
- 性能の再測定で既存のinconclusiveを成功へ変えず、元の測定記録を保持する。

# 0038: actorの順序を制御してlost updateを再現する

`idealis-testing`へScheduleから有限生成器への変換を追加した。TSは`runtime.schedule`、Rustは`Schedule::generator`を使う。ホストが明示した操作境界を1つずつ進め、同じcaseをTSのasync処理とRustの実threadへ再生する。[事前計画](../plans/0038-controlled-schedules.md)と[利用手順](../../docs/testing.md#actorの操作順を制御する)を参照。

## 逐次対照と全操作列

参照モデルの期待値は完了したwrite数。故障版はread時の値+1を後からstoreし、正常版はwriteでatomic incrementする。actorを1つずつ完了させる逐次対照は両版で通る。全操作列では次の追加検出があった。

| actor数 | 完了する操作列 | 正常版の通過 | 故障版の検出 | 故障版の最終値 |
| ------: | -------------: | -----------: | -----------: | -------------- |
|       2 |              6 |          6/6 |          4/6 | 1, 2           |
|       3 |             90 |        90/90 |        84/90 | 1, 2, 3        |

独立TS再帰とRust生成器の完了列が一致し、各prefixの観測もTSとチャネルでgateした実Rust threadで一致した。Loom 0.7.2の2 actor例は、正常版54実行で最終値2、故障版90実行で最終値1/2を返した。permutation/preemption上限と時間制限は指定していない。maxBranchesは1000で、超過すれば例外となる。Loomの列挙回数はactorの操作列数と同じ指標ではない。

LoomにはSeqCstやload bufferingの[既知の制約](https://github.com/tokio-rs/loom/tree/v0.7.2#unsupported-features)がある。この小さい例の最終値集合の一致を、全メモリモデルの互換や任意実装の証明へ一般化しない。

## PBT・縮小・再生

| run                                                     | seed数 | 正常run通過 | 既知故障の検出・再現 | 見逃し | core輸送数 | SUT輸送数 |
| ------------------------------------------------------- | -----: | ----------: | -------------------: | -----: | ---------: | --------: |
| [開発](../runs/0038-schedules-development/result.json)  |     16 |       32/32 |                32/32 |      0 |      2,765 |     5,528 |
| [確認](../runs/0038-schedules-confirmation/result.json) |     32 |       64/64 |                64/64 |      0 |      5,443 |    10,692 |

各seedで2/3 actorを検査するため、run数はseed数の2倍。各正常runは64有効caseを実行した。全96件の故障runが4操作へ縮小され、同じsignatureでTS/Rustの故障版に再現し、正常版は通過した。coreのnative/Wasmの全trialと結果も一致した。有効な単一削除について、同じ失敗を保持しないことを別に検査した。

sourceVersionは`bf1af72014a50828cb94770a8d7bbb8b8a73f38c27a88b10f6e12236bcf3ccc6`で共通。全入力・応答・trialとLoom出力を保存し、同じソース・binaryで再生した。開発seedは0x38a1から16件、確認は0x38c3から32件。これは既知の人工故障に対する評価で、未知の実プロジェクトの不具合発見ではない。

その後、単独Rust consumerでexampleの未宣言serde_json importが失敗した。操作を`Into`で作る例へ修正し、[回帰条件](../plans/0038-packaging-regression.md)を先に記録した。[開発回帰](../runs/0038-packaging-regression-development/result.json)と[確認回帰](../runs/0038-packaging-regression-confirmation/result.json)の最終sourceVersionは`63b8e755e00f8f50ba5e67ef433c264158ad2bb56c19395bdb7b513c528293d0`。同じseedの全trial・core/SUT輸送・Loom出力は元の記録と一致し、再生した。これらは観測済みseedの回帰であり、新しい独立した実験ではない。

## 可搬性と費用

Rust coreの新しい3 tests、native gateの2 tests、共通3 testsをnode:test/Vitestで検査した。実Chromiumは別seed 0x3801で正常・故障、4操作への縮小、両版への再生、work 0を確認した。得た反例はRustの実threadでも再現した。ホストimportとcross-origin isolationは不要。

Wasmは374,454 bytes。0036の350,919 bytesから23,535 bytes増えた。Loomはeval crateのdev dependencyだけに固定し、pure coreにはserde/serde_json以外を追加していない。単独npm/Rust consumerで新しいschedule APIと依存の分離を検査した。

全validationの時間は開発約1.25秒、確認約1.41秒。生成・縮小・TS/Rust照合・実thread起動・IPC・Loomを含み、solverや言語間の速度比較ではない。case数の異なるrunをthroughputの改善としても扱わない。[検証ログと配布consumer](../runs/0038-schedules-verification/result.json)を保存した。

既存mutationの95.4%はTS例の変異結果を再利用した値で、今回のRust coreやスケジュール機能のmutation強度を測っていない。初回のconsumer失敗と、別ブランチのビルド生成物・formatによるlint失敗のログも残した。

## 採用範囲

有限actor列の生成・縮小・保存caseの再生として採用する。逐次対照では検出できなかった故障を同じprefixオラクルで検出し、TS/Rustの実行と修正版への対照を確認できた。PBTのpassedは実行したcaseに限る。全探索は今回の固定96操作列についてのみ検査した。

任意thread/awaitの自動計装、ブロッキング、weak memoryの意味論、動的なactor作成や一般のlivenessは対象外。生成器の直積は1024状態までで、構築費用はSession.maxWorkの外。次は保存caseとprotocolの構造を使うfuzzingを評価する。

# 0022: READMEへ載せる検査器の比較

0019/0021のlarge 4入力を変更せず、tla-tsを加えて同じ期間に再測定する。READMEには1 workerの4実装の比較と、TLC/nativeの1→8 workerの比較を分けて載せる。tla-ts 0.0.10にも並列APIがあるが、今回は1 workerのみを対象にする。tla-tsの並列性能やツール全体の優劣を結論にしない。

## 固定条件

6経路はtla-ts 1、TLC 1/8、modelt Wasm 1、modelt native 1/8。nativeはcheck-safety-parallelでworker数を明示し、1では逐次経路を使う。4入力はcounter-product (5,308,416状態)、function-cube (1,048,576)、parallel-channels (2,097,152)、linear-ring (1,000,000)。全経路で同じsource/configを毎回読み、解析から検査する。Rustコアと入力の最適化・縮小は行わない。

各条件1 warmup + 3測定。経路順は反復ごとに回転し、全経路を直列実行する。測定中は別のbuild/test/replayを開始せず、測定プロセスの終了を確認してから検証を始める。112ではなく4×6×4=96 sampleを保存し、集計対象は72 sample。前のrunの値を新しい表へ混在させない。

TLC v1.7.4、Temurin 21.0.13+9 arm64、-Xmx8g、ParallelGC、ActiveProcessorCount=8、seed16、lncheck final。Node24、--max-old-space-size=8192。tla-ts 0.0.10はworkers=1、version=1.7.4、livenessCheck=final。modelt上限はdepth/states=10,000,000、work=4,000,000,000。これらは厳密な共通RSS上限ではない。timeoutは追加しない。

主指標はprocess spawnから終了・JSON decodeまで。全経路でsource解析、runtime/JVM/Wasm初期化を含む。入力の配置・ログ保存は計測外。補助区間の定義は経路ごとに残し、tla-tsのsource解析込みAPIとmodeltのIR評価API、TLCのInit開始以降を同じkernel時間と呼ばない。READMEには起動込みの比較を載せ、常駐APIの過去の0016結果は別runであることを明示してリンクする。

## 正しさと証拠

解析上の状態数・生成数と完全探索を全sampleで確認し、bounded/empty/error/欠落countを成功へ変えない。tla-tsのraw結果はbigintを十進文字列へ符号化して保存し、modeltに存在するworkの値をtla-tsに捏造しない。smallの正常/遅延不変条件故障を6経路で確認する。失敗ログは保存し、最適化や入力変更で隠さない。

ソース・commit・dirty paths・環境・java/jar/native/Wasm/tla-ts配布物のdigest、raw logs、全sample、RSS sampleを新しいrunへ保存する。中央値/min/maxと条件をREADME・iterationへ記録する。READMEの表は集計結果から生成し、すべての数値を原本と照合する。

replayは全96 sampleの証拠・source/binary digest・条件・判定・計数を検証し、small 4入力×6経路を再実行する。largeの独立した性能再測定とは区別する。終了前にjust check/browser、evaluate/replay-evaluation、新規のcontract/integration検査を実行する。既存runは編集せず、新規ledgerへ追記する。

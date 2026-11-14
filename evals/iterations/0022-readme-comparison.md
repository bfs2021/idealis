# 0022: tla-ts・TLC・modeltのREADME比較

同じ4モデルを6経路で測り直し、READMEへ1枚の比較表を追加した。過去の0016/0019/0021の値は混在させていない。全96 sampleで完全探索・状態数・生成数が一致した。集計対象はwarmupを除く72 sample。

## 固定条件と結果

[事前計画](../plans/0022-readme-comparison.md)を先にコミットした。測定sourceは`74ba626`で、開始時のworking treeはclean。Rustコアと入力は変更していない。Apple M5 arm64、32 GiB、Node 24.22.2、tla-ts 0.0.10、TLC v1.7.4、Temurin 21.0.13+9で測った。

各入力・経路で1 warmup + 3測定。順番を反復ごとに回転し、単独で直列実行した。Nodeは--max-old-space-size=8192、TLCは-Xmx8g、ParallelGC、ActiveProcessorCount=8、seed16、lncheck final。厳密な共通RSS上限ではない。tla-tsはworkers=1、version=1.7.4、livenessCheck=final。並列APIはあるが今回は測っていない。

単位は秒。別processでsource解析、runtime/JVM/Wasm初期化、検査、JSON decodeと終了までを含む。入力の配置・ログ保存は含まない。中央値を示し、最小・最大・全sampleは[本測定](../runs/0022-readme-comparison/result.json)に残す。

| 入力        | tla-ts 1 |  TLC 1 | TLC 8 | Wasm 1 | native 1 | native 8 |
| ----------- | -------: | -----: | ----: | -----: | -------: | -------: |
| カウンタ群  |    5.190 |  6.134 | 6.476 |  3.818 |    2.851 |    3.646 |
| Boolean関数 |    5.388 | 12.647 | 4.444 | 18.666 |    9.345 |    5.101 |
| チャネル群  |   16.371 | 23.825 | 6.807 | 31.415 |   20.733 |    9.537 |
| 幅1のring   |    0.410 |  0.900 | 1.440 |  0.244 |    0.186 |    0.187 |

カウンタ群とringではnative 1 workerが最速。関数とチャネル群ではTLC 8 workersが最速だった。1 worker同士では、tla-tsが関数でnativeの約1.73倍、チャネルで約1.27倍速い。nativeはカウンタでtla-tsの約1.82倍、ringで約2.21倍速い。モデル全般で一方が速いという結果ではない。

関数のtla-ts 1 workerとnative 8 workersは5.388 sと5.101 sで、sampleの範囲は重なる。差を統計的な優位性と解釈しない。tla-ts 8 workersは未測定。WasmはNode上の単一threadで、ブラウザの速度ではない。nativeの幅1モデルはpoolを起動しない。

## 計数と補助区間

状態数/生成数は、counter 5,308,416/21,233,665、function 1,048,576/20,971,521、channels 2,097,152/22,020,097、ring 1,000,000/1,000,001。全経路で1初期状態を使う。native上限はdepth/states=10,000,000、work=4,000,000,000で、全sampleがcompleteになった。

tla-tsのAPI区間はsource/config解析を含み、modeltのAPI区間はfrontendのIR compileを除く。TLCはstdoutのInit開始からsuccessまでで、最終FP検査を含む。同じkernel時間として比較しない。READMEには起動込みを載せ、常駐APIは別の0016測定へリンクした。RSSは200 ms samplingで厳密なpeakではない。

## 再生とREADMEの照合

[manifest](../runs/0022-readme-comparison/manifest.json)にsource、commit、環境、java/jar/native/Wasmとtla-ts配布物のdigestを保存した。[再生](../runs/0022-readme-replay/result.json)では全96 sampleのsource/binary/evidence、入力・経路・判定・計数を検証し、small 4入力×6経路を再実行した。largeの独立した性能再測定ではない。

初回のreplayは、smallの時間にlargeの状態数を添えたtables.jsonも生成した。この表は性能資料として無効。raw sampleとcases.jsonの判定・実計数は正しい。READMEは本測定の値だけから生成した。測定後の報告処理を修正し、以後はreplayで性能表を作らず、README生成もreplay入力を拒否する。測定sampleは変更していない。厳密なsource再生は上記freeze commitをcheckoutする。

[検証記録](../runs/0022-verification/result.json)には、48件のsmall正常/故障対照、未完了・count欠落・不一致の拒否、just check/browser、evaluate/replay-evaluation、READMEの全24数値と状態数の照合を保存する。比較対象は対応する有限4モデルで、TLA+全体の互換性や未知の実装バグの検出を示す評価ではない。

```sh
just readme-bench-test /path/to/java /path/to/tla2tools.jar
just readme-bench /path/to/java /path/to/tla2tools.jar /path/to/jre.tar.gz evals/runs/new-readme-run
just readme-bench-replay /path/to/java /path/to/tla2tools.jar /path/to/jre.tar.gz evals/runs/new-readme-run evals/runs/new-readme-replay
node dist/evals/readme-comparison/readme.js evals/runs/new-readme-run
```

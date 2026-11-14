# TLCと同じ88検査の起動込み比較

同じ混合88件を指定探索worker合計8で実行した。速かったTLCの配置は8 JVM × 各1 workerで、起動込み中央値が11.14 s。modelt nativeは0.446 sで約25.0倍、Wasmは0.776 sで約14.4倍速かった。これは多数の検査を呼び出す実行経路の速度比であり、JVM起動を除いた探索kernelの倍率は測定していない。

0017の約4.8倍はmodelt自身の逐次実行に対する値だった。TLC比と混同しないため、[計画0018](../plans/0018-tlc-throughput.md)を先に記録し、実装を`2de603f3ebc15012164f1e94196e65c4d4edf104`へ固定した。開始時のworking treeはclean。source fingerprintは`4d1ace2014c6ea97279217ec479a31ea1c63e12715228c9321773af9e88615fe`。

## 同じtask群を異なる実行単位へ配置する

各経路で1回ウォームアップ後3回を測定した。中央値 [最小, 最大]、単位は秒。経路順を反復ごとに巡回し、経路間の測定は直列に実行した。

| 経路                     |  起動込みの全件完了時間 | 速かったTLC配置との速度比 |
| ------------------------ | ----------------------: | ------------------------: |
| TLC 1 JVM × 8 worker     | 66.494 [66.059, 68.291] |                    0.17倍 |
| TLC 8 JVM × 各1 worker   | 11.140 [10.890, 13.380] |                    1.00倍 |
| modelt native 8 executor |    0.446 [0.427, 0.503] |                   24.98倍 |
| modelt Wasm 8 executor   |    0.776 [0.725, 0.797] |                   14.36倍 |

入力は0017の11仕様を8巡する混合88 task。同じtask ID・source/config・費用順のtask順を使い、各taskの正常終了・状態数・生成数が一致した。11仕様には線形Ringが含まれ、単一探索のworkerを増やしても同じ深さで探索できる状態数は増えない。この入力群ではTLCも独立した検査を並べる方が速かった。

modeltはbatchごとに新しいNode controllerを起動し、11仕様を解析・IRへコンパイルして8 executorへ登録する。nativeは8子プロセス、Wasmは8 instance。taskを実行する際は登録したIRを使い、結果はキャッシュしない。TLCは公開CLIを使い、88 taskそれぞれで新しいJVMを起動する。TLCの常駐JVMで仕様を再利用する独自driverは作っていない。

全経路でbatch helperのプロセス生成から、結果返却・JSON decode・終了までを測った。入力ファイルの配置と証拠保存は計測外。source解析は計測内だが、modeltは仕様ごとに一度、TLCはtaskごとに行う。OSキャッシュは消していない。起動回数と再利用単位の違いを含む、多数の検査を呼び出す用途の比較として解釈する。

modelt内の`runSchedule`部分の中央値はnativeで296 ms、Wasmで627 ms。新規controllerのmodule読み込み、sourceコンパイル、pool登録と終了を含む全体では446 msと776 msになった。0017の再利用済みpoolの286 ms / 420 msとは条件が異なる。TLCの内部探索だけの時間をこの値から推測しない。

確認用replayではTLCの8 JVM配置が10.715 s、nativeが0.923 s、Wasmが0.953 sだった。nativeのpool起動は主測定の約42-45 msに対し489 msとなり、Wasmの探索部分も増えた。replayはウォームアップと反復数が異なるため主測定へ加えないが、この時間変動も保存する。25倍 / 14倍を固定倍率の保証として採用しない。replay単独の時間比はnative約11.6倍、Wasm約11.2倍で、変動の原因は切り分けていない。

## 単体モデルの比較と実行条件

[0016の起動込み測定](0016-checker-benchmark.md)では、100万状態のRingはTLC 909.77 ms、native 190.77 ms、Wasm 272.66 msだった。同じ単体モデルではnative約4.77倍、Wasm約3.34倍。今回の25倍 / 14倍は独立した検査を繰り返すbatch全体の値であり、単一モデルの高速化率へ転用しない。

- Apple M5、arm64、Node v24.22.2。同じTLC v1.7.4 jar、表示TLC 2.19 rev `5a47802`、Temurin 21.0.13+9のarm64 JRE。
- TLCは`-lncheck final`、seed16、ParallelGC。1 JVMの場合は`-workers 8`、`-Xmx8g`、`ActiveProcessorCount=8`。8 JVMの場合は各`-workers 1`、`-Xmx1g`、`ActiveProcessorCount=1`。
- 指定探索worker数の合計を8に揃えた。JIT・GCなどを含むOS thread数やCPU affinityを同一にした測定ではない。
- TLCの最大heap合計は8 GiB。heapはRSS上限ではなく、Node/Rustのメモリ制約と同一でもない。JVMごとに同時実行先のworking directoryと標準モジュール展開先を隔離した。
- modeltのnative/Wasm binaryは0017から変更していない。WasmはNode上で測定した。対応済みの有限fragmentを対象とし、TLA+全体や未知の分散システムへの一般化は行わない。

## 保存した証拠

[本測定](../runs/0018-tlc-throughput/result.json)に16サンプルを保存し、うち12サンプルを集計した。1,408 taskすべてで判定・状態数・生成数が一致した。JVMの起動引数、生のstdout/stderr、全taskの結果は各サンプルのJSONに保存した。[manifest](../runs/0018-tlc-throughput/manifest.json)にsourceとJava/jar/JRE配布物/native/Wasmのdigestを記録した。

[replay](../runs/0018-tlc-throughput-replay/result.json)は保存した全入力・サンプル・計数・digestを確認し、4経路で352 taskを再実行して一致した。replayは独立した性能サンプルへ加えない。[検証ログ](../runs/0018-verification/result.json)に通常検査、実ブラウザ、既存の評価/replay、Rust実験と今回の測定・再実行を保存した。

```sh
just tlc-throughput "$java" "$jar" "$jre_archive" evals/runs/new-tlc-throughput
just tlc-throughput-replay "$java" "$jar" "$jre_archive" evals/runs/new-tlc-throughput evals/runs/new-tlc-throughput-replay
```

多数の検査をライブラリから呼ぶ用途では、起動とsource解析を償却するmodeltの構成が今回の測定で有利だった。常駐JVMでのAPI比較と、起動・解析を除く探索kernelの比較は別の実験として残す。

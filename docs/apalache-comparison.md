# Apalacheとrtlcの比較

rtlcは有限状態を列挙して到達グラフを検査するRust library、ApalacheはSMTで有界実行や帰納性を検査するtool。ブラウザ内の反復テストにはrtlc、状態を記号的に扱う検査にはApalacheという使い分けが候補になる。これは方式と依存関係からの設計判断であり、大規模モデルの速度順位を示すものではない。

比較日は2026-11-15。Apalacheは[v0.63.0](https://github.com/apalache-mc/apalache/releases/tag/v0.63.0)、rtlcは[今回の計測source](../evals/runs/0027-apalache-pilot/manifest.json)を対象にした。

## 方式と保証

| 観点               | Apalache                                                         | 現在のrtlc                                                                            |
| ------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| 探索               | TLA+をSMTへ変換し、状態を記号的に扱う                            | 値を評価し、BFSで具体的な状態を列挙する                                               |
| 通常の反例なし     | 指定長kまで反例がない                                            | `complete`なら指定した有限モデルの到達グラフを検査済み。予算切れは`bounded`           |
| 帰納性             | 初期成立と1 stepでの保存をSMTで検査できる                        | 専用のSMT帰納性検査はない。有限候補を列挙して調べることは可能                         |
| frontend           | SANYとSnowcatの型検査                                            | rtlaはparser。既存IR生成はtla-tsを使うTS compiler                                     |
| 整数と値           | SMT整数を使い、record/tuple/function等を型で区別する             | signed 32 bit。overflowはエラー。record/sequenceは有限関数としてTLC互換の等価性を使う |
| temporal           | liveness-to-safety変換。対応表ではENABLED、WF/SFは手動で記述する | 対応fragmentで積グラフ/SCCを検査し、列挙可能なENABLEDとWF/SFを処理する                |
| 実行要件           | JVMとSMT solver。Z3が既定で、CVC5も選べる                        | Rust native / Wasm。実行部はJVM/solverに依存しない                                    |
| 実装テストとの接続 | ITF出力、checker/explorer serverによる外部連携                   | modelt側にoracle、照合、履歴、縮小、障害注入、TS/Rust/Go adapterがある                |

Apalacheの[方式](https://apalache-mc.org/docs/apalache/theory.html)、[BMCと帰納性](https://apalache-mc.org/docs/apalache/running.html)、[対応表](https://apalache-mc.org/docs/apalache/features.html)、[型付け](https://apalache-mc.org/docs/apalache/principles/index.html)を参照。rtlcの欄は[実装と対応範囲](rust-safety.md)を基準にする。どちらも全TLA+の互換実装とは扱わない。

構文の基準版も異なる。rtlcはTLC v1.7.4を基準にし、Apalache v0.63.0はSUBSET/UNION/DOMAINの優先順位を修正している。[release note](https://github.com/apalache-mc/apalache/releases/tag/v0.63.0)ではbreaking changeとされる。今回のpilotはこれらを使わず、今後の比較でも版を固定して括弧を明示する。

Apalacheもsymbolic actionの探索で実装からのfeedbackを扱う。[公式の説明](https://apalache-mc.org/)と[explorerの設計](https://apalache-mc.org/docs/adr/010rfc-transition-explorer.html)を参照。今回取得したCLIでもcheckerはShai gRPC、explorerはJSON-RPCとして列挙される。実装テストをrtlcだけの機能とは比較しない。serverの接続・再利用性能は今回未検証。

ApalacheのBMCで反例がなくても、長さを超えた実行については結論が出ない。帰納的不変条件では、初期条件からの成立と遷移による保存を別々に検査する。必要な型・定数・環境仮定が適切で、両方が成立すれば、その条件下の任意長の実行へ結論を広げられる。rtlcの`complete`も、指定した有限モデル内の保証であり、任意のノード数への一般化ではない。

## 同じ小規模モデルでの実測

[計画](../evals/plans/0027-apalache-comparison.md)を先に固定し、同じsource/configを使用した。型注釈はcommentなので両frontendへ同じsourceを渡している。16状態ringと、8要素の0/1関数を全初期状態とする256状態モデルを使った。各条件で1 warmupと3測定、計54測定と18 warmupを行い、backendの順を交互に変えた。

Apple M5、arm64、32 GiB、Node v24.22.2、rustc 1.98.0、Temurin Java 21.0.13。Apalacheはfresh JVM、heap上限4096 MiB、incremental、Z3、oopsla19 encoding。公式はJava 25を推奨し、Java 21以上のbytecode互換を維持している。[実行要件](https://apalache-mc.org/docs/apalache/installation/jvm.html)を参照。rtlcは逐次のmodelt-core JSONL driverを使った。メモリ消費量の実測はしていない。

時間はprocess起動から結果判定まで。下表は中央値と括弧内に最小-最大、単位ms。rtlcのsource経路はfresh Node、TLA+解析・IR生成、native processを含む。IR経路は事前IRからnative processを起動する。ApalacheはTLA+解析・型検査・SMTを含む。反例の検証とarchiveのI/Oは時間から除いた。

| 条件           |      Apalache source→判定 |    rtlc source→判定 | rtlc 事前IR→判定 |
| -------------- | ------------------------: | ------------------: | ---------------: |
| ring正常、k=4  | 1119.62 (1105.14-1207.02) | 73.86 (66.62-76.39) | 2.03 (1.88-3.40) |
| ring正常、k=15 | 1194.96 (1101.21-1208.32) | 67.19 (63.55-67.95) | 1.73 (1.63-1.89) |
| ring故障、k=4  | 1144.49 (1099.52-1256.13) | 61.90 (60.03-69.51) | 1.79 (1.66-1.97) |
| ring故障、k=8  | 1159.92 (1119.66-1164.23) | 65.69 (61.76-79.21) | 1.92 (1.83-2.53) |
| 関数正常、k=4  | 1873.60 (1829.20-2268.08) | 69.26 (63.34-87.61) | 2.44 (2.12-2.60) |
| 関数故障、k=4  | 1152.48 (1140.61-1181.56) | 62.39 (60.98-62.57) | 1.90 (1.85-1.96) |

[全sampleと結果](../evals/runs/0027-apalache-pilot/result.json)にsource、config、IR、raw log、反例、binary/source hashを保存した。小さい6条件のsource経路ではrtlcが約15-27倍短かった。事前IR列とApalache source列の比率を、そのままchecker kernelの差として扱わない。

fresh processの比較で、warmupは常駐JVMのJIT warmupではない。起動準備の費用が大きいモデルを測っている。巨大な集合・関数、長い反例、Apalacheの常駐server、Wasm、並列探索は今回測っていない。hostも専有していない。Apalacheの記号的な状態表現が有利になる条件の評価は残る。

## 同じ「反例なし」でも保証範囲が異なる

| 条件           | Apalacheの応答     | rtlcの応答                                |
| -------------- | ------------------ | ----------------------------------------- |
| ring正常、k=4  | 長さ4まで反例なし  | depthによる`bounded`                      |
| ring正常、k=15 | 長さ15まで反例なし | 16状態の閉グラフで`complete`              |
| ring故障、k=4  | 長さ4まで反例なし  | depthによる`bounded`                      |
| ring故障、k=8  | 8遷移で違反        | 8遷移で違反                               |
| 関数正常、k=4  | 長さ4まで反例なし  | 初期集合だけで全256状態を含み、`complete` |
| 関数故障、k=4  | 初期状態の違反     | 初期状態の違反                            |

故障ringはx=8で初めて違反する。深さ4で見つからないことと、深さ8で見つかることを対照にした。反例はITFとrtlcの値を共通の数値/配列へ読み、初期状態、全遷移、最後の違反、深さを独立したvalidatorで確認した。これはモデルに対する検証であり、実SUTの未知バグ発見には数えていない。

正常ringではApalacheで`Init => Safe`をlength=0、`Safe /\ Next => Safe'`をlength=1で別に検査し、両方が成立した。これは固定16状態のモデルに対する帰納性で、任意パラメータの証明ではない。性能表へ追加sampleとして混ぜていない。

## modeltへ取り込む候補

1. ITFのimport/exportを先に評価する。ApalacheとQuintの反例をmodeltのoracle/照合へ渡せる。[ITF仕様](https://apalache-mc.org/docs/adr/015adr-trace.html)は整数・集合・map・loopを記述する。今回のreaderは整数と固定domainの関数だけで、汎用adapterではない。model value、record、整数幅、actionの対応も定義する必要がある。
2. backendの保証を明示する。`explicit-complete`、`bounded-no-counterexample(k)`、`inductive`と、empty/error/unknownを区別する契約を検討する。予算切れやSMTのunknownを合格にしない。現行public APIの変更は今回行っていない。
3. SMT backendの設計は独立に行う。rtlcの有限IRをそのままZ3へ渡してもApalache相当にはならない。型検査、整数の意味、集合/関数のSMT encoding、有界実行、temporal変換、反例の再構成が必要。z3-wasmを使う場合も、この変換層の実装・評価が必要になる。

現在の目的であるブラウザ・portable library・反復する実装テストではrtlcを維持し、ApalacheをCI等の外部検査backendとして接続する案を推す。次の速度比較では、深さ、domain幅、反例までの距離を別々に増やし、同じ保証範囲の費用を測る。今回の小規模pilotだけでどちらを置き換えるかは決めない。

## 再実行

v0.63.0のrelease artifactを取得し、公式sha256sumで検査してからjarを指定する。

```sh
just apalache-test
just apalache-eval evals/runs/<新規ID> /path/to/java /path/to/apalache.jar
```

必要なApalache packageは約192 MBで、今回のartifact SHA-256は`a756e96af249bf33cd7629667a49472cb26ac8227c48583da139b414834d59df`。JAR自体はrepoへ同梱しない。既存runは上書きせず、新しいIDで実行する。

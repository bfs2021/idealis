# 単一モデルの探索幅とTLC worker scalingを比較する

0018の多検査throughput比較を、TLC自身の単一モデル並列探索の性能と混同しないための実験。外側のpoolは使わず、1つのsource/configを1回のプロセスで完全探索する。TLCは1/2/4/8 worker、modelt native / Wasmは現実装どおり各1 executor。同じモデル・invariant・deadlock設定・状態数・生成数を照合し、modeltが速いという仮説を採用条件にしない。

性能入力を次の4つに固定する。いずれも初期状態を1つにし、大量の初期列挙をBFSの並列化として扱わない。

| 入力                        |           状態数 |                        生成数 | 構造                                                |
| --------------------------- | ---------------: | ----------------------------: | --------------------------------------------------- |
| 4 scalar counters, radix 48 | 48^4 = 5,308,416 |     4 * 48^4 + 1 = 21,233,665 | 各actionが1 counterを巡回更新する直積               |
| Boolean function cube, N=20 | 2^20 = 1,048,576 |    20 * 2^20 + 1 = 20,971,521 | 任意のbitを反転する幅の広いグラフ                   |
| Independent channels, N=7   |  8^7 = 2,097,152 | 12 * 7 * 8^6 + 1 = 22,020,097 | upstream Channelのsend/receiveを独立channel群へ拡張 |
| Scalar ring, N=1,000,000    |        1,000,000 |                     1,000,001 | 探索幅1の対照                                       |

Channelはvendored原本を変更せず、固定初期値から各channelの8状態が到達するように独立なsend/receiveを合成する。根からの層ごとの状態数は、counterが(1+x+...+x^47)^4、cubeが(1+x)^20、channelが(1+2x+2x^2+2x^3+x^4)^7の係数で検算する。各入力の縮小版をTLC 1/8 worker、tla-ts、native/Wasmで先に確認する。正常・後続状態でのinvariant violation対照を持ち、後続状態を探索しない合格やboundedを成功へ変換しない。

1 warmup + 3測定。入力順は固定、同じ入力の経路順を反復ごとに巡回する。経路はtlc-1/tlc-2/tlc-4/tlc-8/native/wasm。プロセスと探索を直列実行し、ビルドや他のテストを同時実行しない。全経路でsource/configを読み、起動・解析・探索・結果decode・終了を含むwall timeを主指標とする。入力配置・結果保存は計測外。OSキャッシュは消さない。

TLCは同じv1.7.4 jarとarm64 Temurin 21.0.13+9、-Xmx8g、ParallelGC、ActiveProcessorCount=8、seed16、-lncheck final、-toolを固定する。worker数以外のJVM設定を変えず、GC/JIT設定の違いをworker scalingへ混ぜない。各JVMのworking directoryと標準モジュール展開先を隔離する。Nodeのold-space上限も8 GiBとするが、nativeやWasmを含むRSS上限の同一性は主張しない。

補助指標として、TLCのEC.TLC_COMPUTING_INIT(2189)からEC.TLC_SUCCESS(2193)までをstdout受信時刻で測る。EC.TLC_INIT_GENERATED1/2も保存する。これは初期生成・探索・最終FP検査などを含む公開イベント区間であり、純粋なBFS kernelではない。modeltはsourceコンパイルとengine準備を除いたcheckSafety相当のAPI時間を測り、IR検証・serialize/decode・初期生成・探索を含む。境界の違いを明記し、直接のkernel倍率を計算しない。TLC自身のworker scalingは同じイベント区間同士でも比較する。

プロセス群のRSSを200 ms間隔で観測し、sampled maxとして保存する。nativeはNodeとRust子プロセスの合計、WasmはNode全体、TLCはJVMを対象にする。厳密なpeakやOSによる強制上限とは扱わない。modeltの上限はdepth/states=10,000,000、work=4,000,000,000。wall timeoutは追加しない。OOM・不完全探索・非対応・プロセス異常は生ログと一緒に保存し、入力サイズを結果に合わせて黙って減らさない。

planを先にコミットし、実装と正しさテストを通したsourceを測定前に固定する。入力・binary・sourceのdigest、commit、dirty paths、全サンプルと生ログを新しいrunへ保存する。3回の中央値・範囲・TLC worker speedup、初期状態数、状態数、生成数、層幅、RSSを記録する。replayでは全証拠のdigestと計数を確認し、縮小版の全経路を再実行する。縮小版replayを大規模入力の独立した性能確認へ数えない。大規模入力の性能replayは同じ条件で新規runとして実行できるようにする。

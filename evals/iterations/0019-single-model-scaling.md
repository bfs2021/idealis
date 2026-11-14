# 単一モデルの並列探索を比較する

探索幅の広い単一モデルではTLCの内部並列化が効いた。関数ハイパーキューブは1 workerの12.81 sから8 workerの4.72 s、独立チャネル群は24.08 sから6.27 sへ短縮した。現在のmodelt nativeはそれぞれ8.89 sと19.86 sで、TLCが約1.9倍、約3.2倍速かった。カウンタ直積はnativeが約2.2倍速く、入力によって優劣が入れ替わる。

[計画0019](../plans/0019-single-model-scaling.md)を先に登録し、実装を`e80683efeb9978cf2d0fabc5437579a097786700`へ固定した。source fingerprintは`6cb7333df59281fe69a574add95d3d4d1442cdad135aeefa16a006493017d043`。開始時のworking treeはclean。checker本体とnative/Wasm binaryは変更していない。

## 幅の広いグラフを固定する

各入力の初期状態は1つ。大量の初期状態生成をworker scalingとして数えない。前の88検査とは異なり、各サンプルで1モデルを1回のプロセスから検査する。外側のpoolや複数JVMによるtask並列化は使わない。

| 入力                               |    状態数 |     生成数 | 最大層幅 |
| ---------------------------------- | --------: | ---------: | -------: |
| Counter product：4 counter、各48値 | 5,308,416 | 21,233,665 |   73,744 |
| Function cube：20 bit              | 1,048,576 | 20,971,521 |  184,756 |
| Parallel channels：7 channel       | 2,097,152 | 22,020,097 |  253,528 |
| Linear ring：幅1の対照             | 1,000,000 |  1,000,001 |        1 |

counterは1変数ずつ巡回更新する直積、cubeは任意のbitを反転する関数、channelはvendoredのupstream Channelのsend/receiveを独立に合成したもの。状態数と生成数を式で検算し、層幅は距離ごとの状態数を表す多項式の係数から求めた。小さい版ではTLC 1/8 worker・tla-ts・native/Wasmの正常結果と計数が一致した。後続状態でinvariantが破れる対照も全実装が検出した。原本のChannelは編集していない。

## 起動・解析込みの全探索

1回ウォームアップ後3回の中央値、単位は秒。経路順を反復ごとに巡回し、測定と他のビルド・テストは同時実行していない。TLCだけが単一探索内で1/2/4/8 workerを使い、modeltは各1 executor。

| 入力              |  TLC 1 |  TLC 2 | TLC 4 | TLC 8 | native 1 | Wasm 1 |
| ----------------- | -----: | -----: | ----: | ----: | -------: | -----: |
| Counter product   |  6.452 |  6.929 | 7.978 | 6.994 |    2.948 |  4.620 |
| Function cube     | 12.814 |  7.850 | 5.307 | 4.721 |    8.888 | 18.167 |
| Parallel channels | 24.076 | 13.553 | 9.065 | 6.273 |   19.864 | 29.924 |
| Linear ring       |  0.930 |  1.343 | 1.371 | 1.438 |    0.190 |  0.254 |

各入力で測ったTLC設定のうち速かったものを使うと、nativeはcounterで約2.19倍速い。cubeではTLCが約1.88倍、channelでは約3.17倍速い。Wasmはcounterで約1.40倍速く、cubeではTLCが約3.85倍、channelでは約4.77倍速かった。幅1のringは別の対照として扱い、広いモデルの平均倍率に混ぜない。

3回の範囲はcounterのTLC 1が6.341-6.657 s、nativeが2.853-3.511 s、cubeのTLC 8が4.373-4.824 s、nativeが8.621-8.911 s、channelのTLC 8が6.181-6.349 s、nativeが19.507-22.494 s。全経路の範囲と生サンプルはresultに保存した。1台のホスト・3回の中央値で、全モデルの固定倍率や統計的な優位性は主張しない。

## 探索区間を補助値として分ける

TLCは公開イベント2189の初期生成開始から2193の成功までをstdout受信時刻で測った。初期生成・BFS・最終FP検査などを含み、純粋なBFS kernelではない。modeltはsourceコンパイルとengine準備を除いたcheckSafety相当のAPI時間で、IRのserialize/decode・検証・初期生成・探索を含む。境界が異なるため、両者の区間値をkernel速度比へ変換しない。

| 入力              | TLC 1の区間 | TLC 8の区間 | TLC区間の1→8速度比 | native API | Wasm API |
| ----------------- | ----------: | ----------: | -----------------: | ---------: | -------: |
| Counter product   |       5.856 |       6.351 |             0.92倍 |      2.868 |    4.528 |
| Function cube     |      12.177 |       4.076 |             2.99倍 |      8.804 |   18.075 |
| Parallel channels |      23.469 |       5.669 |             4.14倍 |     19.786 |   29.837 |
| Linear ring       |       0.357 |       0.857 |             0.42倍 |      0.122 |    0.189 |

単位は秒。cubeとchannelでは起動・解析を外した同じTLC区間同士でも並列化の効果が出た。counterは探索幅が広くてもworker追加による改善がなかった。幅だけでworker数を選ぶ条件にはできず、状態展開の費用や共有処理も含めて校正する必要がある。今回の測定だけで、どの共有処理が律速かは断定しない。

## 環境とメモリ

Apple M5、arm64、Node v24.22.2、同じTLC v1.7.4 jar（TLC 2.19 rev 5a47802）とTemurin 21.0.13+9を使った。TLCは-Xmx8g、ParallelGC、ActiveProcessorCount=8、seed16、-lncheck final、-toolを全worker数で固定した。worker数によってJIT/GCの設定を変えていない。Nodeのold-space上限は8 GiB。heap上限を同じRSS制限として扱わず、CPU affinityも固定していない。

RSSは200 ms間隔のsampled max。TLCはJVM、nativeはNodeとRust子プロセス、WasmはNode全体を観測した。速かったTLC設定での観測値を示す。厳密なpeakや必要最小メモリではなく、TLCのheap・FPSet既定値の費用も含む。

| 入力              | TLCのsampled max |  native |    Wasm |
| ----------------- | ---------------: | ------: | ------: |
| Counter product   |         3290 MiB | 423 MiB | 773 MiB |
| Function cube     |         2541 MiB | 913 MiB | 887 MiB |
| Parallel channels |         3106 MiB | 923 MiB | 880 MiB |
| Linear ring       |          543 MiB | 110 MiB | 160 MiB |

modeltの上限はdepth/states=10,000,000、work=4,000,000,000で、全サンプルがcompleteになった。OOM・不完全探索を成功へ置き換えていない。WasmはNode上の実行で、ブラウザの速度の測定ではない。

## 証拠と次の改善

[本測定](../runs/0019-single-model/result.json)には96サンプル、うち集計対象72サンプルを保存した。全サンプルで正常終了・状態数・生成数が式と一致した。input、source/config、コマンド、生のstdout/stderr、イベント時刻、RSS観測、source/binaryのdigestは[manifest](../runs/0019-single-model/manifest.json)と各sampleにある。

[replay](../runs/0019-single-model-replay/result.json)では元の96サンプルのdigest・判定・計数を確認し、縮小版4入力を全6経路の24回で再実行した。大規模入力の独立した性能確認としては数えない。[検証ログ](../runs/0019-verification/result.json)には、共有テスト各71件、縮小版と故障対照、ブラウザ、既存の評価/replay、Rust実験、今回の測定と再実行を残す。

0018の25倍は多検査の起動・再利用を含むthroughputであり、単一モデルの性能へ一般化できない。今のmodeltはscalarで強く、関数・recordを多く扱う単一モデルではTLCの並列探索に負ける条件が確認できた。この負けた入力を次の改善の基準にする。TLCの内部方式と移植上の制約は[並列探索の調査](../../docs/tlc-parallel-adoption.md)に記録した。

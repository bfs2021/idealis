# 0031: Z3の再利用を含めて比較する

[事前計画](../plans/0031-smt-reuse.md)に従い、同じ17入力でZ3のpush/pop経路を追加した。小さいTTL・amount・schedulerのnativeは約3.0-3.4倍、Wasmは約90-103倍速かった。Boolean UNSATには後退とunknownが残るため、差分制約の限定経路を候補とし、Z3の置き換えは保留する。

## 固定した条件

Apple M5、macOS arm64、Node 24.22.2、Rust 1.98.0。Z3 native/Wasmはともに5.4.1、commit `145c0c51570a357cc558ce898ce77573ba9d4fbf`。専有ホストではない。性能の確認用seedを新しく評価した実験ではなく、0030の固定入力の費用を再測定した追加実験。

各経路は1 warmupと5 sample。入力ごとに2/16/32問い合わせをbatchにし、backendの順序をsampleごとに回転する。計1020計測sample、204 warmup、12 cold sample。Rustのwork予算は2,000,000に固定した。SATモデルを取得し、sampleのモデルをタイマー外でZ3へ戻して検査した。

- `parse-solve`はRustの全source解析、Z3 nativeのreset、Z3 Wasmの新規Solver生成を含む。
- `prepared`は解析を除き、同じ式を同じsolverで再検査してモデルを取得する。Rustは回答をcacheしない。
- `scoped`はZ3のlogicと宣言をcaseごとに読み、assertionの解析・push/check/model/popを毎回行う。Rustは全sourceのparse+solveを行う。

Z3 Wasmの`Solver.pop`は`fromString`が保持する宣言とlogicを戻さなかった。最初のAPI対照は再宣言で失敗したため、計時前に条件を修正してplanへ追記した。矛盾する`x=1`、`x=2`、`x=1`を順番に解き、native/Wasmともassertionが混ざらないことを検査した。固定入力のsourceを最初のassertで分割する処理は、汎用importerではない。

## 再利用したZ3との比較

単位はms/queryの中央値。nativeは両方とも常駐プロセスへのJSONL/SMT-LIB輸送、WasmはNode上のJSON ABI/JS APIを含む。倍率は同じ行のZ3/Rustで、大きいほどRustが速い。ブラウザの速度や言語だけの効果へは一般化しない。

| 入力                            | Rust native | Z3 native |   倍率 | Rust Wasm | Z3 Wasm |    倍率 |
| ------------------------------- | ----------: | --------: | -----: | --------: | ------: | ------: |
| ttl                             |      0.0254 |    0.0822 |  3.23× |    0.0159 |  1.6330 | 102.54× |
| amount                          |      0.0319 |    0.1072 |  3.36× |    0.0182 |  1.6436 |  90.20× |
| chain-4-sat                     |      0.0346 |    0.1228 |  3.55× |    0.0216 |  1.7532 |  81.31× |
| chain-4-unsat                   |      0.0378 |    0.0761 |  2.01× |    0.0180 |  1.4633 |  81.16× |
| chain-32-sat                    |      0.0970 |    0.4056 |  4.18× |    0.0812 |  3.1540 |  38.86× |
| chain-32-unsat                  |      0.1030 |    0.3059 |  2.97× |    0.0859 |  2.6774 |  31.17× |
| chain-128-sat                   |      0.7173 |    2.0627 |  2.88× |    0.6056 | 10.2053 |  16.85× |
| chain-128-unsat                 |      0.4892 |    1.2030 |  2.46× |    0.4999 |  7.4878 |  14.98× |
| chain-512-sat                   |      4.1716 |   11.5343 |  2.76× |    5.5372 | 42.1779 |   7.62× |
| chain-512-unsat                 |      3.8072 |    7.8135 |  2.05× |    5.1420 | 35.7314 |   6.95× |
| boolean-4-sat                   |      0.0194 |    0.0760 |  3.92× |    0.0095 |  1.6058 | 168.43× |
| boolean-4-unsat                 |      0.0350 |    0.0317 |  0.91× |    0.0188 |  1.2808 |  68.18× |
| boolean-8-sat                   |      0.0581 |    0.1107 |  1.90× |    0.0201 |  1.6553 |  82.34× |
| boolean-8-unsat                 |      1.4246 |    0.0415 |  0.03× |    1.2975 |  1.2896 |   0.99× |
| boolean-12-sat                  |      0.0563 |    0.1114 |  1.98× |    0.0213 |  1.6923 |  79.40× |
| boolean-12-unsat (Rust unknown) |     38.7781 |    0.0968 | 対象外 |   40.3128 |  1.4833 |  対象外 |
| scheduler                       |      0.0365 |    0.1103 |  3.02× |    0.0173 |  1.7353 | 100.46× |

Boolean 8のUNSATではZ3 nativeが約34倍速かった。Boolean 12のUNSATはRust native/Wasmともwork予算で停止した。unknownの時間を解決時間の倍率に含めていない。

## 同じ式を再検査する場合

preparedは異なる問い合わせを入れ替えるscopedと別の用途。Z3は保持したsolverの情報を再利用できる。以下は同じrunの中央値で、過去の測定は混在させていない。

| 入力            | Rust native (ms) | Z3 native (ms) | Rust Wasm (ms) | Z3 Wasm (ms) |
| --------------- | ---------------: | -------------: | -------------: | -----------: |
| ttl             |           0.0163 |         1.4416 |         0.0060 |       3.4047 |
| amount          |           0.0206 |         0.1669 |         0.0080 |       1.8205 |
| chain-128-sat   |           0.3678 |         0.8903 |         0.3699 |       4.6569 |
| chain-512-unsat |           2.9432 |         0.5903 |         4.5191 |       5.6406 |
| boolean-8-unsat |           1.3715 |         0.0488 |         1.2899 |       1.2967 |
| scheduler       |           0.0161 |         0.2875 |         0.0058 |       2.2996 |

preparedのchain 512 UNSATでもZ3 nativeが約5倍速い。scopedがすべての入力でZ3の最速経路になるわけではない。用途に合う再利用方法を含めて比較する。

## 起動と容量

TTLだけを各3回、fresh processで測った。WasmはNodeの起動を含む。常駐の問い合わせ時間とは別の指標で、OS cacheの排除は行っていない。

| 経路        | 起動込み中央値 (ms) | 生binaryのbytes |
| ----------- | ------------------: | --------------: |
| rust-native |               2.539 |         517,872 |
| z3-native   |               8.746 |      32,981,520 |
| rust-wasm   |              31.163 |         330,749 |
| z3-wasm     |             289.043 |      39,205,260 |

容量はstrip済みのRust実験と配布されたZ3の実体を測った値。Wasmのglueや圧縮後の容量は含めない。対応する理論が異なるので、機能が等しい実装の比較にはしない。

## 判断と次の実験

事前の「複数の小さいfamilyでnative 1.5倍以上、Wasmでも利益」という候補条件を満たした。整数の差分制約を解く限定経路として検討を続ける。探索途中の整数制約の矛盾検出や節学習がない点と、一般のLIAを扱わない点から、公開backendへの組み込みは保留する。

次はTTL以外の実装からSMT問い合わせを収集する。未対応率、unknown率、Z3 fallbackを含めた総実行時間、追加の故障検出を測る。今回の既知TTL故障の再現を、新しい故障を発見した成果には数えない。

## 保存した証拠

[manifest](../runs/0031-smt-scoped-benchmark/manifest.json)に全source/binaryのSHA-256、実行環境、条件を保存した。[全sample](../runs/0031-smt-scoped-benchmark/trials.json)、[全3モードの中央値・最小・最大](../runs/0031-smt-scoped-benchmark/result.json)、[初期対照](../runs/0031-smt-scoped-benchmark/preflight.json)、[cold出力](../runs/0031-smt-scoped-benchmark/cold.json)を参照できる。0030のrunは編集していない。

[検証run](../runs/0031-smt-verification/result.json)に必須検査、API対照の失敗と修正後のログ、公式Z3 archiveのdigestを保存した。Rust sourceとnative/Wasm binaryは0030の確認runから変えていない。変更したのはZ3の再利用と測定のharnessであり、新しい独立した正しさの確認とは数えない。

# 0024: CPU profileを見てscalar評価と関数検索を調整する

Expression::evalと関数検索に時間を使っていたため、leaf評価と整数添字を調整した。独立したWasmの前後比較ではcounterが4.476→4.136秒、ringが272→255 ms。起動込み時間は約8%・6%減った。nativeのCriterionでもcounter、関数、チャネルを短縮した。0023のWasm後退を全条件で解消したという判定はしない。

## profileから選んだ変更

[計画](../plans/0024-profile-tuning.md)を先に固定し、profile保存後に候補を追記してから実装した。profileはcounter M=24、function N=16、channels N=5、ring N=100,000を各16回完全探索した。前後で状態数・生成数・workが全回一致している。

Wasm counterではevalがself sampleの55.5%、visit_branchが29.7%。native 1でもevalが41.3%だった。Wasm channelsのselectは17.2%、functionのintern検索findは28.0%。2項目を試した。

1. `Literal`、`Read`、`Bound`、`Old`を小さいevalへ切り出し、operator本体をinlineしない関数へ分けた。式の順序、型エラー、短絡、workを変えずに、大きいoperator本体へ入るleaf呼び出しを減らす。
2. `select`とEXCEPTのpath検索では、正の整数keyについて`key-1`番目のkeyが一致した場合に直接参照する。疎な整数domainやrecord、0/負数、missing keyは線形検索を使う。domainが連続すると推測して値を返さない。

TLC v1.7.4のFcnRcdValue.select/takeExcept、IF/論理式の短絡と照合した。API/IRとScalarの表現は追加・変更していない。[評価器](../../crates/modelt-core/src/safety_expressions.rs)、[値の検索](../../crates/modelt-core/src/safety_values.rs)を参照。

leaf切り出しはcounter/ringで改善し、採用した。整数位置の変更単独はfunctionで約5-6%改善したが、channels最大8では7.015→7.469 msに後退した。finalは7.361 msで、leafだけの回より遅い。関数の改善と短いdomainでの費用を分け、全ケースで速くなるとは解釈しない。次はdomain幅別にlookupの採用範囲を調べる。

## 計測用binaryと集計

WasmはNode inspectorのCPU profilerを使い、frontendとengine準備後に開始した。1,000 µs間隔。nativeはsamplyの1,000 Hzで別targetのreleaseを記録した。nativeの名前付きbuildはsymbol strippingだけを変更した計測用buildであり、productionの時間として使わない。[Cargoのprofile設定](https://doc.rust-lang.org/cargo/reference/profiles.html#strip)を参照。

名前付きWasmを別profileでbuildした最初の試行は、非custom section不一致で[error](../runs/0024-profile-wasm-before/error.json)になった。関数順も異なり、その名前をそのままproductionへ使えないため検査が拒否した。`cargo rustc --release -- -C strip=none`で作った名前情報はcode/type/import/function sectionが一致した。productionの末尾へそのname sectionだけを加え、非custom sectionのdigestが完全一致するものを記録した。prefixのバイト列は保持する。importによる関数index変更もテストで拒否する。

全profileは同一workloadを16回実行した観測。下はself/全sampleの重みで、CPU使用率や改善率ではない。inline化でsampleの帰属も変わるため、最終判断はprofilerを外した測定による。native最大8の全thread sampleには待機が多く、待機の割合を計算のCPU費用として扱わない。

| 経路     | 対象                         | before self / 全sample | after self / 全sample |
| -------- | ---------------------------- | ---------------------: | --------------------: |
| wasm 1   | counter-product / 評価器     |    1763 / 3175 (55.5%) |   1462 / 2851 (51.3%) |
| wasm 1   | function-cube / 関数選択     |      229 / 4178 (5.5%) |      71 / 3948 (1.8%) |
| wasm 1   | parallel-channels / 関数選択 |     453 / 2637 (17.2%) |     119 / 2604 (4.6%) |
| native 1 | counter-product / 評価器     |    1107 / 2681 (41.3%) |    923 / 2277 (40.5%) |
| native 1 | function-cube / 関数選択     |      327 / 4336 (7.5%) |      68 / 4083 (1.7%) |
| native 1 | parallel-channels / 関数選択 |     256 / 2419 (10.6%) |     104 / 2366 (4.4%) |

[Wasm before](../runs/0024-summary-wasm-before/result.json)、[after](../runs/0024-summary-wasm-after/result.json)、[native before](../runs/0024-summary-native-before/result.json)、[after](../runs/0024-summary-native-after/result.json)にself/inclusiveの全関数を保存した。raw profileと入力、CLI、binary/source hashは各summaryのsaved先にある。nativeはsamplyのsymbolication sidecarも保存した。recursionはinclusiveに1関数1回だけ数え、sample無しを成功にしない。

## profilerを外したCriterion

0023の14本を維持した。sample size 20、warmup 0.5秒、measurement 1秒、noise threshold 3%。IR decode、frontend、IPC、process起動を除いたnative kernel/APIの計測。表は平均msと95% CI。比率は平均同士の比で、比率そのものの信頼区間ではない。指定8はworker数の上限で、counter/ringは既存の規則により逐次実行する。

| benchmark                    |    before 平均 [95% CI] |     final 平均 [95% CI] | before/final |
| ---------------------------- | ----------------------: | ----------------------: | -----------: |
| `search/counter-product/1`   | 12.285 [11.281, 13.524] |    9.266 [9.233, 9.303] |        1.33× |
| `search/counter-product/8`   | 11.152 [11.042, 11.275] |    9.165 [9.142, 9.195] |        1.22× |
| `search/function-cube/1`     | 11.071 [11.008, 11.145] |    9.416 [9.380, 9.456] |        1.18× |
| `search/function-cube/8`     |    8.911 [8.855, 8.980] |    7.707 [7.677, 7.739] |        1.16× |
| `search/linear-ring/1`       |    1.276 [1.266, 1.288] |    1.037 [1.032, 1.042] |        1.23× |
| `search/linear-ring/8`       |    1.233 [1.225, 1.241] |    1.124 [1.063, 1.210] |        1.10× |
| `search/parallel-channels/1` | 14.856 [14.810, 14.911] | 12.821 [12.755, 12.903] |        1.16× |
| `search/parallel-channels/8` |    8.250 [8.213, 8.288] |    7.361 [7.277, 7.469] |        1.12× |

各段階は[before](../runs/0024-criterion-before/result.json)、[leaf](../runs/0024-criterion-leaves/result.json)、[整数位置](../runs/0024-criterion-positions/result.json)、[final](../runs/0024-criterion-final/result.json)。internなど変更外のmicroの差を、単独の改善効果とは数えない。ホストは専有しておらず、ばらつきと後退も残した。

## 大規模Wasm scalarの交互比較

Apple M5 arm64、32 GiB、Node v24.22.2、rustc 1.98.0。beforeは0023のafterとSHA-256が一致する。counter M=48（5,308,416状態、21,233,665生成）とring N=1,000,000（1,000,000状態、1,000,001生成）を使った。

before/afterの順を各回反転し、各条件1 warmup + 3測定、計16 sample。各回fresh Node、通常release Wasmを使い、profile/build/testを並走させていない。表は中央値と括弧内に最小-最大、単位秒。wallはprocess起動から終了とJSON decodeまで。APIはIR validation/serialization、初期状態生成、探索を含み、frontendとengine起動を除く。

| モデル     | before wall 中央値 (最小-最大) | after wall 中央値 (最小-最大) | before/after | API 中央値 before → after |
| ---------- | -----------------------------: | ----------------------------: | -----------: | ------------------------: |
| カウンタ群 |            4.476 (4.279-4.682) |           4.136 (4.059-4.150) |        1.08× |             4.403 → 4.060 |
| 幅1のring  |            0.272 (0.265-0.274) |           0.255 (0.255-0.258) |        1.07× |             0.203 → 0.190 |

[全sample](../runs/0024-scalar-comparison/result.json)と[環境・binary/source hash](../runs/0024-scalar-comparison/manifest.json)を保存した。完全なresultとworkも前後で同一。これはWasm scalarの比較であり、large複合値やnative最大8、TLC/tla-tsの再測定ではない。過去の値を今回と同時の比較へ混ぜない。

## 検証と再実行

[検証記録](../runs/0024-verification/result.json)にRust全target、fmt/clippy、check/typecheck/unit、portable/runtime、実Chromium、TLA差分、241入力×4指定のfrontier対照、TLCの正常/故障、evaluate/replayを保存した。[wide controls](../runs/0024-wide-controls/result.json)24件で、並列化する層のエラー順序、反例、予算を確認した。再生は[16 sampleのdigest/条件/計数を検査し、small正常/故障8件を再実行](../runs/0024-scalar-replay/result.json)した。large性能の独立した再測定ではない。

追加したleafのcontext/短絡/error/work、疎なdomain/record/missing/extreme integerの対照は変更前から通る。性能だけの変更であり、意味の違いを作るためにテストを弱めていない。profileの集計と名前付与はRed→Greenで検査し、import不一致を拒否するRedも保存した。

```sh
just profile-test
just profile-criterion my-stage evals/runs/my-stage
just profile-record wasm <名前を付けたproduction-Wasm> evals/runs/my-wasm-profile
just profile-record native <名前付きnative-release> evals/runs/my-native-profile
just profile-summary <保存profile> evals/runs/my-summary
just profile-scalar evals/runs/my-scalar-comparison
just profile-replay evals/runs/0024-scalar-comparison evals/runs/my-scalar-replay
```

Wasmの名前付与には`evals/profile-tuning/wasm-names.ts`のwithNamesを使う。通常buildのWasmを保存し、同じCargo profileの`cargo rustc --release -- -C strip=none`と`wasm-tools demangle`で名前情報を作る。function codeとindexが一致しないbuildは拒否する。native profilerのDYLD_LIBRARY_PATH=/usr/libはこの起動にだけ指定し、壊れていたlibiconv参照を回避した。システムのtool設定は変更していない。

再実行は保存binary、同じ環境とsourceを必要とする。`eval-0024-before`、`eval-0024-leaves`、`eval-0024-positions`、`eval-0024-final`と、初回profileの`eval-0024-profiles`をtagで保持する。source/binaryが変わればreplayは拒否する。

残る対象はfunctionのintern検索と、短いdomainに対する整数位置checkの費用。さらにoperator側の評価費用は残っており、Wasm scalarの以前の後退を完全に解消したとの判定はしていない。

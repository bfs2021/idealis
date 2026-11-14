# 0023: Criterionによる値・評価・並列探索の改善

Boolean関数とチャネル群で、native/Wasmの探索時間とRSSを削減した。native最大8 workerの起動込み中央値は変更前に比べて1.37倍・1.53倍、Wasmは1.87倍・1.55倍。Wasmのカウンタは22%、ringは7%遅くなった。scalarを含む全モデルの高速化を達成した結果ではない。

## 実装と判断

[計画](../plans/0023-criterion-safety.md)の6項目を順に実装した。IRとpublic APIは追加していない。

| 項目         | 変更                                                                                         | 判断と限界                                                                                                                                     |
| ------------ | -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| intern       | Cell本体を1箇所に保持し、hash bucketとcollision chainから完全比較する                        | 採用。新規関数のmicroと大規模モデルのRSSで改善。hashだけの一致判定にはしない                                                                   |
| EXCEPT       | domainの正規化を保持し、更新済み関数を再ソートしない。変化しない更新は元の値を返す           | 採用。親の配列コピーは残る。逐次更新、@、missing pathの意味を保つ                                                                              |
| membership   | 到達時に定数型をprepareし、recordの所属結果を再利用する。登録済みASTの参照には寿命を制約する | 採用。recordで改善。関数型への所属microは最終回で後退し、全ての所属判定が速くなるわけではない。動的boundに依存する型と評価エラーは再利用しない |
| 候補buffer   | workerごとの平坦な配列を探索中に再利用し、Refを書き換える                                    | 保持。候補ごとのVec確保を除いた。単独の速度効果はばらつきが大きく、独立した寄与は未確定                                                        |
| 重複検査     | canonical化してから一意な新状態のinvariantを確定順に検査する                                 | 採用。候補列と生成数は残す。functionの並列workは168.5M→110.4M、channelsは318.8M→176.0M。値の重複生成と停止前の先行評価は残る                   |
| worker/batch | 軽いscalarと狭い層は逐次にする。候補数からtaskのparent数を調整する                           | 採用。[測定前に規則を固定](../plans/0023-adaptive-policy.md)。指定worker数は上限。counterの改善は逐次選択によるもの                            |

TLC v1.7.4のWorker.addElement/isSeenState/doNextCheckInvariants、FcnRcdValue/RecordValue.takeExceptと照合した。hash衝突、workerごとのRef、短絡とエラー順序、最短反例、global budgetを維持する。`complete`、`bounded`、`empty`、モデル評価エラーをまとめて成功にしない。

## Criterion

Criterion 0.8.2、sample size 20、warmup 0.5秒、measurement 1秒、noise threshold 3%。実行のtimeoutではなく、サンプル収集の期間。privateな実装をsrcからincludeし、計測用のpublic exportを追加していない。microのiter_batched setup/drop、IR decode、TS frontend、IPC、process起動を対象区間から除く。探索ベンチはSafetyProgram.check_parallel全体を測る。

固定した中規模4モデルの状態数/生成数を事前に照合し、1/8 workerと6 microの計14本を実行した。数値はbefore→finalの平均と95% CI、単位ms。比率は平均同士の比で、比率そのものの信頼区間ではない。

| benchmark                          | before 平均 [95% CI] |  final 平均 [95% CI] |  比率 |
| ---------------------------------- | -------------------: | -------------------: | ----: |
| `values/intern-new-functions`      | 0.937 [0.893, 0.993] | 0.530 [0.524, 0.538] | 1.77× |
| `values/intern-existing-functions` | 0.473 [0.470, 0.476] | 0.491 [0.487, 0.495] | 0.96× |
| `values/except-function`           | 1.307 [1.303, 1.312] | 0.628 [0.553, 0.741] | 2.08× |
| `values/except-record`             | 8.183 [8.156, 8.211] | 4.785 [4.651, 4.918] | 1.71× |
| `membership/function-cube`         | 2.269 [2.260, 2.280] | 2.739 [2.641, 2.846] | 0.83× |
| `membership/parallel-channels`     | 2.737 [2.721, 2.754] | 1.043 [1.026, 1.064] | 2.62× |

段階ごとの生sample・推定値・ソースhashは以下に保存した。bufferの回では、変更していない逐次functionも10.8→44.5 msに変動した。この回だけでbufferの効果や後退を断定しない。固定の共有ホスト上の探索的測定であり、マシン全体のばらつきを除いた結果ではない。

- [before](../runs/0023-criterion-before/result.json)
- [intern](../runs/0023-criterion-intern/result.json)
- [except](../runs/0023-criterion-except/result.json)
- [membership-fixed](../runs/0023-criterion-membership-fixed/result.json)
- [buffer](../runs/0023-criterion-buffer/result.json)
- [dedup](../runs/0023-criterion-dedup/result.json)
- [adaptive](../runs/0023-criterion-adaptive/result.json)
- [final](../runs/0023-criterion-final/result.json)

最初の[membership回](../runs/0023-criterion-membership/result.json)は集計失敗。保存先の判定がbenchmark group名にも一致し、14本の計測に対して22件の推定値を収集した。errorを残して判定を修正し、membership-fixedを別runとして測った。未登録ASTの構造hashを毎回計算する費用も確認し、登録できる参照には寿命を付けてlookupを軽くした。

## 大規模モデルの変更前後

Apple M5 arm64、32 GiB、Node v24.22.2、rustc 1.98.0。0022と同じlarge 4入力を使った。beforeのnative/Wasmは[0022のmanifest](../runs/0022-readme-comparison/manifest.json)とSHA-256が一致する。afterと交互に、各条件1 warmup + 3測定、計96 sampleを実行した。測定中にbuild/test/replayを並走させていない。ホストは専有しておらず、時間の範囲も併記する。

以下はprocess起動から終了・JSON decodeまでの中央値。WasmはNode上の実行。比率はbefore/afterで、大きいほど短縮。TLC/tla-tsはこの回に再測定していない。

| モデル      | native 1: 変更前 → 後 (秒) |  比率 | native 最大8: 変更前 → 後 (秒) |  比率 | Wasm 1: 変更前 → 後 (秒) |  比率 |
| ----------- | -------------------------: | ----: | -----------------------------: | ----: | -----------------------: | ----: |
| カウンタ群  |              3.114 → 3.061 | 1.02× |                  3.781 → 2.996 | 1.26× |            3.944 → 4.822 | 0.82× |
| Boolean関数 |              9.609 → 7.221 | 1.33× |                  5.407 → 3.942 | 1.37× |          19.809 → 10.585 | 1.87× |
| チャネル群  |            22.260 → 17.838 | 1.25× |                 13.500 → 8.829 | 1.53× |          38.771 → 24.964 | 1.55× |
| 幅1のring   |              0.213 → 0.215 | 0.99× |                  0.223 → 0.217 | 1.03× |            0.268 → 0.286 | 0.94× |

API区間はIR validation、serialization、初期状態生成、探索を含み、frontendとengine起動を除く。値は秒。

| モデル      | 経路     | before wall 最小–最大 | after wall 最小–最大 | API 中央値 before → after |
| ----------- | -------- | --------------------: | -------------------: | ------------------------: |
| カウンタ群  | native-1 |           2.951–3.387 |          3.000–3.439 |             3.040 → 2.994 |
| カウンタ群  | native-8 |           3.684–3.791 |          2.974–3.241 |             3.712 → 2.927 |
| カウンタ群  | wasm-1   |           3.846–4.529 |          4.391–7.429 |             3.871 → 4.735 |
| Boolean関数 | native-1 |           9.460–9.935 |          7.084–8.391 |             9.529 → 7.150 |
| Boolean関数 | native-8 |           5.352–5.697 |          3.818–4.293 |             5.330 → 3.868 |
| Boolean関数 | wasm-1   |         19.792–19.879 |        10.379–10.654 |           19.716 → 10.506 |
| チャネル群  | native-1 |         21.014–34.846 |        17.298–21.528 |           22.181 → 17.762 |
| チャネル群  | native-8 |         11.501–14.275 |          7.288–9.401 |            13.407 → 8.731 |
| チャネル群  | wasm-1   |         33.194–41.864 |        22.773–26.607 |           38.647 → 24.877 |
| 幅1のring   | native-1 |           0.212–0.215 |          0.211–0.218 |             0.139 → 0.140 |
| 幅1のring   | native-8 |           0.216–0.231 |          0.212–0.227 |             0.149 → 0.141 |
| 幅1のring   | wasm-1   |           0.260–0.268 |          0.284–0.288 |             0.192 → 0.212 |

RSSは200 msごとの観測最大値で、厳密なpeakではない。nativeはNodeと直下のmodelt-core processを合算する。WasmはNode全体を観測する。各条件の測定3回から最大値を選んだ。単位MiB。

| モデル      | 経路     | RSS before → after | 削減率 |
| ----------- | -------- | -----------------: | -----: |
| Boolean関数 | native-1 |      914.9 → 552.5 |  39.6% |
| Boolean関数 | native-8 |      917.0 → 556.2 |  39.4% |
| Boolean関数 | wasm-1   |      887.0 → 541.6 |  38.9% |
| チャネル群  | native-1 |      923.1 → 604.8 |  34.5% |
| チャネル群  | native-8 |      928.9 → 608.1 |  34.5% |
| チャネル群  | wasm-1   |      878.5 → 659.0 |  25.0% |

[全96 sample・集計](../runs/0023-tuning-comparison/result.json)、[環境・binary・ソースhash](../runs/0023-tuning-comparison/manifest.json)を保存した。Wasm scalarの後退原因は未特定。複合値での効果と区別し、次のprofile対象とする。

## 正しさと再現

- Rust全target、fmt/clippy、TS check/typecheck/unit、実Chromium、portable/runtime、TLA safety差分、241入力×4 worker指定の対照、TLCのsmall正常/故障対照を通過した。[検証記録](../runs/0023-verification/result.json)を参照。
- [replay](../runs/0023-tuning-replay/result.json)で96 sampleのdigest・条件・計数・コマンドと保存値を照合し、同じbinaryでsmall正常/故障48件を再実行した。これはlarge性能の独立した再測定ではない。
- [wide controls](../runs/0023-wide-controls-fixed/result.json)は、幅64のエラー/違反/デッドロック、深さ6の関数反例、state/depth/work制限を2/4/8 workerで計24件検査した。狭いモデルが逐次選択される対照だけに依存しない。
- [wide controls初回](../runs/0023-wide-controls/result.json)はharnessの予算設定ミス。maxWork=1,000,000で完走するモデルをboundedと期待した。失敗を保存し、逐次版が実際にboundedになる事前条件とmaxWork=10,000を使って別runへ再実行した。
- intern/EXCEPTは既存の意味を変えないため、意味論の対照は変更前も通る。性能の基準は保存したbefore。membershipは変更前のwork 25対期待5、dedupはwork 16,997対16,387でRedを保存し、変更後の一致を確認した。実装が無い段階のcompile failureとは区別する。

通常の計測とCI smoke:

```sh
just criterion-test
just criterion-save my-stage evals/runs/my-stage
```

入力はprepareで再コンパイルし、checked-in fixtureとの意味の一致を確認する。fixtureを同梱したため、未生成の.artifactsやcrate単独のcheckoutでもRustのbench smokeを実行できる。Criterionはnativeのdev dependencyで、Wasm runtimeに入らない。

今回の保存binaryと同じ環境での再実行:

```sh
just criterion-compare evals/runs/new-comparison
just criterion-replay evals/runs/0023-tuning-comparison evals/runs/new-replay
```

比較用beforeは`.artifacts/criterion-safety/before/`に必要。今回は0022のbinary hashを固定したため、別環境でbuildしたbinaryを同一の証拠と扱わない。測定時のsourceは`eval-0023-compare`、各Criterion段階は`eval-0023-<stage>`のtagで保持する。replayがsource/binary差分を拒否した場合は、その記録のtagと環境へ戻す。

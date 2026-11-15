# 0021: workerごとのwork予算予約

nativeの単一モデル並列探索に、workerごとのwork creditを採用した。共通予算から256 tick分を予約し、通常の消費はworker専用のcache lineで行う。未使用creditは他workerも消費できる。予約と移動をMutexで同期し、移動中のcreditを消滅した予算として扱わない。式評価全体やArenaをMutexで囲まない。

## 固定条件と全体時間

[0021の計画](../plans/0021-worker-budget.md)に従い、0020と同じ4入力、legacy / parallel 1/2/4/8、1 warmup + 3測定、最大64状態/workerのbatchで測った。入力、反復数、quotaを途中で変えていない。Node 24.22.2、Apple M5 arm64、32 GiB、探索上限は0019と同じ。sourceは`8c14881a90916e44d20ff06d1e5d5308937da03b`、sourceVersionは`9837290521bb9207b0926e8fd5037bb915517b2e6787176faae03a93b9397771`。開始時のworking treeはclean。build・test・replayを並行させず、元の測定プロセスの終了を観測してから次の検証を始めた。

全80 sampleでcomplete、解析上の状態数・生成数が一致した。0020との80組すべてでwork量も一致した。以下は起動・source解析・JSON処理を含む中央値、単位は秒。

| 入力              | legacy | parallel 1 | parallel 2 | parallel 4 | parallel 8 |
| ----------------- | -----: | ---------: | ---------: | ---------: | ---------: |
| counter-product   |  3.153 |      3.020 |      5.545 |      3.570 |      3.673 |
| function-cube     | 10.161 |      9.853 |     13.118 |      6.389 |      5.548 |
| parallel-channels | 24.615 |     22.713 |     29.148 |     17.756 |     12.506 |
| linear-ring       |  0.238 |      0.244 |      0.234 |      0.226 |      0.231 |

同APIのparallel 1→8は、function-cubeが約1.78倍、parallel-channelsが約1.82倍に短縮した。counter-productは約22%遅くなる。linear-ringは幅1なのでpoolを作らず、parallelのworker数を増やしても内部並列探索を実行しない。これを並列化の速度改善として数えない。

parallel-channelsの8 workerは11.056〜22.491 sと幅がある。median 12.506 sだけから一定の速度改善を保証しない。function-cubeの8 workerは5.232〜6.089 s。反復は各3回で、min/maxを削除せず[全sample](../runs/0021-worker-budget/result.json)に残した。2 workerは3つの広い入力すべてでparallel 1より遅い。worker数だけから線形な短縮を予測しない。

## API区間・work・メモリ

API区間はIRの検証、Init生成、探索とpool起動・終了を含む。source解析とnative process起動は除く。RSSは200 ms間隔でNodeとRust子processを合算したsample最大値で、厳密なpeakや強制メモリ上限ではない。

| 入力              | worker | API中央値 s |        work | sample最大RSS MiB |
| ----------------- | -----: | ----------: | ----------: | ----------------: |
| counter-product   |      1 |       2.948 | 106,168,325 |             410.9 |
| counter-product   |      8 |       3.606 | 106,168,325 |             431.5 |
| function-cube     |      1 |       9.749 | 109,051,928 |             915.0 |
| function-cube     |      8 |       5.476 | 168,546,943 |             916.9 |
| parallel-channels |      1 |      22.629 | 231,735,339 |             923.2 |
| parallel-channels |      8 |      12.418 | 318,785,521 |             930.1 |

関数/recordのwork増加は、同じwaveで重複する新状態のinvariantを複数workerが評価するため。この費用も全体予算へ課金する。状態数・生成数の増加による速度差ではない。credit予約前後で同じwork量のまま、8 workerの全体時間はcounter 19.011→3.673 s、function 29.255→5.548 s、channels 54.182→12.506 sになった。0020と0021は別時点の測定であり、同時刻の対照ではない。

0019のnative逐次はfunction 8.888 s、channels 19.864 sだった。今回のlegacyは10.161 sと24.615 sなので、保持方式の変更・実行時の変動を含む後退も残る。この比較だけでArena変更の寄与を分離できない。同じsourceのparallel 1とlegacyも別sampleで、時間の差をAPIの計算内容の差と解釈しない。両者は全入力で同一の逐次経路とworkを返す。

## TLCと採用判断

0019で測ったTLC 8 workerのfunction 4.721 s、channels 6.273 sに対し、今回のnative 8 workerは5.548 sと12.506 s。これらの過去値に対してもTLCの方が速い。TLCを今回同時に再測定してはいないので参考値であり、modeltがTLC全般より速くなったという結果ではない。

採用するのは、明示的なnativeの並列経路とworkerごとの予算予約。defaultの`check` / `check-safety`は逐次を使い、`check_parallel(bounds, workers)` / `check-safety-parallel`で選ぶ。action property・temporal property・fairnessは逐次へfallbackする。Wasmの単一モデル内部並列化は未実装で、native専用requestは拒否する。

次の改善候補は重複するinvariant評価、値のcanonical化、coordinatorのcommit費用。今回これらの内訳をprofileした結果はなく、律速の確定とはしない。人工的な有限モデル4件での評価であり、未知の実装バグの検出やすべてのTLA+仕様への一般化は評価していない。

## 検証と再生

共有予算より大きなconsume、予約単位より小さい予算、他workerの未使用credit、8 threadでの同時消費、正確なbudget境界をRustで確認した。既存のdiamond、local Ref alias、hash衝突、最短traceの反復、depth/state/work境界も通過した。241入力×1/2/4/8 workerの964 reportで、work以外の結果・反例が逐次に一致する。1 workerとaction/temporal/fairness fallbackはworkも一致する。

`just check`、`frontier-safety-test`、`safety-test`、`portable-test`、`runtime-build` / `runtime-test`、`browser`、cargo fmt/clippy、`evaluate` / `replay-evaluation`を実行した。small単一モデルの正常・故障をTLC 1/8 worker・native・Wasmで確認する。Wasmがnative専用requestを拒否する対照もある。

[再生](../runs/0021-worker-budget-replay/result.json)は、保存した全80 sampleのsource/binary/evidence digest、経路・反復・work・判定・計数を検証し、small 4入力×5経路を再実行する。largeの独立した性能再測定ではない。[検証ログ](../runs/0021-verification/result.json)も保存する。0020の幅1への干渉と失敗した最初のreplayは消さず、0021の幅1は単独で測り直した。

```sh
just frontier-safety-test
just frontier-budget-eval /path/to/java /path/to/tla2tools.jar /path/to/jre.tar.gz evals/runs/new-budget-run
just frontier-budget-replay /path/to/java /path/to/tla2tools.jar /path/to/jre.tar.gz evals/runs/new-budget-run evals/runs/new-budget-replay
```

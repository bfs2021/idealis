# 0020: native safetyの層同期並列化

native専用の明示的な並列経路を実装した。既存Arenaの借用、workerのoverlay、canonical Refへの統合、元のBFS順でのcommit、global work予算を持つ。状態のhash衝突時も全状態比較を行い、action/temporal/fairnessは逐次へfallbackする。

## 固定条件と結果

0019と同じ4入力。legacy sequentialとparallel 1/2/4/8、各1 warmup + 3測定。1検査内のpoolを再利用し、batchは最大64状態/worker。sourceは`8d22160015f44324c295b37b80aa4cb2a2ce4853`、sourceVersionは`23778cfbc463cd8d57cfcfd49c7f375df3a0a661ec0591f6a8d8184c5e3911c6`。実行開始時のworking treeはcleanだった。

全80 sampleでcompleteと解析上の状態数・生成数が一致した。各中央値は秒。最後のlinear-ringは測定中に別のbuild/replayを始めたため、速度の判断から除外する。原本のresult.status=completeは判定・計数の一致であり、全timingの隔離を保証しない。[干渉の記録](../runs/0020-timing-overlap/result.json)を参照。

| 入力              | legacy | parallel 1 | parallel 2 | parallel 4 | parallel 8 |
| ----------------- | -----: | ---------: | ---------: | ---------: | ---------: |
| counter-product   |  2.840 |      2.815 |      6.146 |     10.283 |     19.011 |
| function-cube     |  9.199 |      9.448 |     11.519 |     11.519 |     29.255 |
| parallel-channels | 21.506 |     21.666 |     25.480 |     24.342 |     54.182 |

共有AtomicU32を毎tick更新する方式では、3入力すべてで2/4/8 workerがlegacyより遅い。状態数・生成数は増えていないが、wave内で重複した新状態を複数workerが検査するため、関数/recordのwork量も増える。性能の理由でこの版をdefaultにしない。

## 検証と次の変更

Rustのdiamond、最短traceの反復、global budget境界、local Refのalias、既存のhash衝突対照を確認した。TSから241入力を1/2/4/8 workerで検査した964 reportを保存した。work以外の結果は逐次と一致し、1 workerとaction/temporal/fairness fallbackはworkも一致した。Wasmはnative専用requestを拒否する。既存TLCコーパスとsmall単一モデルの正常・故障対照も検査する。

最初のreplayは元のrun終了を確認せず開始したためresult.jsonがまだ存在せず失敗した。[失敗したreplay](../runs/0020-frontier-safety-replay/error.json)を残し、別IDの[再生](../runs/0020-frontier-safety-replay-2/result.json)で全80 sampleのdigest・source・binary・判定を検証した。smallの4入力×5経路を再実行する確認であり、largeの独立した性能再測定ではない。

次は[0021](../plans/0021-worker-budget.md)でworkerごとのcredit予約を検証する。256 tickごとの予約と未使用creditの融通により、共通counterへの毎tickの競合を減らす。BFS、値の統合、batch条件、固定入力と反復数は変えない。

[全測定](../runs/0020-frontier-safety/result.json)、[検証ログ](../runs/0020-verification/result.json)、[方式の説明](../../docs/tlc-parallel-adoption.md)。

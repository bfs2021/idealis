# 0021: workerごとのwork予算予約

0020の測定途中で、counter-productのiteration 1はnative 3.08 s、parallel 2/4/8が6.13/10.17/18.22 sになった。共有workカウンタへの毎tickのCASが競合するという仮説を検証する。0020は中断・編集せず最後まで保存し、この改善は別worktreeで準備する。0020測定中にbuildやtestを走らせない。

## 変更

worker専用のcache lineに残りcreditを保持する。自分のcreditをatomicに消費し、256 tick分をまとめて共通の未予約予算から取得する。共通予算の予約・credit移動だけをMutexで直列化する。他workerの未使用creditも使い切れるようにし、未使用の予約が残っているだけで予算切れにしない。予約中の一時的な移動を残予算0と誤認しない。使用workの合計と返却結果は実消費を数え、maxWorkを超えない。

変更対象はworkの配分のみ。BFSの層・batch最大64状態/worker・commit順・値の統合・対応fragment・public APIを0020から変えない。workerごとの候補生成やinvariantの重複評価への課金も維持する。

## 検証と測定

quotaより小さい全体予算、他workerに予約が偏った場合の融通、全creditの消費、複数threadでの同時消費、consumeの部分消費、maxWorkの正確な境界をテストする。0020のRust/TS差分コーパスと既存TLC対照、Wasm・ブラウザ・portable・runtime検証を再実行する。

0020完了後、0020と同じ4入力・legacy/parallel 1/2/4/8・1 warmup + 3測定・回転順・上限・全体時間/API時間・200 ms RSS samplingで新runへ測定する。入力・反復数・batchサイズ・quotaを結果に合わせて変更しない。0020は別時点の測定として区別する。0019のTLC値も同時に測った新値とは称さない。completeの状態数・生成数、部分結果、反例、work、メモリ、退行を残す。

改善後も関数/recordで逐次を上回らなければopt-inとして残し、自動選択は導入しない。安全性の一致と速度改善を別々に判断する。source/binary digestを固定し、replayは全保存データを検証した上でsmall対照を再実行する。独立したlarge性能再測定とは区別する。

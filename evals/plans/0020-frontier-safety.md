# 0020: native safety探索の層同期並列化

0019の固定入力とTLC v1.7.4のWorker/StateQueue/HeapBasedDiskFPSetを根拠に、nativeに単一モデルの並列経路を追加する。採用する方式は、同じ深さの状態を小さなbatchに分けて展開し、元のBFS順でcommitする層同期BFS。Wasmと既存の逐次APIは維持する。

## 実装条件

- 読み取り専用の既存Arenaを借用し、workerは新規値だけをlocalなoverlayへ保持する。Arena全体のcloneや評価全体のMutexを使わない。commit時にlocal Refをcanonical Refへ変換し、hash衝突時の全状態比較を保つ。
- workerは遷移とstate invariantを評価する。元の親・action・列挙順に結果をcommitし、検査前の状態を次の層で探索しない。型エラーと不変条件違反の順序、deadlock、最短反例、depth/state上限を維持する。
- maxWorkは実際の並列評価に使う全worker共通の予算。重複した新状態のinvariantや停止前の先行評価も課金する。このためwork量と予算境界での部分結果は逐次と同一とは限らない。予算をworker数倍せず、使ったworkが上限を超えたcompleteを返さない。
- action property・temporal property・fairnessを含むモデルは逐次へfallbackする。worker数1も既存の逐次経路を使う。並列経路はnative専用の明示的なrequest/APIで選ぶ。
- Rayonのnative専用poolを1検査内で再利用する。幅1ではpoolを起動しない。出力は小さなbatchに制限するが、既存の1状態あたりの後続状態全体を保持する費用は別途記録する。

## Red / Greenと意味論対照

最初に欠落APIで失敗するRustテストを保存する。diamond・異なるlocal Refの複合値・深さの異なる違反・self loop/閉路とdeadlock・空Init・assumption・型エラーと先行違反・global work/state/depth境界・action/temporal fallback・不正worker数を検査する。既存のTLC差分入力とsmall単一モデルをnative 1/2/4/8 workerで照合する。並列のwork以外の完全探索結果・違反traceは逐次と照合し、TLCも正常/故障の対照に使う。worker failureは成功へ変換しない。

## 固定性能条件

0019のlarge 4入力を変更しない。legacy sequential/native parallel 1/2/4/8を各1 warmup + 3測定、順番を回転して単独実行する。Node/JVM条件・探索上限・生ログ/RSS/全体時間・API時間の区別は0019を引き継ぐ。TLCの入力は変えていないので0019の値は参考値として区別し、今回の新測定と同時実行した比較と称さない。必要なら同じTLC条件で新runへ再測定する。

値保持方式の1worker後退、parallel 2/4/8の実測、work増加、メモリ増加を保存する。速くなることを合格条件にせず、関数/recordが遅い場合は明示的なopt-inのままにし、defaultの自動選択を追加しない。結果を見て入力や反復数を縮めず、修正後は新しいrunへ記録する。

確認はjust check/browser、portable-build/portable-test、runtime-build/runtime-test、safety-test/TLC対照、cargo fmt/clippy、evaluate/replay-evaluation。新規runは不変とし、replayは保存した入力・source digest・binary digest・判定を検証してsmall対照を再実行する。性能の独立した再測定とsmall再生を区別する。

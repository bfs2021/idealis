# 0044: Quintが生成したTLA+の実行確認

PR #52〜#54を最新mainへrebaseし、Quintの実コンパイラがApalache経由で出力したTLA+をrtla → rtla-compiler → rtlcで実行する。手書きのApalache fold / CASEの回帰だけからQuint出力への対応を推測しない。Quint 0.33.1を固定し、使用したApalache、TLC、ソース、生成物と実行binaryの版・SHA-256を記録する。

結果を見る前に、有限counter、整数集合fold、順序に依存するsequence fold、有限非決定選択、sum typeのmatch、入れ子foldの6入力を固定する。各入力に正常なinvariantと違反するinvariantを用意し、同じ生成物の設定だけを切り替える。counterの違反は初期状態以降に到達させる。他の入力では結果値の間違いを検出する対照も含める。新しい意味論や対応範囲が必要なら、観測した失敗をRedとして保存してから修正する。入力の変更や未対応は隠さず記録する。

生成TLA+の書き換え、foldの手展開、CASEの置換はしない。実行用TLC設定と標準モジュールの配置は別ファイルで用意する。TLCの完全探索・状態数・生成数・違反と反例を対照に、idealis native/Wasmの結果と反例を確認する。探索予算はmaxDepth 64、maxStates 10000、maxWork 1000000。unknown・empty・予算切れ・実行基盤エラーは成功へ変えない。対応できない生成構文は理由を記録する。

生成元のQuint、生成TLA+、設定、コマンド、stdout/stderr、判定を保存し、保存物からの再生を回帰にする。再生成にはQuint/Apalache/JVMが必要だが、保存した生成物を検査する通常の回帰からは分離する。正常・違反の両方が実行され、TLCとnative/Wasmが一致して初めて、その入力の対応を確認済みとする。Quint全体やApalacheの全operatorへの互換性、性能改善は主張しない。

compiler-test、rtlcのfmt/clippy/test、portable-test、check/browserを実行する。評価ハーネスを追加した場合はevaluate/replay-evaluationも実行する。結果は新しいrunへ保存して再生し、既存のrunは変更しない。今回はrebaseと実行確認であり、mergeやregistry公開は行わない。

# 0040: SMTの式とcontext APIを拡張する

チェックリストを追加し、定数倍、整数ite、Rust/JSONのassert/push/popとBool literal assumptionsを実装する。無条件の定数代入後、各分岐が整数差分制約または同じ係数でscaledされた差分制約へ落ちる範囲に限定する。一般LIA・他理論・CDCL・学習状態の再利用・SMT-LIB script全体は今回の採用範囲へ含めない。

係数も任意精度とし、正負・0・厳密比較・割り切れない負のboundを検査する。整数iteはguard付きの有限選択肢へ変換し、演算や比較で最大256選択肢に制限する。混在sortと未対応式はerror、展開数の超過はresource errorとする。空and/orはZ3互換なtrue/falseへ変換する。

既存ProblemとJSON solveを保ち、mutableなSolver contextを追加する。prepare後のassertは単一のBool式だけを受け付け、失敗時は元のcontextを保つ。push/popはassertion scopeを管理し、最大256 frameとする。pop underflowはerror。変更時にsourceを再解析し、check時は現在の解析結果を使う。宣言は初期sourceに固定する。これは機能としてのincremental APIで、学習・理論探索の再利用や高速化の主張ではない。

assumptionsは宣言済みBool symbolと正負の組。check内だけで有効にし、未宣言/Int symbol/不正shape/件数超過はerrorにする。上限256。work 0は有効な問い合わせにunknownを返し、modelを付けない。通常check・別handle・pop後へ代入を漏らさない。SMT-LIB check-sat-assumingコマンドの実装済みとは数えない。

実装前のnative/Wasmを固定baselineとして保存し、新規入力の対応率を測る。Z3 5.4.1を独立オラクルとし、同じ式・意図したassertion stackとassumptionsで判定を比較する。SATのRustモデルを元のsourceへ代入してZ3で受理を確認する。native/Wasmの応答も比較する。UNSATはZ3との一致で検査し、unknown/未対応を合格やUNSATへ置き換えない。

開発seed 0x40a1で64件、確認seed 0x40c3で128件を予約する。同じsourceで手書き境界・生成式・context transcriptを実行し、確認には同じ実装と条件を使う。回帰seedは0x4001。source/依存lock/baselineと新binaryのhash、全入力/応答/stack snapshot/Z3結果、失敗・成功ログを新規runへ保存して再生する。SAT/UNSATの両方を含める。実行時間は診断費用として記録し、速度比較とは扱わない。

Red/Green、fmt/clippy/Rust tests、node/Vitest、実Chromium、既存SMT差分、check/browser、runtime-build/test、evaluate/replay、packageと単独consumerを検査する。公開core/backendやSUTを変更する場合だけ追加の適切なチェックを行う。SMT-LIBの意味は公式Core/Ints/参照文書とZ3で確かめ、変更した条件は別plan/runに残す。PRは1本にまとめ、mergeとregistry publishは行わない。

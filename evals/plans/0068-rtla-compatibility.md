# 0068: rtlaとSANYの構文互換性

基準はa242dcbc。rtlaの字句・構文・ASTを対象とし、rtla-compilerの名前解決/level/IR変換、rtlcの探索とは分ける。既存199 source/config組のtla-ts AST比較とSANY直接対照18件は、全TLA+互換の根拠にはならない。

tlaplus/tlaplusのローカルclone d8fb231de4e4b5d02c1b2ed1ac759c086f694021の公式構文corpus（42 txt、355ケース）を全件読み、元の入力・期待AST・error/skip属性・出典とhashを保持する。参照は固定TLC 1.7.4 jar。corpusの世代とjarの世代を区別し、upstreamの期待ラベルと実際のSANY受理の差をrtlaの故障と混同しない。

SANYのCompilationUnitを直接呼ぶ参照adapterをevals側だけで使う。CLIのmodule読込や意味解析により、純粋な構文検査が失敗したように見えることを防ぐ。外部JDK/JVMは任意の比較実行にだけ必要で、rtlaのnative/Wasmの依存には加えない。nativeとWasmの全応答を照合し、SANYのsyntax拒否、rtlaのsyntax/unsupported、process失敗を分ける。監査完了を互換性の全件合格へ置き換えない。

初期の焦点はSUBSET/UNION/DOMAINのprefix優先順位。SANY Operators.javaの8/8、8/8、9/9と現実装10/13の差を、結合順と受理/拒否の両方で確認する。局所的な既存対応の不一致はRedを保存して修正する。LAMBDA、proof、RECURSIVEや全operator語彙を一度に実装しない。

公式corpusは受理/拒否を比較し、ASTの正しさ全体の証明とは区別する。別にprefixの括弧あり/なし・曖昧な境界の固定対照を置き、構文木の結合とTLCで観測できる有限モデルを確認する。誤受理・正しい入力の拒否・ASTの誤結合・仕様上の未対応を分類する。

raw runは上書きせず、before/afterのsource/binary、参照adapter、JDK/jar/corpusのhashと全出力を保存する。SUPPORTへ対応と残る穴をchecklistで反映し、Jevのlive/offline/stagedを同期する。終了前はcheck/browser、rtla native/Wasm/実ブラウザ、portable/compilerの回帰、fmt/clippy、runtime build/test、evaluate/replay、配布consumerを検査する。今回のsyntax監査にTLC config全体やSANYの意味解析の互換性を含めない。

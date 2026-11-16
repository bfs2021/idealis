# rtlaの対応チェックリスト

チェック済みは構文解析とASTの保持に対応する範囲。評価・名前解決やモデル検査の対応とは別。未チェックは未対応またはこのcrateの対象外で、実装予定を約束しない。

## 対応済み

- [x] Module・Expression・TLC Configの型付きASTと、Rustのparse_module / parse_expression / parse_config。
- [x] MODULE、EXTENDS、変数・定数、非再帰operator、関数定義、ASSUME、THEOREMの構文。
- [x] 算術・比較・論理、集合・tuple・record・関数、量化とtuple pattern、EXCEPTと@。
- [x] LET、IF、OTHERを省略したCASE、prime、UNCHANGED、ENABLED、temporal/fairnessの構文。
- [x] infixの優先順位、括弧、コメント、文字列、bulleted junctionとoffside規則。Apalacheの`:=`もASTとして解析する。
- [x] INIT/NEXT/SPECIFICATION、CONSTANT、INVARIANT/PROPERTY、constraintや置換の設定値の保持。実行の可否はcompilerが判断する。
- [x] native JSONLとホストimport不要のWasm ABI v1。
- [x] syntax/unsupportedを区別した位置付きエラー、整数表現と再帰・AST深さの上限検査。

## 非対応・対象外

- [ ] 外部モジュールの読み込み、名前解決、型/level検査、標準operatorの評価。
- [ ] ASTから検査IRへの変換、状態探索、invariantの評価。
- [ ] proof、INSTANCE、nested module、RECURSIVE宣言、一般の高階引数。
- [ ] 全TLA+構文やSANYとの完全互換。

## 制約と根拠

- [x] AST整数は正確に表現できる`2^53-1`まで。再帰的解析とASTの深さは64段までで、TLA+の数値領域自体の制約とは区別する。
- [x] 構文上保持できる設定でも、後段で未対応になる場合を保持する。

[README](README.md)、[公開境界](src/lib.rs)、[parser](src/parser.rs)、[回帰](tests/parser.rs)、[SANYとの対照](../../evals/iterations/0026-rtla-parser.md)を参照。

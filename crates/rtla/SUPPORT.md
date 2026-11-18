# rtlaの対応チェックリスト

チェック済みは構文解析とASTの保持に対応する範囲。評価・名前解決やモデル検査の対応とは別。未チェックは未対応またはこのcrateの対象外で、実装予定を約束しない。

## 対応済み

- [x] Module・Expression・TLC Configの型付きASTと、Rustのparse_module / parse_expression / parse_config。
- [x] MODULE、EXTENDS、変数・定数、通常の名前を持つ非再帰operator、関数定義、ASSUME、THEOREMの構文。
- [x] 既存operator語彙のprefix/infix/postfix定義、記号の定数・formal宣言、operator引数と修飾nonfix参照を既存のDefinition/Declaration/Apply/Identifierへ保持する。変数宣言は識別子に限る。
- [x] 算術・比較・論理、集合・tuple・record・関数、量化とtuple pattern、EXCEPTと@。
- [x] LET、IF、OTHERを省略したCASE、prime、UNCHANGED、ENABLED、temporal/fairnessの構文。
- [x] top-levelの引数なしnamed/unnamed INSTANCE、LOCAL、WITHの式置換、A!B!Op形式の修飾名をASTへ保持する。
- [x] infixの優先順位、括弧、コメント、文字列、bulleted junctionとoffside規則。Apalacheの`:=`もASTとして解析する。
- [x] 周囲のinfix/prefixの結合を変える非括弧のlabelをsyntax拒否する。優先順位を保つbody、label/body全体の明示括弧は保持する。
- [x] SUBSET/UNION/DOMAINのprefix優先順位は1.7.4の10..13を維持する。1.8.0の8/8・8/8・9/9とは区別する。
- [x] INIT/NEXT/SPECIFICATION、CONSTANT、INVARIANT/PROPERTY、constraintや置換の設定値の保持。実行の可否はcompilerが判断する。
- [x] native JSONLとホストimport不要のWasm ABI v1。
- [x] syntax/unsupportedを区別した位置付きエラー、整数表現と再帰・AST深さの上限検査。

## 非対応・対象外

- [ ] 外部モジュールの読み込み、名前解決、型/level検査、標準operatorの評価。
- [ ] ASTから検査IRへの変換、状態探索、invariantの評価。
- [ ] proof、引数付きINSTANCE、LET内のINSTANCE、nested module、RECURSIVE宣言、LAMBDA。
- [ ] 全operator語彙、記号をWITH左辺に使う置換、labelの引数、一般のsubexpression selector。
- [ ] 1.8.0のprefix優先順位を選ぶsyntax dialect、全Unicode operator/bullet/CASE alias。
- [ ] 対応外だがSANYが受理する全構文を、一貫してunsupportedへ分類する診断。
- [ ] 全TLA+構文やSANYとの完全互換。

## 制約と根拠

- [x] AST整数は正確に表現できる`2^53-1`まで。再帰的解析とASTの深さは64段までで、TLA+の数値領域自体の制約とは区別する。
- [x] 構文上保持できる設定でも、後段で未対応になる場合を保持する。

[README](README.md)、[公開境界](src/lib.rs)、[parser](src/parser.rs)、[回帰](tests/parser.rs)、[SANYとの対照](../../evals/iterations/0026-rtla-parser.md)を参照。

[355公式case・13 prefix対照の二版比較](../../evals/iterations/0068-rtla-compatibility.md)と[labelの回帰](tests/labels.rs)を参照。受理率を言語全体の対応率やAST全体の正しさへ読み替えない。

[記号operatorの構文と実行の比較](../../evals/iterations/0069-symbolic-operators.md)、[構文回帰](tests/symbolic.rs)と[固定入力](tests/fixtures/symbolic.json)を参照。記号定義の構文受理と、builtinとの衝突や高階operatorの実行可否を分ける。

# rtla-compilerの対応チェックリスト

チェック済みはrtlaのASTをrtlcの有限検査IRへ変換する範囲。未チェックは未対応または対象外で、実装予定を約束しない。構文解析と探索は他crateが担当する。

## 対応済み

- [x] 共有IRの統合でも既存v1のSYMMETRY設定を保持する。明示v2のSYMMETRYはlowering前に理由を示して拒否する。

- [x] `CompileOptions.irVersion`でv1を既定・明示選択し、明示v2ではexpression定義を共有するSafetyProgramを返す。省略・明示v1のserialize結果を保持し、v1拒否から自動再試行しない。
- [x] v2の定義IDを字句宣言順とphase / membership用途 / target signature順、captureをlexical bindingの宣言順で決める。frame-localなbinder slotとcall / parameter / capture参照を生成する。
- [x] 定義graphの循環・level依存解析を明示stackで処理し、LETスコープで宣言ASTを参照共有する。5000定義のcompile-only対照を持ち、実行成功とは区別する。
- [x] 生成的UNCHANGEDとSPECIFICATION subscriptの変数・tuple別名をparameter / capture経由も含め解決する。ENABLED内にも同じ対象解決を使う。v2では要求されるformal/capture targetをsource依存metadataで解析し、signature別に式定義を共有する。foldの専用frame付き呼出しも共有を維持する。
- [x] v2の引数・captureをcaller lexical frame付きの遅延引数としてrtlcへ渡す。値のmemoizationは行わず、call entryと要求時の課金・予算判定はrtlcが担当する。

- [x] 名前解決、定数・非再帰operatorとLETの展開、lexical scopeの保持。
- [x] 既存語彙のcustom記号operatorを通常の定義として解決する。prefix `-.`、infix、postfix `^+`/`^*`/`^#`とnonfix呼出しをIR v1/v2で扱う。
- [x] builtin・対応する標準moduleからimportした記号とのbinding衝突を拒否する。未使用定義とLET/formal宣言も検査し、SANY全体の意味解析とは区別する。
- [x] 明示したModule集合のcompile_modulesで非再帰EXTENDSと引数なしnamed/unnamed INSTANCE、WITH、LOCALを解決する（IR v1。v2は拒否する）。diamond import、定義元のscope、独立したinstanceを保持する。
- [x] v2で連結と設定上書きを要さない入力は、追加moduleを添えてもbundleの検証が通れば、`compile_modules_with_options`が`compile_with_options`と同じrootの変換を行い、変換に成功した場合は同じIRを返す。連結の検証による拒否はv1と同じmessageになる。
- [x] 反例のaction名には公開名・instanceの修飾名・LETの元のoperator名を使い、内部の束縛識別子を出さない。
- [x] signed 32-bit整数、Bool、文字列、有限集合・関数・record・sequenceの対応式とactionの変換。
- [x] 有限量化、tuple pattern、EXCEPT、全変数の代入・選択・UNCHANGED、列挙できるENABLED。
- [x] IFとCASE。最初の真のarmを選び、OTHERがなく未一致なら評価時エラーにする。未評価の分岐ではエラーにしない。
- [x] INIT/NEXTまたは対応形のSPECIFICATION、定数、INVARIANT/PROPERTY、CHECK_DEADLOCK、定数ASSUME。
- [x] 対応するstate/action/temporal propertyとWF/SFをIRへ変換する。
- [x] 定義元moduleのEXTENDS ApalacheでApaFoldSet / ApaFoldSeqLeft。可視の非再帰2引数operatorに限定し、一般の高階引数とは区別する。
- [x] 同じimport下の`:=`をApalache定義の等号として扱う。
- [x] EXTENDS VariantsでApalache 0.63.0のVariant / VariantTag / VariantGetUnsafe。tag取得はsingletonの文字列domainに限定する。
- [x] EXTENDS TLCのimport自体を受理する。任意のTLC operatorを受理する意味ではない。
- [x] 展開・変数・分岐・step等の上限と未対応入力を拒否し、型付きSafetyProgramを返す。

## TLC互換の追加対応

- [x] CONSTANTで与えるmodel valueとtyped model valueを専用のdata値へ変換する。文字列とは区別する。
- [x] 整数の\div・%・^を専用IRへ変換する。評価時の意味論はrtlcが担当する。
- [x] 有限domainに限定したCHOOSEとtuple pattern。無制限CHOOSEは拒否する。
- [x] CONSTRAINT / ACTION_CONSTRAINTを名前付きpredicateへ変換する。
- [x] PROPERTY内の時間論理forall/exists。domainは定数の有限集合に限定し、通常のstate量化と区別する。
- [x] flat moduleのゼロ引数定義Symm == Permutations(Nodes)とTLC import、CONSTANTへ直接与えた2〜64個の異なるuntyped model value集合をoptionalなsymmetry.modelValuesへlowerする。宣言のequivarianceを自動証明せず、実行時の状態形状と組合せはrtlcが制限する。
- [ ] 一般のSYMMETRY設定、複数群・任意の置換・typed model value・module bundle/INSTANCEとの併用。VIEWの設定とlowering。
- [ ] ALIASによる反例表示、POSTCONDITION、POSSIBLE、PERIODIC、RL_REWARD。
- [x] CONSTANTのoperator置換（<-）と同arityのoperator宣言への置換。module別の定義への値・operator・引数付きテーブル設定。
- [ ] operator宣言そのものへの引数付きテーブル設定。比較元TLCがoperator値をstateへ格納してエラーにする入力を未対応として拒否する。
- [x] IR v1でEXTENDS SequencesのSelectSeqとEXTENDS TLCのSortSeqをlowerする。callbackは可視の非再帰operator名に限定し、SelectSeqは1引数、SortSeqは2引数。定義元scope、LET捕捉とprimeを保持する。
- [x] IR v1でEXTENDS TLCのAssertをlowerする。conditionとmessageの両方を評価し、TRUE以外はmessage付きの評価エラーにする。
- [x] SPECIFICATION内の定数有限domain上のforall/exists fairness。tuple・入れ子・空domainと定数引数のwrapperを扱う。state依存domainと量化したInit/Nextの一般的な分解は対象外。
- [ ] TLCのRandomElement、TLCGet、TLCSet。
- [ ] 最小整数リテラル-2147483648のTLC診断互換。Rust frontendの既存の例外受理と比較元TLCには差がある。計算式-2147483647 - 1は対応する。

## 非対応・対象外

- [ ] IR v2のSelectSeq / SortSeq / Assertのsource lowering。手書きIR v2をcheckerが評価できることと、frontendのsource対応を区別する。
- [ ] v2の代入action template共有、self / mutual recursion。代入actionはinlineを維持する。
- [ ] v2のモジュール連結（ユーザーmoduleのEXTENDS・INSTANCE）と設定の上書き（operator置換・定数表・定義の上書き）。要する入力は、v1と共通の連結と検査が通った場合、v1が受理するときも、loweringの前に`safety IR v2 does not support module linking or configuration operator substitution`で拒否する。v1と既定は従来どおり受理する。

- [ ] 引数付きINSTANCE、LET内のINSTANCE、nested module、WITHの一般のoperator引数・変数名以外への変数置換、一般の高階・再帰operator、LAMBDA。
- [ ] 無限集合の列挙、stateに依存するdomainでの時間論理量化、temporal variable quantification（\AA / \EE）。
- [ ] 無制限CHOOSE、他のApalache/Variants operator。
- [ ] 全モジュールのSANY検査、未使用定義やproofの正しさの判断。
- [ ] 一般のTLA+からSMTへの変換、Quint言語の直接解釈、Quint Connect APIやITFのimport。

## 制約と根拠

- [x] frontendの式nesting / bound variable 40、checkerの定義内式48 / 値64、TS decoder128とv2の動的な評価入れ子128を緩めない。call経路長と定義内式のnestingを区別する。
- [x] 集合foldの値種と順序の制約を守る。文字列・recordや一部の関数を拒否する範囲は[README](README.md)に従う。
- [x] 未対応・型エラー・予算切れを検査成功へ読み替えない。探索予算と結果はrtlcの契約。
- [x] source bundleの追加moduleは256、importの深さは40、展開後のsymbolは4096まで。欠落・循環・衝突を拒否し、ファイルシステムやネットワークを読まない。
- [x] Moduleを直接渡す場合もASTの書き換え深さは64段まで。未使用の定義でも過度に深いASTを拒否する。
- [x] 不正な設定の拒否phase・診断や、存在しないmoduleへの値設定をTLCが無視する場合は同一診断を保証しない。未対応拒否とTLCの評価エラー・無視を比較記録で分ける。

[実装](src/lib.rs)、[回帰](tests/compiler.rs)、[追加対応の回帰](tests/finite_support.rs)、[比較入力](tests/finite-support.json)、[標準演算と公平性の回帰](tests/operators.rs)、[その比較入力](tests/operators-support.json)、[Quint生成物の12設定](../../docs/quint-generated.md)、[既存frontendとの比較](../../evals/iterations/0029-rtla-compiler.md)を参照。
[operator共有の回帰](tests/operator_sharing.rs)も参照。
[記号operatorの回帰](tests/symbolic.rs)、[binding検査](src/symbols.rs)と[構文・TLC比較](../../evals/iterations/0069-symbolic-operators.md)も参照。記号の値としての受け渡し、builtin callbackやoperator定数の一般的な実行を追加した意味ではない。

[operator-sharing cohort](../../evals/tlc-differential/operator-sharing-test.ts)は保存TLCログ、TS/Rust構造、native / Wasm完全応答、線形成長と小予算のbounded / workを検査する。固定した有限fragmentと入力での根拠であり、全TLA+互換や指数的な評価仕事の除去、一般的な速度改善は主張しない。

生成的UNCHANGEDはcallerの字句環境で変数/tuple targetを解決する。通常式の引数・captureを外殻へ代入せず、membership用途を伝播する。action/temporal引数はcall siteごとの非共有anonymous frameでcallerのlexical bindingを保持する。LET本体の自由な`@`は型付きprotocol拒否とする。

- [x] 明示v2の非共有anonymous expression/action/temporal frameを扱い、callerのbinder・old・引数を遅延参照する。入口と引数要求へ各1 workを課金し、v1・既存上限を維持する。LET本体の自由な`@`は型付きprotocol拒否とする。

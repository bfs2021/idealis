# rtla-compiler

[対応範囲・非対応のチェックリスト](SUPPORT.md)を参照。

rtlaの型付きModule/Configを、rtlcのSafetyProgramへ変換するRust frontend。parserとcheckerを別crateに保ち、両者の公開契約をつなぐ。Nodeやsolverには依存しない。

`compile(&module, &config)`は名前解決、定数とoperatorの展開、action列挙、temporal/fairnessを変換する。既存TS frontendの有限fragmentを基礎に、Rust経路では[Quint生成物](../../docs/quint-generated.md)の限定した構文も扱う。明示したユーザーモジュール集合と引数なしINSTANCEにも対応する。一般の高階・再帰operator、無限集合の列挙、symmetry、VIEWは扱わない。解析はrtla、実行時の型・IR検査と探索はrtlcが担当する。

内部ではJSONのIR nodeを組み立て、公開境界で型付きSafetyProgramへ変換する。frontendの性能改善を主張する実装ではない。既存TS frontendは独立した差分比較対象として残す。

root moduleの`EXTENDS Apalache`がある場合、`:=`を同モジュールの定義どおり等号として扱う。`EXTENDS Variants`ではApalache 0.63.0の`Variant`、`VariantTag`、`VariantGetUnsafe`を扱い、tag取得はdomainが一つの文字列keyである関数に限定する。他のVariants operatorと無制限CHOOSEは未対応。`EXTENDS TLC`は受理するが、TLCの全operatorを実装するものではない。

CASEは式・actionともOTHERを省略でき、ソース順で最初の真のarmを選ぶ。OTHERがなく全条件が偽なら、TLC v1.7.4 (`5a47802`) と同様に評価時に未一致エラーとソース位置を返す。未評価の分岐内ではエラーにしない。

defining module が直接 `EXTENDS Apalache` を指定するときだけ、`ApaFoldSet(Op, v, S)` と `ApaFoldSeqLeft(Op, v, seq)` を扱う。`Op` は呼び出し位置で可視の top-level / LET 定義の非再帰 operator 名で、零 arity の引数をちょうど2つ持つ必要がある。累積値が第1引数、要素が第2引数。呼び出し位置で body を展開し、bound slot を2つ使う。一般の高階 operator は未対応のままで、この2つの fold の operator 名だけが例外。LAMBDA、operator parameter、再帰、他の Apalache operator は拒否する。`EXTENDS Apalache` がなければ fold 名も通常の名前解決に従う。展開深さ・bound variable の上限は変更しない。

sequence は先頭から処理し、文字列・record・関数を含む任意の要素種を扱う。集合 fold は TLC v1.7.4 (`5a47802`) の順序に合わせ、整数、Boolean、tuple / sequence、集合、整数 key の関数と、それらの再帰的な組み合わせを扱う。整数は数値順、Boolean は FALSE / TRUE、tuple / sequence と集合は要素数の後に再帰的な辞書順、関数は domain 要素数の後に各 key / value を交互に比較する。集合要素内の全整数 function key の最大値と最小値の差が `i32::MAX` を超える場合は拒否する。TLC の interval / 明示 domain 間の32-bit減算で順序が反転するためで、IR は元の domain 表現を区別しない。primitive 整数や関数の値にこの幅制限は課さない。domain が `1..n` の関数は既存の値表現どおり sequence として扱う。文字列・record をどの深さでも含む集合は、IR に TLC の文字列・field 名の intern ID がないため、それぞれ理由を明示した評価時エラーで拒否する。非整数 key のその他の関数は従来の `Unsupported TLA input: set fold order for strings, records or functions` を維持する。集合foldの model value は未対応で、異なる値種の比較が必要な集合も拒否する。[pinned TLC ログ](../../evals/fixtures/tlc-apalache-folds/README.md)を対照にしており、Apalache 全体との互換性を表さない。

Rust経路はmodel value、有限CHOOSE、整数の\div / % / ^、CONSTRAINT / ACTION_CONSTRAINT、PROPERTY内の定数有限domain上の時間論理量化にも対応する。制約は探索候補を捨てる前に境界のinvariant/propertyを検査する。無解のCHOOSE、ゼロ除算、0^0、overflow、typed model valueの不正比較は評価エラー。空domainの時間論理量化は比較元TLCのtautology診断も含めて検査する。

[固定入力](tests/finite-support.json)をnative/Wasm/TLCへ渡す。既存TS frontendの対応範囲は別で、追加機能はcompileTlaSource / checkTlaSourceから呼ぶ。CriterionではASTからのcompileと、生成済みIRのcheckを別々に測る。parser、IPC、process起動は計測区間へ含めない。

```sh
just compiler-test
just compiler-criterion finite-compiler evals/runs/my-compiler-bench
just tlc-finite-eval evals/runs/my-tlc-check /path/to/java /path/to/tla2tools.jar
```

## モジュールと設定置換

Rustでは`compile_modules(&root, &config, &modules)`へ解析済みModuleの集合を渡す。`compile`は単一module用の既存APIで、外部moduleを自動取得しない。TSの`compileTlaSource` / `checkTlaSource`とnativeの`compile-tla` / `check-tla`には、追加の`modules: { ModuleName: source }`を渡せる。キーとMODULE名が異なる場合は拒否する。modulesを省略した既存要求も受理する。

非再帰EXTENDSと、top-levelの引数なしnamed/unnamed INSTANCEを解決する。WITH右辺は呼び出し側で解決し、束縛名を区別して展開する。変数の置換先は呼び出し側のstate variable名に限定する。LOCALは他moduleへ公開せず、内部の定義から参照できる。引数付きINSTANCE、LET内のINSTANCE、nested module、一般のoperator引数は拒否する。全moduleのSANY検査やlevel検査の代替ではない。

TLC設定の`N <- Bound`、同arityの`Op <- Replacement`を扱う。module別の定義に対する`Op <- [Module] Replacement`、`Value = [Module] 2`、`Op(1) = [Module] 2`も扱う。設定の置換先はrootで解決する。operator宣言そのものへの引数付きテーブル設定は未対応。比較元TLCがoperator値をstateに含めてエラーにする対照を保存する。不正な設定の診断phaseと文言、存在しないmoduleへの値設定の無視まで同一にはしない。

追加moduleは256、importの深さは40、展開後のsymbolは4096まで。ファイルシステムやネットワークを使わず、欠落・循環・名前の衝突はエラーにする。[固定corpus](tests/modules-support.json)と[TLC比較の計画](../../evals/plans/0047-module-linking.md)で対応範囲を検査する。

反例のaction名は公開名、instanceの修飾名、LETの元のoperator名へ戻す。内部の束縛識別子を実装との照合に使う必要はない。

ASTの書き換え深さは64段まで。ModuleをRustから直接渡した場合も、未使用の定義を含めて上限を検査する。

```sh
just module-eval evals/runs/my-module-check /path/to/java /path/to/tla2tools.jar
just module-replay evals/runs/my-module-check evals/runs/my-module-replay /path/to/java /path/to/tla2tools.jar
```

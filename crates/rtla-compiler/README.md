# rtla-compiler

[対応範囲・非対応のチェックリスト](SUPPORT.md)を参照。

rtlaの型付きModule/Configを、rtlcのSafetyProgramへ変換するRust frontend。parserとcheckerを別crateに保ち、両者の公開契約をつなぐ。Nodeやsolverには依存しない。

`compile(&module, &config)`は名前解決、定数とoperatorの展開、action列挙、temporal/fairnessを変換する。既存TS frontendの有限fragmentを基礎に、Rust経路では[Quint生成物](../../docs/quint-generated.md)の限定した構文も扱う。INSTANCE、ユーザーモジュール、一般の高階・再帰operator、無限集合の列挙、symmetry、state/action constraintは扱わない。解析はrtla、実行時の型・IR検査と探索はrtlcが担当する。

内部ではJSONのIR nodeを組み立て、公開境界で型付きSafetyProgramへ変換する。frontendの性能改善を主張する実装ではない。既存TS frontendは独立した差分比較対象として残す。

root moduleの`EXTENDS Apalache`がある場合、`:=`を同モジュールの定義どおり等号として扱う。`EXTENDS Variants`ではApalache 0.63.0の`Variant`、`VariantTag`、`VariantGetUnsafe`を扱い、tag取得はdomainが一つの文字列keyである関数に限定する。他のVariants operatorと一般のCHOOSEは未対応。`EXTENDS TLC`は受理するが、TLCの全operatorを実装するものではない。

CASEは式・actionともOTHERを省略でき、ソース順で最初の真のarmを選ぶ。OTHERがなく全条件が偽なら、TLC v1.7.4 (`5a47802`) と同様に評価時に未一致エラーとソース位置を返す。未評価の分岐内ではエラーにしない。

root module が直接 `EXTENDS Apalache` を指定するときだけ、`ApaFoldSet(Op, v, S)` と `ApaFoldSeqLeft(Op, v, seq)` を扱う。`Op` は呼び出し位置で可視の top-level / LET 定義の非再帰 operator 名で、零 arity の引数をちょうど2つ持つ必要がある。累積値が第1引数、要素が第2引数。呼び出し位置で body を展開し、bound slot を2つ使う。一般の高階 operator は未対応のままで、この2つの fold の operator 名だけが例外。LAMBDA、operator parameter、再帰、他の Apalache operator は拒否する。`EXTENDS Apalache` がなければ fold 名も通常の名前解決に従う。展開深さ・bound variable の上限は変更しない。

sequence は先頭から処理し、文字列・record・関数を含む任意の要素種を扱う。集合 fold は TLC v1.7.4 (`5a47802`) の順序に合わせ、整数、Boolean、tuple / sequence、集合、整数 key の関数と、それらの再帰的な組み合わせを扱う。整数は数値順、Boolean は FALSE / TRUE、tuple / sequence と集合は要素数の後に再帰的な辞書順、関数は domain 要素数の後に各 key / value を交互に比較する。集合要素内の全整数 function key の最大値と最小値の差が `i32::MAX` を超える場合は拒否する。TLC の interval / 明示 domain 間の32-bit減算で順序が反転するためで、IR は元の domain 表現を区別しない。primitive 整数や関数の値にこの幅制限は課さない。domain が `1..n` の関数は既存の値表現どおり sequence として扱う。文字列・record をどの深さでも含む集合は、IR に TLC の文字列・field 名の intern ID がないため、それぞれ理由を明示した評価時エラーで拒否する。非整数 key のその他の関数は従来の `Unsupported TLA input: set fold order for strings, records or functions` を維持する。model value は従来どおり未対応で、異なる値種の比較が必要な集合も拒否する。[pinned TLC ログ](../../evals/fixtures/tlc-apalache-folds/README.md)を対照にしており、Apalache 全体との互換性を表さない。

```sh
just compiler-test
```

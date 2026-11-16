# rtla-compilerの対応チェックリスト

チェック済みはrtlaのASTをrtlcの有限検査IRへ変換する範囲。未チェックは未対応または対象外で、実装予定を約束しない。構文解析と探索は他crateが担当する。

## 対応済み

- [x] 名前解決、定数・非再帰operatorとLETの展開、lexical scopeの保持。
- [x] signed 32-bit整数、Bool、文字列、有限集合・関数・record・sequenceの対応式とactionの変換。
- [x] 有限量化、tuple pattern、EXCEPT、全変数の代入・選択・UNCHANGED、列挙できるENABLED。
- [x] IFとCASE。最初の真のarmを選び、OTHERがなく未一致なら評価時エラーにする。未評価の分岐ではエラーにしない。
- [x] INIT/NEXTまたは対応形のSPECIFICATION、定数、INVARIANT/PROPERTY、CHECK_DEADLOCK、定数ASSUME。
- [x] 対応するstate/action/temporal propertyとWF/SFをIRへ変換する。
- [x] rootのEXTENDS ApalacheでApaFoldSet / ApaFoldSeqLeft。可視の非再帰2引数operatorに限定し、一般の高階引数とは区別する。
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
- [ ] SYMMETRY、VIEWの設定とlowering。
- [ ] ALIASによる反例表示、POSTCONDITION、POSSIBLE、PERIODIC、RL_REWARD。
- [ ] CONSTANTのoperator置換（<-）、module単位・operator引数付き定数設定。
- [ ] SequencesのSelectSeq。
- [ ] TLCのAssert、SortSeq、RandomElement、TLCGet、TLCSet。
- [ ] SPECIFICATIONの量化されたfairness。直接のWF/SF連言は対応済み。
- [ ] 最小整数リテラル-2147483648のTLC診断互換。Rust frontendの既存の例外受理と比較元TLCには差がある。計算式-2147483647 - 1は対応する。

## 非対応・対象外

- [ ] INSTANCE、ユーザーモジュールの読み込み、一般の高階・再帰operator、LAMBDA。
- [ ] 無限集合の列挙、stateに依存するdomainでの時間論理量化、temporal variable quantification（\AA / \EE）。
- [ ] 無制限CHOOSE、他のApalache/Variants operator。
- [ ] 全モジュールのSANY検査、未使用定義やproofの正しさの判断。
- [ ] 一般のTLA+からSMTへの変換、Quint言語の直接解釈、Quint Connect APIやITFのimport。

## 制約と根拠

- [x] 集合foldの値種と順序の制約を守る。文字列・recordや一部の関数を拒否する範囲は[README](README.md)に従う。
- [x] 未対応・型エラー・予算切れを検査成功へ読み替えない。探索予算と結果はrtlcの契約。

[実装](src/lib.rs)、[回帰](tests/compiler.rs)、[追加対応の回帰](tests/finite_support.rs)、[比較入力](tests/finite-support.json)、[Quint生成物の12設定](../../docs/quint-generated.md)、[既存frontendとの比較](../../evals/iterations/0029-rtla-compiler.md)を参照。

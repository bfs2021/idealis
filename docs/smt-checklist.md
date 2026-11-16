# idealis-smtの対応チェックリスト

旧rsmtの拡張を追跡する。チェック済みは、入力の受理だけでなくSAT/UNSAT・具体値・予算切れ・不正入力をnative/WasmとZ3で検査した範囲。試作と公開backendへの採用を分ける。対応済みの細部と資源上限は[対応表](smt-subset.md)を参照。

## 対応済みの基礎

- [x] 任意精度Int、Bool、加減算、比較、等価・distinct。
- [x] 整数差分制約、無条件の整数定数代入。
- [x] Boolean式、Boolのite、同時束縛let。
- [x] SATの全変数モデル、UNSAT、work切れのunknown。
- [x] native/Wasm、JSONL、解析済みProblemのprepare/check/release。

## 今回実装する範囲

- [ ] 定数倍。正負・0・任意精度の係数を扱い、scaled differenceへ落ちる式に限定する。
- [ ] `k * (x - y) <= c`を整数の丸めで正規化する。一般LIAとは区別する。
- [ ] 整数のite。加減算・定数倍・比較・let内でも使い、展開数を制限する。
- [ ] Rust/JSONでassertionの追加とpush/popを提供する。失敗した変更は既存contextを保つ。
- [ ] Bool literalの一時assumptions。次のcheckへ持ち越さない。
- [ ] 空and/orのZ3互換な単位元。
- [ ] 新機能のZ3差分、モデル照合、native/Wasm、ブラウザと単独consumerの検査。

## 次の候補

- [ ] SMT-LIB scriptのpush/pop、複数check、check-sat-assuming。Rust/JSON APIとは別の項目。
- [ ] define-funのmacro展開。
- [ ] SMT-LIBのget-model/get-value。モデル返却自体はRust/JSONで対応済み。
- [ ] Unicodeの引用識別子とSMT-LIB構文・optionの拡張。
- [ ] 一般LIA。例は自由変数を持つ`2*x + 3*y <= 7`。
- [ ] 探索途中の理論矛盾による剪定・理論伝播。
- [ ] CDCL、節学習、Boolean UNSATの改善。
- [ ] 学習状態・理論状態のincrementalな再利用。
- [ ] work切れからの再開と中断の契約。
- [ ] unsat core、unsat assumptions、proof certificate。
- [ ] 非線形Int、div/mod、abs、冪乗。
- [ ] Real/LRA、Real/Int混合。
- [ ] 引数付きUF、独自sort、理論の組み合わせ。
- [ ] BitVec。
- [ ] Array/select/store。
- [ ] String/正規表現。
- [ ] 代数的データ型。
- [ ] 浮動小数点。
- [ ] forall/existsと量化処理。
- [ ] 最適化・MaxSMT。
- [ ] 公開backend・CLIへの統合と、未対応/unknown時の明示的なZ3 fallback。

新しい理論は一括でチェックしない。演算・型・問い合わせ・返却モデルの範囲ごとに条件を固定し、[evals](../evals/README.md)へ結果と制約を残す。

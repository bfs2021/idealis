# RustでSMT-LIBの一部を解く実験

整数の差分制約とBoolean式を解く独立した[`idealis-smt` crate](../crates/idealis-smt)を提供する。実験用crateから移し、nativeとホストimport不要のWasmを同じソースから生成する。idealisの他crateへ依存せず、idealisの公開backendや既存のZ3 adapterには接続していない。

Z3を再利用する比較でも、小さいTTL・amount・schedulerのnative問い合わせは約3.0-3.4倍速かった。Boolean UNSATでは約34倍遅い例と予算切れが残る。[比較条件と全17入力](../evals/iterations/0031-smt-reuse.md)を確認してから用途を判断する。

## 対応する式

SMT-LIBは入力形式で、対応範囲はその一部。[SMT-LIBの定義](https://smt-lib.org/logics.shtml)ではQF_IDLは整数の差分制約、QF_LIAは一般の線形整数式を扱う。この試作は`set-logic QF_LIA`も受け付けるが、定数代入後に差分制約へ変換できる式だけを解く。定数倍は同じ係数の差分へ正規化できる範囲に対応する。[対応チェックリスト](smt-checklist.md)で残りを追跡する。

| 項目     | 対応範囲                                                                                     |
| -------- | -------------------------------------------------------------------------------------------- |
| 宣言     | `Int` / `Bool`の`declare-const`、引数なしの`declare-fun`                                     |
| コマンド | `set-logic QF_IDL` / `QF_LIA`、`assert`、末尾の任意の単一`check-sat`、`:produce-models true` |
| 整数     | 任意精度の整数、加減算、定数倍、`<` / `<=` / `>` / `>=` / `=` / `distinct`                   |
| Boolean  | `and` / `or`、`not`、`xor`、`=>`、等価、Bool / Intの`ite`                                    |
| 束縛     | 同時束縛の`let`、ASCIIの引用識別子、コメント                                                 |
| 正規化   | 無条件の整数定数等式を代入し、`x - y <= c`へ変換。`k * (x - y) <= c`も整数の丸めで正規化     |

例えば次の式は`other = 3`を代入できるため、`amount = 7`を返す。

```smt2
(set-logic QF_LIA)
(declare-const amount Int)
(declare-const other Int)
(assert (= other 3))
(assert (= (+ amount other) 10))
(assert (> amount 0))
(check-sat)
```

`x + y = 7`だけでは両方が自由変数なので拒否する。`2*x + 3*y <= 7`も一般LIAなので対象外。Real、BitVec、配列、量化、非線形乗算、除算、剰余、引数のある未解釈関数は対象外。SMT-LIBスクリプトの`push/pop`や`get-model`も実装していない。再利用は別のJSON protocolで行う。

整数の`ite`はguard付きの選択肢へ変換する。算術・比較で選択肢が256を超えるとresource errorになる。各選択肢が対応する差分制約へ落ちる必要がある。空の`(and)` / `(or)`はZ3 5.4.1のSMT-LIB parserと同じく拒否する。

入力には1 MiB、20万token、4096宣言などの上限がある。構文や型の検査もサブセットに限る。一般のSMT-LIB validatorとしては使わない。

## 求解と結果の契約

Boolean式を分岐し、各枝の整数制約をBellman-Fordの負閉路検査で解く。負閉路があればその枝は矛盾する。整数を有限範囲で列挙してUNSATを推測する方法ではない。

- `sat`は全宣言の整数値と真偽値を返す。整数値はJSONの精度を保つため10進文字列にする。
- `unsat`は探索したすべての枝が矛盾した場合に返す。
- `unknown`は指定した`maxWork`の消費で探索を止めた場合に返す。理由は`work`で、モデル値は返さない。
- `error`は未対応の式、不正な入力、型や資源上限、protocol違反を表す。

workは式の処理と制約の緩和を数える。parserの費用やwall clockの上限ではない。時間切れは追加していない。節学習や探索途中の理論伝播がないため、Booleanの矛盾証明は指数的に増え得る。

## 実験用のAPI

Rustは`idealis_smt::Problem::parse(source)`と`Problem::solve(max_work)`を持つ。`Problem::solve_assuming(&[Assumption], max_work)`は一時的なBool literalを加えて解く。`Assumption { symbol, positive }`は宣言済みBool名と極性を表す。nativeの`idealis-smt` binaryと`idealis-smt.wasm`は共通の`Engine`を使う。[TS側のwire型](../evals/smt-subset/wire.ts)がリクエストと応答を定義する。

```json
{
  "version": 1,
  "request": {
    "kind": "solve",
    "source": "(declare-const x Int)(assert (= x 7))",
    "maxWork": 2000000
  }
}
```

`Solver::parse(source)`は変更可能なcontextを作る。`assert(expression)`、`push(count)`、`pop(count)`、`check(max_work)`、`check_assuming(assumptions, max_work)`を使う。宣言は初期sourceに固定する。push/popはassertionだけを追加・撤回する。変更に失敗した場合は元のcontextを保つ。

JSONの`prepare {source}`はcontextのhandleを返す。次のリクエストを使う。

| kind           | 追加フィールド                  | 動作                                                          |
| -------------- | ------------------------------- | ------------------------------------------------------------- |
| `assert`       | `handle, expression`            | 単一Bool式を追加する。コマンドや複数式は拒否する。            |
| `push` / `pop` | `handle, count`                 | scopeを保存・撤回する。成功時は`updated`を返す。              |
| `check`        | `handle, maxWork, assumptions?` | 現在の式を求解する。assumptionsは`{symbol, positive}`の配列。 |
| `release`      | `handle`                        | contextを破棄する。解放済みhandleはerror。                    |

scopeとassumptionsはそれぞれ最大256件。pop underflow、未宣言/Intのassumption、不正shapeはerrorになる。assumptionsはそのcheckだけで有効。次のcheckや別handleへ持ち越さない。

assert/pop時はsourceを再解析する。変更なしのcheckは解析結果を再利用するが、探索・学習状態や回答は再利用しない。SMT-LIB scriptのpush/pop/check-sat-assumingは引き続き対象外。

[Wasm loader](../evals/smt-subset/wasm.ts)はNodeやZ3へ依存しない。実Chromiumで`crossOriginIsolated === false`のままSAT、UNSAT、unknown、未対応、再利用を検査した。0030で測定した生Wasmは330,749 bytesで、Z3 Wasmは39,205,260 bytesだった。対応機能が異なるため、汎用ソルバー同士の容量比較にはしない。

[評価0040](../evals/iterations/0040-smt-expansion.md)では、定数倍・整数iteの確認用140式と21件のcontext問い合わせをZ3へ照合した。旧版が受理した式は1/140、新版は140/140。新機能を狙った人工入力の対応率で、実用入力全体の対応率や速度の比較ではない。

## 再実行

Node.js 24+、pnpm、just、Rustと`wasm32-unknown-unknown` targetを用意する。確認済みrunのディレクトリは上書きできないので、新しいパスを指定する。

```sh
just rsmt-build
just rsmt-test
just rsmt-example
pnpm build
node --test dist/evals/smt-subset/contract-test.js
just smt-browser
just smt-expansion-test
just rsmt-criterion
just smt-verify development .artifacts/smt-dev
just smt-verify confirmation .artifacts/smt-confirm .artifacts/smt-dev
just smt-bench .artifacts/smt-bench /absolute/path/to/z3
```

Z3 nativeは[公式5.4.1 release](https://github.com/Z3Prover/z3/releases/tag/z3-5.4.1)を使う。比較対象のnpm `z3-solver`も5.4.1。評価に使ったmacOS arm64 archiveのSHA-256は`a4a878121e81dfc297d027dd0b51dc4f08feb4296d6ecace42bfbc87a29d6d58`で、記録した実行binaryのhashとは区別する。

`smt-build` / `smt-test` / `smt-criterion`は既存の呼び出し用に残す。`just package`はidealis-smtのcrate archive、WasmとABI manifestも作り、archiveだけを展開したRust consumerで検査する。過去の0030/0031の性能値は移動前の測定として保持する。

## 採否

差分制約への限定経路は継続検討する。公開backendへの組み込みと自動fallbackは保留した。一般のLIAやBooleanが難しい問題でZ3を置き換える判断はしていない。context APIは追加したが、学習状態の再利用は未実装。

次の実験は、既存のSMTオラクル用途から入力を集め、対応率と総実行時間を測る。サブセットの速さだけでなく、未対応やunknownでZ3へ戻る費用を含めて採否を決める。

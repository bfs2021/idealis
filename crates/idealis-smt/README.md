# idealis-smt

任意精度整数の差分制約とBoolean式を解く、SMT-LIBサブセットのRustライブラリ。idealisの他crate、Node、Z3には依存しない。nativeとホストimport不要のWasmを同じソースから生成する。

`QF_IDL`と、無条件の定数代入や同じ係数の定数倍を正規化して差分制約へ変換できる`QF_LIA`の一部を扱う。Boolean分岐とBellman-Fordの負閉路検査を使う。一般の線形整数式、Real、BitVec、配列、量化は対象外。[対応範囲・制約・実測結果](https://github.com/mizchi/idealis/blob/main/docs/smt-subset.md)を参照。

## Rustから使う

ローカルcheckoutでは依存に`idealis-smt = { path = "../idealis/crates/idealis-smt" }`を追加する。registryへは未公開。

```rust
use idealis_smt::Problem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let problem = Problem::parse(
        "(declare-const x Int)(assert (> x 10))(assert (< x 20))",
    )?;
    let result = problem.solve(100_000);
    assert_eq!(result.status, "sat");
    println!("{}", result.values["x"]);
    Ok(())
}
```

公開境界は`Problem`、`Solver`、`Assumption`、`Solution`、`Stats`、`Error`とJSON protocolの`Engine`。`Solver`はassertion scopeを保持し、`Assumption`は一時的なBool literalを指定する。parser、式、探索状態はprivate。`Solution`はserdeでJSONへ変換でき、整数値は精度を保つため10進文字列にする。

- `sat`は全宣言の具体値を返す。
- `unsat`はすべての枝が矛盾した場合に返す。
- `unknown`はwork予算切れで、モデルを返さない。
- `Problem::parse`の`Error`は未対応、不正入力、型や資源上限を表す。

## assertion scopeと一時assumptions

```rust
use idealis_smt::{Assumption, Solver};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut solver = Solver::parse("(declare-const x Int)(declare-const b Bool)")?;
    solver.push(1)?;
    solver.assert("(= (* 2 x) (ite b 6 8))")?;
    let assumptions = [Assumption { symbol: "b".into(), positive: true }];
    assert_eq!(solver.check_assuming(&assumptions, 100_000)?.values["x"], "3");
    solver.pop(1)?;
    solver.assert("(= x 9)")?;
    assert_eq!(solver.check(100_000).values["x"], "9");
    Ok(())
}
```

宣言は初期sourceに固定し、scopeとassumptionsは各256件まで。失敗したassert/popは元のcontextを保つ。assert/popは再解析し、変更なしのcheckは解析結果だけを再利用する。学習状態の再利用は行わない。整数iteは最大256選択肢まで展開する。一般LIAや非線形乗算、SMT-LIB scriptのpush/popは未対応。

## nativeとWasm

`idealis-smt` binaryはstdin/stdoutでJSONLを扱う。requestにはversion 1と、`solve` / `prepare` / `assert` / `push` / `pop` / `check` / `release`を指定する。prepareは解析結果を保持し、回答はcacheしない。

```sh
just rsmt-build
just rsmt-test
just rsmt-example
just smt-browser
just package
```

Wasmは`wasm32-unknown-unknown`向けの`idealis-smt.wasm`。既存の`smt_*` ABI v1を保ち、SharedArrayBufferを要求しない。読み書きする関数は`smt_input_resize`、`smt_dispatch`、`smt_output_ptr`、`smt_output_len`。[TS loader](https://github.com/mizchi/idealis/blob/main/evals/smt-subset/wasm.ts)で呼び出せる。

`just package`は`idealis-smt-<version>.crate`、`idealis-smt.wasm`とSHA-256/ABI manifestを作る。展開したidealis-smtのarchiveだけで別consumerをビルドし、SAT、UNSAT、unknown、未対応、scope・assumptionsと依存の独立性を検査する。registryへのpublishは行わない。Criterionの測定は`just rsmt-criterion`。

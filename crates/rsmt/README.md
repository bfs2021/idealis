# rsmt

任意精度整数の差分制約とBoolean式を解く、SMT-LIBサブセットのRustライブラリ。modeltの他crate、Node、Z3には依存しない。nativeとホストimport不要のWasmを同じソースから生成する。

`QF_IDL`と、無条件の定数代入で差分制約へ変換できる`QF_LIA`の一部を扱う。Boolean分岐とBellman-Fordの負閉路検査を使う。一般の線形整数式、Real、BitVec、配列、量化は対象外。[対応範囲・制約・実測結果](https://github.com/mizchi/modelt/blob/main/docs/smt-subset.md)を参照。

## Rustから使う

ローカルcheckoutでは依存に`rsmt = { path = "../modelt/crates/rsmt" }`を追加する。registryへは未公開。

```rust
use rsmt::Problem;

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

公開境界は`Problem`、`Solution`、`Stats`、`Error`とJSON protocolの`Engine`。parser、式、探索状態はprivate。`Solution`はserdeでJSONへ変換でき、整数値は精度を保つため10進文字列にする。

- `sat`は全宣言の具体値を返す。
- `unsat`はすべての枝が矛盾した場合に返す。
- `unknown`はwork予算切れで、モデルを返さない。
- `Problem::parse`の`Error`は未対応、不正入力、型や資源上限を表す。

## nativeとWasm

`rsmt` binaryはstdin/stdoutでJSONLを扱う。requestにはversion 1と、`solve` / `prepare` / `check` / `release`を指定する。prepareは解析結果を保持し、回答はcacheしない。

```sh
just rsmt-build
just rsmt-test
just rsmt-example
just smt-browser
just package
```

Wasmは`wasm32-unknown-unknown`向けの`rsmt.wasm`。既存の`smt_*` ABI v1を保ち、SharedArrayBufferを要求しない。読み書きする関数は`smt_input_resize`、`smt_dispatch`、`smt_output_ptr`、`smt_output_len`。[TS loader](https://github.com/mizchi/modelt/blob/main/evals/smt-subset/wasm.ts)で呼び出せる。

`just package`は`rsmt-<version>.crate`、`rsmt.wasm`とSHA-256/ABI manifestを作る。展開したrsmtのarchiveだけで別consumerをビルドし、SAT、UNSAT、unknown、未対応と依存の独立性を検査する。registryへのpublishは行わない。Criterionの測定は`just rsmt-criterion`。

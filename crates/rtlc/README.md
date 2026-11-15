# rtlc

TLCの意味論に合わせた、有限TLA+ fragmentのモデル検査ライブラリ。コンパイル済みIRから状態を生成し、BFS、invariant/deadlock検査、最短反例、temporal property、WF/SFを検査する。nativeと`wasm32-unknown-unknown`で同じ実装を使う。

実行コードは`idealis-core`へ依存しない。RustのTLA+経路ではrtlaが構文解析、rtla-compilerがIR生成を担当する。既存のTypeScript frontendも比較対象として残す。[対応範囲と制約](../../docs/rust-safety.md)を参照。TLC CLIやTLA+全体の互換性を示すcrateではない。

## Rustから使う

ローカルcheckoutでは`rtlc = { path = "../idealis/crates/rtlc" }`を依存に追加する。JSON IRを読む場合は`serde_json`も使う。

```rust
use rtlc::{SafetyBounds, SafetyProgram};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string("program.json")?;
    let program: SafetyProgram = serde_json::from_str(&json)?;
    let result = program.check(SafetyBounds {
        max_depth: 100,
        max_states: 100_000,
        max_work: 10_000_000,
        max_temporal_nodes: None,
    })?;
    println!("{} states, {}", result.states, result.status);
    Ok(())
}
```

`rtlc::ir`に`Expression`、`Branch`、`Step`、`Predicate`、`TemporalFormula`などの公開契約を置く。JSONを経由しない[型付きcounterの例](examples/counter.rs)もある。Arena、探索queue、worker、intern tableはprivate。

既存の`modelt.safety-program` v1を入力形式として維持する。`complete`、`bounded`、`empty`、違反と評価エラーは区別する。`bounded`を安全性の証明として扱わない。`Value`と`CoreError`はidealisと共有するwire契約で、idealis-coreからも同じ型を再exportする。汎用Valueのvariantが存在することをTLA検査側の対応保証にはしない。

nativeの`check_parallel(bounds, workers)`は1〜256 workerの上限を受け取る。軽いscalarや狭いfrontierは逐次実行する。action/temporal propertyとfairnessを含む探索も逐次へfallbackする。Wasmは逐次の`check`を使う。

## 検証と配布

```sh
just rtlc-test
just rtlc-example
just criterion-test
just package
```

`rtlc-test`は全target、fmt/clippyとCriterion smokeを実行する。Criterion14本とfixtureはこのcrateに置く。通常の時間計測は`just criterion-save <stage> <新規runパス>`。

`rtlc`はRustのrlibとしてWasmにもcompileできる。ブラウザ向けのJSONL/host-importなしWasm ABIはidealis-coreが提供する。`just package`はrtlc/idealis-coreの両archiveを作り、展開したcrateだけで単独consumerを検査する。registryへのpublishは行わない。

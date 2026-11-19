# rtlc

[対応範囲・非対応のチェックリスト](SUPPORT.md)を参照。

TLCの意味論に合わせた、有限TLA+ fragmentのモデル検査ライブラリ。コンパイル済みIRから状態を生成し、BFS、invariant/deadlock検査、最短反例、temporal property、WF/SFを検査する。nativeと`wasm32-unknown-unknown`で同じ実装を使う。

実行コードは`idealis-core`へ依存しない。RustのTLA+経路ではrtlaが構文解析、rtla-compilerがIR生成を担当する。既存のTypeScript frontendも比較対象として残す。[対応範囲と制約](../../docs/rust-safety.md)を参照。TLC CLIやTLA+全体の互換性を示すcrateではない。

整数の\div / % / ^、有限CHOOSE、state/action constraint、model value、定数有限domain上の時間論理量化を扱う。探索制約で除外した候補もinvariantと対応するpropertyを検査する。SYMMETRY、VIEW、ディスク退避、checkpoint、simulation/DFID、式単位のcoverageは未対応。各項目は[チェックリスト](SUPPORT.md)で管理する。

model valueのwire表現は`{"type":"model-value","value":"p_1"}`。文字列の`"p_1"`とは異なる値で、反例にもtagを保持する。型名はTLCと同じく名前の先頭1文字とunderscoreで表す。集合foldでmodel valueの順序を使う入力は拒否する。

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

`rtlc::ir`に`Expression`、`Branch`、`Step`、`Predicate`、`TemporalFormula`などの公開契約を置く。JSONを経由しない[型付きcounterの例](examples/counter.rs)もある。Arena、探索queue、worker、intern tableはprivate。`Expression`・`Branch`・`TemporalFormula`・`Value`は深さによらず有界なstackで破棄するため`Drop`を実装し、これらの値から非Copyのfieldをpatternでmoveできない。参照で照合し、`Vec`・`String`のfieldは`std::mem::take`、`Default`を持たない`Expression`・`TemporalFormula`・`Value`のfieldは`Expression::Old`・`Value::Null`などの葉との`std::mem::replace`で取り出す。

既存の`modelt.safety-program` v1を既定の入力形式として維持する。v2ではexpression定義表とcall / parameter / captureを受理し、callerのlexical frameを持つ引数を要求時に毎回評価する。call entryと引数評価へ各1 workを課金する。再帰定義とaction templateの共有は受理しない。定義ID・capture順とwire契約は[IR v2](../../docs/safety-ir-v2.md)を参照。両frontendは明示`irVersion: 2`でこのIRを生成する。型所属ではv2のidentity range map-setを列挙せず検査する。`complete`、`bounded`、`empty`、違反と評価エラーは区別する。`bounded`を安全性の証明として扱わない。`Value`と`CoreError`はidealisと共有するwire契約で、idealis-coreからも同じ型を再exportする。汎用Valueのvariantが存在することをTLA検査側の対応保証にはしない。

nativeの`check_parallel(bounds, workers)`は1〜256 workerの上限を受け取る。軽いscalarや狭いfrontierは逐次実行する。action/temporal propertyとfairnessを含む探索も逐次へfallbackする。Wasmは逐次の`check`を使う。

`check_incremental(bounds, &mut cache, options)`は、`ExpansionOptions { enabled: true, build_digest, max_entries }`で状態ごとの完了したNext評価を再利用する。optionsの既定値は無効で、通常の`check`へ委譲しcacheを参照・更新しない。Init・property・constraintsは毎回検査し、visitedやpropertyの合否を引き継がない。portable値、全Next/definitionのidentityとSHA-256を持つsnapshotを保存・mergeできる。この経路は逐次探索。[契約と証明範囲](../../docs/safety-expansions.md)を参照。

## 検証と配布

```sh
just rtlc-test
just rtlc-example
just criterion-test
just package
```

`rtlc-test`は全target、fmt/clippyとCriterion smokeを実行する。Criterion14本とfixtureはこのcrateに置く。通常の時間計測は`just criterion-save <stage> <新規runパス>`。

`rtlc`はRustのrlibとしてWasmにもcompileできる。ブラウザ向けのJSONL/host-importなしWasm ABIはidealis-coreが提供する。`just package`はrtlc/idealis-coreの両archiveを作り、展開したcrateだけで単独consumerを検査する。registryへのpublishは行わない。

v2のmembership descriptorはcall/parameter/captureと条件分岐を通って要求時ごとに解釈する。通常の値とv1の列挙は維持する。

明示v2のaction/temporal helperはcall siteごとの非共有anonymous frameでcallerの遅延引数を保持する。binder・EXCEPT oldと要求時のprimeを分離し、入口と引数要求へ各1 workを課金する。LET本体の自由な`@`はtyped protocol拒否とし、action template共有・再帰・全TLA+互換を保証しない。

# idealis-oracle

Rustの`#[test]`から複数実装を同じオラクルへ照合し、同じ登録で速度を比較する試作ライブラリ。任意の入力型と`PartialEq`の観測型を使える。TLA+やSMTの検査器、テストランナー、外部プロセスの実行機能は内蔵しない。

## テストへの組み込み

```rust
use idealis_oracle::{Observation, OracleGroup};

#[test]
fn sum_matches_formula() -> Result<(), idealis_oracle::Error> {
    let mut group = OracleGroup::new("sum");
    group
        .case("ten", 10_u64)
        .reference("formula", |input| Observation::value(input * (input + 1) / 2))
        .test_function("iterator", |input| {
            Observation::value((0..=*input).sum::<u64>())
        });
    group.check()?.assert_matched();
    Ok(())
}
```

[`#[test]`と`Result`](https://doc.rust-lang.org/reference/attributes/testing.html)を使い、入力・関数を登録する形は[CriterionのBenchmarkGroup](https://docs.rs/criterion/latest/criterion/struct.BenchmarkGroup.html)を参考にした。CriterionのAPI互換、統計検定、proc macroは提供しない。各公開report型は、保存と集計を呼び出し側で行うために公開している。

`reference`は1つの独立した基準。`Value`との等値か、`Allowed`の許容値集合への包含を調べる。対象実装は単一の`Value`を返す。`Allowed`が空なら`invalid`、対象実装が`Allowed`を返しても`invalid`になる。`unknown`・`invalid`・`error`は、両者が同じ理由でも一致にしない。複数の結果の集約優先順位は`error > mismatch > unknown > invalid > matched`。空のsuite、重複ID、不足した登録は設定エラーとして返す。

入力と観測には所有したimmutableなスナップショットを使う。型に内包された共有可変状態までは検査できない。SUTの初期化、観測の射影、実装間の表現の正規化はcallback側が担当する。callbackの`Result`は`Observation::from(result)`で実行エラーとして保存できる。panicは通常のRustテストと同じように伝播する。

## 保存と再生

`CheckReport<I, O>`は入力・reference・各実装の観測・判定を保持する。`I`と`O`がserdeに対応していればJSONなどへ保存できる。

```rust,ignore
let saved = serde_json::to_string(&group.check()?)?;
let recorded = serde_json::from_str(&saved)?;
group.replay(&recorded)?.assert_matched();
```

`replay`は保存入力で対象実装だけを実行し、保存referenceへ照合する。reference callbackは呼ばない。同じgroup/reference/実装IDを登録する。case未登録なら保存caseを使い、登録済みなら入力も含めて一致を要求する。保存判定は再計算し、件数やID、集約statusの不整合を拒否する。保存データの真正性、ソース・バイナリのhash、モデル版の固定はホストの責任。

## 比較ベンチマーク

```rust,ignore
use idealis_oracle::BenchConfig;

let report = group.bench(BenchConfig {
    baseline: "iterator".into(),
    samples: 20,
    warmup: 3,
    iterations: 10,
    ..BenchConfig::default()
})?;
report.assert_matched();
```

正しさのreferenceと、速度比のbaselineは別に指定する。referenceはpreflightでcaseごとに1回実行し、計測しない。対象実装のpreflightが一致した場合だけwarmupと測定へ進む。全呼び出しの出力を、時計を止めた後に保存referenceへ照合する。途中で不一致や未確定結果が出れば、全caseの速度比を取り消し、部分sampleと失敗箇所を返す。

測るのはcallback呼び出しと出力構築。入力登録・出力の比較・破棄は時計の外に置く。SUT初期化をcallback内で行えば、その費用も含む。[Criterionのtiming loop](https://bheisler.github.io/criterion.rs/book/user_guide/timing_loops.html)とは計測境界が異なるため、比較時は`scope`へ条件を記録する。

sampleごとに開始実装を回し、隣のsampleで順序を逆にする。reportには実行順と、固定iterationsの合計nsを保存する。mean/median/min/maxは1呼び出し当たりのns。速度比は`baselineのmedian / 対象のmedian`で、対象が速いほど1を超える。信頼区間や有意差は出さない。`max_evaluations`はreference・preflight・warmupを含む予定呼び出し数の上限で、壁時計のtimeoutではない。

nativeの`bench`は`Instant`を使う。Wasmは`bench_with_clock(config, clock)`へ単調増加するns時計を渡す。0時間・逆行・累積時間overflowは`unknown`にし、速度比を返さない。時計を関数へ渡すため、このcrateにWasmのJS bindingはない。実行例は[comparison.rs](examples/comparison.rs)、外部実装比較は[評価アダプタ](../../evals/oracle-comparison)を参照。

## 対応範囲

[`idealis-testing`](../idealis-testing)は有限生成器・feedback・縮小を管理する。このcrateは固定入力に対する実装登録・照合・測定・reportを担当する。生成したcaseを`case`へ登録して併用できる。自動PBT、縮小、並列実行、プロセス隔離、任意言語のbindingはこのcrateには含まない。[対応チェックリスト](SUPPORT.md)を参照。

use criterion::{Criterion, criterion_group, criterion_main};
use rtlc::SafetyBounds;
use serde_json::Value;
use std::hint::black_box;
use std::time::Duration;

fn measurements(criterion: &mut Criterion) {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../tests/finite-support.json")).unwrap();
    for name in [
        "model-untyped",
        "state-constraint",
        "action-constraint",
        "choose-order",
        "power-min",
        "temporal-fair",
    ] {
        let case = cases.iter().find(|case| case["name"] == name).unwrap();
        let source = case["source"].as_str().unwrap();
        let configuration = case["config"].as_str().unwrap();
        let module = rtla::parse_module(source).unwrap();
        let config = rtla::parse_config(configuration).unwrap();
        let program = rtla_compiler::compile(&module, &config).unwrap();
        let bounds = SafetyBounds {
            max_depth: 100,
            max_states: 1000,
            max_work: 1_000_000,
            max_temporal_nodes: Some(10000),
        };
        assert_eq!(program.check(bounds).unwrap().status, case["expected"]);
        criterion.bench_function(&format!("compile/{name}"), |bencher| {
            bencher.iter(|| rtla_compiler::compile(black_box(&module), black_box(&config)).unwrap())
        });
        criterion.bench_function(&format!("check/{name}"), |bencher| {
            bencher.iter(|| program.check(black_box(bounds)).unwrap())
        });
    }
}

criterion_group! {name = benches; config = Criterion::default().sample_size(20).warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1)); targets = measurements}
criterion_main!(benches);

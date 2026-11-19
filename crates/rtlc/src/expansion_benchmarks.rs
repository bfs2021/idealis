#![allow(unused_imports, dead_code)]

include!("lib.rs");

use criterion::{BatchSize, BenchmarkId, Criterion};
use serde::Deserialize;
use std::hint::black_box;

#[derive(Deserialize)]
struct Input {
    name: String,
    program: SafetyProgram,
    states: usize,
    generated: u32,
}

fn main() {
    let inputs: Vec<Input> =
        serde_json::from_slice(include_bytes!("../tests/expansions-fixtures.json")).unwrap();
    let mut criterion = Criterion::default().configure_from_args();
    let mut group = criterion.benchmark_group("expansions");
    let bounds = SafetyBounds {
        max_depth: 64,
        max_states: 10000,
        max_work: 1000000,
        max_temporal_nodes: None,
    };
    let digest = "a".repeat(64);
    for input in &inputs {
        let baseline = input.program.check(bounds).unwrap();
        assert_eq!(baseline.status, "complete");
        assert_eq!(baseline.states, input.states);
        assert_eq!(baseline.generated, input.generated);
        let mut cache = SafetyExpansionCache::default();
        let cold = input
            .program
            .check_incremental(bounds, &mut cache, &digest, 10000)
            .unwrap();
        assert_eq!(cold.result.status, baseline.status);
        let warm = input
            .program
            .check_incremental(bounds, &mut cache, &digest, 10000)
            .unwrap();
        assert_eq!(warm.stats.hits as usize, input.states);
        assert_eq!(warm.result.generated, baseline.generated);
        assert_eq!(warm.result.violation, baseline.violation);
        let snapshot = serde_json::to_vec(&cache).unwrap();
        group.bench_function(BenchmarkId::new("uncached", &input.name), |bencher| {
            bencher.iter(|| black_box(input.program.check(black_box(bounds)).unwrap()))
        });
        group.bench_function(BenchmarkId::new("cold", &input.name), |bencher| {
            bencher.iter_batched(
                SafetyExpansionCache::default,
                |mut cache| {
                    black_box(
                        input
                            .program
                            .check_incremental(bounds, &mut cache, &digest, 10000)
                            .unwrap(),
                    );
                    cache
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(BenchmarkId::new("warm", &input.name), |bencher| {
            bencher.iter_batched(
                || cache.clone(),
                |mut cache| {
                    black_box(
                        input
                            .program
                            .check_incremental(bounds, &mut cache, &digest, 10000)
                            .unwrap(),
                    );
                    cache
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_function(BenchmarkId::new("snapshot", &input.name), |bencher| {
            bencher.iter(|| {
                let mut cache = serde_json::from_slice(black_box(&snapshot)).unwrap();
                black_box(
                    input
                        .program
                        .check_incremental(bounds, &mut cache, &digest, 10000)
                        .unwrap(),
                );
                black_box(serde_json::to_vec(&cache).unwrap())
            })
        });
    }
    group.finish();
    criterion.final_summary();
}

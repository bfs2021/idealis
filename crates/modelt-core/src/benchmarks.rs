#![allow(unused_imports, dead_code)]

// Cargo includes the source test modules without their test harness in this target.
include!("lib.rs");

mod measurements {
    use crate::safety::{Expression, Scalar};
    use crate::safety_expressions::EvalContext;
    use crate::safety_values::Arena;
    use crate::{SafetyBounds, SafetyProgram, Value};
    use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
    use serde::Deserialize;
    use serde_json::json;
    use std::hint::black_box;
    use std::time::Duration;

    #[derive(Deserialize)]
    struct Input {
        name: String,
        states: usize,
        generated: u32,
        program: SafetyProgram,
    }

    fn inputs() -> Vec<Input> {
        serde_json::from_slice(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../.artifacts/criterion-safety/inputs.json"
            ))
            .unwrap_or_else(|_| include_bytes!("bench-fixtures.json").to_vec()),
        )
        .unwrap()
    }

    pub fn run(criterion: &mut Criterion) {
        let inputs = inputs();
        let functions: Vec<Vec<_>> = (0..4096)
            .map(|bits| {
                (0..12)
                    .map(|bit| (Scalar::Int(bit + 1), Scalar::Int((bits >> bit) & 1)))
                    .collect()
            })
            .collect();
        let mut group = criterion.benchmark_group("values");
        group.throughput(Throughput::Elements(4096));
        group.bench_function("intern-new-functions", |bencher| {
            bencher.iter_batched(
                || Arena::new(u32::MAX),
                |mut arena| {
                    for entries in black_box(&functions) {
                        black_box(arena.function(entries.clone()).unwrap());
                    }
                    arena
                },
                BatchSize::PerIteration,
            )
        });
        group.bench_function("intern-existing-functions", |bencher| {
            bencher.iter_batched(
                || {
                    let mut arena = Arena::new(u32::MAX);
                    for entries in &functions {
                        arena.function(entries.clone()).unwrap();
                    }
                    arena
                },
                |mut arena| {
                    for entries in black_box(&functions) {
                        black_box(arena.function(entries.clone()).unwrap());
                    }
                    arena
                },
                BatchSize::PerIteration,
            )
        });
        group.bench_function("except-function", |bencher| {
            bencher.iter_batched(
                || {
                    let mut arena = Arena::new(u32::MAX);
                    let value = arena.tuple(vec![Scalar::Int(0); 20]).unwrap();
                    (arena, value)
                },
                |(mut arena, mut value)| {
                    for number in 1..=4096 {
                        value = arena
                            .replace(black_box(value), &[Scalar::Int(1)], Scalar::Int(number))
                            .unwrap();
                    }
                    black_box(value);
                    arena
                },
                BatchSize::PerIteration,
            )
        });
        let update: Expression = serde_json::from_value(json!({"op":"update","target":{"op":"read","slot":0,"next":false},"updates":[
            {"path":[{"op":"literal","value":{"type":"int","value":"1"}},{"op":"data","value":{"type":"string","value":"val"}}],"value":{"op":"add","left":{"op":"old"},"right":{"op":"literal","value":{"type":"int","value":"1"}}}},
            {"path":[{"op":"literal","value":{"type":"int","value":"1"}},{"op":"data","value":{"type":"string","value":"rdy"}}],"value":{"op":"sub","left":{"op":"literal","value":{"type":"int","value":"1"}},"right":{"op":"old"}}}
        ]})).unwrap();
        group.bench_function("except-record", |bencher| {
            bencher.iter_batched(
                || {
                    let mut arena = Arena::new(u32::MAX);
                    let value = arena.data(&record_state(0, 4)).unwrap();
                    (arena, value)
                },
                |(mut arena, mut value)| {
                    for _ in 0..4096 {
                        value = update
                            .eval(EvalContext::new(black_box(&[value]), &[]), &mut arena)
                            .unwrap();
                    }
                    black_box(value);
                    arena
                },
                BatchSize::PerIteration,
            )
        });
        group.finish();
        let mut group = criterion.benchmark_group("membership");
        group.throughput(Throughput::Elements(4096));
        for name in ["function-cube", "parallel-channels"] {
            let input = inputs.iter().find(|input| input.name == name).unwrap();
            group.bench_function(name, |bencher| {
                bencher.iter_batched(
                    || {
                        let mut arena = Arena::new(u32::MAX);
                        let states: Vec<_> = (0..4096)
                            .map(|number| {
                                let state = if name == "function-cube" {
                                    Value::List(
                                        (0..12)
                                            .map(|bit| {
                                                Value::Int(((number >> bit) & 1).to_string())
                                            })
                                            .collect(),
                                    )
                                } else {
                                    record_state(number, 4)
                                };
                                arena.data(&state).unwrap()
                            })
                            .collect();
                        (arena, states)
                    },
                    |(mut arena, states)| {
                        for state in black_box(states) {
                            assert!(
                                input.program.invariants[0]
                                    .expression
                                    .eval(EvalContext::new(&[state], &[]), &mut arena)
                                    .unwrap()
                                    .boolean()
                                    .unwrap()
                            );
                        }
                        arena
                    },
                    BatchSize::PerIteration,
                )
            });
        }
        group.finish();
        let mut group = criterion.benchmark_group("search");
        for input in &inputs {
            group.throughput(Throughput::Elements(u64::from(input.generated)));
            for workers in [1, 8] {
                let bounds = SafetyBounds {
                    max_depth: 100_000,
                    max_states: 100_000,
                    max_work: 100_000_000,
                    max_temporal_nodes: None,
                };
                let result = input.program.check_parallel(bounds, workers).unwrap();
                assert_eq!(result.status, "complete");
                assert_eq!(result.states, input.states);
                assert_eq!(result.generated, input.generated);
                group.bench_with_input(
                    BenchmarkId::new(&input.name, workers),
                    &workers,
                    |bencher, workers| {
                        bencher.iter(|| {
                            black_box(
                                input
                                    .program
                                    .check_parallel(black_box(bounds), *workers)
                                    .unwrap(),
                            )
                        })
                    },
                );
            }
        }
        group.finish();
    }

    fn record_state(mut number: i32, count: usize) -> Value {
        Value::List(
            (0..count)
                .map(|_| {
                    let bits = number & 7;
                    number >>= 3;
                    Value::Record(
                        [
                            ("val", bits & 1),
                            ("rdy", (bits >> 1) & 1),
                            ("ack", (bits >> 2) & 1),
                        ]
                        .into_iter()
                        .map(|(name, value)| (name.into(), Value::Int(value.to_string())))
                        .collect(),
                    )
                })
                .collect(),
        )
    }

    pub fn configuration() -> Criterion {
        Criterion::default()
            .sample_size(20)
            .warm_up_time(Duration::from_millis(500))
            .measurement_time(Duration::from_secs(1))
            .noise_threshold(0.03)
    }
}

criterion::criterion_group! { name = benches; config = measurements::configuration(); targets = measurements::run }
criterion::criterion_main!(benches);

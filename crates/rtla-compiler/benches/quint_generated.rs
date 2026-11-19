use criterion::{Criterion, criterion_group, criterion_main};
use rtla_compiler::{CompileOptions, IrVersion, compile_with_options};
use rtlc::SafetyBounds;
use serde_json::{Value, json};
use std::hint::black_box;
use std::path::Path;
use std::time::Duration;

fn configuration() -> Criterion {
    let criterion = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));
    match std::env::var_os("IDEALIS_QUINT_BENCH_OUTPUT") {
        Some(directory) => criterion.output_directory(Path::new(&directory)),
        None => criterion,
    }
}

fn measurements(criterion: &mut Criterion) {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("../tests/quint-generated.json")).unwrap();
    let bounds = SafetyBounds {
        max_depth: 64,
        max_states: 10000,
        max_work: 1000000,
        max_temporal_nodes: None,
    };
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let configuration = case["config"].as_str().unwrap();
        let module = rtla::parse_module(source).unwrap();
        let config = rtla::parse_config(configuration).unwrap();
        let programs: Vec<_> = [(1, IrVersion::V1), (2, IrVersion::V2)]
            .into_iter()
            .map(|(version, ir_version)| {
                let options = CompileOptions {
                    irVersion: ir_version,
                };
                let program = compile_with_options(&module, &config, options).unwrap();
                assert_eq!(program.version, version);
                let result = program.check(bounds).unwrap();
                assert_eq!(result.status, case["expectedStatus"]);
                assert_eq!(result.states as u64, case["expectedStates"]);
                assert_eq!(result.generated as u64, case["expectedGenerated"]);
                assert!(result.states > 0);
                assert!(result.reason.is_none());
                println!(
                    "IDEALIS_QUINT_PROGRAM {}",
                    json!({
                        "name": name,
                        "version": version,
                        "irBytes": serde_json::to_vec(&program).unwrap().len(),
                        "definitions": program.definitions.as_ref().map_or(0, Vec::len),
                        "states": result.states,
                        "generated": result.generated,
                        "work": result.work,
                    })
                );
                (version, options, program)
            })
            .collect();
        criterion.bench_function(&format!("quint-parse/{name}"), |bencher| {
            bencher.iter(|| {
                (
                    rtla::parse_module(black_box(source)).unwrap(),
                    rtla::parse_config(black_box(configuration)).unwrap(),
                )
            });
        });
        for (version, options, program) in programs {
            criterion.bench_function(&format!("quint-compile/{name}/v{version}"), |bencher| {
                bencher.iter(|| {
                    compile_with_options(black_box(&module), black_box(&config), options).unwrap()
                });
            });
            criterion.bench_function(&format!("quint-check/{name}/v{version}"), |bencher| {
                bencher.iter(|| black_box(&program).check(black_box(bounds)).unwrap());
            });
            criterion.bench_function(&format!("quint-end-to-end/{name}/v{version}"), |bencher| {
                bencher.iter(|| {
                    let module = rtla::parse_module(black_box(source)).unwrap();
                    let config = rtla::parse_config(black_box(configuration)).unwrap();
                    compile_with_options(&module, &config, options)
                        .unwrap()
                        .check(black_box(bounds))
                        .unwrap()
                });
            });
        }
    }
}

criterion_group! {name = benches; config = configuration(); targets = measurements}
criterion_main!(benches);

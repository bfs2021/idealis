use criterion::{Criterion, criterion_group, criterion_main};
use rsmt::Problem;
use std::{hint::black_box, time::Duration};

fn cases(criterion: &mut Criterion) {
    for count in [4, 32, 128] {
        let mut source = (0..count)
            .map(|index| format!("(declare-const x{index} Int)"))
            .collect::<String>();
        source.push_str("(assert (= x0 0))");
        for index in 1..count {
            source.push_str(&format!("(assert (= x{index} (+ x{} 1)))", index - 1));
        }
        let problem = Problem::parse(&source).unwrap();
        criterion.bench_function(&format!("parse-solve-{count}"), |bench| {
            bench.iter(|| Problem::parse(black_box(&source)).unwrap().solve(2_000_000))
        });
        criterion.bench_function(&format!("prepared-{count}"), |bench| {
            bench.iter(|| problem.solve(black_box(2_000_000)))
        });
    }
}
criterion_group! {name=benches;config=Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(2)).sample_size(20);targets=cases}
criterion_main!(benches);

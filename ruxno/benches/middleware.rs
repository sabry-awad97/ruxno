//! Middleware benchmarks

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

fn middleware_benchmark(c: &mut Criterion) {
    c.bench_function("middleware_chain", |b| {
        b.iter(|| {
            // TODO: Benchmark middleware chain execution
            black_box(());
        });
    });
}

criterion_group!(benches, middleware_benchmark);
criterion_main!(benches);

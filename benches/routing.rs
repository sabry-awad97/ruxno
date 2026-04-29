//! Routing benchmarks

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

fn routing_benchmark(c: &mut Criterion) {
    c.bench_function("route_lookup", |b| {
        b.iter(|| {
            // TODO: Benchmark route lookup
            black_box(());
        });
    });
}

criterion_group!(benches, routing_benchmark);
criterion_main!(benches);

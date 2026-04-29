//! Dispatcher benchmarks

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

fn dispatcher_benchmark(c: &mut Criterion) {
    c.bench_function("dispatch_request", |b| {
        b.iter(|| {
            // TODO: Benchmark full request dispatch
            black_box(());
        });
    });
}

criterion_group!(benches, dispatcher_benchmark);
criterion_main!(benches);

//! Columnar Storage Benchmark
//!
//! DEPRECATED: The columnar storage API referenced in this file no longer exists.
//! This file is kept as a stub to allow compilation but benchmarks are disabled.

use criterion::{criterion_group, criterion_main, Criterion};

// Stub benchmark - actual columnar storage benchmarks require a module that doesn't exist
fn bench_stub(c: &mut Criterion) {
    c.benchmark_group("columnar_stub").bench_function("noop", |b| {
        b.iter(|| {});
    });
}

criterion_group!(benches, bench_stub);
criterion_main!(benches);

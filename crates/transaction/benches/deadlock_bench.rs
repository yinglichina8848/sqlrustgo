use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::sync::{Arc, Mutex};

use sqlrustgo_transaction::mvcc::TxId;
use sqlrustgo_transaction::deadlock::DeadlockDetector;

fn setup_detector(n_transactions: usize) -> DeadlockDetector {
    let detector = DeadlockDetector::new();

    for i in 0..n_transactions {
        let tx = TxId::new(i as u64);
        if i > 0 {
            let prev = TxId::new((i - 1) as u64);
            let _ = detector.try_wait_edge(tx, [prev].into_iter().collect());
        }
    }

    detector
}

fn bench_detect_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadlock_detect_cycle");

    for &n in [10, 50, 100, 200].iter() {
        let detector = setup_detector(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let tx_id = black_box(TxId::new(0));
                detector.detect_cycle(tx_id);
            });
        });
    }

    group.finish();
}

fn bench_try_wait_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadlock_try_wait_edge");

    for &n in [10, 50, 100].iter() {
        let detector = Arc::new(Mutex::new(setup_detector(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let d = Arc::clone(&detector);
                let guard = d.lock().unwrap();
                let _ = guard.try_wait_edge(
                    black_box(TxId::new(n as u64 + 1)),
                    black_box([TxId::new(0)].into_iter().collect()),
                );
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_detect_cycle,
    bench_try_wait_edge
);
criterion_main!(benches);
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use std::sync::{Arc, Mutex};

use sqlrustgo_transaction::deadlock::DeadlockDetector;
use sqlrustgo_transaction::mvcc::TxId;

fn setup_linear_chain(n_transactions: usize) -> DeadlockDetector {
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

fn setup_with_cycle(n_transactions: usize) -> DeadlockDetector {
    let detector = DeadlockDetector::new();

    for i in 0..n_transactions {
        let tx = TxId::new(i as u64);
        let next = TxId::new(((i + 1) % n_transactions) as u64);
        detector.add_edge_unsafe(tx, next);
    }

    detector
}

fn bench_detect_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadlock_detect_cycle");

    for &n in [10, 50, 100, 200].iter() {
        let detector = setup_with_cycle(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &_n| {
            b.iter(|| {
                let tx_id = black_box(TxId::new(0));
                detector.detect_cycle(tx_id);
            });
        });
    }

    group.finish();
}

fn bench_try_wait_edge(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadlock_try_wait_edge_ok");

    for &n in [10, 50, 100].iter() {
        let detector = Arc::new(Mutex::new(setup_linear_chain(n)));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let guard = detector.lock().unwrap();
            b.iter(|| {
                let _ = guard.try_wait_edge(
                    black_box(TxId::new((n + 1) as u64)),
                    black_box([TxId::new(0)].into_iter().collect()),
                );
            });
        });
    }

    group.finish();
}

fn bench_try_wait_edge_rejects_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("deadlock_try_wait_edge_rejects_cycle");

    for &n in [10, 50, 100].iter() {
        let detector = setup_linear_chain(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let _ = detector.try_wait_edge(
                    black_box(TxId::new(0)),
                    black_box([TxId::new((n - 1) as u64)].into_iter().collect()),
                );
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_detect_cycle,
    bench_try_wait_edge,
    bench_try_wait_edge_rejects_cycle
);
criterion_main!(benches);

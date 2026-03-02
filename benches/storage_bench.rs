use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo::BPlusTree;

fn bench_bplus_tree_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("bplus_tree_insert");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut tree = BPlusTree::new();
                for i in 0..size {
                    tree.insert(i as i64, i as u32);
                }
            });
        });
    }

    group.finish();
}

fn bench_bplus_tree_search(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 0..10000 {
        tree.insert(i as i64, i as u32);
    }

    let mut group = c.benchmark_group("bplus_tree_search");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let _ = tree.search(i as i64);
                }
            });
        });
    }

    group.finish();
}

fn bench_bplus_tree_range(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 0..10000 {
        tree.insert(i as i64, i as u32);
    }

    let mut group = c.benchmark_group("bplus_tree_range");

    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let _ = tree.range_query(0, size as i64);
            });
        });
    }

    group.finish();
}

fn bench_bplus_tree_keys(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 0..10000 {
        tree.insert(i as i64, i as u32);
    }

    let mut group = c.benchmark_group("bplus_tree_keys");

    group.bench_function("get_all_keys", |b| {
        b.iter(|| {
            let _ = tree.keys();
        });
    });

    group.finish();
}

fn bench_bplus_tree_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("bplus_tree_mixed");

    group.bench_function("50k_operations", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            for i in 0..25000 {
                tree.insert(i as i64, i as u32);
            }
            for i in 0..15000 {
                let _ = tree.search(i as i64);
            }
            let _ = tree.range_query(0, 10000);
            let _ = tree.keys();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bplus_tree_insert,
    bench_bplus_tree_search,
    bench_bplus_tree_range,
    bench_bplus_tree_keys,
    bench_bplus_tree_mixed
);
criterion_main!(benches);

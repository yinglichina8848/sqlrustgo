//! Storage Benchmark Tests
//!
//! Benchmarks for storage engine performance.

use criterion::{criterion_group, criterion_main, Criterion};
use sqlrustgo::storage::{BPlusTree, BufferPool, Page};
use std::sync::Arc;

// ==================== BufferPool Benchmarks ====================

fn bench_buffer_pool_new(c: &mut Criterion) {
    c.bench_function("buffer_pool_new_100", |b| {
        b.iter(|| BufferPool::new(100));
    });
}

fn bench_buffer_pool_insert(c: &mut Criterion) {
    let pool = BufferPool::new(1000);
    c.bench_function("buffer_pool_insert", |b| {
        b.iter(|| {
            let page = Arc::new(Page::new(0));
            pool.insert(page);
        });
    });
}

fn bench_buffer_pool_get(c: &mut Criterion) {
    let pool = BufferPool::new(1000);
    // Pre-populate
    for i in 0..100 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }
    c.bench_function("buffer_pool_get_hit", |b| {
        b.iter(|| pool.get(50));
    });
}

fn bench_buffer_pool_get_miss(c: &mut Criterion) {
    let pool = BufferPool::new(1000);
    // Pre-populate
    for i in 0..100 {
        let page = Arc::new(Page::new(i));
        pool.insert(page);
    }
    c.bench_function("buffer_pool_get_miss", |b| {
        b.iter(|| pool.get(999));
    });
}

fn bench_buffer_pool_many_pages(c: &mut Criterion) {
    c.bench_function("buffer_pool_1000_pages", |b| {
        b.iter(|| {
            let pool = BufferPool::new(1000);
            for i in 0..1000 {
                let page = Arc::new(Page::new(i));
                pool.insert(page);
            }
        });
    });
}

// ==================== B+Tree Benchmarks ====================

fn bench_bplus_tree_insert_single(c: &mut Criterion) {
    c.bench_function("bplus_tree_insert_single", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            tree.insert(1, 1);
        });
    });
}

fn bench_bplus_tree_insert_100(c: &mut Criterion) {
    c.bench_function("bplus_tree_insert_100", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            for i in 1..=100 {
                tree.insert(i as i64, i);
            }
        });
    });
}

fn bench_bplus_tree_insert_1000(c: &mut Criterion) {
    c.bench_function("bplus_tree_insert_1000", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            for i in 1..=1000 {
                tree.insert(i as i64, i);
            }
        });
    });
}

fn bench_bplus_tree_search_existing(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 1..=1000 {
        tree.insert(i as i64, i);
    }
    c.bench_function("bplus_tree_search_existing", |b| {
        b.iter(|| tree.search(500));
    });
}

fn bench_bplus_tree_search_missing(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 1..=1000 {
        tree.insert(i as i64, i);
    }
    c.bench_function("bplus_tree_search_missing", |b| {
        b.iter(|| tree.search(5000));
    });
}

fn bench_bplus_tree_range_scan(c: &mut Criterion) {
    let mut tree = BPlusTree::new();
    for i in 1..=1000 {
        tree.insert(i as i64, i);
    }
    c.bench_function("bplus_tree_range_scan_100", |b| {
        b.iter(|| {
            let _results = tree.range_query(400, 500);
        });
    });
}

// ==================== Page Benchmarks ====================

fn bench_page_new(c: &mut Criterion) {
    c.bench_function("page_new_4kb", |b| {
        b.iter(|| Page::new(0));
    });
}

fn bench_page_new_16kb(c: &mut Criterion) {
    c.bench_function("page_new_16kb", |b| {
        b.iter(|| Page::new(0));
    });
}

fn bench_page_read_write(c: &mut Criterion) {
    let page = Page::new(0);
    c.bench_function("page_read_write", |b| {
        b.iter(|| {
            let mut data = page.data.clone();
            data[0] = 1;
            data[4095] = 1;
            data
        });
    });
}

// ==================== Sequential Insert Benchmarks ====================

fn bench_bplus_tree_sequential_insert_10k(c: &mut Criterion) {
    c.bench_function("bplus_tree_insert_10k_sequential", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            for i in 1..=10000 {
                tree.insert(i as i64, i);
            }
        });
    });
}

fn bench_bplus_tree_random_insert_1k(c: &mut Criterion) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    c.bench_function("bplus_tree_insert_1k_random", |b| {
        b.iter(|| {
            let mut tree = BPlusTree::new();
            // Use hash-based pseudo-random
            for i in 1..=1000 {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let key = (hasher.finish() % 10000) as i64;
                tree.insert(key, i);
            }
        });
    });
}

criterion_group!(
    benches,
    // BufferPool
    bench_buffer_pool_new,
    bench_buffer_pool_insert,
    bench_buffer_pool_get,
    bench_buffer_pool_get_miss,
    bench_buffer_pool_many_pages,
    // B+Tree
    bench_bplus_tree_insert_single,
    bench_bplus_tree_insert_100,
    bench_bplus_tree_insert_1000,
    bench_bplus_tree_search_existing,
    bench_bplus_tree_search_missing,
    bench_bplus_tree_range_scan,
    bench_bplus_tree_sequential_insert_10k,
    bench_bplus_tree_random_insert_1k,
    // Page
    bench_page_new,
    bench_page_new_16kb,
    bench_page_read_write
);
criterion_main!(benches);

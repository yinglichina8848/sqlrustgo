//! ClusteredLeafPage performance benchmarks
//!
//! Benchmarks for ClusteredLeafPage operations including:
//! - Insert throughput
//! - Scan performance
//! - Page split overhead
//! - Encode/decode roundtrip

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_storage::clustered_index::ClusteredLeafPage;
use sqlrustgo_storage::row_format::types::ClusterKey;
use sqlrustgo_types::Value;
use std::hint::black_box;

/// Generate a test cluster key
fn gen_key(id: u64) -> ClusterKey {
    ClusterKey::HiddenRowId(id)
}

/// Generate fixed columns for a record
fn gen_fixed_columns(row_id: u64) -> Vec<Value> {
    vec![
        Value::Integer(row_id as i64),
        Value::Text(format!("row_{}", row_id)),
    ]
}

/// Generate varlen columns
fn gen_varlen_columns(row_id: u64) -> Vec<Option<Vec<u8>>> {
    vec![
        Some(format!("varlen_data_{}", row_id).into_bytes()),
        None, // NULL column
    ]
}

/// Generate null bitmap
fn gen_null_bitmap() -> Vec<bool> {
    vec![false, false, true] // 3 columns, third is NULL
}

/// Benchmark: Single row insert
fn bench_insert_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_insert_single");

    for i in 0..100 {
        let mut page = ClusteredLeafPage::new();

        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            b.iter(|| {
                let mut page = ClusteredLeafPage::new();
                let key = gen_key(i as u64);
                let fixed = gen_fixed_columns(i as u64);
                let varlen = gen_varlen_columns(i as u64);
                let nulls = gen_null_bitmap();

                let _ = page.insert(&key, &fixed, &varlen, &nulls);
                black_box(&page);
            });
        });
    }

    group.finish();
}

/// Benchmark: Sequential insert (filling a page)
fn bench_insert_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_insert_sequential");

    for size in [10, 50, 100, 200].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut page = ClusteredLeafPage::new();
                for i in 0..size {
                    let key = gen_key(i as u64);
                    let fixed = gen_fixed_columns(i as u64);
                    let varlen = gen_varlen_columns(i as u64);
                    let nulls = gen_null_bitmap();

                    if page.insert(&key, &fixed, &varlen, &nulls).is_ok() {
                        black_box(&page);
                    }
                }
            });
        });
    }

    group.finish();
}

/// Benchmark: Random insert (simulating non-sequential key insertion)
fn bench_insert_random(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_insert_random");

    // Use deterministic pseudo-random
    let mut rng = 12345u64;

    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut page = ClusteredLeafPage::new();
                for i in 0..size {
                    // Deterministic "random" key
                    rng ^= rng << 13;
                    rng ^= rng >> 7;
                    rng ^= rng << 17;
                    let key_id = rng % 10000;

                    let key = gen_key(key_id as u64);
                    let fixed = gen_fixed_columns(i as u64);
                    let varlen = gen_varlen_columns(i as u64);
                    let nulls = gen_null_bitmap();

                    let _ = page.insert(&key, &fixed, &varlen, &nulls);
                }
                black_box(&page);
            });
        });
    }

    group.finish();
}

/// Benchmark: Scan all records (full page iteration)
fn bench_scan_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_scan_full");

    for size in [10, 50, 100, 200].iter() {
        // Pre-populate a page
        let mut page = ClusteredLeafPage::new();
        for i in 0..*size {
            let key = gen_key(i as u64);
            let fixed = gen_fixed_columns(i as u64);
            let varlen = gen_varlen_columns(i as u64);
            let nulls = gen_null_bitmap();
            let _ = page.insert(&key, &fixed, &varlen, &nulls);
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut count = 0;
                for i in 0..page.slot_count() {
                    if !page.is_slot_deleted(i) {
                        if page.get_cluster_key(i).is_ok() {
                            count += 1;
                        }
                    }
                }
                black_box(count);
            });
        });
    }

    group.finish();
}

/// Benchmark: Lower bound search
fn bench_search_lower_bound(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_search_lower_bound");

    // Pre-populate a page with sequential keys
    let size = 100;
    let mut page = ClusteredLeafPage::new();
    for i in 0..size {
        let key = gen_key((i * 10) as u64); // Keys: 0, 10, 20, ...
        let fixed = gen_fixed_columns(i as u64);
        let varlen = gen_varlen_columns(i as u64);
        let nulls = gen_null_bitmap();
        let _ = page.insert(&key, &fixed, &varlen, &nulls);
    }

    for search_key in [0, 25, 50, 75, 99, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(search_key),
            search_key,
            |b, &search_key| {
                b.iter(|| {
                    let key = gen_key(search_key as u64);
                    let _ = black_box(page.lower_bound(&key));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Page split
fn bench_page_split(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_page_split");

    for initial_size in [50, 100, 150, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(initial_size),
            initial_size,
            |b, &initial_size| {
                b.iter(|| {
                    let mut page = ClusteredLeafPage::new();
                    for i in 0..initial_size {
                        let key = gen_key(i as u64);
                        let fixed = gen_fixed_columns(i as u64);
                        let varlen = gen_varlen_columns(i as u64);
                        let nulls = gen_null_bitmap();
                        let _ = page.insert(&key, &fixed, &varlen, &nulls);
                    }

                    // Split at middle
                    let split_pos = initial_size / 2;
                    let _ = black_box(page.split(split_pos));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Encode row (without page)
fn bench_encode_row(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_encode");

    for i in 0..1000 {
        group.bench_function(BenchmarkId::from_parameter(i), |b| {
            let key = gen_key(i as u64);
            let fixed = gen_fixed_columns(i as u64);
            let varlen = gen_varlen_columns(i as u64);
            let nulls = gen_null_bitmap();

            b.iter(|| {
                use sqlrustgo_storage::row_format::encoder::encode_row;
                let encoded = encode_row(&key, &fixed, &varlen, &nulls);
                black_box(encoded);
            });
        });
    }

    group.finish();
}

/// Benchmark: Free space calculation
fn bench_free_space(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_free_space");

    // Pre-populate pages of different fullness
    for size in [0, 50, 100, 150, 200].iter() {
        let mut page = ClusteredLeafPage::new();
        for i in 0..*size {
            let key = gen_key(i as u64);
            let fixed = gen_fixed_columns(i as u64);
            let varlen = gen_varlen_columns(i as u64);
            let nulls = gen_null_bitmap();
            let _ = page.insert(&key, &fixed, &varlen, &nulls);
        }

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let space = page.free_space();
                black_box(space);
            });
        });
    }

    group.finish();
}

/// Benchmark: Delete operation
fn bench_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_delete");

    // Pre-populate a page
    let size = 100;
    let mut base_page = ClusteredLeafPage::new();
    for i in 0..size {
        let key = gen_key(i as u64);
        let fixed = gen_fixed_columns(i as u64);
        let varlen = gen_varlen_columns(i as u64);
        let nulls = gen_null_bitmap();
        let _ = base_page.insert(&key, &fixed, &varlen, &nulls);
    }

    for delete_pattern in ["first", "last", "middle", "every_other"].iter() {
        group.bench_function(BenchmarkId::from_parameter(delete_pattern), |b| {
            b.iter(|| {
                let mut page = base_page.clone();
                match *delete_pattern {
                    "first" => {
                        let _ = page.delete(0);
                    }
                    "last" => {
                        let _ = page.delete((size - 1) as u16);
                    }
                    "middle" => {
                        let _ = page.delete(size as u16 / 2);
                    }
                    "every_other" => {
                        for i in (0..size).step_by(2) {
                            let _ = page.delete(i as u16);
                        }
                    }
                    _ => {}
                }
                black_box(&page);
            });
        });
    }

    group.finish();
}

/// Benchmark: Compact operation
fn bench_compact(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_compact");

    // Pre-populate a page with deletions
    for delete_ratio in [0.25, 0.5, 0.75].iter() {
        let size = 100;
        let delete_count = (size as f64 * delete_ratio) as usize;

        group.bench_with_input(
            BenchmarkId::from_parameter(delete_ratio),
            delete_ratio,
            |b, &delete_ratio| {
                b.iter(|| {
                    let mut page = ClusteredLeafPage::new();
                    for i in 0..size {
                        let key = gen_key(i as u64);
                        let fixed = gen_fixed_columns(i as u64);
                        let varlen = gen_varlen_columns(i as u64);
                        let nulls = gen_null_bitmap();
                        let _ = page.insert(&key, &fixed, &varlen, &nulls);
                    }

                    // Delete some records
                    for i in 0..delete_count {
                        let _ = page.delete(i as u16);
                    }

                    // Compact
                    let _ = page.compact();
                    black_box(&page);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Live record count
fn bench_live_record_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("clustered_leaf_live_record_count");

    // Pre-populate a page with deletions
    let size = 100;
    let delete_count = 50;

    let mut base_page = ClusteredLeafPage::new();
    for i in 0..size {
        let key = gen_key(i as u64);
        let fixed = gen_fixed_columns(i as u64);
        let varlen = gen_varlen_columns(i as u64);
        let nulls = gen_null_bitmap();
        let _ = base_page.insert(&key, &fixed, &varlen, &nulls);
    }

    // Delete every other record
    for i in (0..size).step_by(2) {
        let _ = base_page.delete(i as u16);
    }

    group.bench_function("100_slots_50_deleted", |b| {
        b.iter(|| {
            let count = base_page.live_record_count();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_single,
    bench_insert_sequential,
    bench_insert_random,
    bench_scan_full,
    bench_search_lower_bound,
    bench_page_split,
    bench_encode_row,
    bench_free_space,
    bench_delete,
    bench_compact,
    bench_live_record_count
);
criterion_main!(benches);

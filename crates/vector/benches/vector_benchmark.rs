//! Vector index performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlrustgo_vector::{
    flat::FlatIndex,
    ivf::IvfIndex,
    hnsw::HnswIndex,
    metrics::DistanceMetric,
    VectorIndex,
};

fn generate_random_vectors(n: usize, dim: usize) -> Vec<(u64, Vec<f32>)> {
    let mut vectors = Vec::with_capacity(n);
    for i in 0..n {
        let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>() * 2.0 - 1.0).collect();
        vectors.push((i as u64, v));
    }
    vectors
}

fn bench_flat_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat_insert");
    
    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut index = FlatIndex::new(DistanceMetric::Cosine);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(black_box(*id), black_box(v.as_slice()));
                }
            });
        });
    }
    group.finish();
}

fn bench_flat_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("flat_search");
    
    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut index = FlatIndex::new(DistanceMetric::Cosine);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();
            
            b.iter(|| {
                let _ = index.search(black_box(&query), black_box(10));
            });
        });
    }
    group.finish();
}

fn bench_ivf_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivf_insert");
    
    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut index = IvfIndex::new(DistanceMetric::Euclidean, 10);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(black_box(*id), black_box(v.as_slice()));
                }
            });
        });
    }
    group.finish();
}

fn bench_ivf_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivf_search");
    
    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut index = IvfIndex::new(DistanceMetric::Euclidean, 10);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();
            
            b.iter(|| {
                let _ = index.search(black_box(&query), black_box(10));
            });
        });
    }
    group.finish();
}

fn bench_hnsw_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw_insert");
    
    for size in [100, 1000].iter() {
        let vectors = generate_random_vectors(*size, 64);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut index = HnswIndex::new(DistanceMetric::Cosine);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(black_box(*id), black_box(v.as_slice()));
                }
            });
        });
    }
    group.finish();
}

fn bench_hnsw_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw_search");
    
    for size in [100, 1000].iter() {
        let vectors = generate_random_vectors(*size, 64);
        let query = vec![0.5f32; 64];
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut index = HnswIndex::new(DistanceMetric::Cosine);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            
            b.iter(|| {
                let _ = index.search(black_box(&query), black_box(10));
            });
        });
    }
    group.finish();
}

fn bench_scalar_vs_vectorized(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_vs_vectorized");
    
    // This benchmark compares scalar loop vs potential SIMD path
    // Note: The actual SIMD implementation depends on compiler auto-vectorization
    
    let size = 10000;
    let dim = 256;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];
    
    // Manual scalar dot product
    group.bench_function("scalar_dot_product", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            for v in vectors.iter().take(100) {
                for (a, b) in query.iter().zip(v.1.iter()) {
                    sum += a * b;
                }
                black_box(sum);
            }
        });
    });
    
    // Using iterator (may be auto-vectorized)
    group.bench_function("iterator_dot_product", |b| {
        b.iter(|| {
            for v in vectors.iter().take(100) {
                let sum: f32 = query.iter().zip(v.1.iter()).map(|(a, b)| a * b).sum();
                black_box(sum);
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_flat_insert,
    bench_flat_search,
    bench_ivf_insert,
    bench_ivf_search,
    bench_hnsw_insert,
    bench_hnsw_search,
    bench_scalar_vs_vectorized
);
criterion_main!(benches);

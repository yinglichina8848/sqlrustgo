//! Vector index performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlrustgo_vector::{
    flat::FlatIndex,
    ivf::IvfIndex,
    hnsw::HnswIndex,
    metrics::DistanceMetric,
    parallel_knn::ParallelKnn,
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

fn bench_parallel_knn_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_knn_search");
    
    for size in [1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];
        
        // Sequential baseline
        group.bench_with_input(
            BenchmarkId::new("sequential", size), 
            size, 
            |b, &size| {
                let mut index = FlatIndex::new(DistanceMetric::Cosine);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(*id, v);
                }
                
                b.iter(|| {
                    let _ = index.search(black_box(&query), black_box(10));
                });
            }
        );
        
        // Parallel search
        group.bench_with_input(
            BenchmarkId::new("parallel", size), 
            size, 
            |b, &size| {
                let mut index = FlatIndex::new(DistanceMetric::Cosine);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(*id, v);
                }
                let parallel_index = ParallelKnn::new(index);
                
                b.iter(|| {
                    let result = parallel_index.parallel_search(black_box(&query), black_box(10));
                    black_box(result);
                });
            }
        );
    }
    group.finish();
}

fn bench_parallel_batch_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_batch_search");
    
    let sizes = [10, 50, 100];
    for size in sizes.iter() {
        let vectors = generate_random_vectors(10000, 128);
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        
        let queries: Vec<_> = (0..*size)
            .map(|_| (0..128).map(|_| rand::random::<f32>()).collect::<Vec<_>>())
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let results = sqlrustgo_vector::parallel_knn::batch_search(
                    &index, 
                    black_box(&queries[..size]), 
                    black_box(10)
                );
                black_box(results);
            });
        });
    }
    group.finish();
}

fn bench_write_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_throughput");
    
    for size in [1000, 10000, 100000].iter() {
        let dim = 128;
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut index = FlatIndex::new(DistanceMetric::Cosine);
                for i in 0..size {
                    let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
                    let _ = index.insert(black_box(i as u64), black_box(&v));
                }
            });
        });
    }
    group.finish();
}

fn bench_large_scale_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_search");
    
    // 100K vectors for memory-efficient testing
    let size = 100000;
    let dim = 128;
    let mut vectors = Vec::with_capacity(size);
    for i in 0..size {
        let v: Vec<f32> = (0..dim).map(|_| rand::random::<f32>()).collect();
        vectors.push((i as u64, v));
    }
    
    let query = vec![0.5f32; dim];
    
    // Flat index
    group.bench_function("flat_100k", |b| {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        
        b.iter(|| {
            let _ = index.search(black_box(&query), black_box(10));
        });
    });
    
    // IVF index
    group.bench_function("ivf_100k", |b| {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 100);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        index.build_index().unwrap();
        
        b.iter(|| {
            let _ = index.search(black_box(&query), black_box(10));
        });
    });
}

criterion_group!(
    benches,
    bench_flat_insert,
    bench_flat_search,
    bench_ivf_insert,
    bench_ivf_search,
    bench_hnsw_insert,
    bench_hnsw_search,
    bench_scalar_vs_vectorized,
    bench_parallel_knn_search,
    bench_parallel_batch_search,
    bench_write_throughput,
    bench_large_scale_search
);
criterion_main!(benches);

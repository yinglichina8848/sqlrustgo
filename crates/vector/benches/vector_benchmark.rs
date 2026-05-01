//! Vector index performance benchmarks
//!
//! Performance targets (Issue #1343):
//! - 10K vectors KNN < 5ms
//! - 100K vectors KNN < 10ms
//! - 1M vectors KNN < 100ms

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sqlrustgo_vector::{
    flat::FlatIndex, hnsw::HnswIndex, ivf::IvfIndex, ivfpq::IvfpqIndex, metrics::DistanceMetric,
    parallel_knn::ParallelKnn, sql_vector_hybrid::HybridSearcher, VectorIndex,
};

fn generate_random_vectors(n: usize, dim: usize) -> Vec<(u64, Vec<f32>)> {
    let mut vectors = Vec::with_capacity(n);
    for i in 0..n {
        let v: Vec<f32> = (0..dim)
            .map(|_| rand::random::<f32>() * 2.0 - 1.0)
            .collect();
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
                    let _ = index.insert(std::hint::black_box(*id), std::hint::black_box(v.as_slice()));
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
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
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
                    let _ = index.insert(std::hint::black_box(*id), std::hint::black_box(v.as_slice()));
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
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
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
                    let _ = index.insert(std::hint::black_box(*id), std::hint::black_box(v.as_slice()));
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
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
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
                std::hint::black_box(sum);
            }
        });
    });

    // Using iterator (may be auto-vectorized)
    group.bench_function("iterator_dot_product", |b| {
        b.iter(|| {
            for v in vectors.iter().take(100) {
                let sum: f32 = query.iter().zip(v.1.iter()).map(|(a, b)| a * b).sum();
                std::hint::black_box(sum);
            }
        });
    });

    group.finish();
}

fn bench_simd_vs_scalar(c: &mut Criterion) {
    use sqlrustgo_vector::simd_explicit::{
        batch_dot_product_simd, batch_l2_distance_simd, dot_product_simd,
        euclidean_distance_simd,
    };

    let mut group = c.benchmark_group("simd_vs_scalar");

    for dim in [256, 512, 1024, 2048].iter() {
        let query = vec![0.5f32; *dim];
        let vectors: Vec<Vec<f32>> = (0..100)
            .map(|_| (0..*dim).map(|_| rand::random::<f32>()).collect())
            .collect();

        // Scalar dot product
        group.bench_with_input(BenchmarkId::new("scalar_dot", dim), dim, |b, _| {
            b.iter(|| {
                for v in &vectors {
                    let sum: f32 = query.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                    std::hint::black_box(sum);
                }
            });
        });

        // SIMD dot product
        group.bench_with_input(BenchmarkId::new("simd_dot", dim), dim, |b, _| {
            b.iter(|| {
                for v in &vectors {
                    let sum = dot_product_simd(&query, v);
                    std::hint::black_box(sum);
                }
            });
        });

        // Batch SIMD dot product
        group.bench_with_input(BenchmarkId::new("simd_batch_dot", dim), dim, |b, _| {
            b.iter(|| {
                let _ = batch_dot_product_simd(&query, &vectors);
            });
        });

        // Scalar L2 distance
        group.bench_with_input(BenchmarkId::new("scalar_l2", dim), dim, |b, _| {
            b.iter(|| {
                for v in &vectors {
                    let dist: f32 = query
                        .iter()
                        .zip(v.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    std::hint::black_box(dist);
                }
            });
        });

        // SIMD L2 distance
        group.bench_with_input(BenchmarkId::new("simd_l2", dim), dim, |b, _| {
            b.iter(|| {
                for v in &vectors {
                    let dist = euclidean_distance_simd(&query, v);
                    std::hint::black_box(dist);
                }
            });
        });

        // Batch SIMD L2 distance
        group.bench_with_input(BenchmarkId::new("simd_batch_l2", dim), dim, |b, _| {
            b.iter(|| {
                let _ = batch_l2_distance_simd(&query, &vectors);
            });
        });
    }

    group.finish();
}

fn bench_hnsw_ivfpq_e2e(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw_ivfpq_e2e");

    for size in [1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];

        // HNSW search
        group.bench_with_input(BenchmarkId::new("hnsw", size), size, |b, &size| {
            let mut index = HnswIndex::new(DistanceMetric::Euclidean);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });

        // IVFPQ search
        group.bench_with_input(BenchmarkId::new("ivfpq", size), size, |b, &size| {
            let mut index = IvfpqIndex::new(DistanceMetric::Euclidean, 16, 8);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }

    group.finish();
}

fn bench_parallel_knn_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_knn_search");

    for size in [1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];

        // Sequential baseline
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &size| {
            let mut index = FlatIndex::new(DistanceMetric::Cosine);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });

        // Parallel search
        group.bench_with_input(BenchmarkId::new("parallel", size), size, |b, &size| {
            let mut index = FlatIndex::new(DistanceMetric::Cosine);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            let parallel_index = ParallelKnn::new(index);

            b.iter(|| {
                let result = parallel_index.parallel_search(std::hint::black_box(&query), std::hint::black_box(10));
                let _ = std::hint::black_box(result);
            });
        });
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
                    std::hint::black_box(&queries[..size]),
                    std::hint::black_box(10),
                );
                let _ = std::hint::black_box(results);
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
                    let _ = index.insert(std::hint::black_box(i as u64), std::hint::black_box(&v));
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
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
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
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });
}

fn bench_10k_knn_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_10k_knn");
    group.sample_size(50);

    let size = 10_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("flat_cosine", |b| {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.bench_function("hnsw_cosine", |b| {
        let mut index = HnswIndex::with_params(16, 128, 64, DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });
}

fn bench_100k_knn_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_100k_knn");
    group.measurement_time(std::time::Duration::from_secs(2));
    group.sample_size(10);

    let size = 100_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("flat_cosine", |b| {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.bench_function("hnsw_cosine", |b| {
        let mut index = HnswIndex::with_params(16, 64, 32, DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.bench_function("ivf_cosine", |b| {
        let mut index = IvfIndex::new(DistanceMetric::Euclidean, 100);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        index.build_index().unwrap();

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.bench_function("parallel_knn_cosine", |b| {
        let mut index = FlatIndex::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        let parallel_index = ParallelKnn::new(index);

        b.iter(|| {
            let _ = parallel_index.parallel_search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });
}

fn bench_1m_knn_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_1m_knn");
    group.sample_size(10);

    let size = 1_000_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("hnsw_cosine", |b| {
        let mut index = HnswIndex::with_params(16, 128, 64, DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });
}

fn bench_hnsw_parameter_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("hnsw_param_tuning");

    let size = 50_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    for ef_search in [16, 32, 64, 128, 256].iter() {
        group.bench_with_input(
            BenchmarkId::new("ef_search", ef_search),
            ef_search,
            |b, &ef| {
                let mut index = HnswIndex::with_params(16, 128, ef, DistanceMetric::Cosine);
                for (id, v) in vectors.iter() {
                    let _ = index.insert(*id, v);
                }

                b.iter(|| {
                    let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
                });
            },
        );
    }

    for m in [8, 16, 32, 64].iter() {
        group.bench_with_input(BenchmarkId::new("m_param", m), m, |b, &m_val| {
            let mut index = HnswIndex::with_params(m_val, 128, 64, DistanceMetric::Cosine);
            for (id, v) in vectors.iter() {
                let _ = index.insert(*id, v);
            }

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }
}

fn bench_ivf_parameter_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivf_param_tuning");

    let size = 50_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    for nlists in [50, 100, 200, 500].iter() {
        group.bench_with_input(BenchmarkId::new("nlists", nlists), nlists, |b, &nlist| {
            let mut index = IvfIndex::new(DistanceMetric::Euclidean, nlist);
            for (id, v) in vectors.iter() {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }
}

fn bench_hybrid_query_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_query");

    let size = 100_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("hybrid_weighted_scoring", |b| {
        let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let sql_score = 1.0 - (*id as f32 / size as f32);
            let _ = searcher.insert(*id, v, sql_score);
        }

        let sql_scores: Vec<_> = (0..size as u64)
            .map(|id| (id, 1.0 - (id as f32 / size as f32)))
            .collect();

        b.iter(|| {
            let _ = searcher.search_hybrid(std::hint::black_box(&query), &sql_scores, std::hint::black_box(10));
        });
    });

    group.bench_function("hybrid_filtered_search", |b| {
        let mut searcher = HybridSearcher::new(DistanceMetric::Cosine);
        for (id, v) in vectors.iter() {
            let _ = searcher.insert(*id, v, 1.0);
        }

        let predicates = vec![];

        b.iter(|| {
            let _ = searcher.execute_filtered_search(std::hint::black_box(&query), &predicates, std::hint::black_box(10));
        });
    });
}

// ============================================================================
// IVFPQ Benchmarks (Issue #1343)
// ============================================================================

fn bench_ivfpq_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivfpq_insert");

    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 10, 16);
                for (id, v) in vectors.iter().take(size) {
                    let _ = index.insert(std::hint::black_box(*id), std::hint::black_box(v.as_slice()));
                }
            });
        });
    }
    group.finish();
}

fn bench_ivfpq_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivfpq_search");

    for size in [100, 1000, 10000].iter() {
        let vectors = generate_random_vectors(*size, 128);
        let query = vec![0.5f32; 128];

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 10, 16);
            for (id, v) in vectors.iter().take(size) {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }
    group.finish();
}

fn bench_ivfpq_10k_knn(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_ivfpq_10k_knn");
    group.sample_size(50);

    let size = 10_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("ivfpq_cosine", |b| {
        let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 64, 16);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        index.build_index().unwrap();

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.finish();
}

fn bench_ivfpq_100k_knn(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_ivfpq_100k_knn");
    group.measurement_time(std::time::Duration::from_secs(2));
    group.sample_size(10);

    let size = 100_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("ivfpq_cosine", |b| {
        let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 128, 16);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        index.build_index().unwrap();

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.finish();
}

fn bench_ivfpq_1m_knn(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_1343_ivfpq_1m_knn");
    group.sample_size(10);

    let size = 1_000_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    group.bench_function("ivfpq_cosine", |b| {
        let mut index = IvfpqIndex::new(DistanceMetric::Cosine, 256, 16);
        for (id, v) in vectors.iter() {
            let _ = index.insert(*id, v);
        }
        index.build_index().unwrap();

        b.iter(|| {
            let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
        });
    });

    group.finish();
}

fn bench_ivfpq_parameter_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("ivfpq_param_tuning");

    let size = 50_000;
    let dim = 128;
    let vectors = generate_random_vectors(size, dim);
    let query = vec![0.5f32; dim];

    for nlists in [32, 64, 128, 256].iter() {
        group.bench_with_input(BenchmarkId::new("nlist", nlists), nlists, |b, &nlist| {
            let mut index = IvfpqIndex::new(DistanceMetric::Cosine, nlist, 16);
            for (id, v) in vectors.iter() {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }

    for k_sub in [32, 64, 128, 256].iter() {
        group.bench_with_input(BenchmarkId::new("k_sub", k_sub), k_sub, |b, &k| {
            let mut index = IvfpqIndex::with_params(128, 16, k, DistanceMetric::Cosine);
            for (id, v) in vectors.iter() {
                let _ = index.insert(*id, v);
            }
            index.build_index().unwrap();

            b.iter(|| {
                let _ = index.search(std::hint::black_box(&query), std::hint::black_box(10));
            });
        });
    }
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
    bench_simd_vs_scalar,
    bench_hnsw_ivfpq_e2e,
    bench_parallel_knn_search,
    bench_parallel_batch_search,
    bench_write_throughput,
    bench_large_scale_search,
    bench_10k_knn_performance,
    bench_100k_knn_performance,
    bench_1m_knn_performance,
    bench_hnsw_parameter_tuning,
    bench_ivf_parameter_tuning,
    bench_hybrid_query_performance,
    bench_ivfpq_insert,
    bench_ivfpq_search,
    bench_ivfpq_10k_knn,
    bench_ivfpq_100k_knn,
    bench_ivfpq_1m_knn,
    bench_ivfpq_parameter_tuning
);
criterion_main!(benches);

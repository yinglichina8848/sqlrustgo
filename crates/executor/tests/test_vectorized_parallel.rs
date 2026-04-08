//! 向量化并行执行集成测试
//!
//! 测试向量化执行引擎与列式存储的集成，包括性能测试。

use sqlrustgo_executor::parallel_vector_executor::{ParallelVectorExecutor, PartitionInfo};
use sqlrustgo_executor::vector_executor::VectorizedSeqScanExecutor;
use sqlrustgo_executor::vectorization::{simd_agg, AggFunction};
use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value;
use std::time::Instant;

/// ==================== VectorizedSeqScanExecutor 测试 ====================

#[test]
fn test_vectorized_seq_scan_basic() {
    let mut storage = MemoryStorage::new();
    let records = vec![
        vec![Value::Integer(1), Value::Text("Alice".to_string())],
        vec![Value::Integer(2), Value::Text("Bob".to_string())],
        vec![Value::Integer(3), Value::Text("Charlie".to_string())],
    ];
    storage.insert("users", records).unwrap();

    let storage = Box::new(storage);
    let mut executor = VectorizedSeqScanExecutor::new("users".to_string(), storage, vec![], 1024);

    executor.init().unwrap();
    let batch = executor.next_batch().unwrap();

    assert!(batch.is_some());
    let batch = batch.unwrap();
    assert_eq!(batch.num_rows(), 3);
    assert_eq!(batch.num_columns(), 2);
}

#[test]
fn test_vectorized_seq_scan_empty_table() {
    let storage = Box::new(MemoryStorage::new());
    let mut executor =
        VectorizedSeqScanExecutor::new("empty_table".to_string(), storage, vec![], 1024);

    executor.init().unwrap();
    let batch = executor.next_batch().unwrap();
    assert!(batch.is_none());
}

#[test]
fn test_vectorized_seq_scan_projection() {
    let mut storage = MemoryStorage::new();
    let records = vec![
        vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Integer(25),
        ],
        vec![
            Value::Integer(2),
            Value::Text("Bob".to_string()),
            Value::Integer(30),
        ],
    ];
    storage.insert("users", records).unwrap();

    let storage = Box::new(storage);
    let mut executor = VectorizedSeqScanExecutor::new(
        "users".to_string(),
        storage,
        vec![0, 2], // Project only id and age columns
        1024,
    );

    let result = executor.execute_collect().unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].len(), 2); // Only id and age
}

#[test]
fn test_vectorized_seq_scan_batch_processing() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let storage = Box::new(storage);
    let mut executor = VectorizedSeqScanExecutor::new("numbers".to_string(), storage, vec![], 30);

    executor.init().unwrap();
    let mut total_rows = 0;

    while let Ok(Some(batch)) = executor.next_batch() {
        total_rows += batch.num_rows();
    }

    assert_eq!(total_rows, 100);
}

#[test]
fn test_vectorized_seq_scan_all_data_types() {
    let mut storage = MemoryStorage::new();
    let records = vec![
        vec![
            Value::Integer(1),
            Value::Float(1.5),
            Value::Boolean(true),
            Value::Text("hello".to_string()),
        ],
        vec![
            Value::Integer(2),
            Value::Float(2.5),
            Value::Boolean(false),
            Value::Text("world".to_string()),
        ],
    ];
    storage.insert("mixed", records).unwrap();

    let storage = Box::new(storage);
    let mut executor = VectorizedSeqScanExecutor::new("mixed".to_string(), storage, vec![], 1024);

    let result = executor.execute_collect().unwrap();
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].len(), 4);
}

// ==================== PartitionInfo 测试 ====================

#[test]
fn test_partition_info_even_split() {
    let info = PartitionInfo::new(0, 4, 100);
    assert_eq!(info.partition_id, 0);
    assert_eq!(info.num_partitions, 4);
    assert_eq!(info.num_rows, 25);
    assert_eq!(info.row_offset, 0);
}

#[test]
fn test_partition_info_with_remainder() {
    // 100 rows / 3 partitions = 33, 33, 34
    let info0 = PartitionInfo::new(0, 3, 100);
    assert_eq!(info0.num_rows, 34);
    assert_eq!(info0.row_offset, 0);

    let info1 = PartitionInfo::new(1, 3, 100);
    assert_eq!(info1.num_rows, 33);
    assert_eq!(info1.row_offset, 34);

    let info2 = PartitionInfo::new(2, 3, 100);
    assert_eq!(info2.num_rows, 33);
    assert_eq!(info2.row_offset, 67);
}

#[test]
fn test_partition_info_total_coverage() {
    let num_partitions = 4;
    let total_rows = 100;

    let total_covered: usize = (0..num_partitions)
        .map(|i| PartitionInfo::new(i, num_partitions, total_rows).num_rows)
        .sum();

    assert_eq!(total_covered, total_rows);
}

// ==================== ParallelVectorExecutor 测试 ====================

#[test]
fn test_parallel_vector_executor_creation() {
    let mut storage = MemoryStorage::new();
    storage
        .insert(
            "test",
            vec![
                vec![Value::Integer(1)],
                vec![Value::Integer(2)],
                vec![Value::Integer(3)],
            ],
        )
        .unwrap();

    let executor =
        ParallelVectorExecutor::new("test".to_string(), Box::new(storage), 2, vec![], 1024)
            .unwrap();

    assert_eq!(executor.num_partitions(), 2);
}

#[test]
fn test_parallel_scan_all_rows_collected() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![], 30)
            .unwrap();

    let chunks = executor.execute_parallel_scan().unwrap();
    let total_rows: usize = chunks.iter().map(|c| c.num_rows()).sum();

    assert_eq!(total_rows, 100);
}

#[test]
fn test_parallel_scan_agg_count() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![(AggFunction::Count(0), 0)])
        .unwrap();

    assert_eq!(result.values.len(), 1);
    assert_eq!(result.values[0], Value::Integer(100));
}

#[test]
fn test_parallel_scan_agg_sum() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![(AggFunction::Sum(0), 0)])
        .unwrap();

    // Sum of 0..99 = 4950
    assert_eq!(result.values[0], Value::Integer(4950));
}

#[test]
fn test_parallel_scan_agg_min_max() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![(AggFunction::Min(0), 0), (AggFunction::Max(0), 0)])
        .unwrap();

    assert_eq!(result.values.len(), 2);
    assert_eq!(result.values[0], Value::Integer(0));
    assert_eq!(result.values[1], Value::Integer(99));
}

#[test]
fn test_parallel_scan_agg_avg() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![(AggFunction::Avg(0), 0)])
        .unwrap();

    // Average of 0..99 = 49.5
    assert_eq!(result.values[0], Value::Float(49.5));
}

#[test]
fn test_parallel_scan_multiple_threads() {
    // Helper function to create storage
    fn create_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();
        let records: Vec<Vec<Value>> = (0..1000).map(|i| vec![Value::Integer(i as i64)]).collect();
        storage.insert("numbers", records).unwrap();
        storage
    }

    // Test with 1, 2, 4, 8 threads
    for num_threads in [1, 2, 4, 8] {
        let executor = ParallelVectorExecutor::new(
            "numbers".to_string(),
            Box::new(create_storage()),
            num_threads,
            vec![],
            1024,
        )
        .unwrap();

        let chunks = executor.execute_parallel_scan().unwrap();
        let total_rows: usize = chunks.iter().map(|c| c.num_rows()).sum();
        assert_eq!(total_rows, 1000, "Thread count: {}", num_threads);
    }
}

#[test]
fn test_parallel_scan_agg_multiple_functions() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![
            (AggFunction::Count(0), 0),
            (AggFunction::Sum(0), 0),
            (AggFunction::Avg(0), 0),
            (AggFunction::Min(0), 0),
            (AggFunction::Max(0), 0),
        ])
        .unwrap();

    assert_eq!(result.values.len(), 5);
    assert_eq!(result.values[0], Value::Integer(100)); // Count
    assert_eq!(result.values[1], Value::Integer(4950)); // Sum
    assert_eq!(result.values[2], Value::Float(49.5)); // Avg
    assert_eq!(result.values[3], Value::Integer(0)); // Min
    assert_eq!(result.values[4], Value::Integer(99)); // Max
}

// ==================== SIMD 聚合函数测试 ====================

#[test]
fn test_simd_sum_i64_large() {
    let values: Vec<i64> = (0..1_000_000).collect();
    let start = Instant::now();
    let result = simd_agg::sum_i64(&values);
    let elapsed = start.elapsed();

    // Sum of 0..999999 = 499999500000
    assert_eq!(result, 499999500000);
    println!("sum_i64 (1M elements): {:?}", elapsed);
}

#[test]
fn test_simd_sum_f64_large() {
    let values: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    let start = Instant::now();
    let result = simd_agg::sum_f64(&values);
    let elapsed = start.elapsed();

    // Sum should be approximately 499999500000.0
    assert!((result - 499999500000.0).abs() < 1.0);
    println!("sum_f64 (1M elements): {:?}", elapsed);
}

#[test]
fn test_simd_avg_i64() {
    let values: Vec<i64> = (1..=100).collect();
    let avg = simd_agg::avg_i64(&values);
    assert!((avg - 50.5).abs() < 0.001);
}

#[test]
fn test_simd_avg_f64() {
    let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let avg = simd_agg::avg_f64(&values);
    assert!((avg - 50.5).abs() < 0.001);
}

#[test]
fn test_simd_min_max_i64() {
    let values: Vec<i64> = (0..10000).map(|i| if i == 5000 { -1 } else { i }).collect();
    assert_eq!(simd_agg::min_i64(&values), Some(-1));
    assert_eq!(simd_agg::max_i64(&values), Some(9999));
}

#[test]
fn test_simd_count_f64_with_nan() {
    let values: Vec<f64> = vec![1.0, 2.0, f64::NAN, 4.0, 5.0];
    assert_eq!(simd_agg::count_f64(&values), 4);
}

// ==================== 性能测试 ====================

fn generate_test_data(num_rows: usize) -> MemoryStorage {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..num_rows)
        .map(|i| {
            vec![
                Value::Integer(i as i64),
                Value::Integer((i * 2) as i64),
                Value::Float((i as f64) * 1.5),
            ]
        })
        .collect();
    storage.insert("perf_test", records).unwrap();
    storage
}

#[test]
fn test_perf_vectorized_scan_1k_rows() {
    let storage = generate_test_data(1000);
    let storage = Box::new(storage);
    let mut executor =
        VectorizedSeqScanExecutor::new("perf_test".to_string(), storage, vec![], 1024);

    let start = Instant::now();
    executor.init().unwrap();
    let mut count = 0;
    while let Ok(Some(batch)) = executor.next_batch() {
        count += batch.num_rows();
    }
    let elapsed = start.elapsed();

    assert_eq!(count, 1000);
    println!("Vectorized scan 1K rows: {:?}", elapsed);
}

#[test]
fn test_perf_vectorized_scan_100k_rows() {
    let storage = generate_test_data(100_000);
    let storage = Box::new(storage);
    let mut executor =
        VectorizedSeqScanExecutor::new("perf_test".to_string(), storage, vec![], 1024);

    let start = Instant::now();
    executor.init().unwrap();
    let mut count = 0;
    while let Ok(Some(batch)) = executor.next_batch() {
        count += batch.num_rows();
    }
    let elapsed = start.elapsed();

    assert_eq!(count, 100_000);
    println!("Vectorized scan 100K rows: {:?}", elapsed);
}

#[test]
fn test_perf_parallel_scan_1m_rows_1_thread() {
    let storage = generate_test_data(1_000_000);
    let executor =
        ParallelVectorExecutor::new("perf_test".to_string(), Box::new(storage), 1, vec![], 4096)
            .unwrap();

    let start = Instant::now();
    let chunks = executor.execute_parallel_scan().unwrap();
    let total_rows: usize = chunks.iter().map(|c| c.num_rows()).sum();
    let elapsed = start.elapsed();

    assert_eq!(total_rows, 1_000_000);
    println!("Parallel scan 1M rows (1 thread): {:?}", elapsed);
}

#[test]
fn test_perf_parallel_scan_1m_rows_4_threads() {
    let storage = generate_test_data(1_000_000);
    let executor =
        ParallelVectorExecutor::new("perf_test".to_string(), Box::new(storage), 4, vec![], 4096)
            .unwrap();

    let start = Instant::now();
    let chunks = executor.execute_parallel_scan().unwrap();
    let total_rows: usize = chunks.iter().map(|c| c.num_rows()).sum();
    let elapsed = start.elapsed();

    assert_eq!(total_rows, 1_000_000);
    println!("Parallel scan 1M rows (4 threads): {:?}", elapsed);
}

#[test]
fn test_perf_parallel_scan_1m_rows_8_threads() {
    let storage = generate_test_data(1_000_000);
    let executor =
        ParallelVectorExecutor::new("perf_test".to_string(), Box::new(storage), 8, vec![], 4096)
            .unwrap();

    let start = Instant::now();
    let chunks = executor.execute_parallel_scan().unwrap();
    let total_rows: usize = chunks.iter().map(|c| c.num_rows()).sum();
    let elapsed = start.elapsed();

    assert_eq!(total_rows, 1_000_000);
    println!("Parallel scan 1M rows (8 threads): {:?}", elapsed);
}

#[test]
fn test_perf_parallel_agg_1m_rows() {
    let storage = generate_test_data(1_000_000);
    let executor =
        ParallelVectorExecutor::new("perf_test".to_string(), Box::new(storage), 4, vec![0], 4096)
            .unwrap();

    let start = Instant::now();
    let result = executor
        .execute_parallel_scan_agg(vec![
            (AggFunction::Count(0), 0),
            (AggFunction::Sum(0), 0),
            (AggFunction::Avg(0), 0),
            (AggFunction::Min(0), 0),
            (AggFunction::Max(0), 0),
        ])
        .unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result.values[0], Value::Integer(1_000_000));
    println!("Parallel agg 1M rows (4 threads): {:?}", elapsed);
}

#[test]
fn test_perf_simd_sum_10m_elements() {
    let values: Vec<i64> = (0..10_000_000).collect();
    let start = Instant::now();
    let result = simd_agg::sum_i64(&values);
    let elapsed = start.elapsed();

    // Sum of 0..9999999 = 49999995000000
    assert_eq!(result, 49_999_995_000_000);
    println!("SIMD sum 10M elements: {:?}", elapsed);
}

// ==================== 数据类型边界测试 ====================

#[test]
fn test_agg_with_all_zeros() {
    let values: Vec<i64> = vec![0, 0, 0, 0, 0];
    assert_eq!(simd_agg::sum_i64(&values), 0);
    assert_eq!(simd_agg::min_i64(&values), Some(0));
    assert_eq!(simd_agg::max_i64(&values), Some(0));
}

#[test]
fn test_agg_with_negative_values() {
    let values: Vec<i64> = vec![-10, -5, 0, 5, 10];
    assert_eq!(simd_agg::sum_i64(&values), 0);
    assert_eq!(simd_agg::min_i64(&values), Some(-10));
    assert_eq!(simd_agg::max_i64(&values), Some(10));
}

#[test]
fn test_agg_with_large_values() {
    let values: Vec<i64> = vec![i64::MAX / 2, i64::MAX / 2];
    let sum = simd_agg::sum_i64(&values);
    // Should not overflow due to wrapping
    assert!(sum > 0);
}

#[test]
fn test_agg_empty_slice() {
    let values: Vec<i64> = vec![];
    assert_eq!(simd_agg::sum_i64(&values), 0);
    assert!(simd_agg::avg_i64(&values).is_nan());
    assert_eq!(simd_agg::min_i64(&values), None);
    assert_eq!(simd_agg::max_i64(&values), None);
}

#[test]
fn test_agg_single_element() {
    let values: Vec<i64> = vec![42];
    assert_eq!(simd_agg::sum_i64(&values), 42);
    assert_eq!(simd_agg::avg_i64(&values), 42.0);
    assert_eq!(simd_agg::min_i64(&values), Some(42));
    assert_eq!(simd_agg::max_i64(&values), Some(42));
}

// ==================== 并行执行正确性测试 ====================

#[test]
fn test_parallel_results_match_sequential() {
    // Helper to create storage
    fn create_storage() -> MemoryStorage {
        let mut storage = MemoryStorage::new();
        let records: Vec<Vec<Value>> = (0..1000).map(|i| vec![Value::Integer(i as i64)]).collect();
        storage.insert("numbers", records).unwrap();
        storage
    }

    // Sequential scan
    let mut seq_storage = create_storage();
    let seq_storage_box = Box::new(seq_storage);
    let mut seq_executor =
        VectorizedSeqScanExecutor::new("numbers".to_string(), seq_storage_box, vec![], 1024);
    let seq_result = seq_executor.execute_collect().unwrap();

    // Parallel scan (use separate storage)
    let par_storage = create_storage();
    let par_executor = ParallelVectorExecutor::new(
        "numbers".to_string(),
        Box::new(par_storage),
        4,
        vec![],
        1024,
    )
    .unwrap();
    let par_chunks = par_executor.execute_parallel_scan().unwrap();

    // Collect all parallel results
    let mut par_rows: Vec<Vec<Value>> = Vec::new();
    for chunk in par_chunks {
        par_rows.extend(chunk.to_rows());
    }

    // Sort both for comparison (parallel may return in different order)
    let mut seq_rows = seq_result.rows;
    seq_rows.sort_by_key(|r| format!("{:?}", r));
    par_rows.sort_by_key(|r| format!("{:?}", r));

    assert_eq!(seq_rows.len(), par_rows.len());
    for (s, p) in seq_rows.iter().zip(par_rows.iter()) {
        assert_eq!(s, p);
    }
}

#[test]
fn test_parallel_agg_results_match_sequential() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..1000).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    // Calculate expected results using SIMD directly
    let values: Vec<i64> = (0..1000).collect();
    let expected_count = simd_agg::count_i64(&values);
    let expected_sum = simd_agg::sum_i64(&values);
    let expected_avg = simd_agg::avg_i64(&values);
    let expected_min = simd_agg::min_i64(&values).unwrap();
    let expected_max = simd_agg::max_i64(&values).unwrap();

    // Parallel aggregate
    let executor =
        ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![0], 1024)
            .unwrap();

    let result = executor
        .execute_parallel_scan_agg(vec![
            (AggFunction::Count(0), 0),
            (AggFunction::Sum(0), 0),
            (AggFunction::Avg(0), 0),
            (AggFunction::Min(0), 0),
            (AggFunction::Max(0), 0),
        ])
        .unwrap();

    assert_eq!(result.values[0], Value::Integer(expected_count));
    assert_eq!(result.values[1], Value::Integer(expected_sum));
    assert_eq!(result.values[2], Value::Float(expected_avg));
    assert_eq!(result.values[3], Value::Integer(expected_min));
    assert_eq!(result.values[4], Value::Integer(expected_max));
}

// ==================== 内存效率测试 ====================

#[test]
fn test_batch_processing_respects_batch_size() {
    let mut storage = MemoryStorage::new();
    let records: Vec<Vec<Value>> = (0..250).map(|i| vec![Value::Integer(i as i64)]).collect();
    storage.insert("numbers", records).unwrap();

    let storage = Box::new(storage);
    let mut executor = VectorizedSeqScanExecutor::new(
        "numbers".to_string(),
        storage,
        vec![],
        100, // Batch size = 100
    );

    executor.init().unwrap();

    let batch1 = executor.next_batch().unwrap().unwrap();
    let batch2 = executor.next_batch().unwrap().unwrap();
    let batch3 = executor.next_batch().unwrap().unwrap();
    let batch4 = executor.next_batch().unwrap();

    assert_eq!(batch1.num_rows(), 100);
    assert_eq!(batch2.num_rows(), 100);
    assert_eq!(batch3.num_rows(), 50);
    assert!(batch4.is_none()); // No more batches after 250 rows
}

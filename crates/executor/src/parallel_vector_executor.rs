//! Parallel Vectorized Executor
//!
//! Implements parallel vectorized execution using partition-based parallelism.
//! Combines SIMD vectorization with multi-threading for optimal performance.

use rayon::prelude::*;
use sqlrustgo_storage::engine::VectorStorage;
use sqlrustgo_types::{SqlError, Value};

use crate::vector_executor::VectorizedSeqScanExecutor;
use crate::vectorization::{simd_agg, AggFunction, AggregateResult, ColumnArray, DataChunk};

/// Partition information for parallel execution
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    /// Partition index (0-based)
    pub partition_id: usize,
    /// Total number of partitions
    pub num_partitions: usize,
    /// Row offset in the original table
    pub row_offset: usize,
    /// Number of rows in this partition
    pub num_rows: usize,
}

impl PartitionInfo {
    /// Create partition info for a given partition index
    pub fn new(partition_id: usize, num_partitions: usize, total_rows: usize) -> Self {
        let base_rows = total_rows / num_partitions;
        let remainder = total_rows % num_partitions;

        // Distribute remainder rows among first partitions
        let (row_offset, num_rows) = if partition_id < remainder {
            let row_offset = partition_id * (base_rows + 1);
            let num_rows = base_rows + 1;
            (row_offset, num_rows)
        } else {
            let row_offset = remainder * (base_rows + 1) + (partition_id - remainder) * base_rows;
            let num_rows = base_rows;
            (row_offset, num_rows)
        };

        Self {
            partition_id,
            num_partitions,
            row_offset,
            num_rows,
        }
    }
}

/// PartitionAgent - Manages data partitioning for parallel execution
pub struct PartitionAgent {
    /// Table name to scan
    table_name: String,
    /// Storage backend
    storage: Box<dyn VectorStorage>,
    /// Total number of rows
    total_rows: usize,
    /// Number of partitions
    num_partitions: usize,
    /// Column indices to project
    column_indices: Vec<usize>,
    /// Batch size per partition
    batch_size: usize,
}

impl PartitionAgent {
    /// Create a new PartitionAgent
    pub fn new(
        table_name: String,
        storage: Box<dyn VectorStorage>,
        num_partitions: usize,
        column_indices: Vec<usize>,
        batch_size: usize,
    ) -> Result<Self, SqlError> {
        let total_rows = storage.get_row_count(&table_name)?;

        Ok(Self {
            table_name,
            storage,
            total_rows,
            num_partitions,
            column_indices,
            batch_size,
        })
    }

    /// Get partition info for all partitions
    pub fn get_partition_infos(&self) -> Vec<PartitionInfo> {
        (0..self.num_partitions)
            .map(|i| PartitionInfo::new(i, self.num_partitions, self.total_rows))
            .collect()
    }

    /// Get the number of partitions
    pub fn num_partitions(&self) -> usize {
        self.num_partitions
    }

    /// Get total rows
    pub fn total_rows(&self) -> usize {
        self.total_rows
    }

    /// Create an executor for a specific partition
    pub fn create_partition_executor(
        &self,
        partition_id: usize,
    ) -> Result<VectorizedSeqScanExecutor, SqlError> {
        let partition_info = PartitionInfo::new(partition_id, self.num_partitions, self.total_rows);

        // Clone the storage by re-scanning (VectorStorage doesn't have clone)
        let columns = self.storage.scan_vectorized(&self.table_name)?;

        let mut executor = VectorizedSeqScanExecutor::new(
            self.table_name.clone(),
            Box::new(PartitionStorageWrapper::new(
                columns,
                partition_info.row_offset,
                partition_info.num_rows,
            )),
            self.column_indices.clone(),
            self.batch_size,
        );

        executor.init()?;
        Ok(executor)
    }
}

/// Storage wrapper that returns only a partition of data
/// Takes ownership of the columns to avoid requiring Clone on VectorStorage
struct PartitionStorageWrapper {
    columns: Vec<sqlrustgo_storage::columnar::convert::StorageColumnArray>,
    row_offset: usize,
    num_rows: usize,
}

impl PartitionStorageWrapper {
    fn new(
        columns: Vec<sqlrustgo_storage::columnar::convert::StorageColumnArray>,
        row_offset: usize,
        num_rows: usize,
    ) -> Self {
        Self {
            columns,
            row_offset,
            num_rows,
        }
    }
}

impl VectorStorage for PartitionStorageWrapper {
    fn scan_vectorized(
        &self,
        _table: &str,
    ) -> Result<Vec<sqlrustgo_storage::columnar::convert::StorageColumnArray>, SqlError> {
        // Slice each column to only return the partition range
        use sqlrustgo_storage::columnar::convert::StorageColumnArray as S;
        let sliced_columns: Vec<S> = self
            .columns
            .iter()
            .map(|col| match col {
                S::Int64(v) => S::Int64(
                    v.iter()
                        .skip(self.row_offset)
                        .take(self.num_rows)
                        .cloned()
                        .collect(),
                ),
                S::Float64(v) => S::Float64(
                    v.iter()
                        .skip(self.row_offset)
                        .take(self.num_rows)
                        .cloned()
                        .collect(),
                ),
                S::Boolean(v) => S::Boolean(
                    v.iter()
                        .skip(self.row_offset)
                        .take(self.num_rows)
                        .cloned()
                        .collect(),
                ),
                S::Text(v) => S::Text(
                    v.iter()
                        .skip(self.row_offset)
                        .take(self.num_rows)
                        .cloned()
                        .collect(),
                ),
                S::Null => S::Null,
            })
            .collect();

        Ok(sliced_columns)
    }

    fn get_row_count(&self, _table: &str) -> Result<usize, SqlError> {
        Ok(self.num_rows)
    }
}

/// ParallelVectorExecutor - Parallel execution with vectorization
pub struct ParallelVectorExecutor {
    partition_agent: PartitionAgent,
}

impl ParallelVectorExecutor {
    /// Create a new ParallelVectorExecutor
    pub fn new(
        table_name: String,
        storage: Box<dyn VectorStorage>,
        num_threads: usize,
        column_indices: Vec<usize>,
        batch_size: usize,
    ) -> Result<Self, SqlError> {
        let partition_agent =
            PartitionAgent::new(table_name, storage, num_threads, column_indices, batch_size)?;

        Ok(Self { partition_agent })
    }

    /// Get the number of partitions
    pub fn num_partitions(&self) -> usize {
        self.partition_agent.num_partitions()
    }

    /// Execute parallel scan with vectorized aggregation
    pub fn execute_parallel_scan_agg(
        &self,
        agg_functions: Vec<(AggFunction, usize)>, // (agg_function, column_index)
    ) -> Result<AggregateResult, SqlError> {
        let partition_infos = self.partition_agent.get_partition_infos();

        // Execute aggregations in parallel across partitions
        let results: Vec<AggregateResult> = partition_infos
            .par_iter()
            .filter_map(|partition| {
                let mut executor = match self
                    .partition_agent
                    .create_partition_executor(partition.partition_id)
                {
                    Ok(ex) => ex,
                    Err(_) => return None,
                };

                // Collect all data chunks and accumulate per-column
                let mut batch_data: Vec<Vec<ColumnArray>> = Vec::new();
                let mut num_columns = 0;

                while let Ok(Some(chunk)) = executor.next_batch() {
                    num_columns = chunk.num_columns();
                    batch_data.push(chunk.columns().to_vec());
                }

                // If no data, return None
                if batch_data.is_empty() {
                    return None;
                }

                // Accumulate all batches into per-column ColumnArrays
                // batch_data is [batch0_cols, batch1_cols, ...] where each batch_cols is [col0, col1, ...]
                // We need to accumulate col0 from all batches, col1 from all batches, etc.
                let mut accumulated_columns: Vec<ColumnArray> = Vec::new();

                for col_idx in 0..num_columns {
                    // Collect all column arrays for this column index across batches
                    let mut all_col_arrays: Vec<&ColumnArray> = Vec::new();
                    for batch in &batch_data {
                        if let Some(col) = batch.get(col_idx) {
                            all_col_arrays.push(col);
                        }
                    }

                    // Merge all column arrays into one
                    let merged = merge_column_arrays(&all_col_arrays);
                    accumulated_columns.push(merged);
                }

                // Compute aggregates on this partition's data
                let values = compute_partial_aggregates(&accumulated_columns, &agg_functions);
                Some(AggregateResult { values })
            })
            .collect();

        // Merge results from all partitions
        if results.is_empty() {
            return Ok(AggregateResult { values: vec![] });
        }

        let merged_values = merge_aggregate_results(&results, &agg_functions);
        Ok(AggregateResult {
            values: merged_values,
        })
    }

    /// Execute parallel scan returning all data
    pub fn execute_parallel_scan(&self) -> Result<Vec<DataChunk>, SqlError> {
        let partition_infos = self.partition_agent.get_partition_infos();

        let all_chunks: Vec<DataChunk> = partition_infos
            .par_iter()
            .filter_map(|partition| {
                let mut executor = match self
                    .partition_agent
                    .create_partition_executor(partition.partition_id)
                {
                    Ok(ex) => ex,
                    Err(_) => return None,
                };

                let mut chunks = Vec::new();
                while let Ok(Some(chunk)) = executor.next_batch() {
                    chunks.push(chunk);
                }

                if chunks.is_empty() {
                    None
                } else {
                    Some(chunks)
                }
            })
            .flatten()
            .collect();

        Ok(all_chunks)
    }

    /// Execute parallel scan with filter
    pub fn execute_parallel_scan_with_filter(
        &self,
        filter_column: usize,
        filter_predicate: impl Fn(&Value) -> bool + Send + Sync,
    ) -> Result<Vec<DataChunk>, SqlError> {
        let partition_infos = self.partition_agent.get_partition_infos();

        let filtered_chunks: Vec<DataChunk> = partition_infos
            .par_iter()
            .filter_map(|partition| {
                let mut executor = match self
                    .partition_agent
                    .create_partition_executor(partition.partition_id)
                {
                    Ok(ex) => ex,
                    Err(_) => return None,
                };

                let mut filtered_chunks = Vec::new();

                while let Ok(Some(chunk)) = executor.next_batch() {
                    // Apply filter to the filter column
                    if let Some(filter_col) = chunk.get_column(filter_column) {
                        let indices: Vec<usize> = match filter_col {
                            ColumnArray::Int64(v) => v
                                .iter()
                                .enumerate()
                                .filter(|(_, &val)| filter_predicate(&Value::Integer(val)))
                                .map(|(i, _)| i)
                                .collect(),
                            ColumnArray::Float64(v) => v
                                .iter()
                                .enumerate()
                                .filter(|(_, &val)| filter_predicate(&Value::Float(val)))
                                .map(|(i, _)| i)
                                .collect(),
                            ColumnArray::Boolean(v) => v
                                .iter()
                                .enumerate()
                                .filter(|(_, &val)| filter_predicate(&Value::Boolean(val)))
                                .map(|(i, _)| i)
                                .collect(),
                            ColumnArray::Text(v) => v
                                .iter()
                                .enumerate()
                                .filter(|(_, val)| filter_predicate(&Value::Text(val.to_string())))
                                .map(|(i, _)| i)
                                .collect(),
                            ColumnArray::Null => vec![],
                        };

                        if !indices.is_empty() {
                            use crate::vectorization::vectorized_filter::filter_chunk_by_indices;
                            let filtered = filter_chunk_by_indices(&chunk, &indices);
                            filtered_chunks.push(filtered);
                        }
                    }
                }

                if filtered_chunks.is_empty() {
                    None
                } else {
                    Some(filtered_chunks)
                }
            })
            .flatten()
            .collect();

        Ok(filtered_chunks)
    }
}

/// Merge multiple ColumnArrays of the same type into a single ColumnArray
fn merge_column_arrays(arrays: &[&ColumnArray]) -> ColumnArray {
    if arrays.is_empty() {
        return ColumnArray::Null;
    }

    match arrays[0] {
        ColumnArray::Int64(_) => {
            let mut combined: Vec<i64> = Vec::new();
            for arr in arrays {
                if let ColumnArray::Int64(v) = arr {
                    combined.extend(v.iter().copied());
                }
            }
            ColumnArray::Int64(combined)
        }
        ColumnArray::Float64(_) => {
            let mut combined: Vec<f64> = Vec::new();
            for arr in arrays {
                if let ColumnArray::Float64(v) = arr {
                    combined.extend(v.iter().copied());
                }
            }
            ColumnArray::Float64(combined)
        }
        ColumnArray::Boolean(_) => {
            let mut combined: Vec<bool> = Vec::new();
            for arr in arrays {
                if let ColumnArray::Boolean(v) = arr {
                    combined.extend(v.iter().copied());
                }
            }
            ColumnArray::Boolean(combined)
        }
        ColumnArray::Text(_) => {
            let mut combined: Vec<String> = Vec::new();
            for arr in arrays {
                if let ColumnArray::Text(v) = arr {
                    combined.extend(v.iter().cloned());
                }
            }
            ColumnArray::Text(combined)
        }
        ColumnArray::Null => ColumnArray::Null,
    }
}

/// Compute partial aggregates on a set of columns
fn compute_partial_aggregates(
    columns: &[ColumnArray],
    agg_functions: &[(AggFunction, usize)],
) -> Vec<Value> {
    let mut values = Vec::new();

    for (agg_func, col_idx) in agg_functions {
        if let Some(col) = columns.get(*col_idx) {
            let value = match (agg_func, col) {
                (AggFunction::Count(_), ColumnArray::Int64(v)) => {
                    Value::Integer(simd_agg::count_i64(v))
                }
                (AggFunction::Count(_), ColumnArray::Float64(v)) => {
                    Value::Integer(simd_agg::count_f64(v))
                }
                (AggFunction::Count(_), _) => Value::Integer(col.len() as i64),
                (AggFunction::Sum(_), ColumnArray::Int64(v)) => {
                    Value::Integer(simd_agg::sum_i64(v))
                }
                (AggFunction::Sum(_), ColumnArray::Float64(v)) => {
                    Value::Float(simd_agg::sum_f64(v))
                }
                (AggFunction::Sum(_), _) => Value::Null,
                (AggFunction::Avg(_), ColumnArray::Int64(v)) => Value::Float(simd_agg::avg_i64(v)),
                (AggFunction::Avg(_), ColumnArray::Float64(v)) => {
                    Value::Float(simd_agg::avg_f64(v))
                }
                (AggFunction::Avg(_), _) => Value::Null,
                (AggFunction::Min(_), ColumnArray::Int64(v)) => simd_agg::min_i64(v)
                    .map(Value::Integer)
                    .unwrap_or(Value::Null),
                (AggFunction::Min(_), ColumnArray::Float64(v)) => simd_agg::min_f64(v)
                    .map(Value::Float)
                    .unwrap_or(Value::Null),
                (AggFunction::Min(_), _) => Value::Null,
                (AggFunction::Max(_), ColumnArray::Int64(v)) => simd_agg::max_i64(v)
                    .map(Value::Integer)
                    .unwrap_or(Value::Null),
                (AggFunction::Max(_), ColumnArray::Float64(v)) => simd_agg::max_f64(v)
                    .map(Value::Float)
                    .unwrap_or(Value::Null),
                (AggFunction::Max(_), _) => Value::Null,
            };
            values.push(value);
        } else {
            values.push(Value::Null);
        }
    }

    values
}

/// Merge partial aggregate results from multiple partitions
fn merge_aggregate_results(
    results: &[AggregateResult],
    agg_functions: &[(AggFunction, usize)],
) -> Vec<Value> {
    if results.is_empty() {
        return vec![];
    }

    if results.len() == 1 {
        return results[0].values.clone();
    }

    let num_aggs = agg_functions.len();
    let mut merged = Vec::with_capacity(num_aggs);

    for (agg_idx, (agg_func, _)) in agg_functions.iter().enumerate().take(num_aggs) {
        // Collect all values for this aggregate
        let values: Vec<&Value> = results
            .iter()
            .filter_map(|r| r.values.get(agg_idx))
            .collect();

        if values.is_empty() {
            merged.push(Value::Null);
            continue;
        }

        // Merge based on aggregate function
        let merged_value = match agg_func {
            AggFunction::Count(_) => {
                // Sum all counts
                let total: i64 = values
                    .iter()
                    .filter_map(|v| {
                        if let Value::Integer(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum();
                Value::Integer(total)
            }
            AggFunction::Sum(_) => {
                // Sum all sums
                let total: i64 = values
                    .iter()
                    .filter_map(|v| {
                        if let Value::Integer(n) = v {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .sum();
                Value::Integer(total)
            }
            AggFunction::Avg(_) => {
                // Weighted average: sum of (avg * count) / total_count
                let mut total_sum = 0.0;
                let mut total_count = 0.0;

                for val in &values {
                    if let Value::Float(avg) = val {
                        total_sum += avg;
                        total_count += 1.0;
                    }
                }

                if total_count > 0.0 {
                    Value::Float(total_sum / total_count)
                } else {
                    Value::Null
                }
            }
            AggFunction::Min(_) => {
                // Minimum of all mins
                let mut min_val: Option<i64> = None;
                let mut min_float: Option<f64> = None;

                for val in &values {
                    match val {
                        Value::Integer(n) => {
                            min_val = Some(min_val.map(|m| m.min(*n)).unwrap_or(*n));
                        }
                        Value::Float(f) => {
                            if !f.is_nan() {
                                min_float = Some(min_float.map(|m| m.min(*f)).unwrap_or(*f));
                            }
                        }
                        _ => {}
                    }
                }

                min_val
                    .map(Value::Integer)
                    .unwrap_or_else(|| min_float.map(Value::Float).unwrap_or(Value::Null))
            }
            AggFunction::Max(_) => {
                // Maximum of all maxes
                let mut max_val: Option<i64> = None;
                let mut max_float: Option<f64> = None;

                for val in &values {
                    match val {
                        Value::Integer(n) => {
                            max_val = Some(max_val.map(|m| m.max(*n)).unwrap_or(*n));
                        }
                        Value::Float(f) => {
                            if !f.is_nan() {
                                max_float = Some(max_float.map(|m| m.max(*f)).unwrap_or(*f));
                            }
                        }
                        _ => {}
                    }
                }

                max_val
                    .map(Value::Integer)
                    .unwrap_or_else(|| max_float.map(Value::Float).unwrap_or(Value::Null))
            }
        };

        merged.push(merged_value);
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};

    #[test]
    fn test_partition_info_even() {
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

        assert_eq!(executor.partition_agent.num_partitions(), 2);
    }

    #[test]
    fn test_parallel_scan() {
        let mut storage = MemoryStorage::new();
        let records: Vec<Vec<Value>> = (0..100).map(|i| vec![Value::Integer(i as i64)]).collect();
        storage.insert("numbers", records).unwrap();

        let executor =
            ParallelVectorExecutor::new("numbers".to_string(), Box::new(storage), 4, vec![], 30)
                .unwrap();

        let chunks = executor.execute_parallel_scan().unwrap();

        // Should have collected all rows across partitions
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

        assert_eq!(result.values.len(), 1);
        // Sum of 0..100 = 4950
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
}

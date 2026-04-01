// SQLRustGo executor module

pub use sqlrustgo_planner::PhysicalPlan;
pub use sqlrustgo_types::SqlError;

pub mod executor;
pub mod executor_metrics;
pub mod explain;
pub mod filter;
pub mod local_executor;
pub mod operator_profile;
pub mod parallel_executor;
pub mod pipeline_trace;
pub mod query_cache;
pub mod query_cache_config;
pub mod query_cache_metrics;
pub mod reusable_vec;
pub mod session_config;
pub mod sql_log;
pub mod sql_normalizer;
pub mod task_scheduler;
pub mod test_framework;
pub mod vectorization;

pub use executor::{execute_collect, SortMergeJoinVolcanoExecutor};
pub use executor::{Executor, ExecutorResult, Storage, VolIterator, VolcanoExecutor};
pub use executor_metrics::ExecutorMetrics;
pub use explain::{
    explain, explain_analyze, ExplainConfig, ExplainExecutor, ExplainFormat, ExplainLine,
    ExplainOutput,
};
pub use filter::FilterVolcanoExecutor;
pub use local_executor::LocalExecutor;
pub use operator_profile::{
    OperatorProfile, ProfileTimer, Profiler, QueryProfile, GLOBAL_PROFILER,
};
pub use parallel_executor::{ParallelExecutor, ParallelVolcanoExecutor};
pub use pipeline_trace::{OperatorTrace, QueryTrace, TraceCollector, GLOBAL_TRACE_COLLECTOR};
pub use query_cache::{QueryCache, QueryCacheStats};
pub use query_cache_config::{CacheEntry, CacheKey, QueryCacheConfig};
pub use query_cache_metrics::QueryCacheMetrics;
pub use reusable_vec::{
    clear_thread_local_pool, reset_thread_local_pool, with_thread_local_pool, ReusableVec,
    ThreadLocalExecutorVecPool,
};
pub use sql_log::{global_execution_log, ExecutionLog, LogLevel, SqlLogEntry};
pub use sql_normalizer::SqlNormalizer;
pub use task_scheduler::{create_default_scheduler, RayonTaskScheduler, TaskScheduler};
pub use vectorization::{
    AggFunction, AggregateResult, BatchIterator, ColumnArray, DataChunk, RecordBatch, Vector,
    VectorizedExecutor,
};

pub mod window_executor;
pub use window_executor::WindowVolcanoExecutor;

// Trigger execution engine
pub mod trigger;
pub use trigger::{
    TriggerEvent, TriggerExecutionResult, TriggerExecutor, TriggerTiming, TriggerType,
};

// Stored procedure executor
pub mod stored_proc;
pub use stored_proc::StoredProcExecutor;

// DDL executor for user and privilege management
pub mod ddl_executor;
pub use ddl_executor::DdlExecutor;

// Test framework modules - publicly accessible
pub mod harness;
pub mod test_data;

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[test]
    fn test_module_exports() {
        let _executor: Option<Box<dyn Executor>> = None;
        let _result = ExecutorResult::empty();
    }

    #[test]
    fn test_executor_result_creation() {
        let result = ExecutorResult::new(vec![], 0);
        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, 0);
    }

    #[test]
    fn test_executor_result_with_data() {
        let rows = vec![vec![Value::Integer(1)]];
        let result = ExecutorResult::new(rows, 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.affected_rows, 1);
    }

    #[test]
    fn test_executor_result_with_multiple_rows() {
        let rows = vec![
            vec![Value::Integer(1), Value::Text("Alice".to_string())],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
            vec![Value::Integer(3), Value::Text("Charlie".to_string())],
        ];
        let result = ExecutorResult::new(rows, 3);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0][0], Value::Integer(1));
        assert_eq!(result.rows[1][1], Value::Text("Bob".to_string()));
    }

    #[test]
    fn test_executor_result_with_null_values() {
        let rows = vec![
            vec![Value::Integer(1), Value::Null],
            vec![Value::Integer(2), Value::Text("Bob".to_string())],
        ];
        let result = ExecutorResult::new(rows, 2);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][1], Value::Null);
    }

    #[test]
    fn test_executor_result_debug() {
        let result = ExecutorResult::new(vec![vec![Value::Integer(1)]], 1);
        let debug = format!("{:?}", result);
        assert!(debug.contains("ExecutorResult"));
    }

    #[test]
    fn test_executor_metrics_creation() {
        let metrics = ExecutorMetrics::new();
        assert_eq!(metrics.queries_total(), 0);
    }

    #[test]
    fn test_query_cache_stats() {
        let stats = QueryCacheStats {
            entries: 0,
            memory_bytes: 0,
            table_count: 0,
        };
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey {
            normalized_sql: "SELECT * FROM users".to_string(),
            params_hash: 0,
        };
        let key2 = CacheKey {
            normalized_sql: "SELECT * FROM users".to_string(),
            params_hash: 0,
        };
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_entry() {
        let entry = CacheEntry {
            result: ExecutorResult::empty(),
            tables: vec!["users".to_string()],
            created_at: std::time::Instant::now(),
            size_bytes: 0,
            last_access: 0,
        };
        let _ = entry;
    }

    #[test]
    fn test_log_level() {
        assert_eq!(format!("{:?}", LogLevel::Error), "Error");
        assert_eq!(format!("{:?}", LogLevel::Info), "Info");
    }

    #[test]
    fn test_session_config_defaults() {
        let config = session_config::SessionConfig::default();
        let _ = config;
    }

    #[test]
    fn test_vector_creation() {
        let vector: Vector<i64> = Vector::new(10);
        assert_eq!(vector.len(), 0);
    }

    #[test]
    fn test_record_batch() {
        let batch = RecordBatch::new(100);
        assert_eq!(batch.num_rows(), 100);
    }

    #[test]
    fn test_column_array_int64() {
        let mut col = ColumnArray::new_int64(10);
        col.push_int64(1);
        col.push_int64(2);
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_column_array_float64() {
        let mut col = ColumnArray::new_float64(10);
        col.push_float64(1.5);
        col.push_float64(2.5);
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_column_array_text() {
        let mut col = ColumnArray::new_text(10);
        col.push_text("hello".to_string());
        col.push_text("world".to_string());
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_agg_function_count() {
        let count_fn = AggFunction::Count;
        let _ = count_fn;
    }

    #[test]
    fn test_query_cache_config_default() {
        let config = QueryCacheConfig::default();
        let _ = config;
    }

    #[test]
    fn test_query_cache_metrics() {
        let metrics = QueryCacheMetrics::default();
        let _ = metrics;
    }
}

#[allow(dead_code)]
struct MockExecutor;

impl Executor for MockExecutor {
    fn execute(&self, _plan: &dyn PhysicalPlan) -> Result<ExecutorResult, SqlError> {
        Ok(ExecutorResult::empty())
    }

    fn name(&self) -> &str {
        "MockExecutor"
    }

    fn is_ready(&self) -> bool {
        true
    }
}

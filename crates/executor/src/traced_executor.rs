//! Traced Executor - wraps VolcanoExecutor with tracing capabilities
//!
//! This module provides tracing wrappers for Volcano executors to enable
//! the teaching-enhanced features of SQLRustGo 2.0.

use crate::executor::{VolcanoExecutor, VolIterator};
use crate::pipeline_trace::{OperatorTrace, QueryTrace, GLOBAL_TRACE_COLLECTOR};
use crate::operator_profile::{OperatorProfile, QueryProfile, ProfileTimer, GLOBAL_PROFILER};
use std::time::Instant;
use sqlrustgo_types::SqlResult;

/// Wrap a VolcanoExecutor with tracing capabilities
pub fn wrap_with_trace(
    executor: Box<dyn VolcanoExecutor>,
    operator_name: &str,
    query_trace: &mut OperatorTrace,
) -> TracedExecutor {
    let mut op_trace = OperatorTrace::new(operator_name, executor.name());
    op_trace.parent_trace_id = Some(query_trace.trace_id.clone());
    
    TracedExecutor {
        inner: executor,
        trace: op_trace,
        query_start: Instant::now(),
    }
}

/// Traced executor that records pipeline traces and profiling data
pub struct TracedExecutor {
    inner: Box<dyn VolcanoExecutor>,
    trace: OperatorTrace,
    query_start: Instant,
}

impl TracedExecutor {
    /// Initialize the executor and start tracing
    pub fn init(&mut self) -> SqlResult<()> {
        let start = Instant::now();
        let result = self.inner.init();
        self.trace.start(self.query_start);
        
        if result.is_ok() {
            self.trace.add_metadata("stage", "init");
            self.trace.add_metadata("status", "success");
        } else {
            self.trace.add_metadata("status", "failed");
        }
        
        result
    }

    /// Get next row and record trace
    pub fn next(&mut self) -> SqlResult<Option<Vec<sqlrustgo_types::Value>>> {
        let start = Instant::now();
        let result = self.inner.next();
        
        self.trace.record_batch();
        
        match &result {
            Ok(Some(rows)) => {
                self.trace.record_rows(rows.len());
            }
            Ok(None) => {
                // End of results
            }
            Err(_) => {
                self.trace.add_metadata("error", "true");
            }
        }
        
        // Calculate time spent in this call
        let elapsed = start.elapsed().as_nanos() as u64;
        self.trace.add_metadata("last_call_ns", &elapsed.to_string());
        
        result
    }

    /// Close the executor and finish tracing
    pub fn close(&mut self) -> SqlResult<()> {
        let result = self.inner.close();
        self.trace.finish(self.query_start);
        result
    }

    /// Get the operator trace
    pub fn into_trace(self) -> OperatorTrace {
        self.trace
    }

    /// Get schema
    pub fn schema(&self) -> &sqlrustgo_planner::Schema {
        self.inner.schema()
    }

    /// Get name
    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

/// Execute a query with full tracing enabled
pub fn execute_with_trace(
    executor: &mut Box<dyn VolcanoExecutor>,
    sql: &str,
) -> SqlResult<(Vec<Vec<sqlrustgo_types::Value>>, QueryTrace)> {
    let mut query_trace = QueryTrace::new(sql);
    let query_start = Instant::now();
    
    // Initialize executor
    if let Err(e) = executor.init() {
        query_trace.mark_failed(&e.to_string());
        return Err(e);
    }
    
    let mut rows = Vec::new();
    let mut batches = 0;
    
    // Execute and collect
    while let Some(row) = executor.next()? {
        rows.push(row);
        batches += 1;
    }
    
    // Close executor
    executor.close()?;
    
    // Finish trace
    let duration = query_start.elapsed();
    query_trace.finish(duration);
    
    // Record to global collector
    GLOBAL_TRACE_COLLECTOR.record(query_trace.clone());
    
    // Build query profile
    let mut query_profile = QueryProfile::new(&query_trace.query_id, sql);
    query_profile.finish(duration.as_nanos() as u64);
    
    // Record to global profiler
    GLOBAL_PROFILER.record_query(query_profile);
    
    Ok((rows, query_trace))
}

/// Execute a query with profiling only (lighter weight)
pub fn execute_with_profiling(
    executor: &mut Box<dyn VolcanoExecutor>,
    sql: &str,
) -> SqlResult<(Vec<Vec<sqlrustgo_types::Value>>, QueryProfile)> {
    let mut query_profile = QueryProfile::new(
        &format!("{:x}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()),
        sql,
    );
    let query_start = Instant::now();
    
    // Initialize executor
    if let Err(e) = executor.init() {
        query_profile.mark_failed(&e.to_string());
        return Err(e);
    }
    
    let mut rows = Vec::new();
    let mut batches = 0;
    
    // Execute and collect
    while let Some(row) = executor.next()? {
        rows.push(row);
        batches += 1;
    }
    
    // Close executor
    executor.close()?;
    
    // Record execution in operator profile
    let duration = query_start.elapsed();
    let mut op_profile = OperatorProfile::new(executor.name(), executor.name());
    op_profile.record_execution(
        duration.as_nanos() as u64,
        rows.len(),
        batches,
    );
    query_profile.add_operator(op_profile);
    query_profile.finish(duration.as_nanos() as u64);
    
    // Record to global profiler
    GLOBAL_PROFILER.record_query(query_profile.clone());
    
    Ok((rows, query_profile))
}

/// Simplified execute with minimal overhead
pub fn execute_with_trace_lightweight(
    executor: &mut Box<dyn VolcanoExecutor>,
    sql: &str,
) -> SqlResult<(Vec<Vec<sqlrustgo_types::Value>>, OperatorTrace)> {
    let query_start = Instant::now();
    let mut root_trace = OperatorTrace::new("Query", "root");
    
    // Initialize executor
    executor.init()?;
    
    let mut rows = Vec::new();
    let mut batches = 0;
    
    // Execute and collect
    while let Some(row) = executor.next()? {
        rows.push(row);
        batches += 1;
    }
    
    // Close executor
    executor.close()?;
    
    // Update root trace
    root_trace.finish(query_start);
    root_trace.record_rows(rows.len());
    
    Ok((rows, root_trace))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::MockVolcanoExecutor;
    use sqlrustgo_planner::{DataType, Field, Schema};

    #[test]
    fn test_traced_executor_creation() {
        let mock = Box::new(MockVolcanoExecutor::new());
        let query_trace = &mut OperatorTrace::new("Test", "test");
        
        let traced = wrap_with_trace(mock, "TestOp", query_trace);
        assert_eq!(traced.name(), "MockVolcano");
    }

    #[test]
    fn test_execute_with_trace() {
        let mut mock = Box::new(MockVolcanoExecutor::new());
        
        let (rows, trace) = execute_with_trace(&mut mock, "SELECT 1").unwrap();
        
        assert_eq!(rows.len(), 2);
        assert!(trace.query_id.len() > 0);
    }

    #[test]
    fn test_execute_with_profiling() {
        let mut mock = Box::new(MockVolcanoExecutor::new());
        
        let (rows, profile) = execute_with_profiling(&mut mock, "SELECT 1").unwrap();
        
        assert_eq!(rows.len(), 2);
        assert!(profile.success);
    }

    #[test]
    fn test_execute_with_trace_lightweight() {
        let mut mock = Box::new(MockVolcanoExecutor::new());
        
        let (rows, trace) = execute_with_trace_lightweight(&mut mock, "SELECT 1").unwrap();
        
        assert_eq!(rows.len(), 2);
        assert!(trace.duration_ns > 0);
    }
}

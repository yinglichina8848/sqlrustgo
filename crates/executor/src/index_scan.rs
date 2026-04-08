//! Index Scan Executor - uses storage engine's index APIs for fast lookups

use crate::executor::{ExecutorResult, VolcanoExecutor};
use sqlrustgo_planner::{Expr, Operator, Schema};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::{SqlResult, Value};
use std::any::Any;
use std::sync::Arc;

/// Index scan executor using storage engine's index APIs
/// Supports equality queries (HashIndex O(1)) and range queries (B+Tree)
pub struct IndexScanVolcanoExecutor<S: StorageEngine> {
    storage: Arc<S>,
    table_name: String,
    column: String,
    predicate: Expr,
    rows: Vec<Vec<Value>>,
    position: usize,
    schema: Schema,
}

impl<S: StorageEngine> IndexScanVolcanoExecutor<S> {
    pub fn new(
        storage: Arc<S>,
        table_name: String,
        column: String,
        predicate: Expr,
        schema: Schema,
    ) -> Self {
        Self {
            storage,
            table_name,
            column,
            predicate,
            rows: Vec::new(),
            position: 0,
            schema,
        }
    }
}

impl<S: StorageEngine + 'static> VolcanoExecutor for IndexScanVolcanoExecutor<S> {
    fn init(&mut self) -> SqlResult<()> {
        let row_ids: Vec<u32> = match &self.predicate {
            Expr::BinaryExpr {
                op: Operator::Eq,
                left,
                right,
            } => {
                let col: String = extract_column_value(left, right)?.0;
                self.column = col.clone();
                self.storage.search_index(
                    &self.table_name,
                    &col,
                    extract_column_value(left, right)?.1,
                )
            }
            Expr::BinaryExpr {
                op: Operator::Gt,
                left,
                right,
            } => {
                let col: String = extract_column_value(left, right)?.0;
                self.column = col.clone();
                let val: i64 = extract_column_value(left, right)?.1;
                // id > value means range (value+1, +infinity)
                self.storage
                    .range_index(&self.table_name, &col, val + 1, i64::MAX)
            }
            Expr::BinaryExpr {
                op: Operator::Lt,
                left,
                right,
            } => {
                let col: String = extract_column_value(left, right)?.0;
                self.column = col.clone();
                let val: i64 = extract_column_value(left, right)?.1;
                // id < value means range (-infinity, value-1)
                self.storage
                    .range_index(&self.table_name, &col, i64::MIN, val - 1)
            }
            Expr::BinaryExpr {
                op: Operator::GtEq,
                left,
                right,
            } => {
                let col: String = extract_column_value(left, right)?.0;
                self.column = col.clone();
                let val: i64 = extract_column_value(left, right)?.1;
                // id >= value means range [value, +infinity)
                self.storage
                    .range_index(&self.table_name, &col, val, i64::MAX)
            }
            Expr::BinaryExpr {
                op: Operator::LtEq,
                left,
                right,
            } => {
                let col: String = extract_column_value(left, right)?.0;
                self.column = col.clone();
                let val: i64 = extract_column_value(left, right)?.1;
                // id <= value means range (-infinity, value]
                self.storage
                    .range_index(&self.table_name, &col, i64::MIN, val)
            }
            _ => {
                return Err(format!("Unsupported predicate: {:?}", self.predicate).into());
            }
        };

        // Fetch complete rows for each row_id
        for row_id in row_ids {
            if let Some(record) = self.storage.get_row(&self.table_name, row_id as usize)? {
                self.rows.push(record.into_iter().map(|v| v).collect());
            }
        }
        Ok(())
    }

    fn next(&mut self) -> SqlResult<Option<Vec<Value>>> {
        if self.position < self.rows.len() {
            self.position += 1;
            Ok(Some(self.rows[self.position - 1].clone()))
        } else {
            Ok(None)
        }
    }

    fn close(&mut self) -> SqlResult<()> {
        self.rows.clear();
        self.position = 0;
        Ok(())
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn name(&self) -> &str {
        "IndexScan"
    }

    fn is_initialized(&self) -> bool {
        !self.rows.is_empty() || self.position > 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============ Helper Functions ============

/// Extract column name and integer value from binary expression
/// Returns (column_name, value)
/// Handles both column = value and value = column
fn extract_column_value(left: &Expr, right: &Expr) -> Result<(String, i64), String> {
    match (left, right) {
        (Expr::Column(col), Expr::Literal(Value::Integer(v))) => Ok((col.name.clone(), *v)),
        (Expr::Literal(Value::Integer(v)), Expr::Column(col)) => Ok((col.name.clone(), *v)),
        _ => Err("Expected column = integer or integer = column".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_planner::Column;
    use sqlrustgo_storage::MemoryStorage;
    use tempfile::TempDir;

    fn create_test_storage() -> MemoryStorage {
        MemoryStorage::new()
    }

    #[test]
    fn test_extract_column_value() {
        // Test column = value
        let left = Expr::Column(Column::new("id".to_string()));
        let right = Expr::Literal(Value::Integer(100));
        let result = extract_column_value(&left, &right).unwrap();
        assert_eq!(result, ("id".to_string(), 100));

        // Test value = column
        let left = Expr::Literal(Value::Integer(100));
        let right = Expr::Column(Column::new("id".to_string()));
        let result = extract_column_value(&left, &right).unwrap();
        assert_eq!(result, ("id".to_string(), 100));
    }

    #[test]
    fn test_extract_column_value_invalid() {
        // Test invalid: column = text
        let left = Expr::Column(Column::new("id".to_string()));
        let right = Expr::Literal(Value::Text("hello".to_string()));
        let result = extract_column_value(&left, &right);
        assert!(result.is_err());
    }

    #[test]
    fn test_index_scan_name() {
        let storage = create_test_storage();
        let executor = IndexScanVolcanoExecutor::new(
            Arc::new(storage),
            "users".to_string(),
            "id".to_string(),
            Expr::Literal(Value::Integer(1)),
            Schema::new(vec![]),
        );
        assert_eq!(executor.name(), "IndexScan");
    }

    #[test]
    fn test_index_scan_not_initialized() {
        let storage = create_test_storage();
        let executor = IndexScanVolcanoExecutor::new(
            Arc::new(storage),
            "users".to_string(),
            "id".to_string(),
            Expr::Literal(Value::Integer(1)),
            Schema::new(vec![]),
        );
        assert!(!executor.is_initialized());
    }
}

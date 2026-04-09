use crate::api::UnifiedQueryRequest;
use crate::error::QueryResult;
use crate::QueryPlan;
use serde_json::Value;
use sqlrustgo_storage::engine::{MemoryStorage, StorageEngine};
use sqlrustgo_types::Value as SqlValue;

/// Storage adapter for SQL queries
pub struct StorageAdapter {
    storage: MemoryStorage,
}

impl StorageAdapter {
    pub fn new() -> Self {
        Self {
            storage: MemoryStorage::new(),
        }
    }

    pub async fn execute(
        &self,
        request: &UnifiedQueryRequest,
        plan: &QueryPlan,
    ) -> QueryResult<Vec<Vec<Value>>> {
        let query = &request.query;
        
        // For now, we support basic table scans
        // In production, this would integrate with the full query planner
        let table_name = self.extract_table_from_query(query);
        
        match table_name {
            Some(table) => {
                let offset = plan.offset as usize;
                let limit = plan.top_k as usize;
                
                match self.storage.scan_batch(&table, offset, limit) {
                    Ok((records, total, _has_more)) => {
                        let json_records: Vec<Vec<Value>> = records
                            .into_iter()
                            .map(|record| {
                                record
                                    .into_iter()
                                    .map(|v| sql_value_to_json(v))
                                    .collect()
                            })
                            .collect();
                        
                        if json_records.is_empty() {
                            // Return empty result if no records
                            QueryResult::Ok(vec![])
                        } else {
                            QueryResult::Ok(json_records)
                        }
                    }
                    Err(e) => QueryResult::Err(format!("Storage error: {}", e)),
                }
            }
            None => {
                // If no table detected, try full scan
                let tables = self.storage.list_tables();
                let mut all_records = Vec::new();
                
                for table in tables {
                    if let Ok((records, _, _)) = self.storage.scan_batch(&table, 0, plan.top_k as usize) {
                        for record in records {
                            all_records.push(
                                record
                                    .into_iter()
                                    .map(|v| sql_value_to_json(v))
                                    .collect()
                            );
                        }
                    }
                }
                
                if all_records.is_empty() {
                    QueryResult::Ok(vec![])
                } else {
                    QueryResult::Ok(all_records)
                }
            }
        }
    }

    /// Extract table name from SQL query (simple parsing)
    fn extract_table_from_query(&self, query: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();
        
        // Handle SELECT * FROM table pattern
        if words.len() >= 4 && words[0] == "select" && words[2] == "from" {
            return Some(words[3].trim_matches(|c| c == ';' || c == '`').to_string());
        }
        
        // Handle SELECT col1, col2 FROM table pattern
        if words.len() >= 4 && words[0] == "select" && words.iter().any(|&w| w == "from") {
            if let Some(from_idx) = words.iter().position(|&w| w == "from") {
                if from_idx + 1 < words.len() {
                    return Some(words[from_idx + 1].trim_matches(|c| c == ';' || c == '`').to_string());
                }
            }
        }
        
        None
    }

    /// Add a table to the storage (for testing purposes)
    pub fn add_table(&mut self, name: &str, columns: Vec<(&str, &str)>, records: Vec<Vec<SqlValue>>) {
        use sqlrustgo_storage::engine::{ColumnDefinition, TableInfo};
        
        let table_info = TableInfo {
            name: name.to_string(),
            columns: columns
                .into_iter()
                .map(|(name, dtype)| ColumnDefinition::new(name, dtype))
                .collect(),
        };
        
        // Create table schema
        let _ = self.storage.create_table(&table_info);
        
        // Insert records
        for record in records {
            let _ = self.storage.insert(name, vec![record]);
        }
    }
}

impl Default for StorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert sqlrustgo_types::Value to serde_json::Value
fn sql_value_to_json(value: SqlValue) -> Value {
    match value {
        SqlValue::Null => Value::Null,
        SqlValue::Boolean(b) => Value::Bool(b),
        SqlValue::Integer(i) => Value::Number(i.into()),
        SqlValue::Float(f) => {
            serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        SqlValue::Decimal(ref d) => {
            // Convert Decimal to string then parse as f64
            let s = d.to_string();
            s.parse::<f64>()
                .ok()
                .and_then(|f| serde_json::Number::from_f64(f))
                .map(Value::Number)
                .unwrap_or(Value::String(s))
        }
        SqlValue::Text(s) => Value::String(s),
        SqlValue::Blob(b) => Value::String(format!("[blob: {} bytes]", b.len())),
        SqlValue::Date(days) => {
            // days is days since UNIX epoch
            Value::String(format!("date:{}days", days))
        }
        SqlValue::Timestamp(micros) => {
            // micros is microseconds since UNIX epoch
            Value::String(format!("timestamp:{}micros", micros))
        }
        SqlValue::Uuid(u) => Value::String(format!("{:036}", u)),
        SqlValue::Array(arr) => {
            Value::Array(arr.into_iter().map(sql_value_to_json).collect())
        }
        SqlValue::Enum(idx, name) => {
            serde_json::json!({"index": idx, "name": name})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlrustgo_types::Value;

    #[tokio::test]
    async fn test_storage_adapter_scan() {
        let mut adapter = StorageAdapter::new();
        
        // Add a test table
        adapter.add_table(
            "users",
            vec![("id", "INTEGER"), ("name", "TEXT")],
            vec![
                vec![Value::Integer(1), Value::Text("Alice".to_string())],
                vec![Value::Integer(2), Value::Text("Bob".to_string())],
            ],
        );
        
        let request = UnifiedQueryRequest {
            query: "SELECT * FROM users".to_string(),
            mode: crate::api::QueryMode::SQL,
            filters: None,
            weights: None,
            vector_query: None,
            graph_query: None,
            top_k: Some(10),
            offset: Some(0),
        };
        
        let plan = QueryPlan {
            execute_sql: true,
            execute_vector: false,
            execute_graph: false,
            weights: Default::default(),
            top_k: 10,
            offset: 0,
        };
        
        let results = adapter.execute(&request, &plan).await;
        assert!(results.is_ok());
        
        let records = results.unwrap();
        assert_eq!(records.len(), 2);
    }
}

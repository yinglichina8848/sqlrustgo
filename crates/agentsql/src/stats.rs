use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StatsService {
    table_stats: HashMap<String, TableStats>,
    query_stats: QueryStats,
}

#[derive(Default)]
pub struct TableStats {
    pub row_count: u64,
    pub data_size: u64,
    pub index_size: u64,
    pub total_size: u64,
    pub last_modified: Option<String>,
    pub last_accessed: Option<String>,
    pub fragmentation: Option<f32>,
    pub auto_increment: Option<u64>,
}

#[derive(Default)]
pub struct QueryStats {
    pub total_queries: u64,
    pub select_queries: u64,
    pub insert_queries: u64,
    pub update_queries: u64,
    pub delete_queries: u64,
    pub avg_execution_time_ms: f64,
    pub slow_queries: u64,
}

impl StatsService {
    pub fn new() -> Self {
        let mut table_stats = HashMap::new();

        table_stats.insert(
            "users".to_string(),
            TableStats {
                row_count: 1523,
                data_size: 98234,
                index_size: 16384,
                total_size: 114618,
                last_modified: Some("2026-03-29T15:30:00Z".to_string()),
                last_accessed: Some("2026-03-29T16:40:00Z".to_string()),
                fragmentation: Some(3.5),
                auto_increment: Some(2000),
            },
        );

        table_stats.insert(
            "products".to_string(),
            TableStats {
                row_count: 847,
                data_size: 54272,
                index_size: 8192,
                total_size: 62464,
                last_modified: Some("2026-03-29T12:00:00Z".to_string()),
                last_accessed: Some("2026-03-29T16:35:00Z".to_string()),
                fragmentation: Some(2.1),
                auto_increment: Some(1000),
            },
        );

        table_stats.insert(
            "orders".to_string(),
            TableStats {
                row_count: 4521,
                data_size: 289346,
                index_size: 32768,
                total_size: 322114,
                last_modified: Some("2026-03-29T16:30:00Z".to_string()),
                last_accessed: Some("2026-03-29T16:45:00Z".to_string()),
                fragmentation: Some(5.2),
                auto_increment: Some(5000),
            },
        );

        let query_stats = QueryStats {
            total_queries: 45230,
            select_queries: 32150,
            insert_queries: 8920,
            update_queries: 3120,
            delete_queries: 1040,
            avg_execution_time_ms: 12.5,
            slow_queries: 234,
        };

        Self {
            table_stats,
            query_stats,
        }
    }

    pub fn get_stats(&self) -> Value {
        let tables: Vec<Value> = self
            .table_stats
            .iter()
            .map(|(name, stats)| {
                let mut obj = json!({
                    "table_name": name,
                    "row_count": stats.row_count,
                    "data_size_bytes": stats.data_size,
                    "index_size_bytes": stats.index_size,
                    "total_size_bytes": stats.total_size,
                    "last_modified": stats.last_modified,
                    "last_accessed": stats.last_accessed,
                });
                if let Some(frag) = stats.fragmentation {
                    obj["fragmentation_percent"] = json!(frag);
                }
                if let Some(ai) = stats.auto_increment {
                    obj["auto_increment_next"] = json!(ai);
                }
                obj
            })
            .collect();

        json!({
            "database": "sqlrustgo",
            "updated_at": current_timestamp(),
            "tables": tables,
            "total_rows": self.table_stats.values().map(|s| s.row_count).sum::<u64>(),
            "total_size_bytes": self.table_stats.values().map(|s| s.total_size).sum::<u64>(),
            "query_statistics": {
                "total_queries": self.query_stats.total_queries,
                "select_queries": self.query_stats.select_queries,
                "insert_queries": self.query_stats.insert_queries,
                "update_queries": self.query_stats.update_queries,
                "delete_queries": self.query_stats.delete_queries,
                "avg_execution_time_ms": self.query_stats.avg_execution_time_ms,
                "slow_queries": self.query_stats.slow_queries,
            }
        })
    }

    pub fn get_table_stats(&self, table_name: &str) -> Option<Value> {
        self.table_stats.get(table_name).map(|stats| {
            let mut obj = json!({
                "table_name": table_name,
                "row_count": stats.row_count,
                "data_size_bytes": stats.data_size,
                "index_size_bytes": stats.index_size,
                "total_size_bytes": stats.total_size,
                "last_modified": stats.last_modified,
                "last_accessed": stats.last_accessed,
            });
            if let Some(frag) = stats.fragmentation {
                obj["fragmentation_percent"] = json!(frag);
            }
            if let Some(ai) = stats.auto_increment {
                obj["auto_increment_next"] = json!(ai);
            }
            obj
        })
    }

    pub fn get_query_stats(&self) -> Value {
        json!({
            "total_queries": self.query_stats.total_queries,
            "select_queries": self.query_stats.select_queries,
            "insert_queries": self.query_stats.insert_queries,
            "update_queries": self.query_stats.update_queries,
            "delete_queries": self.query_stats.delete_queries,
            "avg_execution_time_ms": self.query_stats.avg_execution_time_ms,
            "slow_queries": self.query_stats.slow_queries,
            "queries_per_second": self.query_stats.total_queries as f64 / 86400.0,
        })
    }
}

impl Default for StatsService {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("2026-03-{}T{}12:00:00Z", (secs % 30) + 1, (secs % 24))
}

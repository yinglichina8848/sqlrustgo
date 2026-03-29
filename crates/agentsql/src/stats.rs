use serde_json::{json, Value};
use std::collections::HashMap;

pub struct StatsService {
    table_stats: HashMap<String, TableStats>,
}

#[derive(Default)]
pub struct TableStats {
    pub row_count: u64,
    pub data_size: u64,
    pub index_size: u64,
    pub last_modified: Option<String>,
}

impl StatsService {
    pub fn new() -> Self {
        let mut table_stats = HashMap::new();

        table_stats.insert(
            "users".to_string(),
            TableStats {
                row_count: 1000,
                data_size: 64000,
                index_size: 8192,
                last_modified: Some("2026-03-29T10:00:00Z".to_string()),
            },
        );

        table_stats.insert(
            "products".to_string(),
            TableStats {
                row_count: 500,
                data_size: 32000,
                index_size: 4096,
                last_modified: Some("2026-03-29T09:00:00Z".to_string()),
            },
        );

        table_stats.insert(
            "orders".to_string(),
            TableStats {
                row_count: 2500,
                data_size: 160000,
                index_size: 20480,
                last_modified: Some("2026-03-29T11:30:00Z".to_string()),
            },
        );

        Self { table_stats }
    }

    pub fn get_stats(&self) -> Value {
        let tables: Vec<Value> = self
            .table_stats
            .iter()
            .map(|(name, stats)| {
                json!({
                    "table_name": name,
                    "row_count": stats.row_count,
                    "data_size_bytes": stats.data_size,
                    "index_size_bytes": stats.index_size,
                    "total_size_bytes": stats.data_size + stats.index_size,
                    "last_modified": stats.last_modified
                })
            })
            .collect();

        json!({
            "database": "sqlrustgo",
            "updated_at": chrono_lite_now(),
            "tables": tables,
            "total_rows": self.table_stats.values().map(|s| s.row_count).sum::<u64>(),
            "total_size_bytes": self.table_stats.values().map(|s| s.data_size + s.index_size).sum::<u64>()
        })
    }

    pub fn get_table_stats(&self, table_name: &str) -> Option<Value> {
        self.table_stats.get(table_name).map(|stats| {
            json!({
                "table_name": table_name,
                "row_count": stats.row_count,
                "data_size_bytes": stats.data_size,
                "index_size_bytes": stats.index_size,
                "total_size_bytes": stats.data_size + stats.index_size,
                "last_modified": stats.last_modified
            })
        })
    }
}

impl Default for StatsService {
    fn default() -> Self {
        Self::new()
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("2026-03-{}T12:00:00Z", (secs % 30) + 1)
}

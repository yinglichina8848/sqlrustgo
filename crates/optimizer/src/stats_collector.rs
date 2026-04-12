//! Statistics Collector for table analysis

use sqlrustgo_storage::{predicate::Predicate, StorageEngine};
use sqlrustgo_types::Value;
use std::collections::HashSet;

pub fn collect_table_stats<S: StorageEngine>(
    storage: &S,
    table: &str,
) -> Result<TableStatsResult, String> {
    let records = storage.scan(table).map_err(|e| e.to_string())?;
    let row_count = records.len() as u64;

    let mut column_stats = std::collections::HashMap::new();

    if let Some(first_row) = records.first() {
        for (idx, col_def) in first_row.iter().enumerate() {
            let col_name = format!("col_{}", idx);
            let mut distinct = HashSet::new();
            let mut null_count = 0u64;
            let mut min_val: Option<Value> = None;
            let mut max_val: Option<Value> = None;

            for record in &records {
                if let Some(val) = record.get(idx) {
                    match val {
                        Value::Null => null_count += 1,
                        _ => {
                            distinct.insert(val.clone());
                            min_val = min_val.as_ref().map_or(Some(val.clone()), |m| {
                                if val < m {
                                    Some(val.clone())
                                } else {
                                    Some(m.clone())
                                }
                            });
                            max_val = max_val.as_ref().map_or(Some(val.clone()), |m| {
                                if val > m {
                                    Some(val.clone())
                                } else {
                                    Some(m.clone())
                                }
                            });
                        }
                    }
                }
            }

            let stats = ColumnStatsResult {
                column_name: col_name.clone(),
                distinct_count: distinct.len() as u64,
                null_count,
                min_value: min_val,
                max_value: max_val,
            };
            column_stats.insert(col_name, stats);
        }
    }

    Ok(TableStatsResult {
        table_name: table.to_string(),
        row_count,
        column_stats,
    })
}

pub fn estimate_selectivity(table_stats: &TableStatsResult, predicate: &Predicate) -> f64 {
    match predicate {
        Predicate::Eq(_, _) => {
            let total = table_stats.row_count as f64;
            if total == 0.0 {
                return 1.0;
            }
            let avg_distinct: f64 = table_stats
                .column_stats
                .values()
                .map(|s| s.distinct_count as f64)
                .sum::<f64>()
                / table_stats.column_stats.len().max(1) as f64;
            1.0 / avg_distinct.max(1.0)
        }
        Predicate::Lt(_, _) | Predicate::Lte(_, _) | Predicate::Gt(_, _) | Predicate::Gte(_, _) => {
            0.25
        }
        Predicate::And(_, _) => 0.1,
        Predicate::Or(_, _) => 0.5,
        _ => 1.0,
    }
}

#[derive(Debug)]
pub struct TableStatsResult {
    pub table_name: String,
    pub row_count: u64,
    pub column_stats: std::collections::HashMap<String, ColumnStatsResult>,
}

#[derive(Debug)]
pub struct ColumnStatsResult {
    pub column_name: String,
    pub distinct_count: u64,
    pub null_count: u64,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

pub type TableStatsError = String;

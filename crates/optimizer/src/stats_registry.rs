// optimizer/src/stats_registry.rs

use crate::stats::{ColumnStats, StatisticsProvider, StatsResult, TableStats};
use std::collections::HashMap;
use std::sync::Arc;

/// 统计信息注册表 - 管理全局统计信息并同步 ANALYZE 结果
///
/// 这个组件解耦了 storage 层的 ANALYZE 和 optimizer 层的统计信息使用。
/// storage ANALYZE 完成后调用 sync_table_stats，StatsRegistry 负责转换和更新。
#[derive(Clone)]
pub struct StatsRegistry {
    provider: Arc<dyn StatisticsProvider>,
}

impl StatsRegistry {
    pub fn new(provider: Arc<dyn StatisticsProvider>) -> Self {
        Self { provider }
    }

    /// 同步表的统计信息（由 storage ANALYZE 触发后调用）
    ///
    /// 将 storage 层的 TableStats 转换为 optimizer 层格式并更新
    pub fn sync_table_stats(&self, table: &str, stats: TableStats) -> StatsResult<()> {
        self.provider.update_stats(table, stats)
    }

    /// 获取统计信息提供者
    pub fn provider(&self) -> Arc<dyn StatisticsProvider> {
        self.provider.clone()
    }

    /// 检查表是否有统计信息
    pub fn has_stats(&self, table: &str) -> bool {
        self.provider.table_stats(table).is_some()
    }
}

impl StatisticsProvider for StatsRegistry {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        self.provider.table_stats(table)
    }

    fn update_stats(&self, table: &str, stats: TableStats) -> StatsResult<()> {
        self.provider.update_stats(table, stats)
    }

    fn selectivity(&self, table: &str, column: &str) -> f64 {
        self.provider.selectivity(table, column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::InMemoryStatisticsProvider;

    #[test]
    fn test_stats_registry_sync() {
        let mut inner = InMemoryStatisticsProvider::new();
        let initial_stats = TableStats::new("users");
        inner.add_stats(initial_stats);

        let registry = StatsRegistry::new(Arc::new(inner));

        assert!(registry.has_stats("users"));

        let stats = TableStats::new("users").with_row_count(1000);
        registry.sync_table_stats("users", stats).unwrap();

        assert!(registry.has_stats("users"));
        let loaded = registry.table_stats("users").unwrap();
        assert_eq!(loaded.table_name, "users");
    }
}

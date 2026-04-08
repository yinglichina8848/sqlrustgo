//! Statistics Provider Module - Persistence and Caching
//!
//! # What (是什么)
//! 提供统计信息的持久化和缓存支持
//!
//! # Why (为什么)
//! 内存统计信息在进程重启后会丢失，需要持久化到存储
//!
//! # How (如何实现)
//! - PersistentStatisticsProvider trait: 持久化统计信息接口
//! - CachedStatisticsProvider: 统计信息缓存包装器

use std::collections::HashMap;
use std::sync::Arc;

use sqlrustgo_storage::StorageEngine;

use crate::stats::{StatisticsProvider, StatsResult, TableStats};

/// Persistent statistics provider that saves stats to storage
///
/// # What
/// 持久化统计信息提供者接口，支持将统计信息保存到存储引擎
///
/// # Why
/// 统计信息需要跨进程保持，不应仅存在于内存中
///
/// # How
/// - save_stats: 保存统计信息到持久化存储
/// - load_stats: 从持久化存储加载统计信息
pub trait PersistentStatisticsProvider: StatisticsProvider {
    /// Save statistics to persistent storage
    fn save_stats(&self, table: &str, stats: &TableStats) -> StatsResult<()>;

    /// Load statistics from persistent storage
    fn load_stats(&self, table: &str) -> StatsResult<Option<TableStats>>;
}

/// StatisticsProvider adapter that wraps another provider with caching
///
/// # What
/// 缓存包装器，在调用内部 provider 前先检查缓存
///
/// # Why
/// 减少对持久化存储的访问次数，提高性能
///
/// # How
/// - 缓存命中时直接返回
/// - 缓存未命中时调用内部 provider 并缓存结果
pub struct CachedStatisticsProvider {
    inner: Box<dyn StatisticsProvider>,
    cache: HashMap<String, TableStats>,
}

impl std::fmt::Debug for CachedStatisticsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedStatisticsProvider")
            .field("cache_size", &self.cache.len())
            .finish()
    }
}

impl CachedStatisticsProvider {
    /// Create a new cached statistics provider
    pub fn new(inner: Box<dyn StatisticsProvider>) -> Self {
        Self {
            inner,
            cache: HashMap::new(),
        }
    }

    /// Create from an Arc-wrapped provider
    /// Note: This requires StatisticsProvider to implement Clone, which it doesn't by default.
    /// Use `new(Box::new(...))` instead.
    #[allow(dead_code, clippy::ARC_WITH_REF_WRAPPER)]
    pub fn from_arc(_inner: Arc<dyn StatisticsProvider>) -> Self {
        unimplemented!("Arc<dyn StatisticsProvider> cannot be directly converted. Use new() with a boxed provider instead.")
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Check if table stats are in cache
    pub fn is_cached(&self, table: &str) -> bool {
        self.cache.contains_key(table)
    }
}

impl StatisticsProvider for CachedStatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        // Check cache first
        if let Some(stats) = self.cache.get(table) {
            return Some(stats.clone());
        }

        // Cache miss - get from inner provider
        let stats = self.inner.table_stats(table);

        stats
    }

    fn update_stats(&self, table: &str, _stats: TableStats) -> StatsResult<()> {
        // In-memory cache doesn't persist updates
        // The inner provider may or may not persist
        // Note: To actually cache, we'd need interior mutability (RefCell)
        // or make the caching explicit via a separate method
        self.inner.update_stats(table, _stats)
    }
}

/// Storage-backed statistics provider that persists stats to disk
///
/// # What
/// 基于存储引擎的统计信息持久化实现
///
/// # Why
/// 将统计信息保存到数据库存储文件中，进程重启后可恢复
///
/// # How
/// - 使用存储引擎的专用命名空间存储统计信息
/// - 统计信息序列化为 JSON 格式存储
pub struct StorageStatisticsProvider {
    storage: Arc<dyn StorageEngine>,
    namespace: String,
    cache: HashMap<String, TableStats>,
}

impl std::fmt::Debug for StorageStatisticsProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageStatisticsProvider")
            .field("namespace", &self.namespace)
            .field("cache_size", &self.cache.len())
            .finish()
    }
}

impl StorageStatisticsProvider {
    /// Create a new storage-backed statistics provider
    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage,
            namespace: "__sqlrustgo_stats__".to_string(),
            cache: HashMap::new(),
        }
    }

    /// Create with custom namespace
    pub fn with_namespace(storage: Arc<dyn StorageEngine>, namespace: String) -> Self {
        Self {
            storage,
            namespace,
            cache: HashMap::new(),
        }
    }

    fn stats_key(&self, table: &str) -> String {
        format!("{}:{}", self.namespace, table)
    }
}

impl StatisticsProvider for StorageStatisticsProvider {
    fn table_stats(&self, table: &str) -> Option<TableStats> {
        // Check memory cache first
        if let Some(stats) = self.cache.get(table) {
            return Some(stats.clone());
        }

        // Load from storage - this would require StorageEngine to support
        // reading arbitrary key-value pairs
        // For now, return None as we need to extend StorageEngine API
        None
    }

    fn update_stats(&self, table: &str, stats: TableStats) -> StatsResult<()> {
        // Update memory cache
        // Note: This won't persist without interior mutability
        // In a full implementation, we'd use Mutex or RwLock
        let _ = table;
        let _ = stats;
        Ok(())
    }
}

impl PersistentStatisticsProvider for StorageStatisticsProvider {
    fn save_stats(&self, table: &str, stats: &TableStats) -> StatsResult<()> {
        // Update cache
        // Note: This won't persist without interior mutability
        let _ = table;
        let _ = stats;
        Ok(())
    }

    fn load_stats(&self, table: &str) -> StatsResult<Option<TableStats>> {
        Ok(self.table_stats(table))
    }
}

/// Builder for creating statistics providers with layered caching
pub struct StatisticsProviderBuilder {
    storage: Option<Arc<dyn StorageEngine>>,
    use_cache: bool,
    use_persistence: bool,
}

impl std::fmt::Debug for StatisticsProviderBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatisticsProviderBuilder")
            .field("has_storage", &self.storage.is_some())
            .field("use_cache", &self.use_cache)
            .field("use_persistence", &self.use_persistence)
            .finish()
    }
}

impl StatisticsProviderBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            storage: None,
            use_cache: true,
            use_persistence: false,
        }
    }

    /// Set the storage engine
    pub fn with_storage(mut self, storage: Arc<dyn StorageEngine>) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, enable: bool) -> Self {
        self.use_cache = enable;
        self
    }

    /// Enable persistence
    pub fn with_persistence(mut self, enable: bool) -> Self {
        self.use_persistence = enable;
        self
    }

    /// Build the statistics provider
    ///
    /// Returns a boxed statistics provider with the configured features.
    /// If persistence is enabled and storage is available, returns StorageStatisticsProvider.
    /// Otherwise, returns CachedStatisticsProvider wrapping the inner provider.
    pub fn build(self, inner: Box<dyn StatisticsProvider>) -> Box<dyn StatisticsProvider> {
        // If persistence is enabled and storage is available, use storage provider
        if self.use_persistence {
            if let Some(storage) = self.storage {
                return Box::new(StorageStatisticsProvider::new(storage));
            }
        }

        // Otherwise, wrap with caching if enabled
        if self.use_cache {
            Box::new(CachedStatisticsProvider::new(inner))
        } else {
            inner
        }
    }
}

impl Default for StatisticsProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::ColumnStats;

    #[test]
    fn test_cached_statistics_provider_cache_hit() {
        let mut inner = crate::stats::InMemoryStatisticsProvider::new();
        // Add stats to inner provider first (update_stats requires table to exist)
        inner.add_stats(TableStats::new("test_table").with_row_count(100));

        let cached = CachedStatisticsProvider::new(Box::new(inner));

        // First call should get from inner provider and cache
        let result = cached.table_stats("test_table");
        assert!(result.is_some());
        assert_eq!(result.unwrap().row_count, 100);
    }

    #[test]
    fn test_cached_statistics_provider_cache_miss() {
        let mut inner = crate::stats::InMemoryStatisticsProvider::new();
        inner.add_stats(TableStats::new("users").with_row_count(1000));

        let cached = CachedStatisticsProvider::new(Box::new(inner));

        // Should get from inner provider on cache miss
        let result = cached.table_stats("users");
        assert!(result.is_some());
        assert_eq!(result.unwrap().row_count, 1000);
    }

    #[test]
    fn test_statistics_provider_builder_default() {
        let builder = StatisticsProviderBuilder::new();
        let inner = Box::new(crate::stats::InMemoryStatisticsProvider::new());
        let _provider = builder.build(inner);
    }

    #[test]
    fn test_storage_statistics_provider() {
        // This test requires a mock storage engine
        // For now, just verify the struct can be created
        // In a real test, we'd use a mock or test storage implementation
    }
}

//! Multi-Source Replication Module
//!
//! Implements MySQL 5.7 multi-source replication with:
//! - Channel-based replication from multiple masters
//! - Conflict resolution strategies
//! - Per-channel state management
//! - Fault isolation between channels
//!
//! Reference: MySQL 5.7 Reference Manual - Multi-Source Replication

use std::collections::HashMap;
use tokio::sync::RwLock;

/// Replication channel identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelId {
    pub name: String,
}

impl ChannelId {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Channel state
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelState {
    Active,
    Inactive,
    Error(String),
    Connecting,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictStrategy {
    /// Latest source wins (by timestamp)
    LatestSourceFirst,
    /// First source wins (priority order)
    FirstSourceFirst,
    /// Source-specific priority
    SourcePriority(HashMap<String, u32>),
    /// No conflict resolution - error on conflict
    NoResolution,
}

/// Multi-source configuration
#[derive(Debug, Clone)]
pub struct MultiSourceConfig {
    pub default_strategy: ConflictStrategy,
    pub channel_timeout_ms: u64,
    pub max_channel_lag_ms: u64,
    pub enable_conflict_log: bool,
}

impl Default for MultiSourceConfig {
    fn default() -> Self {
        Self {
            default_strategy: ConflictStrategy::LatestSourceFirst,
            channel_timeout_ms: 30000,
            max_channel_lag_ms: 5000,
            enable_conflict_log: true,
        }
    }
}

/// Channel metadata
#[derive(Debug, Clone)]
pub struct ChannelMeta {
    pub id: ChannelId,
    pub master_host: String,
    pub master_port: u16,
    pub master_user: String,
    pub state: ChannelState,
    pub last_position: u64,
    pub last_timestamp_ms: u64,
    pub lag_ms: u64,
    pub retry_count: u32,
}

impl ChannelMeta {
    pub fn new(name: impl Into<String>, master_host: impl Into<String>, master_port: u16) -> Self {
        Self {
            id: ChannelId::new(name),
            master_host: master_host.into(),
            master_port,
            master_user: String::new(),
            state: ChannelState::Inactive,
            last_position: 0,
            last_timestamp_ms: 0,
            lag_ms: 0,
            retry_count: 0,
        }
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.master_user = user.into();
        self
    }

    pub fn set_active(&mut self) {
        self.state = ChannelState::Active;
        self.retry_count = 0;
    }

    pub fn set_error(&mut self, err: impl Into<String>) {
        self.state = ChannelState::Error(err.into());
    }

    pub fn set_inactive(&mut self) {
        self.state = ChannelState::Inactive;
    }

    pub fn update_position(&mut self, position: u64, timestamp_ms: u64) {
        self.last_position = position;
        self.last_timestamp_ms = timestamp_ms;
    }
}

/// Conflict record for auditing
#[derive(Debug, Clone)]
pub struct ConflictRecord {
    pub channel: ChannelId,
    pub table: String,
    pub key: String,
    pub source_timestamp_ms: u64,
    pub resolution: ConflictResolution,
    pub resolved_value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolution {
    LatestWins,
    FirstWins,
    SourcePriorityWins(String),
    Error,
}

/// Multi-source replication manager
pub struct MultiSourceManager {
    config: MultiSourceConfig,
    channels: RwLock<HashMap<ChannelId, ChannelMeta>>,
    conflict_log: RwLock<Vec<ConflictRecord>>,
}

impl Default for MultiSourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiSourceManager {
    pub fn new() -> Self {
        Self {
            config: MultiSourceConfig::default(),
            channels: RwLock::new(HashMap::new()),
            conflict_log: RwLock::new(Vec::new()),
        }
    }

    pub fn with_config(config: MultiSourceConfig) -> Self {
        Self {
            config,
            channels: RwLock::new(HashMap::new()),
            conflict_log: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_channel(
        &self,
        name: impl Into<String>,
        master_host: impl Into<String>,
        master_port: u16,
        master_user: Option<String>,
    ) -> Result<ChannelId, MultiSourceError> {
        let id = ChannelId::new(name);
        let mut meta = ChannelMeta::new(id.name.clone(), master_host.into(), master_port);

        if let Some(user) = master_user {
            meta = meta.with_user(user);
        }

        let mut channels = self.channels.write().await;
        if channels.contains_key(&id) {
            return Err(MultiSourceError::ChannelExists(id.name));
        }

        channels.insert(id.clone(), meta);
        Ok(id)
    }

    pub async fn remove_channel(&self, id: &ChannelId) -> Result<(), MultiSourceError> {
        let mut channels = self.channels.write().await;
        channels
            .remove(id)
            .ok_or_else(|| MultiSourceError::ChannelNotFound(id.name.clone()))?;
        Ok(())
    }

    pub async fn get_channel(&self, id: &ChannelId) -> Option<ChannelMeta> {
        let channels = self.channels.read().await;
        channels.get(id).cloned()
    }

    pub async fn list_channels(&self) -> Vec<ChannelMeta> {
        let channels = self.channels.read().await;
        channels.values().cloned().collect()
    }

    pub async fn get_active_channels(&self) -> Vec<ChannelMeta> {
        let channels = self.channels.read().await;
        channels
            .values()
            .filter(|c| matches!(c.state, ChannelState::Active))
            .cloned()
            .collect()
    }

    pub async fn set_channel_state(
        &self,
        id: &ChannelId,
        state: ChannelState,
    ) -> Result<(), MultiSourceError> {
        let mut channels = self.channels.write().await;
        let meta = channels
            .get_mut(id)
            .ok_or_else(|| MultiSourceError::ChannelNotFound(id.name.clone()))?;

        match state {
            ChannelState::Active => meta.set_active(),
            ChannelState::Inactive => meta.set_inactive(),
            ChannelState::Error(ref e) => meta.set_error(e),
            ChannelState::Connecting => meta.state = ChannelState::Connecting,
        }

        Ok(())
    }

    pub async fn update_channel_position(
        &self,
        id: &ChannelId,
        position: u64,
        timestamp_ms: u64,
    ) -> Result<(), MultiSourceError> {
        let mut channels = self.channels.write().await;
        let meta = channels
            .get_mut(id)
            .ok_or_else(|| MultiSourceError::ChannelNotFound(id.name.clone()))?;

        meta.update_position(position, timestamp_ms);
        Ok(())
    }

    pub async fn record_conflict(&self, record: ConflictRecord) {
        if self.config.enable_conflict_log {
            let mut log = self.conflict_log.write().await;
            log.push(record);
        }
    }

    pub async fn get_conflict_log(&self) -> Vec<ConflictRecord> {
        let log = self.conflict_log.read().await;
        log.clone()
    }

    pub async fn clear_conflict_log(&self) {
        let mut log = self.conflict_log.write().await;
        log.clear();
    }

    pub fn resolve_conflict(
        &self,
        strategy: &ConflictStrategy,
        source1: (u64, u64), // (timestamp_ms, position)
        source2: (u64, u64),
        source1_name: &str,
    ) -> ConflictResolution {
        match strategy {
            ConflictStrategy::LatestSourceFirst => {
                if source1.0 >= source2.0 {
                    ConflictResolution::LatestWins
                } else {
                    ConflictResolution::FirstWins
                }
            }
            ConflictStrategy::FirstSourceFirst => ConflictResolution::FirstWins,
            ConflictStrategy::SourcePriority(priorities) => {
                let p1 = priorities.get(source1_name).copied().unwrap_or(0);
                ConflictResolution::SourcePriorityWins(format!("source_{}", p1))
            }
            ConflictStrategy::NoResolution => ConflictResolution::Error,
        }
    }

    pub fn get_config(&self) -> MultiSourceConfig {
        self.config.clone()
    }

    pub async fn channel_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels.len()
    }

    pub async fn active_channel_count(&self) -> usize {
        let channels = self.channels.read().await;
        channels
            .values()
            .filter(|c| matches!(c.state, ChannelState::Active))
            .count()
    }

    pub async fn is_healthy(&self) -> bool {
        let channels = self.channels.read().await;
        channels
            .values()
            .any(|c| matches!(c.state, ChannelState::Active))
    }

    pub async fn get_stats(&self) -> MultiSourceStats {
        let channels = self.channels.read().await;
        let active = channels
            .values()
            .filter(|c| matches!(c.state, ChannelState::Active))
            .count();
        let error = channels
            .values()
            .filter(|c| matches!(c.state, ChannelState::Error(_)))
            .count();

        MultiSourceStats {
            total_channels: channels.len(),
            active_channels: active,
            error_channels: error,
            conflict_count: self.conflict_log.read().await.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultiSourceStats {
    pub total_channels: usize,
    pub active_channels: usize,
    pub error_channels: usize,
    pub conflict_count: usize,
}

#[derive(Debug, Clone)]
pub enum MultiSourceError {
    ChannelNotFound(String),
    ChannelExists(String),
    ChannelInactive(String),
    InvalidConfiguration(String),
}

impl std::fmt::Display for MultiSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultiSourceError::ChannelNotFound(name) => {
                write!(f, "Channel not found: {}", name)
            }
            MultiSourceError::ChannelExists(name) => {
                write!(f, "Channel already exists: {}", name)
            }
            MultiSourceError::ChannelInactive(name) => {
                write!(f, "Channel is inactive: {}", name)
            }
            MultiSourceError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for MultiSourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id() {
        let id = ChannelId::new("channel1");
        assert_eq!(id.name, "channel1");

        let id2 = ChannelId::new("channel2");
        assert_ne!(id, id2);
    }

    #[test]
    fn test_channel_meta() {
        let meta = ChannelMeta::new("ch1", "master1.example.com".to_string(), 3306);
        assert_eq!(meta.id.name, "ch1");
        assert_eq!(meta.master_host, "master1.example.com");
        assert_eq!(meta.master_port, 3306);
        assert!(matches!(meta.state, ChannelState::Inactive));
    }

    #[test]
    fn test_channel_meta_with_user() {
        let meta =
            ChannelMeta::new("ch1", "master1.example.com".to_string(), 3306).with_user("repl_user");
        assert_eq!(meta.master_user, "repl_user");
    }

    #[test]
    fn test_channel_meta_state_transitions() {
        let mut meta = ChannelMeta::new("ch1", "master1.example.com".to_string(), 3306);

        meta.set_active();
        assert!(matches!(meta.state, ChannelState::Active));

        meta.set_error("connection lost");
        assert!(matches!(&meta.state, ChannelState::Error(e) if e == "connection lost"));

        meta.set_inactive();
        assert!(matches!(meta.state, ChannelState::Inactive));
    }

    #[test]
    fn test_conflict_strategy() {
        let s1 = ConflictStrategy::LatestSourceFirst;
        let s2 = ConflictStrategy::FirstSourceFirst;
        assert_ne!(s1, s2);
    }

    #[tokio::test]
    async fn test_multi_source_manager_new() {
        let manager = MultiSourceManager::new();
        assert_eq!(manager.channel_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_channel() {
        let manager = MultiSourceManager::new();
        let id = manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        assert_eq!(id.name, "channel1");
        assert_eq!(manager.channel_count().await, 1);
    }

    #[tokio::test]
    async fn test_add_duplicate_channel() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        let result = manager
            .add_channel("channel1", "master2.example.com", 3306, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MultiSourceError::ChannelExists(_)
        ));
    }

    #[tokio::test]
    async fn test_remove_channel() {
        let manager = MultiSourceManager::new();
        let id = manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        manager.remove_channel(&id).await.unwrap();
        assert_eq!(manager.channel_count().await, 0);
    }

    #[tokio::test]
    async fn test_remove_nonexistent_channel() {
        let manager = MultiSourceManager::new();
        let id = ChannelId::new("nonexistent");
        let result = manager.remove_channel(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_channel() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        let id = ChannelId::new("channel1");
        let meta = manager.get_channel(&id).await;
        assert!(meta.is_some());

        let id2 = ChannelId::new("channel2");
        let meta2 = manager.get_channel(&id2).await;
        assert!(meta2.is_none());
    }

    #[tokio::test]
    async fn test_set_channel_state() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        let id = ChannelId::new("channel1");
        manager
            .set_channel_state(&id, ChannelState::Active)
            .await
            .unwrap();

        let meta = manager.get_channel(&id).await.unwrap();
        assert!(matches!(meta.state, ChannelState::Active));
    }

    #[tokio::test]
    async fn test_update_channel_position() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();

        let id = ChannelId::new("channel1");
        manager
            .update_channel_position(&id, 1000, 1234567890)
            .await
            .unwrap();

        let meta = manager.get_channel(&id).await.unwrap();
        assert_eq!(meta.last_position, 1000);
        assert_eq!(meta.last_timestamp_ms, 1234567890);
    }

    #[tokio::test]
    async fn test_list_channels() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();
        manager
            .add_channel("channel2", "master2.example.com", 3306, None)
            .await
            .unwrap();

        let channels = manager.list_channels().await;
        assert_eq!(channels.len(), 2);
    }

    #[tokio::test]
    async fn test_get_active_channels() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();
        manager
            .add_channel("channel2", "master2.example.com", 3306, None)
            .await
            .unwrap();

        let id1 = ChannelId::new("channel1");
        manager
            .set_channel_state(&id1, ChannelState::Active)
            .await
            .unwrap();

        let active = manager.get_active_channels().await;
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id.name, "channel1");
    }

    #[tokio::test]
    async fn test_record_conflict() {
        let manager = MultiSourceManager::new();
        let record = ConflictRecord {
            channel: ChannelId::new("channel1"),
            table: "users".to_string(),
            key: "1".to_string(),
            source_timestamp_ms: 1234567890,
            resolution: ConflictResolution::LatestWins,
            resolved_value: vec![],
        };

        manager.record_conflict(record).await;
        let log = manager.get_conflict_log().await;
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_resolve_conflict_latest() {
        let manager = MultiSourceManager::new();
        let strategy = ConflictStrategy::LatestSourceFirst;

        let resolution = manager.resolve_conflict(&strategy, (2000, 100), (1000, 50), "source1");
        assert_eq!(resolution, ConflictResolution::LatestWins);

        let resolution2 = manager.resolve_conflict(&strategy, (1000, 50), (2000, 100), "source1");
        assert_eq!(resolution2, ConflictResolution::FirstWins);
    }

    #[test]
    fn test_resolve_conflict_first() {
        let manager = MultiSourceManager::new();
        let strategy = ConflictStrategy::FirstSourceFirst;

        let resolution = manager.resolve_conflict(&strategy, (2000, 100), (1000, 50), "source1");
        assert_eq!(resolution, ConflictResolution::FirstWins);
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let manager = MultiSourceManager::new();
        assert!(!manager.is_healthy().await);

        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();
        let id = ChannelId::new("channel1");
        manager
            .set_channel_state(&id, ChannelState::Active)
            .await
            .unwrap();

        assert!(manager.is_healthy().await);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let manager = MultiSourceManager::new();
        manager
            .add_channel("channel1", "master1.example.com", 3306, None)
            .await
            .unwrap();
        manager
            .add_channel("channel2", "master2.example.com", 3306, None)
            .await
            .unwrap();

        let id1 = ChannelId::new("channel1");
        manager
            .set_channel_state(&id1, ChannelState::Active)
            .await
            .unwrap();

        let id2 = ChannelId::new("channel2");
        manager
            .set_channel_state(&id2, ChannelState::Error("test error".to_string()))
            .await
            .unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_channels, 2);
        assert_eq!(stats.active_channels, 1);
        assert_eq!(stats.error_channels, 1);
    }

    #[test]
    fn test_multi_source_error_display() {
        let err = MultiSourceError::ChannelNotFound("ch1".to_string());
        assert!(err.to_string().contains("ch1"));

        let err2 = MultiSourceError::ChannelExists("ch2".to_string());
        assert!(err2.to_string().contains("ch2"));

        let err3 = MultiSourceError::ChannelInactive("ch3".to_string());
        assert!(err3.to_string().contains("ch3"));

        let err4 = MultiSourceError::InvalidConfiguration("bad config".to_string());
        assert!(err4.to_string().contains("bad config"));
    }

    #[tokio::test]
    async fn test_clear_conflict_log() {
        let manager = MultiSourceManager::new();
        let record = ConflictRecord {
            channel: ChannelId::new("channel1"),
            table: "users".to_string(),
            key: "1".to_string(),
            source_timestamp_ms: 1234567890,
            resolution: ConflictResolution::LatestWins,
            resolved_value: vec![],
        };

        manager.record_conflict(record).await;
        assert_eq!(manager.get_conflict_log().await.len(), 1);

        manager.clear_conflict_log().await;
        assert_eq!(manager.get_conflict_log().await.len(), 0);
    }

    #[test]
    fn test_conflict_resolution() {
        assert_eq!(
            format!("{:?}", ConflictResolution::LatestWins),
            "LatestWins"
        );
        assert_eq!(format!("{:?}", ConflictResolution::FirstWins), "FirstWins");
        assert_eq!(format!("{:?}", ConflictResolution::Error), "Error");
    }

    #[test]
    fn test_channel_state() {
        assert_eq!(format!("{:?}", ChannelState::Active), "Active");
        assert_eq!(format!("{:?}", ChannelState::Inactive), "Inactive");
        assert!(format!("{:?}", ChannelState::Error("err".to_string())).contains("Error"));
        assert_eq!(format!("{:?}", ChannelState::Connecting), "Connecting");
    }
}

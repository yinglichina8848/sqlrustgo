//! MySQL-compatible Replication Module
//!
//! This module provides basic master-slave replication support:
//! - Binary log (binlog) management
//! - Replication commands (SHOW MASTER STATUS, etc.)
//! - GTID-based replication configuration
//!
//! For full MySQL 5.7 replication, this is a foundation.

use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;

/// Replication role
#[derive(Debug, Clone, PartialEq)]
pub enum ReplicationRole {
    Master,
    Slave,
    Standalone,
}

/// Binlog event types (MySQL 5.7 compatible)
#[derive(Debug, Clone)]
pub enum BinlogEventType {
    Query,
    WriteRows,
    UpdateRows,
    DeleteRows,
    Xid,
    Rotate,
    FormatDescription,
}

/// Binlog event
#[derive(Debug, Clone)]
pub struct BinlogEvent {
    pub event_type: BinlogEventType,
    pub server_id: u32,
    pub log_pos: u64,
    pub timestamp: u32,
    pub database: String,
    pub table: Option<String>,
    pub sql: Option<String>,
    pub affected_rows: u64,
}

/// Binary log manager
pub struct BinlogManager {
    /// Current binlog file
    pub current_file: RwLock<String>,
    /// Current log position
    pub current_pos: RwLock<u64>,
    /// Server ID
    server_id: u32,
    /// Binlog events buffer
    events: AsyncRwLock<Vec<BinlogEvent>>,
    /// Enabled status
    enabled: RwLock<bool>,
}

impl Default for BinlogManager {
    fn default() -> Self {
        Self::new(1)
    }
}

impl BinlogManager {
    pub fn new(server_id: u32) -> Self {
        Self {
            current_file: RwLock::new("mysql-bin.000001".to_string()),
            current_pos: RwLock::new(4), // Start after header
            server_id,
            events: AsyncRwLock::new(Vec::new()),
            enabled: RwLock::new(true),
        }
    }

    /// Enable binlog
    pub fn enable(&self) {
        *self.enabled.write().unwrap() = true;
    }

    /// Disable binlog
    pub fn disable(&self) {
        *self.enabled.write().unwrap() = false;
    }

    /// Check if binlog is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read().unwrap()
    }

    /// Write an event to binlog
    pub async fn write_event(&self, event: BinlogEvent) -> Result<u64, String> {
        if !self.is_enabled() {
            return Err("Binlog is disabled".to_string());
        }

        let new_pos = {
            let mut pos = self.current_pos.write().unwrap();
            let new_pos = *pos + 1;
            *pos = new_pos;
            new_pos
        };

        self.events.write().await.push(event);
        Ok(new_pos)
    }

    /// Get current binlog status
    pub fn get_status(&self) -> BinlogStatus {
        BinlogStatus {
            file: self.current_file.read().unwrap().clone(),
            position: *self.current_pos.read().unwrap(),
            server_id: self.server_id,
            enabled: self.is_enabled(),
        }
    }

    /// Rotate binlog file
    pub async fn rotate(&self, new_file: String) -> Result<(), String> {
        let mut file = self.current_file.write().unwrap();
        *file = new_file;

        let mut pos = self.current_pos.write().unwrap();
        *pos = 4;

        Ok(())
    }

    /// Get recent events
    pub async fn get_events(&self, limit: usize) -> Vec<BinlogEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }
}

/// Binlog status
#[derive(Debug, Clone)]
pub struct BinlogStatus {
    pub file: String,
    pub position: u64,
    pub server_id: u32,
    pub enabled: bool,
}

/// Replication configuration
#[derive(Debug, Clone)]
pub struct ReplicationConfig {
    /// Server ID (must be unique in replication group)
    pub server_id: u32,
    /// Replication role
    pub role: ReplicationRole,
    /// Master host (for slave)
    pub master_host: Option<String>,
    /// Master port (for slave)
    pub master_port: Option<u16>,
    /// Master user (for slave)
    pub master_user: Option<String>,
    /// Relay log directory
    pub relay_log_dir: Option<String>,
    /// Read-only mode (for slave)
    pub read_only: bool,
    /// GTID mode enabled
    pub gtid_mode: bool,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            server_id: 1,
            role: ReplicationRole::Standalone,
            master_host: None,
            master_port: None,
            master_user: None,
            relay_log_dir: None,
            read_only: true,
            gtid_mode: false,
        }
    }
}

/// Global replication state
pub struct ReplicationState {
    config: RwLock<ReplicationConfig>,
    binlog: Arc<BinlogManager>,
    slave_status: RwLock<Option<SlaveStatus>>,
}

impl Default for ReplicationState {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplicationState {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(ReplicationConfig::default()),
            binlog: Arc::new(BinlogManager::new(1)),
            slave_status: RwLock::new(None),
        }
    }

    pub fn with_config(config: ReplicationConfig) -> Self {
        Self {
            config: RwLock::new(config.clone()),
            binlog: Arc::new(BinlogManager::new(config.server_id)),
            slave_status: RwLock::new(None),
        }
    }

    /// Get binlog manager
    pub fn binlog(&self) -> &Arc<BinlogManager> {
        &self.binlog
    }

    /// Get replication configuration
    pub fn config(&self) -> ReplicationConfig {
        self.config.read().unwrap().clone()
    }

    /// Update replication configuration
    pub fn update_config(&self, config: ReplicationConfig) {
        *self.config.write().unwrap() = config;
    }

    /// Get master status (SHOW MASTER STATUS)
    pub fn get_master_status(&self) -> Option<MasterStatus> {
        let config = self.config.read().unwrap();

        if config.role != ReplicationRole::Master {
            return None;
        }

        let binlog_status = self.binlog.get_status();

        Some(MasterStatus {
            file: binlog_status.file,
            position: binlog_status.position,
            binlog_do_db: Vec::new(),
            binlog_ignore_db: Vec::new(),
            executed_gtid_set: String::new(),
        })
    }

    /// Get slave status (SHOW SLAVE STATUS)
    pub fn get_slave_status(&self) -> Option<SlaveStatus> {
        let config = self.config.read().unwrap();

        if config.role != ReplicationRole::Slave {
            return None;
        }

        self.slave_status.read().unwrap().clone()
    }

    /// Set slave status
    pub fn set_slave_status(&self, status: SlaveStatus) {
        *self.slave_status.write().unwrap() = Some(status);
    }
}

/// Master status
#[derive(Debug, Clone)]
pub struct MasterStatus {
    pub file: String,
    pub position: u64,
    pub binlog_do_db: Vec<String>,
    pub binlog_ignore_db: Vec<String>,
    pub executed_gtid_set: String,
}

/// Slave status
#[derive(Debug, Clone)]
pub struct SlaveStatus {
    pub master_host: String,
    pub master_port: u16,
    pub master_user: String,
    pub read_only: bool,
    pub slave_io_running: bool,
    pub slave_sql_running: bool,
    pub last_error: Option<String>,
    pub seconds_behind_master: Option<u32>,
    pub relay_log_file: String,
    pub relay_log_pos: u64,
    pub master_log_file: String,
    pub read_master_log_pos: u64,
}

// ============================================================================
// GTID (Global Transaction ID) Support
// ============================================================================

/// GTID interval representing a range of transactions
#[derive(Debug, Clone, PartialEq)]
pub struct GtidInterval {
    pub sid: u64,
    pub start: u64,
    pub end: u64,
}

impl GtidInterval {
    pub fn new(sid: u64, start: u64, end: u64) -> Self {
        Self { sid, start, end }
    }

    pub fn contains(&self, gno: u64) -> bool {
        gno >= self.start && gno <= self.end
    }

    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }

    pub fn is_empty(&self) -> bool {
        self.start > self.end
    }
}

/// GTID set tracking executed transactions
#[derive(Debug, Clone, Default)]
pub struct GtidSet {
    pub intervals: Vec<GtidInterval>,
}

impl GtidSet {
    pub fn new() -> Self {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn add_interval(&mut self, interval: GtidInterval) {
        self.intervals.push(interval);
        self.normalize();
    }

    pub fn contains(&self, sid: u64, gno: u64) -> bool {
        self.intervals
            .iter()
            .any(|i| i.sid == sid && i.contains(gno))
    }

    fn normalize(&mut self) {
        self.intervals.sort_by_key(|i| (i.sid, i.start));
        let mut merged: Vec<GtidInterval> = Vec::new();

        for interval in &self.intervals {
            if let Some(last) = merged.last_mut() {
                if last.sid == interval.sid && last.end + 1 >= interval.start {
                    last.end = last.end.max(interval.end);
                    continue;
                }
            }
            merged.push(interval.clone());
        }
        self.intervals = merged;
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.iter().all(|i| i.is_empty())
    }

    pub fn len(&self) -> u64 {
        self.intervals.iter().map(|i| i.len()).sum()
    }
}

impl std::fmt::Display for GtidSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self
            .intervals
            .iter()
            .map(|i| format!("{}-{}", i.start, i.end))
            .collect();
        write!(f, "{}", parts.join(":"))
    }
}

/// GTID manager for tracking and persisting GTID state
pub struct GtidManager {
    server_id: u32,
    executed: RwLock<GtidSet>,
    purged: RwLock<GtidSet>,
    wait_timeout_ms: u64,
}

impl GtidManager {
    pub fn new(server_id: u32) -> Self {
        Self {
            server_id,
            executed: RwLock::new(GtidSet::new()),
            purged: RwLock::new(GtidSet::new()),
            wait_timeout_ms: 1000,
        }
    }

    pub fn with_timeout(server_id: u32, wait_timeout_ms: u64) -> Self {
        Self {
            server_id,
            executed: RwLock::new(GtidSet::new()),
            purged: RwLock::new(GtidSet::new()),
            wait_timeout_ms,
        }
    }

    pub fn add_gtid(&self, sid: u64, gno: u64) {
        let interval = GtidInterval::new(sid, gno, gno);
        let mut executed = self.executed.write().unwrap();
        executed.add_interval(interval);
    }

    pub fn add_gtid_interval(&self, sid: u64, start: u64, end: u64) {
        let interval = GtidInterval::new(sid, start, end);
        let mut executed = self.executed.write().unwrap();
        executed.add_interval(interval);
    }

    pub fn contains(&self, sid: u64, gno: u64) -> bool {
        self.executed.read().unwrap().contains(sid, gno)
    }

    pub fn get_executed_set(&self) -> GtidSet {
        self.executed.read().unwrap().clone()
    }

    pub fn get_executed_string(&self) -> String {
        self.executed.read().unwrap().to_string()
    }

    pub fn purge_gtids(&self, sid: u64, gno: u64) {
        let interval = GtidInterval::new(sid, 0, gno);
        let mut purged = self.purged.write().unwrap();
        purged.add_interval(interval);
    }

    pub fn get_purged_set(&self) -> GtidSet {
        self.purged.read().unwrap().clone()
    }

    pub fn get_wait_timeout_ms(&self) -> u64 {
        self.wait_timeout_ms
    }

    pub fn set_wait_timeout_ms(&mut self, timeout_ms: u64) {
        self.wait_timeout_ms = timeout_ms;
    }

    pub fn get_server_id(&self) -> u32 {
        self.server_id
    }

    pub fn generate_gtid(&self, sid: u64, gno: u64) -> GtidEvent {
        GtidEvent {
            sid,
            gno,
            server_id: self.server_id,
        }
    }

    pub fn get_gtid_count(&self) -> u64 {
        self.executed.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.executed.read().unwrap().is_empty()
    }
}

/// GTID event for replication
#[derive(Debug, Clone)]
pub struct GtidEvent {
    pub sid: u64,
    pub gno: u64,
    pub server_id: u32,
}

impl GtidEvent {
    pub fn new(sid: u64, gno: u64, server_id: u32) -> Self {
        Self {
            sid,
            gno,
            server_id,
        }
    }
}

// ============================================================================
// Semi-synchronous Replication Support
// ============================================================================

/// Semi-sync replication state
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SemiSyncState {
    #[default]
    Off,
    WaitServer,
    WaitSlave,
}

/// Semi-sync replica info
#[derive(Debug)]
pub struct SemiSyncReplica {
    pub node_id: u32,
    pub ack_channel: tokio::sync::mpsc::Sender<u64>,
    pub last_ack_time: RwLock<Option<std::time::Instant>>,
    pub lag_ms: u64,
}

impl Clone for SemiSyncReplica {
    fn clone(&self) -> Self {
        let (tx, _) = tokio::sync::mpsc::channel(100);
        Self {
            node_id: self.node_id,
            ack_channel: tx,
            last_ack_time: RwLock::new(None),
            lag_ms: self.lag_ms,
        }
    }
}

impl SemiSyncReplica {
    pub fn new(node_id: u32) -> Self {
        let (tx, _) = tokio::sync::mpsc::channel(100);
        Self {
            node_id,
            ack_channel: tx,
            last_ack_time: RwLock::new(None),
            lag_ms: 0,
        }
    }

    pub fn record_ack(&self) {
        let mut last_ack = self.last_ack_time.write().unwrap();
        *last_ack = Some(std::time::Instant::now());
    }

    pub fn get_last_ack_elapsed_ms(&self) -> Option<u64> {
        self.last_ack_time
            .read()
            .unwrap()
            .map(|t| t.elapsed().as_millis() as u64)
    }

    pub fn set_lag(&mut self, lag_ms: u64) {
        self.lag_ms = lag_ms;
    }

    pub fn get_lag(&self) -> u64 {
        self.lag_ms
    }
}

/// Semi-sync replication manager
pub struct SemiSyncManager {
    state: RwLock<SemiSyncState>,
    replicas: RwLock<Vec<SemiSyncReplica>>,
    wait_timeout_ms: u64,
    wait_count: RwLock<u32>,
    ack_timeout_ms: u64,
}

impl Default for SemiSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SemiSyncManager {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(SemiSyncState::Off),
            replicas: RwLock::new(Vec::new()),
            wait_timeout_ms: 10000,
            wait_count: RwLock::new(1),
            ack_timeout_ms: 1000,
        }
    }

    pub fn with_config(wait_timeout_ms: u64, wait_count: u32, ack_timeout_ms: u64) -> Self {
        Self {
            state: RwLock::new(SemiSyncState::Off),
            replicas: RwLock::new(Vec::new()),
            wait_timeout_ms,
            wait_count: RwLock::new(wait_count),
            ack_timeout_ms,
        }
    }

    pub fn enable(&self) {
        *self.state.write().unwrap() = SemiSyncState::WaitServer;
    }

    pub fn disable(&self) {
        *self.state.write().unwrap() = SemiSyncState::Off;
    }

    pub fn get_state(&self) -> SemiSyncState {
        self.state.read().unwrap().clone()
    }

    pub fn add_replica(&self, node_id: u32) {
        let replica = SemiSyncReplica::new(node_id);
        self.replicas.write().unwrap().push(replica);
    }

    pub fn remove_replica(&self, node_id: u32) {
        self.replicas
            .write()
            .unwrap()
            .retain(|r| r.node_id != node_id);
    }

    pub fn get_replica_count(&self) -> usize {
        self.replicas.read().unwrap().len()
    }

    pub fn get_wait_count(&self) -> u32 {
        *self.wait_count.read().unwrap()
    }

    pub fn set_wait_count(&self, count: u32) {
        *self.wait_count.write().unwrap() = count;
    }

    pub fn get_wait_timeout_ms(&self) -> u64 {
        self.wait_timeout_ms
    }

    pub fn set_wait_timeout_ms(&mut self, timeout_ms: u64) {
        self.wait_timeout_ms = timeout_ms;
    }

    pub fn get_ack_timeout_ms(&self) -> u64 {
        self.ack_timeout_ms
    }

    pub fn set_ack_timeout_ms(&mut self, timeout_ms: u64) {
        self.ack_timeout_ms = timeout_ms;
    }

    pub async fn wait_for_ack(&self, lsn: u64) -> Result<(), SemiSyncError> {
        if *self.state.read().unwrap() == SemiSyncState::Off {
            return Ok(());
        }

        let replicas = self.replicas.read().unwrap();
        let required_acks = *self.wait_count.read().unwrap();

        let mut acked = 0;
        for replica in replicas.iter() {
            if let Some(elapsed) = replica.get_last_ack_elapsed_ms() {
                if elapsed <= self.ack_timeout_ms {
                    acked += 1;
                }
            }
        }

        if acked >= required_acks as usize {
            Ok(())
        } else {
            Err(SemiSyncError::ack_timeout(lsn))
        }
    }

    pub fn record_ack(&self, node_id: u32) {
        if let Some(replica) = self
            .replicas
            .write()
            .unwrap()
            .iter_mut()
            .find(|r| r.node_id == node_id)
        {
            replica.record_ack();
        }
    }

    pub fn is_enabled(&self) -> bool {
        *self.state.read().unwrap() != SemiSyncState::Off
    }
}

#[derive(Debug, Clone)]
pub struct SemiSyncError {
    pub message: String,
    pub lsn: Option<u64>,
}

impl std::fmt::Display for SemiSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SemiSyncError {}

impl SemiSyncError {
    pub fn ack_timeout(lsn: u64) -> Self {
        Self {
            message: format!("Semi-sync ACK timeout for LSN {}", lsn),
            lsn: Some(lsn),
        }
    }

    pub fn no_replica() -> Self {
        Self {
            message: "No semi-sync replica available".to_string(),
            lsn: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binlog_manager() {
        let manager = BinlogManager::new(1);
        assert!(manager.is_enabled());

        let status = manager.get_status();
        assert_eq!(status.server_id, 1);
    }

    #[test]
    fn test_replication_state() {
        let state = ReplicationState::new();
        let config = state.config();

        assert_eq!(config.server_id, 1);
        assert_eq!(config.role, ReplicationRole::Standalone);
    }

    #[tokio::test]
    async fn test_binlog_events() {
        let manager = BinlogManager::new(1);

        let event = BinlogEvent {
            event_type: BinlogEventType::Query,
            server_id: 1,
            log_pos: 0,
            timestamp: 1234567890,
            database: "test".to_string(),
            table: Some("users".to_string()),
            sql: Some("INSERT INTO users VALUES (1)".to_string()),
            affected_rows: 1,
        };

        let pos = manager.write_event(event).await.unwrap();
        assert!(pos > 0);
    }

    #[test]
    fn test_binlog_manager_disable() {
        let manager = BinlogManager::new(1);
        assert!(manager.is_enabled());

        manager.disable();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_binlog_manager_default() {
        let manager = BinlogManager::default();
        let status = manager.get_status();
        assert_eq!(status.server_id, 1);
        assert_eq!(status.file, "mysql-bin.000001");
        assert_eq!(status.position, 4);
    }

    #[tokio::test]
    async fn test_binlog_rotate() {
        let manager = BinlogManager::new(1);

        let result = manager.rotate("mysql-bin.000002".to_string()).await;
        assert!(result.is_ok());

        let status = manager.get_status();
        assert_eq!(status.file, "mysql-bin.000002");
        assert_eq!(status.position, 4);
    }

    #[tokio::test]
    async fn test_binlog_get_events() {
        let manager = BinlogManager::new(1);

        for i in 0..5 {
            let event = BinlogEvent {
                event_type: BinlogEventType::Query,
                server_id: 1,
                log_pos: i as u64,
                timestamp: 1234567890 + i,
                database: "test".to_string(),
                table: None,
                sql: Some(format!("SELECT {}", i)),
                affected_rows: 0,
            };
            manager.write_event(event).await.unwrap();
        }

        let events = manager.get_events(3).await;
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_binlog_status_debug() {
        let status = BinlogStatus {
            file: "mysql-bin.000001".to_string(),
            position: 100,
            server_id: 1,
            enabled: true,
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("mysql-bin.000001"));
    }

    #[test]
    fn test_replication_role_debug() {
        assert_eq!(format!("{:?}", ReplicationRole::Master), "Master");
        assert_eq!(format!("{:?}", ReplicationRole::Slave), "Slave");
        assert_eq!(format!("{:?}", ReplicationRole::Standalone), "Standalone");
    }

    #[test]
    fn test_binlog_event_type_debug() {
        assert_eq!(format!("{:?}", BinlogEventType::Query), "Query");
        assert_eq!(format!("{:?}", BinlogEventType::WriteRows), "WriteRows");
        assert_eq!(format!("{:?}", BinlogEventType::UpdateRows), "UpdateRows");
        assert_eq!(format!("{:?}", BinlogEventType::DeleteRows), "DeleteRows");
    }

    #[test]
    fn test_replication_config_default() {
        let config = ReplicationConfig::default();

        assert_eq!(config.server_id, 1);
        assert_eq!(config.role, ReplicationRole::Standalone);
        assert!(config.master_host.is_none());
        assert!(config.master_port.is_none());
        assert!(config.master_user.is_none());
        assert!(config.relay_log_dir.is_none());
        assert!(config.read_only);
        assert!(!config.gtid_mode);
    }

    #[test]
    fn test_replication_config_custom() {
        let config = ReplicationConfig {
            server_id: 42,
            role: ReplicationRole::Slave,
            master_host: Some("master.example.com".to_string()),
            master_port: Some(3306),
            master_user: Some("repl_user".to_string()),
            relay_log_dir: Some("/var/log/mysql".to_string()),
            read_only: false,
            gtid_mode: true,
        };

        assert_eq!(config.server_id, 42);
        assert_eq!(config.role, ReplicationRole::Slave);
        assert_eq!(config.master_host, Some("master.example.com".to_string()));
        assert_eq!(config.master_port, Some(3306));
        assert!(config.gtid_mode);
    }

    #[test]
    fn test_replication_state_with_config() {
        let config = ReplicationConfig {
            server_id: 100,
            role: ReplicationRole::Master,
            ..Default::default()
        };
        let state = ReplicationState::with_config(config);

        assert_eq!(state.config().server_id, 100);
        assert_eq!(state.config().role, ReplicationRole::Master);
    }

    #[test]
    fn test_replication_state_update_config() {
        let state = ReplicationState::new();

        let new_config = ReplicationConfig {
            server_id: 200,
            role: ReplicationRole::Slave,
            master_host: Some("master.example.com".to_string()),
            ..Default::default()
        };
        state.update_config(new_config);

        let config = state.config();
        assert_eq!(config.server_id, 200);
        assert_eq!(config.role, ReplicationRole::Slave);
    }

    #[test]
    fn test_get_master_status_not_master() {
        let state = ReplicationState::new();
        let status = state.get_master_status();
        assert!(status.is_none());
    }

    #[test]
    fn test_get_master_status_as_master() {
        let state = ReplicationState::new();
        state.update_config(ReplicationConfig {
            role: ReplicationRole::Master,
            ..Default::default()
        });

        let status = state.get_master_status();
        assert!(status.is_some());

        let status = status.unwrap();
        assert_eq!(status.file, "mysql-bin.000001");
    }

    #[test]
    fn test_get_slave_status_not_slave() {
        let state = ReplicationState::new();
        let status = state.get_slave_status();
        assert!(status.is_none());
    }

    #[test]
    fn test_slave_status_debug() {
        let status = SlaveStatus {
            master_host: "master.example.com".to_string(),
            master_port: 3306,
            master_user: "repl".to_string(),
            read_only: true,
            slave_io_running: true,
            slave_sql_running: true,
            last_error: None,
            seconds_behind_master: Some(0),
            relay_log_file: "relay-bin.000001".to_string(),
            relay_log_pos: 100,
            master_log_file: "mysql-bin.000001".to_string(),
            read_master_log_pos: 500,
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("master.example.com"));
    }

    #[test]
    fn test_master_status_debug() {
        let status = MasterStatus {
            file: "mysql-bin.000001".to_string(),
            position: 1000,
            binlog_do_db: vec!["db1".to_string()],
            binlog_ignore_db: vec!["db2".to_string()],
            executed_gtid_set: "1-10".to_string(),
        };

        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("mysql-bin.000001"));
    }

    // GTID tests
    #[test]
    fn test_gtid_interval_contains() {
        let interval = GtidInterval::new(1, 10, 20);
        assert!(interval.contains(10));
        assert!(interval.contains(15));
        assert!(interval.contains(20));
        assert!(!interval.contains(9));
        assert!(!interval.contains(21));
    }

    #[test]
    fn test_gtid_interval_len() {
        let interval = GtidInterval::new(1, 10, 20);
        assert_eq!(interval.len(), 11);
    }

    #[test]
    fn test_gtid_interval_empty() {
        let interval = GtidInterval::new(1, 10, 9);
        assert!(interval.is_empty());
    }

    #[test]
    fn test_gtid_interval_debug() {
        let interval = GtidInterval::new(1, 10, 20);
        let debug_str = format!("{:?}", interval);
        assert!(debug_str.contains("sid: 1"));
    }

    #[test]
    fn test_gtid_interval_clone() {
        let interval = GtidInterval::new(1, 10, 20);
        let cloned = interval.clone();
        assert_eq!(cloned.sid, interval.sid);
        assert_eq!(cloned.start, interval.start);
        assert_eq!(cloned.end, interval.end);
    }

    #[test]
    fn test_gtid_set_new() {
        let set = GtidSet::new();
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_gtid_set_add_and_contains() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 10));
        set.add_interval(GtidInterval::new(2, 1, 5));

        assert!(set.contains(1, 5));
        assert!(set.contains(1, 10));
        assert!(!set.contains(1, 11));
        assert!(set.contains(2, 3));
        assert!(!set.contains(2, 10));
    }

    #[test]
    fn test_gtid_set_normalization() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 5));
        set.add_interval(GtidInterval::new(1, 6, 10));
        set.add_interval(GtidInterval::new(1, 3, 7));

        assert!(set.contains(1, 2));
        assert!(set.contains(1, 5));
        assert!(set.contains(1, 8));
        assert!(set.contains(1, 10));
        assert_eq!(set.len(), 10);
    }

    #[test]
    fn test_gtid_set_clone() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 10));

        let cloned = set.clone();
        assert!(cloned.contains(1, 5));
    }

    #[test]
    fn test_gtid_manager_add_and_check() {
        let manager = GtidManager::new(1);
        manager.add_gtid(1, 100);
        manager.add_gtid(1, 200);

        assert!(manager.contains(1, 100));
        assert!(manager.contains(1, 200));
        assert!(!manager.contains(1, 150));
        assert_eq!(manager.get_gtid_count(), 2);
    }

    #[test]
    fn test_gtid_manager_generate_gtid_event() {
        let manager = GtidManager::new(42);
        let event = manager.generate_gtid(1, 100);

        assert_eq!(event.sid, 1);
        assert_eq!(event.gno, 100);
        assert_eq!(event.server_id, 42);
    }

    #[test]
    fn test_gtid_manager_get_server_id() {
        let manager = GtidManager::new(42);
        assert_eq!(manager.get_server_id(), 42);
    }

    #[test]
    fn test_gtid_set_display() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 10));
        set.add_interval(GtidInterval::new(2, 5, 15));

        let display = format!("{}", set);
        assert!(display.contains("1-10"));
        assert!(display.contains("5-15"));
    }

    #[test]
    fn test_gtid_event_debug() {
        let event = GtidEvent {
            sid: 1,
            gno: 100,
            server_id: 42,
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("sid: 1"));
    }

    #[test]
    fn test_binlog_event_debug() {
        let event = BinlogEvent {
            event_type: BinlogEventType::Query,
            server_id: 1,
            log_pos: 100,
            timestamp: 1234567890,
            database: "test".to_string(),
            table: Some("users".to_string()),
            sql: Some("SELECT 1".to_string()),
            affected_rows: 0,
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("test"));
    }

    // Semi-sync tests
    #[test]
    fn test_semi_sync_state_default() {
        let state = SemiSyncState::default();
        assert_eq!(state, SemiSyncState::Off);
    }

    #[test]
    fn test_semi_sync_state_variants() {
        assert_eq!(format!("{:?}", SemiSyncState::Off), "Off");
        assert_eq!(format!("{:?}", SemiSyncState::WaitServer), "WaitServer");
        assert_eq!(format!("{:?}", SemiSyncState::WaitSlave), "WaitSlave");
    }

    #[test]
    fn test_semi_sync_manager_enable_disable() {
        let manager = SemiSyncManager::new();
        assert!(!manager.is_enabled());

        manager.enable();
        assert!(manager.is_enabled());
        assert_eq!(manager.get_state(), SemiSyncState::WaitServer);

        manager.disable();
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_semi_sync_manager_add_remove_replica() {
        let manager = SemiSyncManager::new();
        manager.add_replica(1);
        manager.add_replica(2);

        assert_eq!(manager.get_replica_count(), 2);

        manager.remove_replica(1);
        assert_eq!(manager.get_replica_count(), 1);
    }

    #[test]
    fn test_semi_sync_manager_wait_count() {
        let manager = SemiSyncManager::new();
        assert_eq!(manager.get_wait_count(), 1);

        manager.set_wait_count(3);
        assert_eq!(manager.get_wait_count(), 3);
    }

    #[tokio::test]
    async fn test_semi_sync_manager_wait_for_ack_disabled() {
        let manager = SemiSyncManager::new();
        let result = manager.wait_for_ack(100).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_semi_sync_replica_record_ack() {
        let mut replica = SemiSyncReplica::new(1);
        replica.record_ack();

        assert!(replica.get_last_ack_elapsed_ms().is_some());
    }

    #[test]
    fn test_semi_sync_replica_lag() {
        let mut replica = SemiSyncReplica::new(1);
        replica.set_lag(50);

        assert_eq!(replica.get_lag(), 50);
    }

    #[test]
    fn test_semi_sync_replica_debug() {
        let replica = SemiSyncReplica::new(1);
        let debug_str = format!("{:?}", replica);
        assert!(debug_str.contains("node_id: 1"));
    }

    #[test]
    fn test_semi_sync_error_display() {
        let err = SemiSyncError::ack_timeout(123);
        assert!(err.message.contains("123"));

        let err2 = SemiSyncError::no_replica();
        assert!(err2.message.contains("No semi-sync replica"));
    }

    #[test]
    fn test_semi_sync_error_clone() {
        let err = SemiSyncError::ack_timeout(100);
        let cloned = err.clone();
        assert!(cloned.message.contains("100"));
    }

    #[test]
    fn test_semi_sync_manager_config() {
        let mut manager = SemiSyncManager::with_config(5000, 2, 500);
        assert_eq!(manager.get_wait_timeout_ms(), 5000);
        assert_eq!(manager.get_ack_timeout_ms(), 500);

        manager.set_wait_timeout_ms(10000);
        manager.set_ack_timeout_ms(1000);
        assert_eq!(manager.get_wait_timeout_ms(), 10000);
        assert_eq!(manager.get_ack_timeout_ms(), 1000);
    }

    #[test]
    fn test_semi_sync_manager_remove_nonexistent_replica() {
        let manager = SemiSyncManager::new();
        manager.add_replica(1);
        manager.remove_replica(99);
        assert_eq!(manager.get_replica_count(), 1);
    }

    #[tokio::test]
    async fn test_binlog_write_disabled() {
        let manager = BinlogManager::new(1);
        manager.disable();

        let event = BinlogEvent {
            event_type: BinlogEventType::Query,
            server_id: 1,
            log_pos: 0,
            timestamp: 1234567890,
            database: "test".to_string(),
            table: None,
            sql: Some("SELECT 1".to_string()),
            affected_rows: 0,
        };

        let result = manager.write_event(event).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_gtid_set_len() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 10));
        assert_eq!(set.len(), 10);

        set.add_interval(GtidInterval::new(2, 1, 5));
        assert_eq!(set.len(), 15);
    }

    #[test]
    fn test_gtid_set_contains_after_merge() {
        let mut set = GtidSet::new();
        set.add_interval(GtidInterval::new(1, 1, 5));
        set.add_interval(GtidInterval::new(1, 4, 8));

        assert!(set.contains(1, 3));
        assert!(set.contains(1, 5));
        assert!(set.contains(1, 6));
        assert!(set.contains(1, 8));
    }
}

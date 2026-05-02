//! Semi-Synchronous Replication Module
//!
//! Implements MySQL 5.7 semi-synchronous replication with:
//! - AFTER_SYNC mode: Master waits for slave to receive relay log before committing
//! - AFTER_COMMIT mode: Master waits for slave to commit after commit
//!
//! Reference: MySQL 5.7 Reference Manual - Semisynchronous Replication

use std::sync::RwLock;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// Semi-sync replication mode
#[derive(Debug, Clone, PartialEq)]
pub enum SemiSyncMode {
    /// Master waits for slave to receive relay log before commit
    /// (MySQL 5.7+ default and recommended)
    AfterSync,
    /// Master waits for slave to commit after local commit
    /// (Legacy mode, less safe)
    AfterCommit,
}

/// Semi-sync replica state
#[derive(Debug, Clone)]
pub struct SemiSyncReplicaState {
    pub node_id: u32,
    pub connected: bool,
    pub last_ack_time: Option<Instant>,
    pub lag_ms: u64,
    pub last_position: u64,
}

impl SemiSyncReplicaState {
    pub fn new(node_id: u32) -> Self {
        Self {
            node_id,
            connected: true,
            last_ack_time: None,
            lag_ms: 0,
            last_position: 0,
        }
    }

    pub fn record_ack(&mut self, position: u64) {
        self.last_ack_time = Some(Instant::now());
        self.last_position = position;
    }

    pub fn set_disconnected(&mut self) {
        self.connected = false;
    }

    pub fn set_connected(&mut self) {
        self.connected = true;
    }

    pub fn set_lag(&mut self, lag_ms: u64) {
        self.lag_ms = lag_ms;
    }
}

/// ACK sender for communication between master and replica
#[derive(Debug, Clone)]
pub struct AckSender {
    node_id: u32,
    sender: mpsc::Sender<u64>,
}

impl AckSender {
    pub fn new(node_id: u32) -> (Self, mpsc::Receiver<u64>) {
        let (sender, receiver) = mpsc::channel(100);
        (Self { node_id, sender }, receiver)
    }

    pub async fn send_ack(&self, lsn: u64) -> Result<(), mpsc::error::SendError<u64>> {
        self.sender.send(lsn).await
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }
}

/// Semi-sync master configuration
#[derive(Debug, Clone)]
pub struct SemiSyncMasterConfig {
    pub mode: SemiSyncMode,
    pub timeout_ms: u64,
    pub wait_count: u32,
    pub ack_timeout_ms: u64,
    pub replica_lag_max_ms: u64,
}

impl Default for SemiSyncMasterConfig {
    fn default() -> Self {
        Self {
            mode: SemiSyncMode::AfterSync,
            timeout_ms: 10000,
            wait_count: 1,
            ack_timeout_ms: 1000,
            replica_lag_max_ms: 5000,
        }
    }
}

/// Semi-sync master state
pub struct SemiSyncMaster {
    config: RwLock<SemiSyncMasterConfig>,
    replicas: RwLock<Vec<SemiSyncReplicaState>>,
    ack_senders: RwLock<Vec<AckSender>>,
    state: RwLock<SemiSyncMasterState>,
    total_acks: RwLock<u64>,
    total_timeouts: RwLock<u64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SemiSyncMasterState {
    #[default]
    Off,
    On,
    Degraded,
}

impl Default for SemiSyncMaster {
    fn default() -> Self {
        Self::new()
    }
}

impl SemiSyncMaster {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(SemiSyncMasterConfig::default()),
            replicas: RwLock::new(Vec::new()),
            ack_senders: RwLock::new(Vec::new()),
            state: RwLock::new(SemiSyncMasterState::Off),
            total_acks: RwLock::new(0),
            total_timeouts: RwLock::new(0),
        }
    }

    pub fn with_config(config: SemiSyncMasterConfig) -> Self {
        Self {
            config: RwLock::new(config),
            replicas: RwLock::new(Vec::new()),
            ack_senders: RwLock::new(Vec::new()),
            state: RwLock::new(SemiSyncMasterState::Off),
            total_acks: RwLock::new(0),
            total_timeouts: RwLock::new(0),
        }
    }

    pub fn enable(&self) {
        *self.state.write().unwrap() = SemiSyncMasterState::On;
    }

    pub fn disable(&self) {
        *self.state.write().unwrap() = SemiSyncMasterState::Off;
    }

    pub fn get_state(&self) -> SemiSyncMasterState {
        self.state.read().unwrap().clone()
    }

    pub fn is_enabled(&self) -> bool {
        match *self.state.read().unwrap() {
            SemiSyncMasterState::On | SemiSyncMasterState::Degraded => true,
            SemiSyncMasterState::Off => false,
        }
    }

    pub fn register_replica(&self, node_id: u32) -> AckSender {
        let replica = SemiSyncReplicaState::new(node_id);
        self.replicas.write().unwrap().push(replica);

        let (ack_sender, _receiver) = AckSender::new(node_id);
        self.ack_senders.write().unwrap().push(ack_sender.clone());
        ack_sender
    }

    pub fn unregister_replica(&self, node_id: u32) {
        self.replicas
            .write()
            .unwrap()
            .retain(|r| r.node_id != node_id);
        self.ack_senders
            .write()
            .unwrap()
            .retain(|s| s.node_id() != node_id);
    }

    pub fn get_replica_count(&self) -> usize {
        self.replicas.read().unwrap().len()
    }

    pub fn get_connected_replica_count(&self) -> usize {
        self.replicas
            .read()
            .unwrap()
            .iter()
            .filter(|r| r.connected)
            .count()
    }

    pub fn get_config(&self) -> SemiSyncMasterConfig {
        self.config.read().unwrap().clone()
    }

    pub fn set_mode(&self, mode: SemiSyncMode) {
        self.config.write().unwrap().mode = mode;
    }

    pub fn set_timeout(&self, timeout_ms: u64) {
        self.config.write().unwrap().timeout_ms = timeout_ms;
    }

    pub fn set_wait_count(&self, count: u32) {
        self.config.write().unwrap().wait_count = count;
    }

    pub fn set_ack_timeout(&self, ack_timeout_ms: u64) {
        self.config.write().unwrap().ack_timeout_ms = ack_timeout_ms;
    }

    pub fn get_total_acks(&self) -> u64 {
        *self.total_acks.read().unwrap()
    }

    pub fn get_total_timeouts(&self) -> u64 {
        *self.total_timeouts.read().unwrap()
    }

    pub async fn wait_for_slaves(&self, lsn: u64) -> Result<(), SemiSyncError> {
        if !self.is_enabled() {
            return Ok(());
        }

        {
            let state = self.state.read().unwrap().clone();
            if state == SemiSyncMasterState::Off {
                return Ok(());
            }
        }

        let config = self.config.read().unwrap().clone();
        let replicas = {
            let r = self.replicas.read().unwrap();
            if r.is_empty() {
                *self.total_timeouts.write().unwrap() += 1;
                return Err(SemiSyncError::NoReplica);
            }
            r.clone()
        };

        let required_acks = config.wait_count.min(replicas.len() as u32) as usize;
        let timeout = Duration::from_millis(config.timeout_ms);
        let ack_timeout = Duration::from_millis(config.ack_timeout_ms);
        let start = Instant::now();

        loop {
            let acked_count = {
                let replicas = self.replicas.read().unwrap();
                replicas
                    .iter()
                    .filter(|r| {
                        r.connected && r.last_ack_time.is_some_and(|t| t.elapsed() <= ack_timeout)
                    })
                    .count()
            };

            if acked_count >= required_acks {
                *self.total_acks.write().unwrap() += 1;
                return Ok(());
            }

            if start.elapsed() >= timeout {
                *self.total_timeouts.write().unwrap() += 1;

                let has_connected = {
                    let replicas = self.replicas.read().unwrap();
                    replicas.iter().any(|r| r.connected)
                };
                if has_connected {
                    *self.state.write().unwrap() = SemiSyncMasterState::Degraded;
                }
                return Err(SemiSyncError::AckTimeout(lsn));
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    pub fn record_slave_ack(&self, node_id: u32, lsn: u64) {
        if let Some(replica) = self
            .replicas
            .write()
            .unwrap()
            .iter_mut()
            .find(|r| r.node_id == node_id)
        {
            replica.record_ack(lsn);
        }
    }

    pub fn set_replica_disconnected(&self, node_id: u32) {
        if let Some(replica) = self
            .replicas
            .write()
            .unwrap()
            .iter_mut()
            .find(|r| r.node_id == node_id)
        {
            replica.set_disconnected();
        }

        if self.get_connected_replica_count() == 0 && self.get_replica_count() > 0 {
            *self.state.write().unwrap() = SemiSyncMasterState::Degraded;
        }
    }

    pub fn set_replica_connected(&self, node_id: u32) {
        if let Some(replica) = self
            .replicas
            .write()
            .unwrap()
            .iter_mut()
            .find(|r| r.node_id == node_id)
        {
            replica.set_connected();
        }

        if self.get_connected_replica_count() > 0
            && *self.state.read().unwrap() == SemiSyncMasterState::Degraded
        {
            *self.state.write().unwrap() = SemiSyncMasterState::On;
        }
    }

    pub fn get_replica_statuses(&self) -> Vec<SemiSyncReplicaState> {
        self.replicas.read().unwrap().clone()
    }

    pub fn get_stats(&self) -> SemiSyncStats {
        SemiSyncStats {
            state: self.get_state(),
            replica_count: self.get_replica_count(),
            connected_count: self.get_connected_replica_count(),
            total_acks: self.get_total_acks(),
            total_timeouts: self.get_total_timeouts(),
            mode: self.get_config().mode,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemiSyncStats {
    pub state: SemiSyncMasterState,
    pub replica_count: usize,
    pub connected_count: usize,
    pub total_acks: u64,
    pub total_timeouts: u64,
    pub mode: SemiSyncMode,
}

#[derive(Debug, Clone)]
pub enum SemiSyncError {
    AckTimeout(u64),
    NoReplica,
    NotEnabled,
    Internal(String),
}

impl std::fmt::Display for SemiSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemiSyncError::AckTimeout(lsn) => write!(f, "Semi-sync ACK timeout for LSN {}", lsn),
            SemiSyncError::NoReplica => write!(f, "No semi-sync replica available"),
            SemiSyncError::NotEnabled => write!(f, "Semi-sync replication is not enabled"),
            SemiSyncError::Internal(msg) => write!(f, "Internal semi-sync error: {}", msg),
        }
    }
}

impl std::error::Error for SemiSyncError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semi_sync_mode_default() {
        let config = SemiSyncMasterConfig::default();
        assert_eq!(config.mode, SemiSyncMode::AfterSync);
        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.wait_count, 1);
    }

    #[test]
    fn test_semi_sync_master_new() {
        let master = SemiSyncMaster::new();
        assert!(!master.is_enabled());
        assert_eq!(master.get_state(), SemiSyncMasterState::Off);
    }

    #[test]
    fn test_semi_sync_master_enable_disable() {
        let master = SemiSyncMaster::new();
        master.enable();
        assert!(master.is_enabled());
        assert_eq!(master.get_state(), SemiSyncMasterState::On);

        master.disable();
        assert!(!master.is_enabled());
        assert_eq!(master.get_state(), SemiSyncMasterState::Off);
    }

    #[test]
    fn test_register_unregister_replica() {
        let master = SemiSyncMaster::new();
        let _sender = master.register_replica(1);
        assert_eq!(master.get_replica_count(), 1);

        master.unregister_replica(1);
        assert_eq!(master.get_replica_count(), 0);
    }

    #[test]
    fn test_replica_connection_state() {
        let master = SemiSyncMaster::new();
        master.register_replica(1);
        master.register_replica(2);

        assert_eq!(master.get_connected_replica_count(), 2);

        master.set_replica_disconnected(1);
        assert_eq!(master.get_connected_replica_count(), 1);

        master.set_replica_connected(1);
        assert_eq!(master.get_connected_replica_count(), 2);
    }

    #[test]
    fn test_set_mode() {
        let master = SemiSyncMaster::new();
        master.set_mode(SemiSyncMode::AfterCommit);
        assert_eq!(master.get_config().mode, SemiSyncMode::AfterCommit);

        master.set_mode(SemiSyncMode::AfterSync);
        assert_eq!(master.get_config().mode, SemiSyncMode::AfterSync);
    }

    #[test]
    fn test_set_timeout() {
        let master = SemiSyncMaster::new();
        master.set_timeout(5000);
        assert_eq!(master.get_config().timeout_ms, 5000);
    }

    #[test]
    fn test_set_wait_count() {
        let master = SemiSyncMaster::new();
        master.register_replica(1);
        master.register_replica(2);

        master.set_wait_count(2);
        assert_eq!(master.get_config().wait_count, 2);
    }

    #[tokio::test]
    async fn test_wait_for_slaves_disabled() {
        let master = SemiSyncMaster::new();
        let result = master.wait_for_slaves(100).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_slaves_no_replicas() {
        let master = SemiSyncMaster::new();
        master.enable();
        let result = master.wait_for_slaves(100).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_record_slave_ack() {
        let master = SemiSyncMaster::new();
        master.register_replica(1);
        master.record_slave_ack(1, 100);
        assert_eq!(master.get_total_acks(), 0);
    }

    #[test]
    fn test_semi_sync_stats() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.register_replica(1);
        master.register_replica(2);

        let stats = master.get_stats();
        assert_eq!(stats.state, SemiSyncMasterState::On);
        assert_eq!(stats.replica_count, 2);
        assert_eq!(stats.connected_count, 2);
        assert_eq!(stats.mode, SemiSyncMode::AfterSync);
    }

    #[test]
    fn test_semi_sync_error_display() {
        let err = SemiSyncError::AckTimeout(123);
        assert!(err.to_string().contains("123"));

        let err2 = SemiSyncError::NoReplica;
        assert!(err2.to_string().contains("No semi-sync replica"));

        let err3 = SemiSyncError::NotEnabled;
        assert!(err3.to_string().contains("not enabled"));
    }

    #[test]
    fn test_semi_sync_replica_state() {
        let mut replica = SemiSyncReplicaState::new(1);
        assert!(replica.connected);
        assert_eq!(replica.node_id, 1);

        replica.record_ack(100);
        assert!(replica.last_ack_time.is_some());
        assert_eq!(replica.last_position, 100);

        replica.set_disconnected();
        assert!(!replica.connected);

        replica.set_connected();
        assert!(replica.connected);

        replica.set_lag(50);
        assert_eq!(replica.lag_ms, 50);
    }

    #[test]
    fn test_semi_sync_master_state() {
        assert_eq!(format!("{:?}", SemiSyncMasterState::Off), "Off");
        assert_eq!(format!("{:?}", SemiSyncMasterState::On), "On");
        assert_eq!(format!("{:?}", SemiSyncMasterState::Degraded), "Degraded");
    }

    #[test]
    fn test_ack_sender() {
        let (sender, mut receiver) = AckSender::new(1);
        assert_eq!(sender.node_id(), 1);

        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                sender.send_ack(100).await.unwrap();
                let received = receiver.recv().await.unwrap();
                assert_eq!(received, 100);
            });
    }

    #[test]
    fn test_semi_sync_mode() {
        assert_eq!(format!("{:?}", SemiSyncMode::AfterSync), "AfterSync");
        assert_eq!(format!("{:?}", SemiSyncMode::AfterCommit), "AfterCommit");
    }

    #[test]
    fn test_degraded_state_when_all_disconnected() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.register_replica(1);

        master.set_replica_disconnected(1);
        assert_eq!(master.get_state(), SemiSyncMasterState::Degraded);
    }

    #[test]
    fn test_recover_from_degraded() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.register_replica(1);
        master.set_replica_disconnected(1);

        assert_eq!(master.get_state(), SemiSyncMasterState::Degraded);

        master.set_replica_connected(1);
        assert_eq!(master.get_state(), SemiSyncMasterState::On);
    }

    #[test]
    fn test_multiple_replicas() {
        let master = SemiSyncMaster::new();
        master.register_replica(1);
        master.register_replica(2);
        master.register_replica(3);

        assert_eq!(master.get_replica_count(), 3);
        assert_eq!(master.get_connected_replica_count(), 3);

        master.unregister_replica(2);
        assert_eq!(master.get_replica_count(), 2);
    }

    #[test]
    fn test_config_clone() {
        let config = SemiSyncMasterConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.mode, config.mode);
        assert_eq!(cloned.timeout_ms, config.timeout_ms);
    }

    #[test]
    fn test_stats_clone() {
        let master = SemiSyncMaster::new();
        master.enable();
        let stats = master.get_stats();
        let cloned = stats.clone();
        assert_eq!(cloned.state, SemiSyncMasterState::On);
    }
}

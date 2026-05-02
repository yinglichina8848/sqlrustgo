//! Semi-synchronous Replication Module
//!
//! MySQL 5.7 compatible semi-synchronous replication support.
//!
//! # Key Concepts
//!
//! - **AFTER_SYNC mode** (5.7+): Master waits for slave to receive relay log
//!   BEFORE committing to storage
//! - **AFTER_COMMIT mode** (old): Master waits for slave to commit AFTER master
//!   commits
//! - **Timeout mechanism**: Master falls back to async if slave doesn't ACK
//!   within timeout
//! - **ACK collector**: Master waits for configured number of slave ACKs
//!
//! # Example
//!
//! ```ignore
//! use sqlrustgo_distributed::semisync::{SemiSyncMaster, SemiSyncMode};
//!
//! let mut master = SemiSyncMaster::new();
//! master.set_mode(SemiSyncMode::AfterSync);
//! master.set_timeout_ms(10000);
//! master.set_wait_count(1);
//! master.enable();
//! ```

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::Duration;

/// Semi-sync replication mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemiSyncMode {
    /// Wait for relay log receipt (MySQL 5.7+)
    /// Master sends transaction to slaves and waits for ACK before committing
    #[default]
    AfterSync,
    /// Wait for slave commit (legacy mode)
    /// Master commits first, then waits for slave to commit
    AfterCommit,
}

/// Semi-sync master status (MySQL compatible)
#[derive(Debug, Clone, Default)]
pub struct SemiSyncMasterStatus {
    /// Rpl_semi_sync_master_status - ON if semi-sync is enabled
    pub rpl_semi_sync_master_status: bool,
    /// Rpl_semi_sync_master_clients - number of semi-sync slaves
    pub rpl_semi_sync_master_clients: u32,
    /// Rpl_semi_sync_master_yes_transactions - transactions with semi-sync
    pub rpl_semi_sync_master_yes_transactions: u64,
    /// Rpl_semi_sync_master_no_transactions - transactions without semi-sync
    pub rpl_semi_sync_master_no_transactions: u64,
}

/// Semi-sync slave status
#[derive(Debug, Clone, Default)]
pub struct SemiSyncSlaveStatus {
    /// Whether semi-sync is enabled on slave
    pub rpl_semi_sync_slave_status: bool,
    /// Master UUID this slave is connected to
    pub master_uuid: Option<String>,
    /// Last ACK sent time
    pub last_ack_time: Option<std::time::Instant>,
}

/// Semi-sync master state
pub struct SemiSyncMaster {
    /// Whether semi-sync is enabled
    enabled: AtomicBool,
    /// Semi-sync replication mode
    mode: RwLock<SemiSyncMode>,
    /// Timeout in milliseconds before falling back to async
    timeout_ms: RwLock<u64>,
    /// Number of ACKs to wait for
    wait_count: RwLock<u32>,
    /// ACK counter for tracking received ACKs
    ack_counter: AtomicU32,
    /// Status information
    status: RwLock<SemiSyncMasterStatus>,
    /// Transaction counter for yes responses
    yes_transactions: AtomicU64,
    /// Transaction counter for no responses
    no_transactions: AtomicU64,
}

impl Default for SemiSyncMaster {
    fn default() -> Self {
        Self::new()
    }
}

impl SemiSyncMaster {
    /// Create a new SemiSyncMaster with default settings
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            mode: RwLock::new(SemiSyncMode::default()),
            timeout_ms: RwLock::new(10000), // 10 seconds default
            wait_count: RwLock::new(1),
            ack_counter: AtomicU32::new(0),
            status: RwLock::new(SemiSyncMasterStatus::default()),
            yes_transactions: AtomicU64::new(0),
            no_transactions: AtomicU64::new(0),
        }
    }

    /// Create with custom configuration
    pub fn with_config(timeout_ms: u64, wait_count: u32, mode: SemiSyncMode) -> Self {
        Self {
            enabled: AtomicBool::new(false),
            mode: RwLock::new(mode),
            timeout_ms: RwLock::new(timeout_ms),
            wait_count: RwLock::new(wait_count),
            ack_counter: AtomicU32::new(0),
            status: RwLock::new(SemiSyncMasterStatus::default()),
            yes_transactions: AtomicU64::new(0),
            no_transactions: AtomicU64::new(0),
        }
    }

    /// Enable semi-synchronous replication
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
        let mut status = self.status.write().unwrap();
        status.rpl_semi_sync_master_status = true;
    }

    /// Disable semi-synchronous replication
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        let mut status = self.status.write().unwrap();
        status.rpl_semi_sync_master_status = false;
    }

    /// Check if semi-sync is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Get current semi-sync mode
    pub fn get_mode(&self) -> SemiSyncMode {
        *self.mode.read().unwrap()
    }

    /// Set semi-sync mode
    pub fn set_mode(&self, mode: SemiSyncMode) {
        *self.mode.write().unwrap() = mode;
    }

    /// Get timeout in milliseconds
    pub fn get_timeout_ms(&self) -> u64 {
        *self.timeout_ms.read().unwrap()
    }

    /// Set timeout in milliseconds
    pub fn set_timeout_ms(&self, timeout_ms: u64) {
        *self.timeout_ms.write().unwrap() = timeout_ms;
    }

    /// Get wait count (number of ACKs to wait for)
    pub fn get_wait_count(&self) -> u32 {
        *self.wait_count.read().unwrap()
    }

    /// Set wait count
    pub fn set_wait_count(&self, count: u32) {
        *self.wait_count.write().unwrap() = count;
    }

    /// Get current status
    pub fn get_status(&self) -> SemiSyncMasterStatus {
        let status = self.status.read().unwrap();
        SemiSyncMasterStatus {
            rpl_semi_sync_master_status: status.rpl_semi_sync_master_status,
            rpl_semi_sync_master_clients: status.rpl_semi_sync_master_clients,
            rpl_semi_sync_master_yes_transactions: self
                .yes_transactions
                .load(Ordering::SeqCst),
            rpl_semi_sync_master_no_transactions: self
                .no_transactions
                .load(Ordering::SeqCst),
        }
    }

    /// Update client count
    pub fn set_client_count(&self, count: u32) {
        let mut status = self.status.write().unwrap();
        status.rpl_semi_sync_master_clients = count;
    }

    /// Increment ACK counter
    pub fn increment_ack(&self) -> u32 {
        self.ack_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Reset ACK counter
    pub fn reset_ack_counter(&self) {
        self.ack_counter.store(0, Ordering::SeqCst);
    }

    /// Get current ACK count
    pub fn get_ack_count(&self) -> u32 {
        self.ack_counter.load(Ordering::SeqCst)
    }

    /// Record a successful semi-sync transaction
    pub fn record_yes_transaction(&self) {
        self.yes_transactions.fetch_add(1, Ordering::SeqCst);
    }

    /// Record a transaction that fell back to async
    pub fn record_no_transaction(&self) {
        self.no_transactions.fetch_add(1, Ordering::SeqCst);
    }

    /// Wait for ACKs from slaves
    ///
    /// Returns Ok(()) if required ACKs received, Err if timeout
    pub async fn wait_for_acks(&self, required: u32) -> Result<(), SemiSyncTimeoutError> {
        if !self.is_enabled() {
            return Ok(());
        }

        let timeout_ms = self.get_timeout_ms();
        let start = std::time::Instant::now();

        while self.get_ack_count() < required {
            if start.elapsed().as_millis() as u64 >= timeout_ms {
                return Err(SemiSyncTimeoutError {
                    required,
                    received: self.get_ack_count(),
                    timeout_ms,
                });
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

/// Error indicating ACK timeout
#[derive(Debug, Clone)]
pub struct SemiSyncTimeoutError {
    /// Number of ACKs required
    pub required: u32,
    /// Number of ACKs received before timeout
    pub received: u32,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

impl std::fmt::Display for SemiSyncTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Semi-sync timeout: required {} ACKs but only received {} within {}ms",
            self.required, self.received, self.timeout_ms
        )
    }
}

impl std::error::Error for SemiSyncTimeoutError {}

/// Semi-sync slave state
pub struct SemiSyncSlave {
    /// Whether semi-sync is enabled
    enabled: AtomicBool,
    /// Master UUID this slave is connected to
    master_uuid: RwLock<Option<String>>,
    /// Whether ACK has been sent for current transaction
    ack_sent: AtomicBool,
    /// Slave status
    status: RwLock<SemiSyncSlaveStatus>,
}

impl Default for SemiSyncSlave {
    fn default() -> Self {
        Self::new()
    }
}

impl SemiSyncSlave {
    /// Create a new SemiSyncSlave
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            master_uuid: RwLock::new(None),
            ack_sent: AtomicBool::new(false),
            status: RwLock::new(SemiSyncSlaveStatus::default()),
        }
    }

    /// Enable semi-synchronous replication
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::SeqCst);
        let mut status = self.status.write().unwrap();
        status.rpl_semi_sync_slave_status = true;
    }

    /// Disable semi-synchronous replication
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        let mut status = self.status.write().unwrap();
        status.rpl_semi_sync_slave_status = false;
    }

    /// Check if semi-sync is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    /// Set master UUID
    pub fn set_master_uuid(&self, uuid: Option<String>) {
        *self.master_uuid.write().unwrap() = uuid.clone();
        let mut status = self.status.write().unwrap();
        status.master_uuid = uuid;
    }

    /// Get master UUID
    pub fn get_master_uuid(&self) -> Option<String> {
        self.master_uuid.read().unwrap().clone()
    }

    /// Mark ACK as sent for current transaction
    pub fn mark_ack_sent(&self) {
        self.ack_sent.store(true, Ordering::SeqCst);
        let mut status = self.status.write().unwrap();
        status.last_ack_time = Some(std::time::Instant::now());
    }

    /// Reset ACK sent flag
    pub fn reset_ack_sent(&self) {
        self.ack_sent.store(false, Ordering::SeqCst);
    }

    /// Check if ACK was sent
    pub fn is_ack_sent(&self) -> bool {
        self.ack_sent.load(Ordering::SeqCst)
    }

    /// Get slave status
    pub fn get_status(&self) -> SemiSyncSlaveStatus {
        self.status.read().unwrap().clone()
    }
}

/// Semi-sync error types
#[derive(Debug, Clone)]
pub enum SemiSyncError {
    /// Timeout waiting for slave ACK
    Timeout(SemiSyncTimeoutError),
    /// No slaves available
    NoSlaves,
    /// Invalid configuration
    InvalidConfig(String),
}

impl std::fmt::Display for SemiSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemiSyncError::Timeout(e) => write!(f, "Timeout: {}", e),
            SemiSyncError::NoSlaves => write!(f, "No semi-sync slaves available"),
            SemiSyncError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for SemiSyncError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semi_sync_mode_default() {
        assert_eq!(SemiSyncMode::default(), SemiSyncMode::AfterSync);
    }

    #[test]
    fn test_semi_sync_mode_debug() {
        assert_eq!(format!("{:?}", SemiSyncMode::AfterSync), "AfterSync");
        assert_eq!(format!("{:?}", SemiSyncMode::AfterCommit), "AfterCommit");
    }

    #[test]
    fn test_semi_sync_master_new() {
        let master = SemiSyncMaster::new();
        assert!(!master.is_enabled());
        assert_eq!(master.get_mode(), SemiSyncMode::AfterSync);
        assert_eq!(master.get_timeout_ms(), 10000);
        assert_eq!(master.get_wait_count(), 1);
    }

    #[test]
    fn test_semi_sync_master_enable_disable() {
        let master = SemiSyncMaster::new();
        assert!(!master.is_enabled());

        master.enable();
        assert!(master.is_enabled());

        master.disable();
        assert!(!master.is_enabled());
    }

    #[test]
    fn test_semi_sync_master_set_mode() {
        let master = SemiSyncMaster::new();
        assert_eq!(master.get_mode(), SemiSyncMode::AfterSync);

        master.set_mode(SemiSyncMode::AfterCommit);
        assert_eq!(master.get_mode(), SemiSyncMode::AfterCommit);
    }

    #[test]
    fn test_semi_sync_master_set_timeout() {
        let master = SemiSyncMaster::new();
        assert_eq!(master.get_timeout_ms(), 10000);

        master.set_timeout_ms(5000);
        assert_eq!(master.get_timeout_ms(), 5000);
    }

    #[test]
    fn test_semi_sync_master_set_wait_count() {
        let master = SemiSyncMaster::new();
        assert_eq!(master.get_wait_count(), 1);

        master.set_wait_count(3);
        assert_eq!(master.get_wait_count(), 3);
    }

    #[test]
    fn test_semi_sync_master_ack_counter() {
        let master = SemiSyncMaster::new();
        assert_eq!(master.get_ack_count(), 0);

        master.increment_ack();
        assert_eq!(master.get_ack_count(), 1);

        master.increment_ack();
        assert_eq!(master.get_ack_count(), 2);

        master.reset_ack_counter();
        assert_eq!(master.get_ack_count(), 0);
    }

    #[test]
    fn test_semi_sync_master_status() {
        let master = SemiSyncMaster::new();
        let status = master.get_status();
        assert!(!status.rpl_semi_sync_master_status);
        assert_eq!(status.rpl_semi_sync_master_clients, 0);
        assert_eq!(status.rpl_semi_sync_master_yes_transactions, 0);
        assert_eq!(status.rpl_semi_sync_master_no_transactions, 0);

        master.enable();
        master.set_client_count(2);
        master.record_yes_transaction();
        master.record_yes_transaction();
        master.record_no_transaction();

        let status = master.get_status();
        assert!(status.rpl_semi_sync_master_status);
        assert_eq!(status.rpl_semi_sync_master_clients, 2);
        assert_eq!(status.rpl_semi_sync_master_yes_transactions, 2);
        assert_eq!(status.rpl_semi_sync_master_no_transactions, 1);
    }

    #[test]
    fn test_semi_sync_master_with_config() {
        let master = SemiSyncMaster::with_config(5000, 2, SemiSyncMode::AfterCommit);
        assert_eq!(master.get_timeout_ms(), 5000);
        assert_eq!(master.get_wait_count(), 2);
        assert_eq!(master.get_mode(), SemiSyncMode::AfterCommit);
    }

    #[test]
    fn test_semi_sync_slave_new() {
        let slave = SemiSyncSlave::new();
        assert!(!slave.is_enabled());
        assert!(slave.get_master_uuid().is_none());
        assert!(!slave.is_ack_sent());
    }

    #[test]
    fn test_semi_sync_slave_enable_disable() {
        let slave = SemiSyncSlave::new();
        assert!(!slave.is_enabled());

        slave.enable();
        assert!(slave.is_enabled());

        slave.disable();
        assert!(!slave.is_enabled());
    }

    #[test]
    fn test_semi_sync_slave_master_uuid() {
        let slave = SemiSyncSlave::new();
        assert!(slave.get_master_uuid().is_none());

        slave.set_master_uuid(Some("uuid-123".to_string()));
        assert_eq!(slave.get_master_uuid(), Some("uuid-123".to_string()));

        slave.set_master_uuid(None);
        assert!(slave.get_master_uuid().is_none());
    }

    #[test]
    fn test_semi_sync_slave_ack() {
        let slave = SemiSyncSlave::new();
        assert!(!slave.is_ack_sent());

        slave.mark_ack_sent();
        assert!(slave.is_ack_sent());

        slave.reset_ack_sent();
        assert!(!slave.is_ack_sent());
    }

    #[test]
    fn test_semi_sync_slave_status() {
        let slave = SemiSyncSlave::new();
        let status = slave.get_status();
        assert!(!status.rpl_semi_sync_slave_status);

        slave.enable();
        slave.set_master_uuid(Some("uuid-456".to_string()));
        slave.mark_ack_sent();

        let status = slave.get_status();
        assert!(status.rpl_semi_sync_slave_status);
        assert_eq!(status.master_uuid, Some("uuid-456".to_string()));
        assert!(status.last_ack_time.is_some());
    }

    #[tokio::test]
    async fn test_semi_sync_master_wait_for_acks_disabled() {
        let master = SemiSyncMaster::new();
        // When disabled, wait_for_acks should return immediately
        let result = master.wait_for_acks(1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_semi_sync_master_wait_for_acks_success() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.set_wait_count(2);

        // Simulate receiving ACKs
        master.increment_ack();
        master.increment_ack();

        let result = master.wait_for_acks(2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_semi_sync_master_wait_for_acks_timeout() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.set_timeout_ms(50); // 50ms timeout
        master.set_wait_count(3); // Wait for 3, but only 0 received

        let result = master.wait_for_acks(3).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_semi_sync_timeout_error_display() {
        let err = SemiSyncTimeoutError {
            required: 3,
            received: 1,
            timeout_ms: 10000,
        };
        let display = format!("{}", err);
        assert!(display.contains("required 3"));
        assert!(display.contains("received 1"));
        assert!(display.contains("10000"));
    }

    #[test]
    fn test_semi_sync_error_display() {
        let err = SemiSyncError::NoSlaves;
        assert_eq!(format!("{}", err), "No semi-sync slaves available");

        let err = SemiSyncError::InvalidConfig("bad value".to_string());
        assert_eq!(format!("{}", err), "Invalid config: bad value");

        let timeout_err = SemiSyncTimeoutError {
            required: 2,
            received: 1,
            timeout_ms: 5000,
        };
        let err = SemiSyncError::Timeout(timeout_err);
        assert!(format!("{}", err).contains("Timeout"));
    }

    #[test]
    fn test_semi_sync_master_record_transactions() {
        let master = SemiSyncMaster::new();

        master.record_yes_transaction();
        master.record_yes_transaction();
        master.record_no_transaction();

        let status = master.get_status();
        assert_eq!(status.rpl_semi_sync_master_yes_transactions, 2);
        assert_eq!(status.rpl_semi_sync_master_no_transactions, 1);
    }

    #[test]
    fn test_semi_sync_master_default() {
        let master = SemiSyncMaster::default();
        assert!(!master.is_enabled());
        assert_eq!(master.get_mode(), SemiSyncMode::AfterSync);
    }

    #[test]
    fn test_semi_sync_slave_default() {
        let slave = SemiSyncSlave::default();
        assert!(!slave.is_enabled());
    }

    #[test]
    fn test_semi_sync_master_set_client_count() {
        let master = SemiSyncMaster::new();
        assert_eq!(master.get_status().rpl_semi_sync_master_clients, 0);

        master.set_client_count(5);
        assert_eq!(master.get_status().rpl_semi_sync_master_clients, 5);
    }

    #[test]
    fn test_semi_sync_mode_clone() {
        let mode = SemiSyncMode::AfterSync;
        let cloned = mode;
        assert_eq!(cloned, SemiSyncMode::AfterSync);

        let mode2 = SemiSyncMode::AfterCommit;
        let cloned2 = mode2;
        assert_eq!(cloned2, SemiSyncMode::AfterCommit);
    }

    #[test]
    fn test_semi_sync_master_status_clone() {
        let master = SemiSyncMaster::new();
        master.enable();
        master.record_yes_transaction();
        master.record_no_transaction();

        let status = master.get_status();
        let cloned = status.clone();

        assert_eq!(
            cloned.rpl_semi_sync_master_yes_transactions,
            status.rpl_semi_sync_master_yes_transactions
        );
        assert_eq!(
            cloned.rpl_semi_sync_master_no_transactions,
            status.rpl_semi_sync_master_no_transactions
        );
    }
}
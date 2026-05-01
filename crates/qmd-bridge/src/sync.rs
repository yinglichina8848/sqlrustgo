//! Synchronization management between SQLRustGo and QMD

use crate::config::QmdConfig;
use crate::error::QmdResult;
use crate::types::{SyncState, SyncStatus};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Synchronization manager for QMD data sync
pub struct SyncManager {
    #[allow(dead_code)]
    config: QmdConfig,
    state: Arc<Mutex<SyncStateInner>>,
}

/// Internal sync state
struct SyncStateInner {
    state: SyncState,
    last_sync: Option<Instant>,
    items_synced: u64,
    error: Option<String>,
}

impl SyncManager {
    /// Create a new sync manager
    pub fn new(config: QmdConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(SyncStateInner {
                state: SyncState::Idle,
                last_sync: None,
                items_synced: 0,
                error: None,
            })),
        }
    }

    /// Get current sync status
    pub fn status(&self) -> QmdResult<SyncStatus> {
        let state = self.state.lock().map_err(|e| {
            crate::error::QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        Ok(SyncStatus {
            last_sync: state
                .last_sync
                .map(|i| i.elapsed().as_secs() as i64)
                .unwrap_or(0),
            items_synced: state.items_synced,
            state: state.state,
            error: state.error.clone(),
        })
    }

    /// Start a sync operation
    pub fn start_sync(&self) -> QmdResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            crate::error::QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        if state.state == SyncState::Syncing {
            return Err(crate::error::QmdBridgeError::Sync(
                "Sync already in progress".to_string(),
            ));
        }

        state.state = SyncState::Syncing;
        state.error = None;
        tracing::info!("Sync started");

        Ok(())
    }

    /// Complete sync operation
    pub fn complete_sync(&self, items_synced: u64) -> QmdResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            crate::error::QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        state.state = SyncState::Completed;
        state.last_sync = Some(Instant::now());
        state.items_synced = items_synced;
        tracing::info!(items = items_synced, "Sync completed");

        Ok(())
    }

    /// Fail sync operation
    pub fn fail_sync(&self, error: String) -> QmdResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            crate::error::QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        state.state = SyncState::Failed;
        state.error = Some(error.clone());
        tracing::error!(error = %error, "Sync failed");

        Ok(())
    }

    /// Reset sync state to idle
    pub fn reset(&self) -> QmdResult<()> {
        let mut state = self.state.lock().map_err(|e| {
            crate::error::QmdBridgeError::Sync(format!("Failed to acquire lock: {}", e))
        })?;

        state.state = SyncState::Idle;
        state.error = None;
        Ok(())
    }

    /// Check if sync is needed based on time elapsed
    pub fn is_sync_needed(&self, interval_secs: u64) -> bool {
        let state = self.state.lock().ok();

        if let Some(state) = state {
            if state.state == SyncState::Syncing {
                return false;
            }

            if let Some(last_sync) = state.last_sync {
                return last_sync.elapsed() > Duration::from_secs(interval_secs);
            }

            true // Never synced
        } else {
            true
        }
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new(QmdConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_manager_status() {
        let manager = SyncManager::default();

        let status = manager.status().unwrap();
        assert_eq!(status.state, SyncState::Idle);
    }

    #[test]
    fn test_sync_lifecycle() {
        let manager = SyncManager::default();

        // Start sync
        manager.start_sync().unwrap();
        assert_eq!(manager.status().unwrap().state, SyncState::Syncing);

        // Complete sync
        manager.complete_sync(100).unwrap();
        let status = manager.status().unwrap();
        assert_eq!(status.state, SyncState::Completed);
        assert_eq!(status.items_synced, 100);
    }

    #[test]
    fn test_sync_failure() {
        let manager = SyncManager::default();

        manager.start_sync().unwrap();
        manager.fail_sync("Connection failed".to_string()).unwrap();

        let status = manager.status().unwrap();
        assert_eq!(status.state, SyncState::Failed);
        assert_eq!(status.error.as_deref(), Some("Connection failed"));
    }

    #[test]
    fn test_is_sync_needed() {
        let manager = SyncManager::default();

        // Never synced, should need sync
        assert!(manager.is_sync_needed(60));

        // After successful sync, should not need immediate sync
        manager.start_sync().unwrap();
        manager.complete_sync(10).unwrap();
        assert!(!manager.is_sync_needed(3600)); // 1 hour interval
    }
}

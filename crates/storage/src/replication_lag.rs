//! Replication Lag Monitor
//!
//! Monitors replication lag between master and slave nodes.
//! Alerts when lag exceeds configured threshold.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ReplicationLagMonitor {
    master_lsn: Arc<AtomicU64>,
    master_timestamp: Arc<AtomicU64>,
    slave_applied_lsn: Arc<AtomicU64>,
    slave_applied_timestamp: Arc<AtomicU64>,
    lag_threshold_ms: u64,
    last_warning_time: Arc<AtomicU64>,
    warning_interval_ms: u64,
}

impl ReplicationLagMonitor {
    pub fn new(lag_threshold_ms: u64) -> Self {
        Self {
            master_lsn: Arc::new(AtomicU64::new(0)),
            master_timestamp: Arc::new(AtomicU64::new(0)),
            slave_applied_lsn: Arc::new(AtomicU64::new(0)),
            slave_applied_timestamp: Arc::new(AtomicU64::new(0)),
            lag_threshold_ms,
            last_warning_time: Arc::new(AtomicU64::new(0)),
            warning_interval_ms: 60_000,
        }
    }

    pub fn with_warning_interval(mut self, interval_ms: u64) -> Self {
        self.warning_interval_ms = interval_ms;
        self
    }

    pub fn update_master_info(&self, lsn: u64, timestamp_ms: u64) {
        self.master_lsn.store(lsn, Ordering::SeqCst);
        self.master_timestamp.store(timestamp_ms, Ordering::SeqCst);
    }

    pub fn report_applied(&self, lsn: u64) {
        let now = current_timestamp_ms();
        self.slave_applied_lsn.store(lsn, Ordering::SeqCst);
        self.slave_applied_timestamp.store(now, Ordering::SeqCst);
    }

    pub fn current_lag_ms(&self) -> u64 {
        let master_ts = self.master_timestamp.load(Ordering::SeqCst);
        let slave_ts = self.slave_applied_timestamp.load(Ordering::SeqCst);

        if master_ts == 0 || slave_ts == 0 {
            return 0;
        }

        master_ts.saturating_sub(slave_ts)
    }

    pub fn current_lag_events(&self) -> u64 {
        let master_lsn = self.master_lsn.load(Ordering::SeqCst);
        let slave_lsn = self.slave_applied_lsn.load(Ordering::SeqCst);

        master_lsn.saturating_sub(slave_lsn)
    }

    pub fn is_lag_exceeding_threshold(&self) -> bool {
        self.current_lag_ms() > self.lag_threshold_ms
    }

    pub fn should_warn(&self) -> bool {
        let now = current_timestamp_ms();
        let last_warning = self.last_warning_time.load(Ordering::SeqCst);

        if now.saturating_sub(last_warning) >= self.warning_interval_ms {
            self.last_warning_time.store(now, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn check_and_warn(&self) -> Option<LagWarning> {
        if self.is_lag_exceeding_threshold() && self.should_warn() {
            Some(LagWarning {
                lag_ms: self.current_lag_ms(),
                lag_events: self.current_lag_events(),
                threshold_ms: self.lag_threshold_ms,
            })
        } else {
            None
        }
    }

    pub fn lag_threshold_ms(&self) -> u64 {
        self.lag_threshold_ms
    }

    pub fn reset(&self) {
        self.master_lsn.store(0, Ordering::SeqCst);
        self.master_timestamp.store(0, Ordering::SeqCst);
        self.slave_applied_lsn.store(0, Ordering::SeqCst);
        self.slave_applied_timestamp.store(0, Ordering::SeqCst);
    }
}

impl Default for ReplicationLagMonitor {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Clone for ReplicationLagMonitor {
    fn clone(&self) -> Self {
        Self {
            master_lsn: self.master_lsn.clone(),
            master_timestamp: self.master_timestamp.clone(),
            slave_applied_lsn: self.slave_applied_lsn.clone(),
            slave_applied_timestamp: self.slave_applied_timestamp.clone(),
            lag_threshold_ms: self.lag_threshold_ms,
            last_warning_time: self.last_warning_time.clone(),
            warning_interval_ms: self.warning_interval_ms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LagWarning {
    pub lag_ms: u64,
    pub lag_events: u64,
    pub threshold_ms: u64,
}

impl LagWarning {
    pub fn message(&self) -> String {
        format!(
            "Replication lag warning: {}ms ({} events) exceeds threshold {}ms",
            self.lag_ms, self.lag_events, self.threshold_ms
        )
    }
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub struct ReplicationStatus {
    pub master_lsn: u64,
    pub slave_applied_lsn: u64,
    pub lag_ms: u64,
    pub lag_events: u64,
    pub is_healthy: bool,
}

impl ReplicationLagMonitor {
    pub fn status(&self) -> ReplicationStatus {
        let master_lsn = self.master_lsn.load(Ordering::SeqCst);
        let slave_applied_lsn = self.slave_applied_lsn.load(Ordering::SeqCst);
        let lag_ms = self.current_lag_ms();
        let lag_events = self.current_lag_events();

        ReplicationStatus {
            master_lsn,
            slave_applied_lsn,
            lag_ms,
            lag_events,
            is_healthy: !self.is_lag_exceeding_threshold(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lag_monitor_creation() {
        let monitor = ReplicationLagMonitor::new(5000);
        assert_eq!(monitor.lag_threshold_ms(), 5000);
        assert_eq!(monitor.current_lag_ms(), 0);
    }

    #[test]
    fn test_lag_calculation() {
        let monitor = ReplicationLagMonitor::new(1000);

        monitor.update_master_info(100, current_timestamp_ms());

        std::thread::sleep(Duration::from_millis(10));

        monitor.report_applied(50);

        let lag = monitor.current_lag_ms();
        assert!(lag >= 0);
        assert_eq!(monitor.current_lag_events(), 50);
    }

    #[test]
    fn test_threshold_detection() {
        let monitor = ReplicationLagMonitor::new(100);

        monitor.update_master_info(1000, current_timestamp_ms());
        monitor.report_applied(0);

        assert!(monitor.is_lag_exceeding_threshold());
    }

    #[test]
    fn test_no_lag() {
        let monitor = ReplicationLagMonitor::new(1000);

        let now = current_timestamp_ms();
        monitor.update_master_info(100, now);
        monitor.report_applied(100);

        assert!(!monitor.is_lag_exceeding_threshold());
    }

    #[test]
    fn test_warning_rate_limiting() {
        let monitor = ReplicationLagMonitor::new(1).with_warning_interval(1000);

        monitor.update_master_info(100, current_timestamp_ms());
        monitor.report_applied(0);

        assert!(monitor.should_warn());
        assert!(!monitor.should_warn());
    }

    #[test]
    fn test_lag_warning_message() {
        let warning = LagWarning {
            lag_ms: 5000,
            lag_events: 100,
            threshold_ms: 1000,
        };

        let msg = warning.message();
        assert!(msg.contains("5000ms"));
        assert!(msg.contains("100 events"));
        assert!(msg.contains("1000ms"));
    }
}

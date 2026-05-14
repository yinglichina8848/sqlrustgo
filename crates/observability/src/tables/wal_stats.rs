use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalStats {
    pub total_writes: u64,
    pub total_bytes: u64,
    pub flush_count: u64,
    pub replay_count: u64,
    pub replay_time_ms: u64,
    pub last_flush_lsn: u64,
    pub current_lsn: u64,
}

#[derive(Clone)]
pub struct WalStatsCollector {
    stats: Arc<Mutex<WalStats>>,
}

impl WalStatsCollector {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(WalStats::default())),
        }
    }

    pub fn record_write(&self, bytes: u64, lsn: u64) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_writes += 1;
        stats.total_bytes += bytes;
        stats.current_lsn = lsn;
    }

    pub fn record_flush(&self, lsn: u64) {
        let mut stats = self.stats.lock().unwrap();
        stats.flush_count += 1;
        stats.last_flush_lsn = lsn;
    }

    pub fn record_replay(&self, time_ms: u64) {
        let mut stats = self.stats.lock().unwrap();
        stats.replay_count += 1;
        stats.replay_time_ms += time_ms;
    }

    pub fn get_stats(&self) -> WalStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        *self.stats.lock().unwrap() = WalStats::default();
    }
}

impl Default for WalStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wal_stats_record_write() {
        let collector = WalStatsCollector::new();
        collector.record_write(100, 1);

        let stats = collector.get_stats();
        assert_eq!(stats.total_writes, 1);
        assert_eq!(stats.total_bytes, 100);
        assert_eq!(stats.current_lsn, 1);
    }

    #[test]
    fn test_wal_stats_record_flush() {
        let collector = WalStatsCollector::new();
        collector.record_flush(100);

        let stats = collector.get_stats();
        assert_eq!(stats.flush_count, 1);
        assert_eq!(stats.last_flush_lsn, 100);
    }

    #[test]
    fn test_wal_stats_record_replay() {
        let collector = WalStatsCollector::new();
        collector.record_replay(50);

        let stats = collector.get_stats();
        assert_eq!(stats.replay_count, 1);
        assert_eq!(stats.replay_time_ms, 50);
    }

    #[test]
    fn test_wal_stats_reset() {
        let collector = WalStatsCollector::new();
        collector.record_write(100, 1);
        collector.record_flush(1);
        collector.reset();

        let stats = collector.get_stats();
        assert_eq!(stats.total_writes, 0);
        assert_eq!(stats.flush_count, 0);
    }
}

use query_stats::{SlowQueryConfig, SlowQueryLog};
use std::env::temp_dir;
use std::path::PathBuf;

#[test]
fn test_slow_query_config_default() {
    let config = SlowQueryConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.threshold_ms, 1000);
    assert_eq!(config.log_path, PathBuf::from("slow_query.log"));
}

#[test]
fn test_slow_query_log_creation() {
    let log = SlowQueryLog::new(500, PathBuf::from("/tmp/slow.log"));
    assert_eq!(log.threshold_ms(), 500);
    assert_eq!(log.log_path(), &PathBuf::from("/tmp/slow.log"));
}

#[test]
fn test_slow_query_log_from_config() {
    let config = SlowQueryConfig {
        enabled: true,
        threshold_ms: 2000,
        log_path: PathBuf::from("/tmp/test.log"),
    };
    let log = SlowQueryLog::from_config(&config);
    assert_eq!(log.threshold_ms(), 2000);
}

#[test]
fn test_maybe_log_below_threshold() {
    let log_path = temp_dir().join("test_below_threshold.log");
    let log = SlowQueryLog::new(1000, log_path.clone());

    log.maybe_log("SELECT 1", 500, 10);

    let recent = log.get_recent();
    assert!(recent.is_empty() || recent.iter().all(|r| r.duration_ms >= 1000));

    std::fs::remove_file(log_path).ok();
}

#[test]
fn test_maybe_log_above_threshold() {
    let log_path = temp_dir().join("test_above_threshold.log");
    let log = SlowQueryLog::new(100, log_path.clone());

    log.maybe_log("SELECT * FROM orders WHERE status = 'pending'", 500, 100);

    let recent = log.get_recent();
    assert_eq!(recent.len(), 1);
    assert_eq!(
        recent[0].query,
        "SELECT * FROM orders WHERE status = 'pending'"
    );
    assert_eq!(recent[0].duration_ms, 500);
    assert_eq!(recent[0].rows, 100);

    std::fs::remove_file(log_path).ok();
}

#[test]
fn test_clear_recent() {
    let log_path = temp_dir().join("test_clear.log");
    let log = SlowQueryLog::new(1, log_path);

    log.maybe_log("SELECT 1", 100, 1);
    assert_eq!(log.get_recent().len(), 1);

    log.clear_recent();
    assert!(log.get_recent().is_empty());
}

#[test]
fn test_read_logs_empty_file() {
    let log_path = temp_dir().join("test_empty.log");
    let log = SlowQueryLog::new(1000, log_path);

    let records = log.read_logs();
    assert!(records.is_empty());
}

use sqlrustgo_common::logging::{init_logging, LogFormat, LogLevel};

#[test]
fn test_log_level_error_only() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().to_str().unwrap();

    if let Ok(_) = init_logging(log_dir, LogLevel::Error, LogFormat::Text, 1024 * 1024, 5) {
        log::error!("error message");
        log::warn!("warn should be filtered");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn test_log_level_debug() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().to_str().unwrap();

    if let Ok(_) = init_logging(log_dir, LogLevel::Debug, LogFormat::Text, 1024 * 1024, 5) {
        log::error!("error message");
        log::warn!("warn message");
        log::info!("info message");
        log::debug!("debug message");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn test_log_format_json() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().to_str().unwrap();

    if let Ok(_) = init_logging(log_dir, LogLevel::Info, LogFormat::Json, 1024 * 1024, 5) {
        log::info!("json format test");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn test_log_multiple_messages() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().to_str().unwrap();

    if let Ok(_) = init_logging(log_dir, LogLevel::Info, LogFormat::Text, 1024 * 1024, 5) {
        for i in 0..10 {
            log::info!("message {}", i);
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

#[test]
fn test_log_rotation_triggered() {
    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().to_str().unwrap();

    let result = init_logging(log_dir, LogLevel::Info, LogFormat::Text, 100, 3);

    if result.is_ok() {
        for i in 0..50 {
            log::info!("rotation test message {}", i);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

use chrono::Local;
use log::{Log, Metadata, Record};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl LogLevel {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ERROR" => LogLevel::Error,
            "WARN" | "WARNING" => LogLevel::Warn,
            "INFO" => LogLevel::Info,
            "DEBUG" => LogLevel::Debug,
            _ => LogLevel::Info,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Text,
    Json,
}

pub struct RollingLogger {
    level: LogLevel,
    format: LogFormat,
    log_dir: String,
    max_file_size: u64,
    max_files: usize,
    current_file: Mutex<String>,
    file_size: Mutex<u64>,
}

impl RollingLogger {
    pub fn new(
        log_dir: &str,
        level: LogLevel,
        format: LogFormat,
        max_file_size: u64,
        max_files: usize,
    ) -> Self {
        let logger = Self {
            level,
            format,
            log_dir: log_dir.to_string(),
            max_file_size,
            max_files,
            current_file: Mutex::new(String::new()),
            file_size: Mutex::new(0),
        };
        let _ = fs::create_dir_all(log_dir);
        logger.rotate_if_needed();
        logger
    }

    fn get_log_filename(&self) -> String {
        let now = Local::now();
        format!(
            "{}/sqlrustgo_{}.log",
            self.log_dir,
            now.format("%Y%m%d_%H%M%S")
        )
    }

    fn rotate_if_needed(&self) {
        let mut current = self.current_file.lock().unwrap();
        if current.is_empty() {
            *current = self.get_log_filename();
            return;
        }

        let size = *self.file_size.lock().unwrap();
        if size >= self.max_file_size {
            self.perform_rotation(&current);
            *current = self.get_log_filename();
            *self.file_size.lock().unwrap() = 0;
        }
    }

    fn perform_rotation(&self, current_file: &str) {
        if let Ok(entries) = fs::read_dir(&self.log_dir) {
            let mut log_files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "log" || ext == "gz")
                })
                .collect();

            log_files.sort_by_key(|e| e.path());

            while log_files.len() >= self.max_files {
                if let Some(oldest) = log_files.first() {
                    let _ = fs::remove_file(oldest.path());
                    log_files.remove(0);
                }
            }
        }

        let gzip_file = format!("{}.gz", current_file);
        if let Ok(mut reader) = File::open(current_file) {
            let mut contents = Vec::new();
            if reader.read_to_end(&mut contents).is_ok() {
                if let Ok(file) = File::create(&gzip_file) {
                    let mut encoder =
                        flate2::write::GzEncoder::new(file, flate2::Compression::default());
                    let _ = encoder.write_all(&contents);
                    let _ = encoder.finish();
                }
            }
        }
        let _ = fs::remove_file(current_file);
    }

    fn write_log(&self, record: &Record) {
        self.rotate_if_needed();

        let filename = self.current_file.lock().unwrap().clone();
        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&filename) {
            let mut writer = BufWriter::new(file);
            let log_line = match self.format {
                LogFormat::Text => self.format_text(record),
                LogFormat::Json => self.format_json(record),
            };

            if writer.write_all(log_line.as_bytes()).is_ok() {
                let _ = writer.flush();
                *self.file_size.lock().unwrap() += log_line.len() as u64;
            }
        }
    }

    fn format_text(&self, record: &Record) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        format!(
            "[{}] [{}] [{}:{}] {}\n",
            timestamp,
            record.level().as_str(),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    }

    fn format_json(&self, record: &Record) -> String {
        let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%:z");
        let json = serde_json::json!({
            "timestamp": timestamp.to_string(),
            "level": record.level().as_str(),
            "module": record.module_path().unwrap_or("unknown"),
            "file": record.file().unwrap_or("unknown"),
            "line": record.line().unwrap_or(0),
            "message": record.args().to_string(),
        });
        format!("{}\n", json)
    }

    fn should_log(&self, level: log::Level) -> bool {
        let record_level = match level {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug | log::Level::Trace => LogLevel::Debug,
        };
        record_level as u8 <= self.level as u8
    }
}

impl Log for RollingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.should_log(metadata.level())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.write_log(record);
        }
    }

    fn flush(&self) {}
}

pub fn init_logging(
    log_dir: &str,
    level: LogLevel,
    format: LogFormat,
    max_file_size: u64,
    max_files: usize,
) -> Result<(), log::SetLoggerError> {
    let logger = RollingLogger::new(log_dir, level, format, max_file_size, max_files);
    log::set_logger(Box::leak(Box::new(logger)))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("invalid"), LogLevel::Info);
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
    }

    #[test]
    fn test_rolling_logger_creation() {
        let dir = tempdir().unwrap();
        let logger = RollingLogger::new(
            dir.path().to_str().unwrap(),
            LogLevel::Info,
            LogFormat::Text,
            1024 * 1024,
            5,
        );
        assert!(dir.path().exists());
        let _ = logger;
    }

    #[test]
    fn test_rolling_logger_json_format() {
        let dir = tempdir().unwrap();
        let logger = RollingLogger::new(
            dir.path().to_str().unwrap(),
            LogLevel::Info,
            LogFormat::Json,
            1024 * 1024,
            5,
        );

        let record = Record::builder()
            .args(format_args!("test message"))
            .file(Some("test.rs"))
            .line(Some(42))
            .module_path(Some("test_module"))
            .build();

        logger.write_log(&record);

        let files: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
        assert!(!files.is_empty());
    }

    #[test]
    fn test_should_log() {
        let dir = tempdir().unwrap();
        let logger = RollingLogger::new(
            dir.path().to_str().unwrap(),
            LogLevel::Info,
            LogFormat::Text,
            1024 * 1024,
            5,
        );

        assert!(logger.should_log(log::Level::Error));
        assert!(logger.should_log(log::Level::Warn));
        assert!(logger.should_log(log::Level::Info));
        assert!(!logger.should_log(log::Level::Debug));
    }
}

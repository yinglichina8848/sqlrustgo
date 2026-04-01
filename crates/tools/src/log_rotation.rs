//! Log Rotation
//!
//! Provides log rotation with:
//! - Size-based rotation
//! - Time-based rotation (daily, hourly)
//! - Retention policies
//! - Compression of old logs

use anyhow::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use structopt::StructOpt;

#[derive(Debug, Clone)]
pub enum RotationStrategy {
    Size(u64),
    Time(TimeRotation),
    Both { size: u64, time: TimeRotation },
}

#[derive(Debug, Clone)]
pub enum TimeRotation {
    Hourly,
    Daily,
    Weekly,
}

#[derive(Debug, Clone)]
pub struct LogRotationConfig {
    pub strategy: RotationStrategy,
    pub max_files: usize,
    pub compress: bool,
    pub directory: PathBuf,
    pub prefix: String,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            strategy: RotationStrategy::Size(100 * 1024 * 1024),
            max_files: 10,
            compress: true,
            directory: PathBuf::from("logs"),
            prefix: "sqlrustgo".to_string(),
        }
    }
}

pub struct LogRotator {
    config: LogRotationConfig,
    current_size: u64,
    last_rotation: SystemTime,
    current_file: Option<PathBuf>,
}

impl LogRotator {
    pub fn new(config: LogRotationConfig) -> Result<Self> {
        let rotator = Self {
            config: config.clone(),
            current_size: 0,
            last_rotation: SystemTime::now(),
            current_file: None,
        };

        fs::create_dir_all(&config.directory)?;
        Ok(rotator)
    }

    pub fn get_current_log_path(&self) -> PathBuf {
        self.current_file
            .clone()
            .unwrap_or_else(|| self.generate_log_path())
    }

    fn generate_log_path(&self) -> PathBuf {
        let timestamp = chrono_lite_timestamp();
        self.config
            .directory
            .join(format!("{}_{}.log", self.config.prefix, timestamp))
    }

    pub fn should_rotate(&self) -> bool {
        if let Some(ref path) = self.current_file {
            if !path.exists() {
                return true;
            }
        }

        match &self.config.strategy {
            RotationStrategy::Size(max_size) => self.current_size >= *max_size,
            RotationStrategy::Time(time_rot) => self.check_time_rotation(time_rot),
            RotationStrategy::Both { size, time } => {
                self.current_size >= *size || self.check_time_rotation(time)
            }
        }
    }

    fn check_time_rotation(&self, time: &TimeRotation) -> bool {
        let elapsed = SystemTime::now()
            .duration_since(self.last_rotation)
            .unwrap_or(Duration::ZERO);

        match time {
            TimeRotation::Hourly => elapsed >= Duration::from_secs(3600),
            TimeRotation::Daily => elapsed >= Duration::from_secs(86400),
            TimeRotation::Weekly => elapsed >= Duration::from_secs(604800),
        }
    }

    pub fn rotate(&mut self) -> Result<Option<PathBuf>> {
        if !self.should_rotate() {
            return Ok(None);
        }

        let old_file = self.current_file.take();

        if let Some(ref path) = old_file {
            if path.exists() {
                if self.config.compress {
                    self.compress_log(path)?;
                }
                self.enforce_retention()?;
            }
        }

        let new_path = self.generate_log_path();
        File::create(&new_path)?;
        self.current_file = Some(new_path.clone());
        self.current_size = 0;
        self.last_rotation = SystemTime::now();

        println!("Rotated log to: {}", new_path.display());
        Ok(Some(new_path))
    }

    fn compress_log(&self, path: &Path) -> Result<()> {
        let compressed_path = path.with_extension("log.gz");
        let input = File::open(path)?;
        let output = File::create(&compressed_path)?;
        let mut encoder = GzEncoder::new(output, Compression::default());

        let mut reader = BufReader::new(input);
        io::copy(&mut reader, &mut encoder)?;
        encoder.finish()?;

        fs::remove_file(path)?;
        println!(
            "Compressed: {} -> {}",
            path.display(),
            compressed_path.display()
        );
        Ok(())
    }

    pub fn decompress_log(&self, path: &Path) -> Result<PathBuf> {
        let output_path = path.with_extension("decompressed.log");
        let input = File::open(path)?;
        let mut output = File::create(&output_path)?;
        let mut decoder = GzDecoder::new(input);
        io::copy(&mut decoder, &mut output)?;
        Ok(output_path)
    }

    fn enforce_retention(&self) -> Result<()> {
        let mut files = self.list_log_files()?;

        if files.len() >= self.config.max_files {
            let to_delete = files.len() - self.config.max_files;
            for _ in 0..to_delete {
                if let Some(oldest) = files.pop() {
                    println!("Removing old log: {}", oldest.display());
                    fs::remove_file(oldest).ok();
                }
            }
        }
        Ok(())
    }

    pub fn list_log_files(&self) -> Result<Vec<PathBuf>> {
        let mut files: Vec<PathBuf> = Vec::new();

        if !self.config.directory.exists() {
            return Ok(files);
        }

        for entry in fs::read_dir(&self.config.directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if filename.starts_with(&self.config.prefix)
                    && (filename.ends_with(".log") || filename.ends_with(".log.gz"))
                {
                    files.push(path);
                }
            }
        }

        files.sort_by_key(|p| {
            p.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });

        Ok(files)
    }

    pub fn write(&mut self, message: &str) -> Result<()> {
        if self.should_rotate() {
            self.rotate()?;
        }

        let path = self.get_current_log_path();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        file.write_all(message.as_bytes())?;
        self.current_size += message.len() as u64;
        Ok(())
    }

    pub fn read_logs(&self, count: usize) -> Result<Vec<String>> {
        let path = self.get_current_log_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        Ok(lines.into_iter().rev().take(count).rev().collect())
    }

    pub fn get_total_size(&self) -> Result<u64> {
        let files = self.list_log_files()?;
        let mut total: u64 = 0;

        for path in files {
            if let Ok(meta) = fs::metadata(&path) {
                total += meta.len();
            }
        }

        Ok(total)
    }
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    let mut year = 1970;
    let mut remaining_days = days as i64;

    while remaining_days >= 365 {
        let leap_years = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days >= leap_years {
            remaining_days -= leap_years;
            year += 1;
        } else {
            break;
        }
    }

    let days_per_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

    let mut month = 1;
    for (i, days_in_month) in days_per_month.iter().enumerate() {
        let actual_days = if is_leap && i == 1 {
            29
        } else {
            *days_in_month
        };
        if remaining_days < actual_days as i64 {
            break;
        }
        remaining_days -= actual_days as i64;
        month = i + 2;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    )
}

pub fn create_rotating_logger(config: LogRotationConfig) -> Result<LogRotator> {
    LogRotator::new(config)
}

#[derive(Debug, StructOpt)]
pub struct LogRotationCommand {
    #[structopt(subcommand)]
    pub action: LogRotationAction,
}

#[derive(Debug, StructOpt)]
pub enum LogRotationAction {
    /// Rotate logs now
    Rotate {
        /// Log directory
        #[structopt(short = "d", long = "dir", default_value = "logs")]
        directory: PathBuf,
        /// Max file size in MB
        #[structopt(short = "s", long = "size", default_value = "100")]
        max_size_mb: u64,
        /// Max number of files to retain
        #[structopt(short = "m", long = "max-files", default_value = "10")]
        max_files: usize,
    },
    /// List log files
    List {
        /// Log directory
        #[structopt(short = "d", long = "dir", default_value = "logs")]
        directory: PathBuf,
    },
    /// Show total log size
    Size {
        /// Log directory
        #[structopt(short = "d", long = "dir", default_value = "logs")]
        directory: PathBuf,
    },
    /// Compress old log files
    Compress {
        /// Log directory
        #[structopt(short = "d", long = "dir", default_value = "logs")]
        directory: PathBuf,
    },
    /// Clean old logs beyond retention
    Clean {
        /// Log directory
        #[structopt(short = "d", long = "dir", default_value = "logs")]
        directory: PathBuf,
        /// Maximum files to retain
        #[structopt(short = "m", long = "max-files", default_value = "10")]
        max_files: usize,
    },
}

pub fn run_log_rotation_cmd(cmd: LogRotationCommand) -> Result<()> {
    match cmd.action {
        LogRotationAction::Rotate {
            directory,
            max_size_mb,
            max_files,
        } => {
            let config = LogRotationConfig {
                strategy: RotationStrategy::Size(max_size_mb * 1024 * 1024),
                max_files,
                compress: true,
                directory,
                prefix: "sqlrustgo".to_string(),
            };
            let mut rotator = LogRotator::new(config)?;
            if let Some(path) = rotator.rotate()? {
                println!("Rotated to: {}", path.display());
            } else {
                println!("No rotation needed");
            }
        }
        LogRotationAction::List { directory } => {
            let config = LogRotationConfig {
                strategy: RotationStrategy::Time(TimeRotation::Daily),
                max_files: 10,
                compress: true,
                directory,
                prefix: "sqlrustgo".to_string(),
            };
            let rotator = LogRotator::new(config)?;
            let files = rotator.list_log_files()?;
            if files.is_empty() {
                println!("No log files found");
            } else {
                for f in &files {
                    if let Ok(meta) = fs::metadata(f) {
                        let size_kb = meta.len() / 1024;
                        println!("{} ({} KB)", f.display(), size_kb);
                    }
                }
            }
        }
        LogRotationAction::Size { directory } => {
            let config = LogRotationConfig {
                strategy: RotationStrategy::Time(TimeRotation::Daily),
                max_files: 10,
                compress: true,
                directory,
                prefix: "sqlrustgo".to_string(),
            };
            let rotator = LogRotator::new(config)?;
            let total = rotator.get_total_size()?;
            println!(
                "Total log size: {} KB ({} MB)",
                total / 1024,
                total / 1024 / 1024
            );
        }
        LogRotationAction::Compress { directory } => {
            let config = LogRotationConfig {
                strategy: RotationStrategy::Time(TimeRotation::Daily),
                max_files: 10,
                compress: true,
                directory,
                prefix: "sqlrustgo".to_string(),
            };
            let rotator = LogRotator::new(config)?;
            let files = rotator.list_log_files()?;
            for f in files {
                if f.extension().and_then(|e| e.to_str()) == Some("log") {
                    rotator.compress_log(&f)?;
                }
            }
        }
        LogRotationAction::Clean {
            directory,
            max_files,
        } => {
            let config = LogRotationConfig {
                strategy: RotationStrategy::Time(TimeRotation::Daily),
                max_files,
                compress: true,
                directory,
                prefix: "sqlrustgo".to_string(),
            };
            let rotator = LogRotator::new(config)?;
            rotator.enforce_retention()?;
            println!("Cleaned old logs");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_log_rotation_config_default() {
        let config = LogRotationConfig::default();
        assert_eq!(config.max_files, 10);
        assert!(config.compress);
    }

    #[test]
    fn test_size_rotation_trigger() {
        let dir = tempdir().unwrap();
        let config = LogRotationConfig {
            strategy: RotationStrategy::Size(100),
            max_files: 3,
            compress: false,
            directory: dir.path().to_path_buf(),
            prefix: "test".to_string(),
        };

        let mut rotator = LogRotator::new(config).unwrap();
        assert!(!rotator.should_rotate());
    }

    #[test]
    fn test_generate_log_path() {
        let dir = tempdir().unwrap();
        let config = LogRotationConfig {
            strategy: RotationStrategy::Time(TimeRotation::Daily),
            max_files: 5,
            compress: false,
            directory: dir.path().to_path_buf(),
            prefix: "app".to_string(),
        };

        let rotator = LogRotator::new(config).unwrap();
        let path = rotator.get_current_log_path();

        assert!(path.to_str().unwrap().starts_with("app_"));
        assert!(path.to_str().unwrap().ends_with(".log"));
    }

    #[test]
    fn test_list_log_files_empty() {
        let dir = tempdir().unwrap();
        let config = LogRotationConfig {
            strategy: RotationStrategy::Time(TimeRotation::Daily),
            max_files: 5,
            compress: false,
            directory: dir.path().to_path_buf(),
            prefix: "test".to_string(),
        };

        let rotator = LogRotator::new(config).unwrap();
        let files = rotator.list_log_files().unwrap();
        assert!(files.is_empty());
    }
}

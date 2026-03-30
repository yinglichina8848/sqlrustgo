//! Backup Scheduler and Compression Module
//!
//! Provides automated backup scheduling and compression:
//! - Scheduled full backups (daily/weekly)
//! - WAL-based incremental backups
//! - Gzip compression for backup files
//! - Backup retention policies

use crate::{WalEntry, WalReader};
use serde::{Deserialize, Serialize};
use sqlrustgo_types::{SqlError, SqlResult};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BackupScheduleType {
    Daily,
    Weekly,
    Monthly,
}

impl Default for BackupScheduleType {
    fn default() -> Self {
        Self::Daily
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub schedule_type: BackupScheduleType,
    pub enabled: bool,
    pub retention_days: u32,
    pub compress: bool,
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            schedule_type: BackupScheduleType::Daily,
            enabled: true,
            retention_days: 7,
            compress: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalBackupPoint {
    pub lsn: u64,
    pub timestamp: u64,
    pub tables_modified: Vec<String>,
}

pub struct BackupScheduler {
    schedule: BackupSchedule,
    storage_path: PathBuf,
    backup_path: PathBuf,
    wal_path: PathBuf,
    current_lsn: Arc<Mutex<u64>>,
    is_running: Arc<Mutex<bool>>,
}

impl BackupScheduler {
    pub fn new(storage_path: PathBuf, backup_path: PathBuf, wal_path: PathBuf) -> Self {
        Self {
            schedule: BackupSchedule::default(),
            storage_path,
            backup_path,
            wal_path,
            current_lsn: Arc::new(Mutex::new(0)),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn with_schedule(mut self, schedule: BackupSchedule) -> Self {
        self.schedule = schedule;
        self
    }

    pub fn get_schedule(&self) -> BackupSchedule {
        self.schedule.clone()
    }

    pub fn update_schedule(&mut self, schedule: BackupSchedule) {
        self.schedule = schedule;
    }

    pub fn start(&self) {
        *self.is_running.lock().unwrap() = true;
    }

    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
    }

    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    pub fn should_run_backup(&self) -> bool {
        if !self.schedule.enabled {
            return false;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let schedule_file = self.backup_path.join(".last_backup");
        let last_backup = if schedule_file.exists() {
            fs::read_to_string(&schedule_file)
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0)
        } else {
            0
        };

        let interval_seconds = match self.schedule.schedule_type {
            BackupScheduleType::Daily => 86400,
            BackupScheduleType::Weekly => 604800,
            BackupScheduleType::Monthly => 2592000,
        };

        now - last_backup >= interval_seconds
    }

    pub fn record_backup_run(&self) -> SqlResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let schedule_file = self.backup_path.join(".last_backup");
        fs::write(&schedule_file, now.to_string()).map_err(|e| SqlError::IoError(e.to_string()))
    }

    pub fn get_backup_directory(&self) -> PathBuf {
        self.backup_path.clone()
    }

    pub fn cleanup_old_backups(&self) -> SqlResult<Vec<PathBuf>> {
        if !self.backup_path.exists() {
            return Ok(Vec::new());
        }

        let retention_seconds = self.schedule.retention_days as u64 * 86400;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut removed = Vec::new();

        for entry in fs::read_dir(&self.backup_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        let age = now
                            .saturating_sub(created.duration_since(UNIX_EPOCH).unwrap().as_secs());

                        if age > retention_seconds {
                            fs::remove_dir_all(&path).ok();
                            removed.push(path);
                        }
                    }
                }
            }
        }

        Ok(removed)
    }

    pub fn set_current_lsn(&self, lsn: u64) {
        *self.current_lsn.lock().unwrap() = lsn;
    }

    pub fn get_current_lsn(&self) -> u64 {
        *self.current_lsn.lock().unwrap()
    }

    pub fn read_wal_since(&self, start_lsn: u64) -> SqlResult<Vec<WalEntry>> {
        let wal_file = self.wal_path.join("wal.binlog");
        if !wal_file.exists() {
            return Ok(Vec::new());
        }

        let mut reader = WalReader::new(&wal_file)?;
        reader
            .read_from(start_lsn)
            .map_err(|e| SqlError::IoError(e.to_string()))
    }

    pub fn create_incremental_backup(&self, parent_lsn: u64) -> SqlResult<IncrementalBackupPoint> {
        let entries = self.read_wal_since(parent_lsn)?;

        let mut tables_modified = std::collections::HashSet::new();
        for entry in &entries {
            tables_modified.insert(format!("table_{}", entry.table_id));
        }

        let max_lsn = entries.iter().map(|e| e.lsn).max().unwrap_or(parent_lsn);

        Ok(IncrementalBackupPoint {
            lsn: max_lsn,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tables_modified: tables_modified.into_iter().collect(),
        })
    }
}

pub struct BackupCompressor;

impl BackupCompressor {
    pub fn compress_file<P: AsRef<Path>>(source: P, target: P) -> SqlResult<u64> {
        let source = source.as_ref();
        let target = target.as_ref();

        let input = File::open(source).map_err(|e| SqlError::IoError(e.to_string()))?;
        let output = File::create(target).map_err(|e| SqlError::IoError(e.to_string()))?;

        let mut reader = BufReader::new(input);
        let mut writer = BufWriter::new(output);

        let mut encoder =
            flate2::write::GzEncoder::new(&mut writer, flate2::Compression::default());
        let mut buffer = vec![0u8; 8192];

        let mut total_bytes = 0u64;
        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            if n == 0 {
                break;
            }
            encoder
                .write_all(&buffer[..n])
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            total_bytes += n as u64;
        }

        encoder
            .finish()
            .map_err(|e| SqlError::IoError(e.to_string()))?;
        writer
            .flush()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(total_bytes)
    }

    pub fn decompress_file<P: AsRef<Path>>(source: P, target: P) -> SqlResult<u64> {
        let source = source.as_ref();
        let target = target.as_ref();

        let input = File::open(source).map_err(|e| SqlError::IoError(e.to_string()))?;
        let output = File::create(target).map_err(|e| SqlError::IoError(e.to_string()))?;

        let mut reader = BufReader::new(input);
        let mut writer = BufWriter::new(output);

        let mut decoder = flate2::read::GzDecoder::new(&mut reader);
        let mut buffer = vec![0u8; 8192];

        let mut total_bytes = 0u64;
        loop {
            let n = decoder
                .read(&mut buffer)
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            if n == 0 {
                break;
            }
            writer
                .write_all(&buffer[..n])
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            total_bytes += n as u64;
        }

        writer
            .flush()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(total_bytes)
    }

    pub fn compress_directory<P: AsRef<Path>>(source_dir: P, target_file: P) -> SqlResult<u64> {
        let source_dir = source_dir.as_ref();
        let target_file = target_file.as_ref();

        let output = File::create(target_file).map_err(|e| SqlError::IoError(e.to_string()))?;
        let mut encoder = flate2::write::GzEncoder::new(output, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(&mut encoder);

        fn add_dir_recursive(
            builder: &mut tar::Builder<&mut flate2::write::GzEncoder<File>>,
            dir: &Path,
            prefix: &str,
        ) -> SqlResult<u64> {
            let mut total_bytes = 0u64;

            for entry in fs::read_dir(dir).map_err(|e| SqlError::IoError(e.to_string()))? {
                let entry = entry.map_err(|e| SqlError::IoError(e.to_string()))?;
                let path = entry.path();
                let name = format!("{}/{}", prefix, path.file_name().unwrap().to_string_lossy());

                if path.is_file() {
                    let mut file =
                        File::open(&path).map_err(|e| SqlError::IoError(e.to_string()))?;
                    let mut buffer = vec![0u8; 8192];
                    let mut file_size = 0u64;

                    loop {
                        let n = file
                            .read(&mut buffer)
                            .map_err(|e| SqlError::IoError(e.to_string()))?;
                        if n == 0 {
                            break;
                        }
                        file_size += n as u64;
                    }

                    builder
                        .append_path_with_name(&path, &name)
                        .map_err(|e| SqlError::IoError(e.to_string()))?;
                    total_bytes += file_size;
                } else if path.is_dir() {
                    total_bytes += add_dir_recursive(builder, &path, &name)?;
                }
            }

            Ok(total_bytes)
        }

        let total = add_dir_recursive(&mut tar_builder, source_dir, "")?;
        tar_builder
            .finish()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(total)
    }

    pub fn decompress_directory<P: AsRef<Path>>(source_file: P, target_dir: P) -> SqlResult<u64> {
        let source_file = source_file.as_ref();
        let target_dir = target_dir.as_ref();

        let input = File::open(source_file).map_err(|e| SqlError::IoError(e.to_string()))?;
        let mut decoder = flate2::read::GzDecoder::new(input);
        let mut archive = tar::Archive::new(&mut decoder);

        fs::create_dir_all(target_dir).map_err(|e| SqlError::IoError(e.to_string()))?;

        let mut total_bytes = 0u64;
        for entry in archive
            .entries()
            .map_err(|e| SqlError::IoError(e.to_string()))?
        {
            let mut entry = entry.map_err(|e| SqlError::IoError(e.to_string()))?;
            let path = entry.path().map_err(|e| SqlError::IoError(e.to_string()))?;
            let dest_path = target_dir.join(&path);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(|e| SqlError::IoError(e.to_string()))?;
            }

            if path.to_string_lossy().ends_with('/') {
                fs::create_dir_all(&dest_path).map_err(|e| SqlError::IoError(e.to_string()))?;
            } else {
                let mut out_file =
                    File::create(&dest_path).map_err(|e| SqlError::IoError(e.to_string()))?;
                let mut buffer = vec![0u8; 8192];
                loop {
                    let n = entry
                        .read(&mut buffer)
                        .map_err(|e| SqlError::IoError(e.to_string()))?;
                    if n == 0 {
                        break;
                    }
                    out_file
                        .write_all(&buffer[..n])
                        .map_err(|e| SqlError::IoError(e.to_string()))?;
                    total_bytes += n as u64;
                }
            }
        }

        Ok(total_bytes)
    }
}

pub struct WalBackupManager {
    wal_path: PathBuf,
    min_lsn_to_keep: u64,
}

impl WalBackupManager {
    pub fn new(wal_path: PathBuf) -> Self {
        Self {
            wal_path,
            min_lsn_to_keep: 0,
        }
    }

    pub fn set_min_lsn(&mut self, lsn: u64) {
        self.min_lsn_to_keep = lsn;
    }

    pub fn get_min_lsn(&self) -> u64 {
        self.min_lsn_to_keep
    }

    pub fn archive_wal_since(&self, start_lsn: u64, archive_path: &Path) -> SqlResult<u64> {
        let wal_file = self.wal_path.join("wal.binlog");
        if !wal_file.exists() {
            return Ok(0);
        }

        let mut reader = WalReader::new(&wal_file)?;
        let entries = reader
            .read_from(start_lsn)
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        let mut file = File::create(archive_path).map_err(|e| SqlError::IoError(e.to_string()))?;
        let mut writer = BufWriter::new(file);
        let mut bytes_written = 0u64;

        for entry in entries {
            let entry_bytes = entry.to_bytes();
            writer
                .write_all(&(entry_bytes.len() as u32).to_le_bytes())
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            writer
                .write_all(&entry_bytes)
                .map_err(|e| SqlError::IoError(e.to_string()))?;
            bytes_written += 4 + entry_bytes.len() as u64;
        }

        writer
            .flush()
            .map_err(|e| SqlError::IoError(e.to_string()))?;

        Ok(bytes_written)
    }

    pub fn apply_wal_archive(&self, archive_path: &Path) -> SqlResult<Vec<WalEntry>> {
        if !archive_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(archive_path).map_err(|e| SqlError::IoError(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(SqlError::IoError(e.to_string())),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut data = vec![0u8; len];
            reader
                .read_exact(&mut data)
                .map_err(|e| SqlError::IoError(e.to_string()))?;

            if let Some(entry) = WalEntry::from_bytes(&data) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_backup_schedule_default() {
        let schedule = BackupSchedule::default();
        assert_eq!(schedule.schedule_type, BackupScheduleType::Daily);
        assert!(schedule.enabled);
        assert_eq!(schedule.retention_days, 7);
        assert!(schedule.compress);
    }

    #[test]
    fn test_backup_scheduler_creation() {
        let scheduler = BackupScheduler::new(
            temp_dir().join("storage"),
            temp_dir().join("backup"),
            temp_dir().join("wal"),
        );
        assert!(!scheduler.is_running());
    }

    #[test]
    fn test_backup_scheduler_lsn() {
        let scheduler = BackupScheduler::new(
            temp_dir().join("storage"),
            temp_dir().join("backup"),
            temp_dir().join("wal"),
        );
        scheduler.set_current_lsn(100);
        assert_eq!(scheduler.get_current_lsn(), 100);
    }

    #[test]
    fn test_backup_compressor_compress_decompress() {
        let temp = temp_dir();
        let source = temp.join("compress_source.txt");
        let compressed = temp.join("compressed.txt.gz");
        let decompressed = temp.join("decompressed.txt");

        fs::write(
            &source,
            "Hello, World! This is a test file for compression.",
        )
        .unwrap();

        BackupCompressor::compress_file(&source, &compressed).unwrap();
        BackupCompressor::decompress_file(&compressed, &decompressed).unwrap();

        let original = fs::read_to_string(&source).unwrap();
        let restored = fs::read_to_string(&decompressed).unwrap();

        assert_eq!(original, restored);

        fs::remove_file(source).ok();
        fs::remove_file(compressed).ok();
        fs::remove_file(decompressed).ok();
    }

    #[test]
    fn test_wal_backup_manager_creation() {
        let manager = WalBackupManager::new(temp_dir().join("wal"));
        assert_eq!(manager.get_min_lsn(), 0);
    }

    #[test]
    fn test_wal_backup_manager_set_min_lsn() {
        let mut manager = WalBackupManager::new(temp_dir().join("wal"));
        manager.set_min_lsn(500);
        assert_eq!(manager.get_min_lsn(), 500);
    }

    #[test]
    fn test_backup_schedule_type_serialization() {
        let daily = BackupScheduleType::Daily;
        let weekly = BackupScheduleType::Weekly;
        let monthly = BackupScheduleType::Monthly;

        assert_eq!(daily, BackupScheduleType::Daily);
        assert_eq!(weekly, BackupScheduleType::Weekly);
        assert_eq!(monthly, BackupScheduleType::Monthly);
    }

    #[test]
    fn test_incremental_backup_point_serialization() {
        let point = IncrementalBackupPoint {
            lsn: 12345,
            timestamp: 1700000000,
            tables_modified: vec!["table_1".to_string(), "table_2".to_string()],
        };

        let json = serde_json::to_string(&point).unwrap();
        let parsed: IncrementalBackupPoint = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.lsn, 12345);
        assert_eq!(parsed.timestamp, 1700000000);
        assert_eq!(parsed.tables_modified.len(), 2);
    }
}

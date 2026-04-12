//! Data Scale Validation Module
//!
//! Provides validation to ensure benchmark results are trustworthy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataScaleInfo {
    pub row_count: usize,
    pub avg_row_size_bytes: usize,
    pub total_size_bytes: usize,
    pub ram_bytes: u64,
    pub fits_in_memory: bool,
}

impl DataScaleInfo {
    pub fn new(row_count: usize, avg_row_size_bytes: usize) -> Self {
        let total_size_bytes = row_count * avg_row_size_bytes;
        let ram_bytes = get_system_ram();
        let fits_in_memory = total_size_bytes < (ram_bytes as usize);

        Self {
            row_count,
            avg_row_size_bytes,
            total_size_bytes,
            ram_bytes,
            fits_in_memory,
        }
    }

    pub fn validate(&self) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        let total_mb = self.total_size_bytes as f64 / 1_000_000.0;
        let ram_gb = self.ram_bytes as f64 / 1_000_000_000.0;

        if !self.fits_in_memory {
            warnings.push(ValidationWarning {
                level: WarningLevel::Warn,
                message: format!(
                    "Dataset ({:.1} MB) fits in memory ({:.1} GB RAM) - results should be accurate",
                    total_mb, ram_gb
                ),
            });
        } else {
            warnings.push(ValidationWarning {
                level: WarningLevel::Warn,
                message: format!(
                    "WARNING: Dataset ({:.1} MB) may exceed RAM ({:.1} GB) - results may be unreliable due to disk I/O",
                    total_mb, ram_gb
                ),
            });
        }

        if self.row_count < 1000 {
            warnings.push(ValidationWarning {
                level: WarningLevel::Warn,
                message: format!(
                    "Small dataset ({} rows) - results may not be statistically significant",
                    self.row_count
                ),
            });
        }

        warnings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub level: WarningLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WarningLevel {
    Info,
    Warn,
    Error,
}

fn get_system_ram() -> u64 {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("hw.memsize").output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                return s.trim().parse().unwrap_or(8_000_000_000);
            }
        }
        8_000_000_000 // default 8GB
    }

    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|kb| kb * 1024)
            })
            .unwrap_or(8_000_000_000)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        8_000_000_000 // default 8GB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_scale_info() {
        let info = DataScaleInfo::new(1000000, 100);
        assert!(info.fits_in_memory);
        assert_eq!(info.total_size_bytes, 100_000_000);
    }

    #[test]
    fn test_validation_warnings() {
        let info = DataScaleInfo::new(100, 100); // small dataset
        let warnings = info.validate();
        assert!(!warnings.is_empty());
    }
}

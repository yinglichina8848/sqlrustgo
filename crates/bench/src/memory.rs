//! Memory monitoring and limits for benchmark safety

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Default memory limit: 10GB
pub const DEFAULT_MEMORY_LIMIT_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// Get current process memory usage in bytes
/// Returns None if the operation fails (e.g., on unsupported platforms)
#[cfg(target_os = "macos")]
pub fn get_process_memory_bytes() -> Option<u64> {
    use std::process::Command;

    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;

    let rss_str = String::from_utf8_lossy(&output.stdout);
    let rss_kb: u64 = rss_str.trim().parse().ok()?;
    Some(rss_kb * 1024)
}

#[cfg(target_os = "linux")]
pub fn get_process_memory_bytes() -> Option<u64> {
    use std::fs;

    let status_path = format!("/proc/{}/status", std::process::id());
    let content = fs::read_to_string(status_path).ok()?;

    for line in content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let rss_kb: u64 = parts[1].parse().ok()?;
                return Some(rss_kb * 1024);
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn get_process_memory_bytes() -> Option<u64> {
    None
}

/// Memory limiter that checks memory usage periodically
pub struct MemoryLimiter {
    limit_bytes: u64,
    exceeded: Arc<AtomicBool>,
}

impl MemoryLimiter {
    /// Create a new memory limiter with the specified limit
    pub fn new(limit_bytes: u64) -> Self {
        Self {
            limit_bytes,
            exceeded: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create with default 10GB limit
    pub fn default_limit() -> Self {
        Self::new(DEFAULT_MEMORY_LIMIT_BYTES)
    }

    /// Check if memory limit is exceeded
    pub fn is_exceeded(&self) -> bool {
        if let Some(current) = get_process_memory_bytes() {
            let exceeded = current > self.limit_bytes;
            if exceeded {
                self.exceeded.store(true, Ordering::SeqCst);
            }
            exceeded
        } else {
            // If we can't measure memory, allow execution but warn
            tracing::warn!("Cannot measure process memory, skipping limit check");
            false
        }
    }

    /// Get current memory usage as a string for display
    pub fn get_current_usage_string(&self) -> String {
        match get_process_memory_bytes() {
            Some(bytes) => {
                let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                let limit_gb = self.limit_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                format!("{:.2} GB / {:.2} GB limit ({:.1}%)", gb, limit_gb, (bytes as f64 / self.limit_bytes as f64) * 100.0)
            }
            None => "Unknown".to_string(),
        }
    }

    /// Check and warn if memory usage is high (> 80%)
    pub fn check_and_warn(&self) {
        if let Some(current) = get_process_memory_bytes() {
            let usage = current as f64 / self.limit_bytes as f64;
            if usage > 0.8 {
                tracing::warn!(
                    "High memory usage: {:.1}% of limit ({:.2} GB / {:.2} GB)",
                    usage * 100.0,
                    current as f64 / (1024.0 * 1024.0 * 1024.0),
                    self.limit_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
                );
            }
        }
    }

    /// Get the limit in bytes
    pub fn limit_bytes(&self) -> u64 {
        self.limit_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limiter_creation() {
        let limiter = MemoryLimiter::new(5_000_000_000);
        assert_eq!(limiter.limit_bytes(), 5_000_000_000);
    }

    #[test]
    fn test_default_limit_is_10gb() {
        let limiter = MemoryLimiter::default_limit();
        assert_eq!(limiter.limit_bytes(), DEFAULT_MEMORY_LIMIT_BYTES);
    }

    #[test]
    fn test_get_current_usage() {
        let limiter = MemoryLimiter::default_limit();
        let usage = limiter.get_current_usage_string();
        // Should return a string, not "Unknown" on supported platforms
        println!("Memory usage: {}", usage);
    }
}

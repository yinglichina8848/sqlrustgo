use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum FirewallError {
    #[error("SQL injection detected: {0}")]
    SqlInjectionDetected(String),
    #[error("Query timeout exceeded: {0}s")]
    QueryTimeout(u64),
    #[error("Row limit exceeded: {0} rows")]
    RowLimitExceeded(usize),
    #[error("Full table scan detected: {0}")]
    FullTableScanDetected(String),
    #[error("Blocked pattern: {0}")]
    BlockedPattern(String),
    #[error("IP blocked: {0}")]
    IpBlocked(String),
    #[error("Rate limit exceeded for user: {0}")]
    RateLimitExceeded(String),
    #[error("Batch operation not allowed: {0}")]
    BatchOperationNotAllowed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallConfig {
    pub enabled: bool,
    pub query_timeout_secs: u64,
    pub max_rows: usize,
    pub allow_full_table_scans: bool,
    pub allow_batch_delete: bool,
    pub allow_batch_update: bool,
    pub block_sql_keywords: bool,
}

impl Default for FirewallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            query_timeout_secs: 30,
            max_rows: 10000,
            allow_full_table_scans: false,
            allow_batch_delete: false,
            allow_batch_update: false,
            block_sql_keywords: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallStats {
    pub total_checked: u64,
    pub injections_blocked: u64,
    pub queries_blocked: u64,
    pub alerts_triggered: u64,
}

#[derive(Debug, Clone)]
pub struct BlacklistPattern {
    pub pattern: String,
    pub description: String,
    pub severity: ThreatSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct WhitelistPattern {
    pub sql_pattern: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IpBlacklist {
    pub blocked_ips: HashSet<String>,
}

impl IpBlacklist {
    pub fn new() -> Self {
        Self {
            blocked_ips: HashSet::new(),
        }
    }

    pub fn add(&mut self, ip: &str) {
        self.blocked_ips.insert(ip.to_string());
    }

    pub fn remove(&mut self, ip: &str) {
        self.blocked_ips.remove(ip);
    }

    pub fn contains(&self, ip: &str) -> bool {
        self.blocked_ips.contains(ip)
    }
}

pub struct SqlFirewall {
    config: FirewallConfig,
    blacklist: Vec<BlacklistPattern>,
    whitelist: Vec<WhitelistPattern>,
    ip_blacklist: IpBlacklist,
    stats: FirewallStats,
}

impl SqlFirewall {
    pub fn new(config: FirewallConfig) -> Self {
        Self {
            config,
            blacklist: Self::default_blacklist(),
            whitelist: Vec::new(),
            ip_blacklist: IpBlacklist::new(),
            stats: FirewallStats::default(),
        }
    }

    pub fn with_blacklist(mut self, patterns: Vec<BlacklistPattern>) -> Self {
        self.blacklist = patterns;
        self
    }

    pub fn with_whitelist(mut self, patterns: Vec<WhitelistPattern>) -> Self {
        self.whitelist = patterns;
        self
    }

    fn default_blacklist() -> Vec<BlacklistPattern> {
        vec![
            BlacklistPattern {
                pattern: r"UNION\s+(ALL\s+)?SELECT".to_string(),
                description: "UNION-based SQL injection".to_string(),
                severity: ThreatSeverity::Critical,
            },
            BlacklistPattern {
                pattern: r"('\s*OR\s*'1'\s*=\s*'1)".to_string(),
                description: "Classic OR injection".to_string(),
                severity: ThreatSeverity::Critical,
            },
            BlacklistPattern {
                pattern: r"(DROP\s+TABLE|DELETE\s+FROM\s+--)".to_string(),
                description: "Destructive SQL command".to_string(),
                severity: ThreatSeverity::High,
            },
            BlacklistPattern {
                pattern: r"(EXEC\s*\(|EXECUTE\s*\(|xp_)".to_string(),
                description: "Stored procedure injection".to_string(),
                severity: ThreatSeverity::High,
            },
            BlacklistPattern {
                pattern: r"(\bOR\b.*\bOR\b.*\bOR\b)".to_string(),
                description: "Excessive OR conditions".to_string(),
                severity: ThreatSeverity::Medium,
            },
            BlacklistPattern {
                pattern: r"(--|\#|\/\*)".to_string(),
                description: "SQL comment injection".to_string(),
                severity: ThreatSeverity::Medium,
            },
            BlacklistPattern {
                pattern: r"(INTO\s+OUTFILE|INTO\s+DUMPFILE)".to_string(),
                description: "File write attempt".to_string(),
                severity: ThreatSeverity::Critical,
            },
            BlacklistPattern {
                pattern: r"(\bLOAD_FILE\b|\bBENCHMARK\b|\bSLEEP\b)".to_string(),
                description: "Information gathering/time-based injection".to_string(),
                severity: ThreatSeverity::Medium,
            },
        ]
    }

    pub fn check_sql(&mut self, sql: &str) -> Result<(), FirewallError> {
        self.stats.total_checked += 1;

        if !self.config.enabled {
            return Ok(());
        }

        let sql_upper = sql.to_uppercase();
        let sql_lower = sql.to_lowercase();

        if self.is_whitelisted(&sql_lower) {
            return Ok(());
        }

        for pattern in &self.blacklist {
            if self.matches_pattern(sql, &pattern.pattern) {
                self.stats.injections_blocked += 1;
                return Err(FirewallError::SqlInjectionDetected(
                    pattern.description.clone(),
                ));
            }
        }

        if self.config.block_sql_keywords {
            let dangerous_keywords = [
                "UNION",
                "EXEC",
                "EXECUTE",
                "XP_",
                "LOAD_FILE",
                "INTO OUTFILE",
                "INTO DUMPFILE",
            ];
            for keyword in dangerous_keywords {
                if sql_upper.contains(keyword) {
                    self.stats.injections_blocked += 1;
                    return Err(FirewallError::BlockedPattern(keyword.to_string()));
                }
            }
        }

        Ok(())
    }

    fn is_whitelisted(&self, sql: &str) -> bool {
        for pattern in &self.whitelist {
            if pattern.enabled && self.matches_pattern(sql, &pattern.sql_pattern) {
                return true;
            }
        }
        false
    }

    fn matches_pattern(&self, sql: &str, pattern: &str) -> bool {
        if let Ok(regex) = regex::Regex::new(&format!("(?i){}", pattern)) {
            regex.is_match(sql)
        } else {
            sql.to_lowercase().contains(&pattern.to_lowercase())
        }
    }

    pub fn check_ip(&self, ip: &str) -> Result<(), FirewallError> {
        if self.ip_blacklist.contains(ip) {
            return Err(FirewallError::IpBlocked(ip.to_string()));
        }
        Ok(())
    }

    pub fn check_query_timeout(&self, elapsed_secs: u64) -> Result<(), FirewallError> {
        if elapsed_secs > self.config.query_timeout_secs {
            return Err(FirewallError::QueryTimeout(elapsed_secs));
        }
        Ok(())
    }

    pub fn check_row_limit(&self, row_count: usize) -> Result<(), FirewallError> {
        if row_count > self.config.max_rows {
            return Err(FirewallError::RowLimitExceeded(row_count));
        }
        Ok(())
    }

    pub fn check_full_table_scan(
        &mut self,
        sql: &str,
        table_name: &str,
    ) -> Result<(), FirewallError> {
        if self.config.allow_full_table_scans {
            return Ok(());
        }

        let sql_upper = sql.to_uppercase();
        let has_where = sql_upper.contains("WHERE");
        let has_index_hint = sql_upper.contains("USE INDEX") || sql_upper.contains("FORCE INDEX");

        if !has_where && !has_index_hint && sql.contains(table_name) && !sql_upper.contains("JOIN")
        {
            self.stats.queries_blocked += 1;
            return Err(FirewallError::FullTableScanDetected(table_name.to_string()));
        }

        Ok(())
    }

    pub fn check_batch_operation(&self, sql: &str) -> Result<(), FirewallError> {
        let sql_upper = sql.to_uppercase();

        if sql_upper.starts_with("DELETE FROM")
            && !self.config.allow_batch_delete
            && !sql_upper.contains("WHERE")
        {
            return Err(FirewallError::BatchOperationNotAllowed(
                "DELETE".to_string(),
            ));
        }

        if sql_upper.starts_with("UPDATE")
            && !self.config.allow_batch_update
            && !sql_upper.contains("WHERE")
        {
            return Err(FirewallError::BatchOperationNotAllowed(
                "UPDATE".to_string(),
            ));
        }

        Ok(())
    }

    pub fn add_blacklist_pattern(&mut self, pattern: BlacklistPattern) {
        self.blacklist.push(pattern);
    }

    pub fn add_whitelist_pattern(&mut self, pattern: WhitelistPattern) {
        self.whitelist.push(pattern);
    }

    pub fn block_ip(&mut self, ip: &str) {
        self.ip_blacklist.add(ip);
    }

    pub fn unblock_ip(&mut self, ip: &str) {
        self.ip_blacklist.remove(ip);
    }

    pub fn get_stats(&self) -> &FirewallStats {
        &self.stats
    }

    pub fn get_config(&self) -> &FirewallConfig {
        &self.config
    }

    pub fn update_config(&mut self, config: FirewallConfig) {
        self.config = config;
    }
}

impl Default for SqlFirewall {
    fn default() -> Self {
        Self::new(FirewallConfig::default())
    }
}

pub type SharedFirewall = Arc<parking_lot::RwLock<SqlFirewall>>;

pub fn create_shared_firewall(config: FirewallConfig) -> SharedFirewall {
    Arc::new(parking_lot::RwLock::new(SqlFirewall::new(config)))
}

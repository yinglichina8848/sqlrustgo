use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AlertError {
    #[error("Alert channel error: {0}")]
    ChannelError(String),
    #[error("Alert queue full")]
    QueueFull,
    #[error("Alert send failed: {0}")]
    SendFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: super::firewall::ThreatSeverity,
    pub alert_type: AlertType,
    pub message: String,
    pub source_ip: Option<String>,
    pub sql_pattern: Option<String>,
    pub user: Option<String>,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    SqlInjection,
    QueryTimeout,
    RowLimitExceeded,
    FullTableScan,
    BatchOperationBlocked,
    IpBlocked,
    RateLimitExceeded,
    BlacklistViolation,
    WhitelistViolation,
    ConfigChange,
}

impl Alert {
    pub fn new(
        severity: super::firewall::ThreatSeverity,
        alert_type: AlertType,
        message: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            severity,
            alert_type,
            message,
            source_ip: None,
            sql_pattern: None,
            user: None,
            acknowledged: false,
        }
    }

    pub fn with_source_ip(mut self, ip: String) -> Self {
        self.source_ip = Some(ip);
        self
    }

    pub fn with_sql_pattern(mut self, sql: String) -> Self {
        self.sql_pattern = Some(sql);
        self
    }

    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub flush_interval_secs: u64,
    pub alert_on_injection: bool,
    pub alert_on_timeout: bool,
    pub alert_on_full_table_scan: bool,
    pub alert_on_batch_blocked: bool,
    pub alert_on_ip_blocked: bool,
    pub min_severity_for_alert: super::firewall::ThreatSeverity,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 1000,
            flush_interval_secs: 5,
            alert_on_injection: true,
            alert_on_timeout: true,
            alert_on_full_table_scan: true,
            alert_on_batch_blocked: true,
            alert_on_ip_blocked: true,
            min_severity_for_alert: super::firewall::ThreatSeverity::Medium,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AlertStats {
    pub total_alerts: u64,
    pub alerts_by_type: std::collections::HashMap<AlertType, u64>,
    pub alerts_by_severity: std::collections::HashMap<super::firewall::ThreatSeverity, u64>,
    pub acknowledged_alerts: u64,
}

pub struct AlertManager {
    config: AlertConfig,
    alerts: Vec<Alert>,
    stats: AlertStats,
}

impl AlertManager {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            alerts: Vec::new(),
            stats: AlertStats::default(),
        }
    }

    pub fn send_alert(&mut self, alert: Alert) -> Result<(), AlertError> {
        if !self.config.enabled {
            return Ok(());
        }

        if alert.severity < self.config.min_severity_for_alert {
            return Ok(());
        }

        if self.alerts.len() >= self.config.buffer_size {
            return Err(AlertError::QueueFull);
        }

        self.stats.total_alerts += 1;
        *self
            .stats
            .alerts_by_type
            .entry(alert.alert_type)
            .or_insert(0) += 1;
        *self
            .stats
            .alerts_by_severity
            .entry(alert.severity)
            .or_insert(0) += 1;

        self.alerts.push(alert);
        Ok(())
    }

    pub fn send_sql_injection_alert(
        &mut self,
        ip: Option<String>,
        sql: Option<String>,
        description: &str,
    ) -> Result<(), AlertError> {
        if !self.config.alert_on_injection {
            return Ok(());
        }

        let alert = Alert::new(
            super::firewall::ThreatSeverity::Critical,
            AlertType::SqlInjection,
            format!("SQL injection detected: {}", description),
        )
        .with_source_ip(ip.unwrap_or_else(|| "unknown".to_string()))
        .with_sql_pattern(sql.unwrap_or_else(|| "unknown".to_string()));

        self.send_alert(alert)
    }

    pub fn send_timeout_alert(&mut self, query: &str, elapsed_secs: u64) -> Result<(), AlertError> {
        if !self.config.alert_on_timeout {
            return Ok(());
        }

        let alert = Alert::new(
            super::firewall::ThreatSeverity::Medium,
            AlertType::QueryTimeout,
            format!("Query timeout after {}s: {}", elapsed_secs, query),
        )
        .with_sql_pattern(query.to_string());

        self.send_alert(alert)
    }

    pub fn send_full_table_scan_alert(&mut self, table: &str, sql: &str) -> Result<(), AlertError> {
        if !self.config.alert_on_full_table_scan {
            return Ok(());
        }

        let alert = Alert::new(
            super::firewall::ThreatSeverity::Medium,
            AlertType::FullTableScan,
            format!("Full table scan detected on table: {}", table),
        )
        .with_sql_pattern(sql.to_string());

        self.send_alert(alert)
    }

    pub fn send_batch_blocked_alert(
        &mut self,
        operation: &str,
        sql: &str,
        ip: Option<String>,
    ) -> Result<(), AlertError> {
        if !self.config.alert_on_batch_blocked {
            return Ok(());
        }

        let alert = Alert::new(
            super::firewall::ThreatSeverity::Medium,
            AlertType::BatchOperationBlocked,
            format!("Batch {} operation blocked", operation),
        )
        .with_sql_pattern(sql.to_string())
        .with_source_ip(ip.unwrap_or_else(|| "unknown".to_string()));

        self.send_alert(alert)
    }

    pub fn send_ip_blocked_alert(&mut self, ip: &str) -> Result<(), AlertError> {
        if !self.config.alert_on_ip_blocked {
            return Ok(());
        }

        let alert = Alert::new(
            super::firewall::ThreatSeverity::High,
            AlertType::IpBlocked,
            format!("IP blocked: {}", ip),
        )
        .with_source_ip(ip.to_string());

        self.send_alert(alert)
    }

    pub fn get_alerts(&self) -> &[Alert] {
        &self.alerts
    }

    pub fn get_alerts_mut(&mut self) -> &mut Vec<Alert> {
        &mut self.alerts
    }

    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }

    pub fn acknowledge_alert(&mut self, alert_id: &str) -> Result<(), AlertError> {
        if let Some(alert) = self.alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledge();
            self.stats.acknowledged_alerts += 1;
            Ok(())
        } else {
            Err(AlertError::SendFailed(format!(
                "Alert {} not found",
                alert_id
            )))
        }
    }

    pub fn get_stats(&self) -> &AlertStats {
        &self.stats
    }

    pub fn update_config(&mut self, config: AlertConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &AlertConfig {
        &self.config
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new(AlertConfig::default())
    }
}

pub type SharedAlertManager = Arc<parking_lot::RwLock<AlertManager>>;

pub fn create_shared_alert_manager(config: AlertConfig) -> SharedAlertManager {
    Arc::new(parking_lot::RwLock::new(AlertManager::new(config)))
}

//! Audit logging system
//!
//! Provides comprehensive audit logging for security compliance.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub user: String,
    pub ip: String,
    pub details: String,
    pub session_id: u64,
    pub duration_ms: Option<u64>,
    pub rows: Option<u64>,
}

impl AuditRecord {
    pub fn new(event_type: &str, user: &str, ip: &str, details: &str) -> Self {
        Self {
            id: 0,
            timestamp: current_timestamp(),
            event_type: event_type.to_string(),
            user: user.to_string(),
            ip: ip.to_string(),
            details: details.to_string(),
            session_id: 0,
            duration_ms: None,
            rows: None,
        }
    }

    pub fn with_session(mut self, session_id: u64) -> Self {
        self.session_id = session_id;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn with_rows(mut self, rows: u64) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

#[derive(Debug, Clone)]
pub enum AuditEvent {
    Login {
        user: String,
        success: bool,
        ip: String,
    },
    Logout {
        user: String,
        session_id: u64,
    },
    ExecuteSql {
        user: String,
        sql: String,
        duration_ms: u64,
        rows: u64,
        session_id: u64,
    },
    DDL {
        user: String,
        sql: String,
        session_id: u64,
    },
    DML {
        user: String,
        sql: String,
        table: String,
        session_id: u64,
    },
    Grant {
        user: String,
        privilege: String,
        object: String,
        to_user: String,
    },
    Revoke {
        user: String,
        privilege: String,
        object: String,
        from_user: String,
    },
    Error {
        user: String,
        error: String,
        session_id: u64,
    },
    SessionStart {
        user: String,
        ip: String,
    },
    SessionEnd {
        session_id: u64,
    },
}

impl AuditEvent {
    pub fn to_record(&self) -> AuditRecord {
        match self {
            AuditEvent::Login { user, success, ip } => {
                AuditRecord::new("LOGIN", user, ip, &format!("success={}", success))
            }
            AuditEvent::Logout { user, session_id } => {
                AuditRecord::new("LOGOUT", user, "", &format!("session_id={}", session_id))
                    .with_session(*session_id)
            }
            AuditEvent::ExecuteSql {
                user,
                sql,
                duration_ms,
                rows,
                session_id,
            } => AuditRecord::new("EXECUTE_SQL", user, "", sql)
                .with_session(*session_id)
                .with_duration(*duration_ms)
                .with_rows(*rows),
            AuditEvent::DDL {
                user,
                sql,
                session_id,
            } => AuditRecord::new("DDL", user, "", sql).with_session(*session_id),
            AuditEvent::DML {
                user,
                sql,
                table,
                session_id,
            } => AuditRecord::new("DML", user, "", &format!("{} on {}", sql, table))
                .with_session(*session_id),
            AuditEvent::Grant {
                user,
                privilege,
                object,
                to_user,
            } => AuditRecord::new(
                "GRANT",
                user,
                "",
                &format!("{} on {} to {}", privilege, object, to_user),
            ),
            AuditEvent::Revoke {
                user,
                privilege,
                object,
                from_user,
            } => AuditRecord::new(
                "REVOKE",
                user,
                "",
                &format!("{} on {} from {}", privilege, object, from_user),
            ),
            AuditEvent::Error {
                user,
                error,
                session_id,
            } => AuditRecord::new("ERROR", user, "", error).with_session(*session_id),
            AuditEvent::SessionStart { user, ip } => {
                AuditRecord::new("SESSION_START", user, ip, "")
            }
            AuditEvent::SessionEnd { session_id } => {
                AuditRecord::new("SESSION_END", "", "", &format!("session_id={}", session_id))
                    .with_session(*session_id)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: PathBuf,
    pub log_login: bool,
    pub log_sql: bool,
    pub log_ddl: bool,
    pub log_dml: bool,
    pub log_grant_revokes: bool,
    pub log_errors: bool,
    pub retention_days: u32,
    pub async_write: bool,
    pub max_events_in_memory: usize,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: PathBuf::from("audit.log"),
            log_login: true,
            log_sql: true,
            log_ddl: true,
            log_dml: true,
            log_grant_revokes: true,
            log_errors: true,
            retention_days: 90,
            async_write: false,
            max_events_in_memory: 10000,
        }
    }
}

pub struct AuditManager {
    config: AuditConfig,
    events: Arc<Mutex<VecDeque<AuditRecord>>>,
    next_id: Arc<Mutex<u64>>,
}

impl AuditManager {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            events: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(AuditConfig::default())
    }

    pub fn log_event(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }

        let should_log = match &event {
            AuditEvent::Login { .. } => self.config.log_login,
            AuditEvent::Logout { .. } => self.config.log_login,
            AuditEvent::ExecuteSql { .. } => self.config.log_sql,
            AuditEvent::DDL { .. } => self.config.log_ddl,
            AuditEvent::DML { .. } => self.config.log_dml,
            AuditEvent::Grant { .. } | AuditEvent::Revoke { .. } => self.config.log_grant_revokes,
            AuditEvent::Error { .. } => self.config.log_errors,
            AuditEvent::SessionStart { .. } | AuditEvent::SessionEnd { .. } => true,
        };

        if !should_log {
            return;
        }

        let mut record = event.to_record();
        record.id = self.next_id();

        self.add_record(record);
    }

    fn next_id(&self) -> u64 {
        let mut id = self.next_id.lock().unwrap();
        let result = *id;
        *id += 1;
        result
    }

    fn add_record(&self, mut record: AuditRecord) {
        let mut events = self.events.lock().unwrap();
        record.id = self.next_id();
        events.push_back(record.clone());

        if events.len() > self.config.max_events_in_memory {
            events.pop_front();
        }

        drop(events);

        if !self.config.async_write {
            if let Err(e) = self.write_to_file(&record) {
                eprintln!("Failed to write audit record: {}", e);
            }
        }
    }

    fn write_to_file(&self, record: &AuditRecord) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)?;

        writeln!(file, "{}", record.to_json())?;
        file.flush()
    }

    pub fn log_login(&self, user: &str, success: bool, ip: &str) {
        self.log_event(AuditEvent::Login {
            user: user.to_string(),
            success,
            ip: ip.to_string(),
        });
    }

    pub fn log_logout(&self, user: &str, session_id: u64) {
        self.log_event(AuditEvent::Logout {
            user: user.to_string(),
            session_id,
        });
    }

    pub fn log_sql(&self, user: &str, sql: &str, duration_ms: u64, rows: u64, session_id: u64) {
        self.log_event(AuditEvent::ExecuteSql {
            user: user.to_string(),
            sql: sql.to_string(),
            duration_ms,
            rows,
            session_id,
        });
    }

    pub fn log_ddl(&self, user: &str, sql: &str, session_id: u64) {
        self.log_event(AuditEvent::DDL {
            user: user.to_string(),
            sql: sql.to_string(),
            session_id,
        });
    }

    pub fn log_grant(&self, user: &str, privilege: &str, object: &str, to_user: &str) {
        self.log_event(AuditEvent::Grant {
            user: user.to_string(),
            privilege: privilege.to_string(),
            object: object.to_string(),
            to_user: to_user.to_string(),
        });
    }

    pub fn log_revoke(&self, user: &str, privilege: &str, object: &str, from_user: &str) {
        self.log_event(AuditEvent::Revoke {
            user: user.to_string(),
            privilege: privilege.to_string(),
            object: object.to_string(),
            from_user: from_user.to_string(),
        });
    }

    pub fn log_error(&self, user: &str, error: &str, session_id: u64) {
        self.log_event(AuditEvent::Error {
            user: user.to_string(),
            error: error.to_string(),
            session_id,
        });
    }

    pub fn query_logs(&self, filter: &AuditFilter) -> Vec<AuditRecord> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|r| filter.matches(r))
            .cloned()
            .collect()
    }

    pub fn get_recent(&self, count: usize) -> Vec<AuditRecord> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(count).cloned().collect()
    }

    pub fn get_stats(&self) -> AuditStats {
        let events = self.events.lock().unwrap();
        let total = events.len();

        let logins = events.iter().filter(|e| e.event_type == "LOGIN").count();
        let sqls = events
            .iter()
            .filter(|e| e.event_type == "EXECUTE_SQL")
            .count();
        let ddls = events.iter().filter(|e| e.event_type == "DDL").count();
        let errors = events.iter().filter(|e| e.event_type == "ERROR").count();

        AuditStats {
            total_events: total,
            logins,
            sql_executions: sqls,
            ddls,
            errors,
        }
    }

    pub fn flush(&self) -> io::Result<()> {
        let events: Vec<AuditRecord> = {
            let events_guard = self.events.lock().unwrap();
            events_guard.iter().cloned().collect()
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)?;

        for record in &events {
            writeln!(file, "{}", record.to_json())?;
        }
        file.flush()
    }

    pub fn clear_memory(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }
}

#[derive(Debug, Clone)]
pub struct AuditFilter {
    pub event_types: Option<Vec<String>>,
    pub users: Option<Vec<String>>,
    pub from_time: Option<u64>,
    pub to_time: Option<u64>,
    pub session_id: Option<u64>,
}

impl AuditFilter {
    pub fn new() -> Self {
        Self {
            event_types: None,
            users: None,
            from_time: None,
            to_time: None,
            session_id: None,
        }
    }

    pub fn with_event_types(mut self, types: Vec<&str>) -> Self {
        self.event_types = Some(types.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_users(mut self, users: Vec<&str>) -> Self {
        self.users = Some(users.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn with_time_range(mut self, from: u64, to: u64) -> Self {
        self.from_time = Some(from);
        self.to_time = Some(to);
        self
    }

    pub fn with_session(mut self, session_id: u64) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn matches(&self, record: &AuditRecord) -> bool {
        if let Some(ref types) = self.event_types {
            if !types.contains(&record.event_type) {
                return false;
            }
        }

        if let Some(ref users) = self.users {
            if !users.contains(&record.user) {
                return false;
            }
        }

        if let Some(from) = self.from_time {
            if record.timestamp < from {
                return false;
            }
        }

        if let Some(to) = self.to_time {
            if record.timestamp > to {
                return false;
            }
        }

        if let Some(session) = self.session_id {
            if record.session_id != session {
                return false;
            }
        }

        true
    }
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_events: usize,
    pub logins: usize,
    pub sql_executions: usize,
    pub ddls: usize,
    pub errors: usize,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_record_creation() {
        let record = AuditRecord::new("LOGIN", "alice", "127.0.0.1", "success=true");
        assert_eq!(record.event_type, "LOGIN");
        assert_eq!(record.user, "alice");
    }

    #[test]
    fn test_audit_event_to_record() {
        let event = AuditEvent::Login {
            user: "bob".to_string(),
            success: true,
            ip: "192.168.1.1".to_string(),
        };
        let record = event.to_record();
        assert_eq!(record.event_type, "LOGIN");
        assert!(record.details.contains("true"));
    }

    #[test]
    fn test_audit_filter() {
        let filter = AuditFilter::new()
            .with_event_types(vec!["LOGIN", "LOGOUT"])
            .with_users(vec!["alice"]);

        let record = AuditRecord::new("LOGIN", "alice", "127.0.0.1", "");
        assert!(filter.matches(&record));

        let record2 = AuditRecord::new("EXECUTE_SQL", "alice", "127.0.0.1", "");
        assert!(!filter.matches(&record2));

        let record3 = AuditRecord::new("LOGIN", "bob", "127.0.0.1", "");
        assert!(!filter.matches(&record3));
    }

    #[test]
    fn test_audit_manager() {
        let config = AuditConfig {
            enabled: true,
            log_path: PathBuf::from("/tmp/audit_test.log"),
            ..Default::default()
        };
        let manager = AuditManager::new(config);

        manager.log_login("alice", true, "127.0.0.1");
        manager.log_sql("alice", "SELECT * FROM users", 10, 5, 1);

        let stats = manager.get_stats();
        assert_eq!(stats.logins, 1);
        assert_eq!(stats.sql_executions, 1);
    }

    #[test]
    fn test_json_serialization() {
        let record = AuditRecord::new("LOGIN", "alice", "127.0.0.1", "success=true");
        let json = record.to_json();
        let parsed = AuditRecord::from_json(&json).unwrap();
        assert_eq!(parsed.user, "alice");
    }
}

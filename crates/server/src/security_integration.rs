//! Security integration for SQLRustGo server
//!
//! Provides integration between server components and security features.

use sqlrustgo_security::{
    AuditConfig, AuditEvent, AuditFilter, AuditManager, AuditRecord, AuditStats, Session,
    SessionManager, SessionStatus,
};
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Clone)]
pub struct SecurityIntegration {
    audit_manager: Arc<AuditManager>,
    session_manager: Arc<SessionManager>,
}

impl SecurityIntegration {
    pub fn new() -> Self {
        Self {
            audit_manager: Arc::new(AuditManager::with_default_config()),
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    pub fn with_audit_config(config: AuditConfig) -> Self {
        Self {
            audit_manager: Arc::new(AuditManager::new(config)),
            session_manager: Arc::new(SessionManager::new()),
        }
    }

    pub fn audit(&self) -> &AuditManager {
        &self.audit_manager
    }

    pub fn sessions(&self) -> &SessionManager {
        &self.session_manager
    }

    pub fn create_secure_session(&self, user: String, ip: String) -> u64 {
        let session_id = self
            .session_manager
            .create_session(user.clone(), ip.clone());
        self.audit_manager.log_event(AuditEvent::SessionStart {
            user: user.clone(),
            ip: ip.clone(),
        });
        self.audit_manager.log_event(AuditEvent::Login {
            user,
            success: true,
            ip,
        });
        session_id
    }

    pub fn close_secure_session(&self, session_id: u64, user: String) {
        self.session_manager.close_session(session_id);
        self.audit_manager
            .log_event(AuditEvent::SessionEnd { session_id });
        self.audit_manager
            .log_event(AuditEvent::Logout { user, session_id });
    }

    pub fn log_sql_execution(
        &self,
        user: &str,
        sql: &str,
        duration_ms: u64,
        rows: u64,
        session_id: u64,
    ) {
        self.session_manager.update_activity(session_id);
        self.audit_manager
            .log_sql(user, sql, duration_ms, rows, session_id);
    }

    pub fn log_ddl(&self, user: &str, sql: &str, session_id: u64) {
        self.session_manager.update_activity(session_id);
        self.audit_manager.log_ddl(user, sql, session_id);
    }

    pub fn log_grant(&self, user: &str, privilege: &str, object: &str, to_user: &str) {
        self.audit_manager
            .log_grant(user, privilege, object, to_user);
    }

    pub fn log_error(&self, user: &str, error: &str, session_id: u64) {
        self.audit_manager.log_error(user, error, session_id);
    }

    pub fn cleanup_idle_sessions(&self) -> usize {
        let closed = self.session_manager.close_idle_sessions();
        for _ in 0..closed {
            let sessions = self.session_manager.get_all_sessions();
            if let Some(closed_session) =
                sessions.iter().find(|s| s.status == SessionStatus::Closed)
            {
                self.audit_manager.log_event(AuditEvent::SessionEnd {
                    session_id: closed_session.id,
                });
            }
        }
        closed
    }

    pub fn get_security_stats(&self) -> SecurityStats {
        let audit_stats = self.audit_manager.get_stats();
        SecurityStats {
            audit_total_events: audit_stats.total_events,
            audit_logins: audit_stats.logins,
            audit_sql_executions: audit_stats.sql_executions,
            audit_ddls: audit_stats.ddls,
            audit_errors: audit_stats.errors,
            active_sessions: self.session_manager.get_active_session_count(),
            total_sessions: self.session_manager.get_session_count(),
        }
    }
}

impl Default for SecurityIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub audit_total_events: usize,
    pub audit_logins: usize,
    pub audit_sql_executions: usize,
    pub audit_ddls: usize,
    pub audit_errors: usize,
    pub active_sessions: usize,
    pub total_sessions: usize,
}

pub struct SecurityGuard {
    integration: SecurityIntegration,
    session_id: u64,
    user: String,
    start_time: std::time::Instant,
}

impl SecurityGuard {
    pub fn new(integration: SecurityIntegration, user: String, session_id: u64) -> Self {
        Self {
            integration,
            session_id,
            user,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn log_query(&self, sql: &str, rows: u64) {
        let duration = self.start_time.elapsed().as_millis() as u64;
        self.integration
            .log_sql_execution(&self.user, sql, duration, rows, self.session_id);
    }

    pub fn log_ddl(&self, sql: &str) {
        self.integration.log_ddl(&self.user, sql, self.session_id);
    }

    pub fn log_error(&self, error: &str) {
        self.integration
            .log_error(&self.user, error, self.session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_integration() -> SecurityIntegration {
        let config = AuditConfig {
            enabled: true,
            log_path: PathBuf::from("/tmp/audit_test.log"),
            ..Default::default()
        };
        SecurityIntegration::with_audit_config(config)
    }

    #[test]
    fn test_create_session() {
        let security = create_test_integration();
        let session_id =
            security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        assert_eq!(session_id, 1);
        assert_eq!(security.session_manager.get_session_count(), 1);
    }

    #[test]
    fn test_close_session() {
        let security = create_test_integration();
        let session_id =
            security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        security.close_secure_session(session_id, "alice".to_string());

        let session = security.session_manager.get_session(session_id).unwrap();
        assert_eq!(session.status, SessionStatus::Closed);
    }

    #[test]
    fn test_sql_execution_logging() {
        let security = create_test_integration();
        let session_id =
            security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        security.log_sql_execution("alice", "SELECT * FROM users", 10, 5, session_id);

        let stats = security.get_security_stats();
        assert_eq!(stats.audit_sql_executions, 1);
    }

    #[test]
    fn test_ddl_logging() {
        let security = create_test_integration();
        let session_id =
            security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        security.log_ddl("alice", "CREATE TABLE test (id INT)", session_id);

        let stats = security.get_security_stats();
        assert_eq!(stats.audit_ddls, 1);
    }

    #[test]
    fn test_grant_logging() {
        let security = create_test_integration();
        security.create_secure_session("admin".to_string(), "127.0.0.1".to_string());

        security.log_grant("admin", "SELECT", "users", "alice");

        let filter = AuditFilter::new().with_event_types(vec!["GRANT"]);
        let grants = security.audit_manager.query_logs(&filter);
        assert_eq!(grants.len(), 1);
    }

    #[test]
    fn test_security_stats() {
        let security = create_test_integration();
        security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        let stats = security.get_security_stats();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_sessions, 1);
    }

    #[test]
    fn test_security_guard() {
        let security = create_test_integration();
        let session_id =
            security.create_secure_session("alice".to_string(), "127.0.0.1".to_string());

        let guard = SecurityGuard::new(security.clone(), "alice".to_string(), session_id);
        guard.log_query("SELECT 1", 1);

        let stats = security.get_security_stats();
        assert_eq!(stats.audit_sql_executions, 1);
    }
}

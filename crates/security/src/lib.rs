//! Security Module - Audit logging and session management
//!
//! Provides security features:
//! - Audit logging for SQL execution, login, DDL, DML
//! - Session management
//! - TLS configuration
//! - SQL Firewall for injection detection and query control

pub mod audit;
pub mod firewall;
pub mod session;
pub mod tls;

pub use audit::{AuditConfig, AuditEvent, AuditFilter, AuditManager, AuditRecord, AuditStats};
pub use firewall::{
    create_shared_firewall, BlacklistPattern, FirewallConfig, FirewallError, FirewallStats,
    SharedFirewall, SqlFirewall, ThreatSeverity, WhitelistPattern,
};
pub use session::{Session, SessionManager, SessionStatus};
pub use tls::{CertificateManager, TlsConfig};

#[cfg(test)]
mod firewall_tests;

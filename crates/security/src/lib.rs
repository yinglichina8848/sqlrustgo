//! Security Module - Audit logging and session management
//!
//! Provides security features:
//! - Audit logging for SQL execution, login, DDL, DML
//! - Session management
//! - TLS configuration

pub mod audit;
pub mod session;
pub mod tls;

pub use audit::{AuditConfig, AuditEvent, AuditManager, AuditRecord};
pub use session::{Session, SessionManager};
pub use tls::{CertificateManager, TlsConfig};

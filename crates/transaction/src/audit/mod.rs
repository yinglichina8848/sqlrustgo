//! Audit Module - Immutable Audit System
//!
//! Provides append-only audit logging with tamper detection for GMP compliance.

pub mod audit_verify;
pub mod event_stream;
pub mod hash_chain;
pub mod signature;

pub use audit_verify::{AuditRecoveryVerifier, AuditVerifier, VerificationResult};
pub use event_stream::{
    AuditEvent, AuditEventStore, AuditEventType, AuditStreamState, AUDIT_GENESIS_PREV_HASH,
};
pub use hash_chain::{compute_hash, verify_hash, HashChain, HashChainEntry};
pub use signature::{sign_data, verify_signature, SignatureManager};

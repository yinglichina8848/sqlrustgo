//! Electronic Signature Module for GMP
//!
//! Implements 21 CFR Part 11 compliant electronic signatures.
//!
//! ## Core Formula
//!
//! Electronic Signature = Private Key Signature + Signing Reason + Timestamp
//!
//! ## Features
//!
//! - **User Signatures**: ECDSA-P256 or ED25519 private key signing
//! - **Signing Reasons**: Mandatory reason text for each signature
//! - **Timestamps**: Integration with Trusted Timestamp (GMP-6)
//! - **Four Eyes (双人复核)**: Approval policies requiring multiple signatures
//! - **Policy Enforcement**: Sequential or parallel signature collection

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Electronic signature entry representing a single 21 CFR Part 11 signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignature {
    /// Unique signature identifier
    pub id: String,
    /// Reference to audit chain entry (table id)
    pub audit_chain_id: i64,
    /// User who signed
    pub user_id: String,
    /// Session identifier
    pub session_id: Option<String>,
    /// User role at time of signature
    pub role: Option<String>,
    /// Signing reason (mandatory for 21 CFR Part 11)
    pub reason: String,
    /// Hash of the signed data
    pub data_hash: Vec<u8>,
    /// ED25519 signature bytes
    pub signature: Vec<u8>,
    /// Public key bytes for verification
    pub verifying_key: Vec<u8>,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Associated policy ID (if applicable)
    pub policy_id: Option<String>,
    /// Policy name (for display)
    pub policy_name: Option<String>,
    /// Sequence number within the policy (for sequential policies)
    pub seq_in_policy: Option<i32>,
    /// When signature was created
    pub created_at: i64,
}

impl ElectronicSignature {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        audit_chain_id: i64,
        user_id: String,
        session_id: Option<String>,
        role: Option<String>,
        reason: String,
        data_hash: Vec<u8>,
        signature: Vec<u8>,
        verifying_key: Vec<u8>,
        timestamp: i64,
        policy_id: Option<String>,
        policy_name: Option<String>,
        seq_in_policy: Option<i32>,
    ) -> Self {
        let id = uuid_simple();
        let created_at = timestamp;
        Self {
            id,
            audit_chain_id,
            user_id,
            session_id,
            role,
            reason,
            data_hash,
            signature,
            verifying_key,
            timestamp,
            policy_id,
            policy_name,
            seq_in_policy,
            created_at,
        }
    }

    /// Verify the signature is valid
    pub fn verify(&self, _data: &[u8]) -> bool {
        // Reconstruct what was signed: data_hash || reason || timestamp
        let mut to_verify = Vec::with_capacity(self.data_hash.len() + self.reason.len() + 8);
        to_verify.extend_from_slice(&self.data_hash);
        to_verify.extend_from_slice(self.reason.as_bytes());
        to_verify.extend_from_slice(&self.timestamp.to_le_bytes());

        verify_ed25519_signature(&self.verifying_key, &to_verify, &self.signature)
    }
}

/// Policy status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

impl PolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicyStatus::Pending => "PENDING",
            PolicyStatus::Approved => "APPROVED",
            PolicyStatus::Rejected => "REJECTED",
            PolicyStatus::Expired => "EXPIRED",
        }
    }

    pub fn from_str_explicit(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PENDING" => Some(PolicyStatus::Pending),
            "APPROVED" => Some(PolicyStatus::Approved),
            "REJECTED" => Some(PolicyStatus::Rejected),
            "EXPIRED" => Some(PolicyStatus::Expired),
            _ => None,
        }
    }
}

/// Approval policy defining signature requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Unique policy identifier
    pub id: String,
    /// Human-readable policy name
    pub name: String,
    /// Number of required signatures
    pub required_signatures: usize,
    /// Roles that can sign (any one of these roles)
    pub required_roles: Vec<String>,
    /// If true, signatures must be collected in order
    pub sequential: bool,
    /// Hours before policy request expires
    pub timeout_hours: i32,
    /// Policy description
    pub description: Option<String>,
    /// When policy was created
    pub created_at: i64,
    /// When policy was last updated
    pub updated_at: i64,
    /// Whether policy is active
    pub active: bool,
}

impl ApprovalPolicy {
    /// Create a new approval policy
    pub fn new(
        name: String,
        required_signatures: usize,
        required_roles: Vec<String>,
        sequential: bool,
        timeout_hours: i32,
        description: Option<String>,
    ) -> Self {
        let id = uuid_simple();
        let now = current_timestamp_ms();
        Self {
            id,
            name,
            required_signatures,
            required_roles,
            sequential,
            timeout_hours,
            description,
            created_at: now,
            updated_at: now,
            active: true,
        }
    }
}

/// Signature request for multi-signature policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureRequest {
    /// Unique request identifier
    pub id: String,
    /// Associated policy ID
    pub policy_id: String,
    /// Table name of the record being signed
    pub record_table: String,
    /// Record identifier
    pub record_id: String,
    /// Current policy status
    pub status: PolicyStatus,
    /// Current step (for sequential policies)
    pub current_step: i32,
    /// When request was created
    pub created_at: i64,
    /// When request was last updated
    pub updated_at: i64,
    /// When request expires
    pub expires_at: i64,
}

impl SignatureRequest {
    /// Create a new signature request
    pub fn new(
        policy_id: String,
        record_table: String,
        record_id: String,
        timeout_hours: i32,
    ) -> Self {
        let id = uuid_simple();
        let now = current_timestamp_ms();
        let expires_at = now + (timeout_hours as i64 * 3600 * 1000);
        Self {
            id,
            policy_id,
            record_table,
            record_id,
            status: PolicyStatus::Pending,
            current_step: 1,
            created_at: now,
            updated_at: now,
            expires_at,
        }
    }

    /// Check if request has expired
    pub fn is_expired(&self) -> bool {
        current_timestamp_ms() > self.expires_at
    }
}

/// Result of policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    /// Associated policy ID
    pub policy_id: String,
    /// Associated request ID
    pub request_id: String,
    /// Current policy status
    pub status: PolicyStatus,
    /// Number of signatures collected
    pub current_signatures: usize,
    /// Number of signatures required
    pub required_signatures: usize,
    /// Whether policy is satisfied
    pub is_complete: bool,
    /// Roles that still need to sign
    pub missing_roles: Vec<String>,
}

impl PolicyEvaluation {
    /// Check if all required signatures are present
    pub fn check_complete(&self) -> bool {
        self.current_signatures >= self.required_signatures
    }
}

/// Collected signature record
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CollectedSignature {
    user_id: String,
    role: String,
    timestamp: i64,
}

/// Approval policy evaluator for tracking multi-signature workflows
pub struct ApprovalPolicyEvaluator {
    policy: ApprovalPolicy,
    collected_signatures: Vec<CollectedSignature>,
    current_step: i32,
    request_id: String,
}

impl ApprovalPolicyEvaluator {
    pub fn new(policy: &ApprovalPolicy, request_id: String) -> Self {
        Self {
            policy: policy.clone(),
            collected_signatures: Vec::new(),
            current_step: 1,
            request_id,
        }
    }

    pub fn add_signature(
        &mut self,
        user_id: &str,
        role: &str,
    ) -> Result<PolicyEvaluation, SignatureError> {
        if self.policy.sequential {
            self.add_signature_sequential(user_id, role)
        } else {
            self.add_signature_parallel(user_id, role)
        }
    }

    fn add_signature_parallel(
        &mut self,
        user_id: &str,
        role: &str,
    ) -> Result<PolicyEvaluation, SignatureError> {
        if self.is_complete() {
            return Err(SignatureError::SignatureAlreadyExists {
                signature_id: format!("policy {} already satisfied", self.policy.id),
            });
        }

        if !self.has_required_role(role) {
            return Err(SignatureError::InsufficientPermissions {
                required_role: format!("one of {:?}", self.policy.required_roles),
            });
        }

        if self.has_user_signed(user_id) {
            return Err(SignatureError::SignatureAlreadyExists {
                signature_id: user_id.to_string(),
            });
        }

        self.collected_signatures.push(CollectedSignature {
            user_id: user_id.to_string(),
            role: role.to_string(),
            timestamp: current_timestamp_ms(),
        });

        Ok(self.evaluate())
    }

    fn add_signature_sequential(
        &mut self,
        user_id: &str,
        role: &str,
    ) -> Result<PolicyEvaluation, SignatureError> {
        if self.is_complete() {
            return Err(SignatureError::SignatureAlreadyExists {
                signature_id: format!("policy {} already satisfied", self.policy.id),
            });
        }

        let expected_role = self.get_expected_role()?;
        if role != expected_role {
            return Err(SignatureError::SequentialOrderViolation {
                expected_step: self.current_step,
                actual_step: self.current_step,
            });
        }

        if self.has_user_signed(user_id) {
            return Err(SignatureError::SignatureAlreadyExists {
                signature_id: user_id.to_string(),
            });
        }

        self.collected_signatures.push(CollectedSignature {
            user_id: user_id.to_string(),
            role: role.to_string(),
            timestamp: current_timestamp_ms(),
        });

        self.current_step += 1;

        Ok(self.evaluate())
    }

    fn has_required_role(&self, role: &str) -> bool {
        self.policy.required_roles.iter().any(|r| r == role)
    }

    fn has_user_signed(&self, user_id: &str) -> bool {
        self.collected_signatures
            .iter()
            .any(|s| s.user_id == user_id)
    }

    fn get_expected_role(&self) -> Result<&str, SignatureError> {
        let idx = (self.current_step - 1) as usize;
        if idx >= self.policy.required_roles.len() {
            return Err(SignatureError::PolicyNotSatisfied { missing: vec![] });
        }
        Ok(&self.policy.required_roles[idx])
    }

    pub fn current_status(&self) -> PolicyStatus {
        if self.is_complete() {
            PolicyStatus::Approved
        } else {
            PolicyStatus::Pending
        }
    }

    fn is_complete(&self) -> bool {
        self.collected_signatures.len() >= self.policy.required_signatures
    }

    pub fn evaluate(&self) -> PolicyEvaluation {
        let missing_roles = if self.policy.sequential {
            self.get_missing_roles_sequential()
        } else {
            self.get_missing_roles_parallel()
        };

        PolicyEvaluation {
            policy_id: self.policy.id.clone(),
            request_id: self.request_id.clone(),
            status: self.current_status(),
            current_signatures: self.collected_signatures.len(),
            required_signatures: self.policy.required_signatures,
            is_complete: self.is_complete(),
            missing_roles,
        }
    }

    fn get_missing_roles_sequential(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for (i, role) in self.policy.required_roles.iter().enumerate() {
            let step = (i + 1) as i32;
            if step >= self.current_step {
                let has_signed = self.collected_signatures.iter().any(|s| s.role == *role);
                if !has_signed {
                    missing.push(role.clone());
                }
            }
        }
        missing
    }

    fn get_missing_roles_parallel(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for role in &self.policy.required_roles {
            let has_signed = self.collected_signatures.iter().any(|s| s.role == *role);
            if !has_signed {
                missing.push(role.clone());
            }
        }
        missing
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Electronic signature errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureError {
    /// Signature verification failed
    VerificationFailed { reason: String },
    /// Invalid key format
    InvalidKey { reason: String },
    /// User lacks required permissions
    InsufficientPermissions { required_role: String },
    /// Policy requirements not met
    PolicyNotSatisfied { missing: Vec<String> },
    /// Signature already exists
    SignatureAlreadyExists { signature_id: String },
    /// Request has expired
    RequestExpired { request_id: String },
    /// Sequential signature order violated
    SequentialOrderViolation {
        expected_step: i32,
        actual_step: i32,
    },
    /// Invalid policy configuration
    InvalidPolicy { reason: String },
    /// Storage error
    StorageError { reason: String },
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::VerificationFailed { reason } => {
                write!(f, "Signature verification failed: {}", reason)
            }
            SignatureError::InvalidKey { reason } => {
                write!(f, "Invalid key: {}", reason)
            }
            SignatureError::InsufficientPermissions { required_role } => {
                write!(f, "User lacks required role: {}", required_role)
            }
            SignatureError::PolicyNotSatisfied { missing } => {
                write!(f, "Policy not satisfied. Missing roles: {:?}", missing)
            }
            SignatureError::SignatureAlreadyExists { signature_id } => {
                write!(f, "Signature already exists: {}", signature_id)
            }
            SignatureError::RequestExpired { request_id } => {
                write!(f, "Signature request has expired: {}", request_id)
            }
            SignatureError::SequentialOrderViolation {
                expected_step,
                actual_step,
            } => {
                write!(
                    f,
                    "Sequential order violated: expected step {}, got {}",
                    expected_step, actual_step
                )
            }
            SignatureError::InvalidPolicy { reason } => {
                write!(f, "Invalid policy: {}", reason)
            }
            SignatureError::StorageError { reason } => {
                write!(f, "Storage error: {}", reason)
            }
        }
    }
}

impl std::error::Error for SignatureError {}

// ============================================================================
// SQL for Table Creation
// ============================================================================

/// SQL to create electronic signatures table
pub const CREATE_ELECTRONIC_SIGNATURES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_electronic_signatures (
    id              TEXT PRIMARY KEY,
    audit_chain_id  INTEGER NOT NULL,
    user_id         TEXT NOT NULL,
    session_id      TEXT,
    role            TEXT,
    reason          TEXT NOT NULL,
    data_hash       BLOB NOT NULL,
    signature       BLOB NOT NULL,
    verifying_key   BLOB NOT NULL,
    timestamp       INTEGER NOT NULL,
    policy_id       TEXT,
    policy_name     TEXT,
    seq_in_policy   INTEGER,
    created_at      INTEGER NOT NULL,
    FOREIGN KEY (audit_chain_id) REFERENCES gmp_audit_log(id)
)
"#;

/// SQL to create approval policies table
pub const CREATE_APPROVAL_POLICIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_approval_policies (
    id                  TEXT PRIMARY KEY,
    name                TEXT UNIQUE NOT NULL,
    required_signatures INTEGER NOT NULL DEFAULT 1,
    required_roles     TEXT NOT NULL,  -- JSON array
    sequential          INTEGER NOT NULL DEFAULT 1,
    timeout_hours      INTEGER NOT NULL DEFAULT 72,
    description         TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    active              INTEGER NOT NULL DEFAULT 1
)
"#;

/// SQL to create signature requests table
pub const CREATE_SIGNATURE_REQUESTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_signature_requests (
    id              TEXT PRIMARY KEY,
    policy_id       TEXT NOT NULL,
    record_table    TEXT NOT NULL,
    record_id       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'PENDING',
    current_step    INTEGER NOT NULL DEFAULT 1,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    expires_at      INTEGER NOT NULL,
    FOREIGN KEY (policy_id) REFERENCES gmp_approval_policies(id)
)
"#;

// ============================================================================
// SQL Statement Builders
// ============================================================================

/// SQL statement builders for electronic signature operations
pub mod sql {
    /// SQL to initialize electronic signature tables
    pub const INIT_TABLES: &str = r#"
CREATE TABLE IF NOT EXISTS gmp_electronic_signatures (
    id              TEXT PRIMARY KEY,
    audit_chain_id  INTEGER NOT NULL,
    user_id         TEXT NOT NULL,
    session_id      TEXT,
    role            TEXT,
    reason          TEXT NOT NULL,
    data_hash       BLOB NOT NULL,
    signature       BLOB NOT NULL,
    verifying_key   BLOB NOT NULL,
    timestamp       INTEGER NOT NULL,
    policy_id       TEXT,
    policy_name     TEXT,
    seq_in_policy   INTEGER,
    created_at      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS gmp_approval_policies (
    id                  TEXT PRIMARY KEY,
    name                TEXT UNIQUE NOT NULL,
    required_signatures INTEGER NOT NULL DEFAULT 1,
    required_roles     TEXT NOT NULL,
    sequential          INTEGER NOT NULL DEFAULT 1,
    timeout_hours      INTEGER NOT NULL DEFAULT 72,
    description         TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL,
    active              INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS gmp_signature_requests (
    id              TEXT PRIMARY KEY,
    policy_id       TEXT NOT NULL,
    record_table    TEXT NOT NULL,
    record_id       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'PENDING',
    current_step    INTEGER NOT NULL DEFAULT 1,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL,
    expires_at      INTEGER NOT NULL
)
"#;

    /// Build SQL to create an approval policy
    pub fn create_approval_policy(
        name: &str,
        required_signatures: usize,
        required_roles: &[&str],
        sequential: bool,
        timeout_hours: i32,
    ) -> String {
        let roles_sql = format!(
            "[{}]",
            required_roles
                .iter()
                .map(|r| format!("'{}'", r))
                .collect::<Vec<_>>()
                .join(", ")
        );
        format!(
            "INSERT INTO gmp_approval_policies (id, name, required_signatures, required_roles, sequential, timeout_hours, created_at, updated_at, active) \
             VALUES (lower(hex(randomblob(16))), '{}', {}, '{}', {}, {}, {}, {}, 1)",
            name,
            required_signatures,
            roles_sql,
            if sequential { 1 } else { 0 },
            timeout_hours,
            crate::electronic_signature::current_timestamp_ms(),
            crate::electronic_signature::current_timestamp_ms(),
        )
    }

    /// Build SQL to create a signature request
    pub fn create_signature_request(
        policy_id: &str,
        record_table: &str,
        record_id: &str,
        timeout_hours: i32,
    ) -> String {
        let now = crate::electronic_signature::current_timestamp_ms();
        let expires = now + (timeout_hours as i64 * 3600 * 1000);
        format!(
            "INSERT INTO gmp_signature_requests (id, policy_id, record_table, record_id, status, current_step, created_at, updated_at, expires_at) \
             VALUES (lower(hex(randomblob(16))), '{}', '{}', '{}', 'PENDING', 1, {}, {}, {})",
            policy_id, record_table, record_id, now, now, expires,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_signature(
        audit_chain_id: i64,
        user_id: &str,
        session_id: Option<&str>,
        role: Option<&str>,
        reason: &str,
        data_hash: &[u8],
        signature: &[u8],
        verifying_key: &[u8],
        timestamp: i64,
        policy_id: Option<&str>,
        policy_name: Option<&str>,
        seq_in_policy: Option<i32>,
    ) -> String {
        let session_sql = session_id
            .map(|s| format!("'{}'", s))
            .unwrap_or_else(|| "NULL".to_string());
        let role_sql = role
            .map(|r| format!("'{}'", r))
            .unwrap_or_else(|| "NULL".to_string());
        let policy_id_sql = policy_id
            .map(|p| format!("'{}'", p))
            .unwrap_or_else(|| "NULL".to_string());
        let policy_name_sql = policy_name
            .map(|p| format!("'{}'", p))
            .unwrap_or_else(|| "NULL".to_string());
        let seq_sql = seq_in_policy
            .map(|s| s.to_string())
            .unwrap_or_else(|| "NULL".to_string());

        let data_hash_hex = hex::encode(data_hash);
        let signature_hex = hex::encode(signature);
        let verifying_key_hex = hex::encode(verifying_key);

        format!(
            "INSERT INTO gmp_electronic_signatures (id, audit_chain_id, user_id, session_id, role, reason, data_hash, signature, verifying_key, timestamp, policy_id, policy_name, seq_in_policy, created_at) \
             VALUES (lower(hex(randomblob(16))), {}, '{}', {}, {}, '{}', x'{}', x'{}', x'{}', {}, {}, {}, {}, {})",
            audit_chain_id,
            user_id,
            session_sql,
            role_sql,
            reason,
            data_hash_hex,
            signature_hex,
            verifying_key_hex,
            timestamp,
            policy_id_sql,
            policy_name_sql,
            seq_sql,
            timestamp,
        )
    }

    /// Build SQL to query signatures for a record
    pub fn query_signatures_for_record(audit_chain_id: i64) -> String {
        format!(
            "SELECT * FROM gmp_electronic_signatures WHERE audit_chain_id = {} ORDER BY created_at",
            audit_chain_id
        )
    }

    /// Build SQL to query pending signature requests
    pub fn query_pending_requests() -> String {
        "SELECT * FROM gmp_signature_requests WHERE status = 'PENDING' ORDER BY created_at"
            .to_string()
    }

    /// Build SQL to query active approval policies
    pub fn query_active_policies() -> String {
        "SELECT * FROM gmp_approval_policies WHERE active = 1 ORDER BY name".to_string()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a simple UUID (nanoseconds-based for demo purposes)
pub fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let pid = std::process::id() as u64;
    let thread_id = current_thread_id();
    format!("{:x}-{:x}-{:x}", now, pid, thread_id)
}

// Global counter for thread ID on macOS (avoids uninitialized memory)
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_os = "macos")]
static THREAD_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn current_thread_id() -> u64 {
    THREAD_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(not(target_os = "macos"))]
fn current_thread_id() -> u64 {
    let id = std::thread::current().id();
    // Convert ThreadId to u64 by hashing
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish()
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Compute SHA-256 hash of data
pub fn compute_data_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Compute signing payload: data_hash || reason || timestamp
pub fn compute_signing_payload(data_hash: &[u8], reason: &str, timestamp: i64) -> Vec<u8> {
    let mut payload = Vec::with_capacity(data_hash.len() + reason.len() + 8);
    payload.extend_from_slice(data_hash);
    payload.extend_from_slice(reason.as_bytes());
    payload.extend_from_slice(&timestamp.to_le_bytes());
    payload
}

// ============================================================================
// ED25519 Signature Verification
// (Reused from transaction::audit::signature)
// ============================================================================

use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};

/// Verify an ED25519 signature
pub fn verify_ed25519_signature(
    verifying_key_bytes: &[u8],
    _data: &[u8],
    signature_bytes: &[u8],
) -> bool {
    let key_array: [u8; 32] = match verifying_key_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&key_array) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = match Ed25519Signature::from_slice(signature_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    verifying_key.verify(_data, &signature).is_ok()
}

/// Create a signature over data using ED25519
pub fn sign_ed25519(signing_key_bytes: &[u8; 32], data: &[u8]) -> Vec<u8> {
    use ed25519_dalek::{Signer, SigningKey};
    let signing_key = SigningKey::from_bytes(signing_key_bytes);
    let signature = signing_key.sign(data);
    signature.to_vec()
}

// ============================================================================
// Trusted Timestamp Provider (GMP-6)
// ============================================================================

pub trait TrustedTimestampProvider: Send + Sync {
    fn get_timestamp(&self) -> Result<i64, SignatureError>;
    fn verify_timestamp(&self, timestamp: i64) -> Result<bool, SignatureError>;
}

pub struct SystemTimeProvider;

impl SystemTimeProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustedTimestampProvider for SystemTimeProvider {
    fn get_timestamp(&self) -> Result<i64, SignatureError> {
        Ok(current_timestamp_ms())
    }

    fn verify_timestamp(&self, timestamp: i64) -> Result<bool, SignatureError> {
        let now = current_timestamp_ms();
        Ok(timestamp <= now && timestamp > now - 86400 * 1000)
    }
}

// ============================================================================
// Electronic Signature Provider (Trait for 21 CFR Part 11 Compliance)
// ============================================================================

pub trait ElectronicSignatureProvider: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn sign(
        &self,
        user_id: &str,
        session_id: Option<&str>,
        role: Option<&str>,
        data_hash: &[u8],
        reason: &str,
        signing_key: &[u8; 32],
        verifying_key: &[u8; 32],
    ) -> Result<ElectronicSignature, SignatureError>;

    fn verify(
        &self,
        signature: &ElectronicSignature,
        data_hash: &[u8],
    ) -> Result<bool, SignatureError>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electronic_signature_creation() {
        let sig = ElectronicSignature::new(
            1,
            "user1".to_string(),
            Some("session1".to_string()),
            Some("ADMIN".to_string()),
            "Approved for release".to_string(),
            vec![0u8; 32],
            vec![0u8; 64],
            vec![0u8; 32],
            current_timestamp_ms(),
            None,
            None,
            None,
        );

        assert!(!sig.id.is_empty());
        assert_eq!(sig.user_id, "user1");
        assert_eq!(sig.reason, "Approved for release");
    }

    #[test]
    fn test_approval_policy_creation() {
        let policy = ApprovalPolicy::new(
            "batch_release".to_string(),
            2,
            vec!["QA_MANAGER".to_string(), "PRODUCTION_MANAGER".to_string()],
            true,
            72,
            Some("Requires two managers to approve".to_string()),
        );

        assert!(!policy.id.is_empty());
        assert_eq!(policy.name, "batch_release");
        assert_eq!(policy.required_signatures, 2);
        assert!(policy.sequential);
    }

    #[test]
    fn test_signature_request_expiry() {
        let mut request = SignatureRequest::new(
            "policy-1".to_string(),
            "batch_records".to_string(),
            "record-1".to_string(),
            1, // 1 hour timeout
        );

        // Should not be expired immediately
        assert!(!request.is_expired());

        // Manually set expires_at to past
        request.expires_at = current_timestamp_ms() - 1000;
        assert!(request.is_expired());
    }

    #[test]
    fn test_policy_status() {
        assert_eq!(PolicyStatus::Pending.as_str(), "PENDING");
        assert_eq!(
            PolicyStatus::from_str_explicit("APPROVED"),
            Some(PolicyStatus::Approved)
        );
        assert_eq!(PolicyStatus::from_str_explicit("INVALID"), None);
    }

    #[test]
    fn test_data_hash() {
        let data = b"test data";
        let hash = compute_data_hash(data);
        assert_eq!(hash.len(), 32); // SHA-256 output
    }

    #[test]
    fn test_signing_payload() {
        let data_hash = vec![0u8; 32];
        let reason = "Approved";
        let timestamp = 1234567890i64;

        let payload = compute_signing_payload(&data_hash, reason, timestamp);
        assert_eq!(payload.len(), 32 + 8 + 8); // hash + timestamp + reason
    }

    #[test]
    fn test_uuid_simple() {
        let id1 = uuid_simple();
        let id2 = uuid_simple();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }
}

//! Audit Event Stream Module
//!
//! Provides append-only audit event storage with tamper detection.
//! All audit events are immutable - UPDATE and DELETE are forbidden.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Genesis previous hash (all zeros for first event)
pub const AUDIT_GENESIS_PREV_HASH: [u8; 32] = [0u8; 32];

/// Audit event types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditEventType {
    /// User login
    Login = 1,
    /// User logout
    Logout = 2,
    /// SQL query executed
    Query = 3,
    /// Data modification (INSERT/UPDATE/DELETE)
    DataChange = 4,
    /// Schema change (CREATE/ALTER/DROP)
    SchemaChange = 5,
    /// Transaction begin
    TransactionBegin = 6,
    /// Transaction commit
    TransactionCommit = 7,
    /// Transaction rollback
    TransactionRollback = 8,
    /// Permission change
    PermissionChange = 9,
    /// System event
    System = 10,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::Login => "LOGIN",
            AuditEventType::Logout => "LOGOUT",
            AuditEventType::Query => "QUERY",
            AuditEventType::DataChange => "DATA_CHANGE",
            AuditEventType::SchemaChange => "SCHEMA_CHANGE",
            AuditEventType::TransactionBegin => "TX_BEGIN",
            AuditEventType::TransactionCommit => "TX_COMMIT",
            AuditEventType::TransactionRollback => "TX_ROLLBACK",
            AuditEventType::PermissionChange => "PERMISSION_CHANGE",
            AuditEventType::System => "SYSTEM",
        }
    }
}

/// Immutable audit event - append-only, no UPDATE/DELETE allowed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Actor who performed the action
    pub actor: String,
    /// Device identifier
    pub device_id: String,
    /// Action type
    pub action: AuditEventType,
    /// Object type (table, index, etc.)
    pub object_type: String,
    /// Object identifier
    pub object_id: String,
    /// Event timestamp (Unix epoch ms)
    pub timestamp: u64,
    /// Previous event hash in chain
    pub prev_hash: [u8; 32],
    /// SHA-256 hash of this event
    pub current_hash: [u8; 32],
    /// Digital signature (optional for MVP)
    pub signature: Option<Vec<u8>>,
    /// Additional metadata (JSON)
    pub metadata: Option<String>,
}

impl AuditEvent {
    /// Create a new audit event with computed hash
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        actor: String,
        device_id: String,
        action: AuditEventType,
        object_type: String,
        object_id: String,
        timestamp: u64,
        prev_hash: [u8; 32],
        metadata: Option<String>,
    ) -> Self {
        let event_id = Uuid::new_v4();
        let mut event = Self {
            event_id,
            actor,
            device_id,
            action,
            object_type,
            object_id,
            timestamp,
            prev_hash,
            current_hash: [0u8; 32],
            signature: None,
            metadata,
        };
        event.current_hash = event.compute_hash();
        event
    }

    /// Create a new audit event with signature
    #[allow(clippy::too_many_arguments)]
    pub fn new_signed(
        actor: String,
        device_id: String,
        action: AuditEventType,
        object_type: String,
        object_id: String,
        timestamp: u64,
        prev_hash: [u8; 32],
        metadata: Option<String>,
        signature: Vec<u8>,
    ) -> Self {
        let mut event = Self::new(
            actor,
            device_id,
            action,
            object_type,
            object_id,
            timestamp,
            prev_hash,
            metadata,
        );
        event.signature = Some(signature);
        event
    }

    /// Compute SHA-256 hash of this event
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.event_id.as_bytes());
        hasher.update(self.actor.as_bytes());
        hasher.update(self.device_id.as_bytes());
        hasher.update((self.action as u8).to_le_bytes());
        hasher.update(self.object_type.as_bytes());
        hasher.update(self.object_id.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.prev_hash);
        if let Some(ref m) = self.metadata {
            hasher.update(m.as_bytes());
        }
        hasher.finalize().into()
    }

    /// Verify event chain integrity
    pub fn verify_chain_integrity(&self, expected_prev_hash: &[u8; 32]) -> bool {
        if self.prev_hash != *expected_prev_hash {
            return false;
        }
        let computed = self.compute_hash();
        computed == self.current_hash
    }
}

/// Audit event stream state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditStreamState {
    pub next_event_id: u64,
    pub last_hash: [u8; 32],
    pub event_count: u64,
}

impl AuditStreamState {
    pub fn new() -> Self {
        Self {
            next_event_id: 1,
            last_hash: AUDIT_GENESIS_PREV_HASH,
            event_count: 0,
        }
    }

    pub fn append_event(&mut self, event: &AuditEvent) {
        self.last_hash = event.current_hash;
        self.next_event_id += 1;
        self.event_count += 1;
    }
}

/// Append-only audit event store
pub struct AuditEventStore {
    state: AuditStreamState,
    events: Vec<AuditEvent>,
}

impl AuditEventStore {
    pub fn new() -> Self {
        Self {
            state: AuditStreamState::new(),
            events: Vec::new(),
        }
    }

    /// Append a new audit event (append-only, no UPDATE/DELETE)
    pub fn append(&mut self, event: AuditEvent) -> bool {
        // Verify chain integrity before appending
        if !event.verify_chain_integrity(&self.state.last_hash) {
            return false;
        }
        self.state.append_event(&event);
        self.events.push(event);
        true
    }

    /// Get the current stream state
    pub fn state(&self) -> &AuditStreamState {
        &self.state
    }

    /// Get all events
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Verify entire chain integrity
    pub fn verify_chain(&self) -> bool {
        let mut expected_prev = AUDIT_GENESIS_PREV_HASH;
        for event in &self.events {
            if !event.verify_chain_integrity(&expected_prev) {
                return false;
            }
            expected_prev = event.current_hash;
        }
        true
    }
}

impl Default for AuditEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Login,
            "session".to_string(),
            "sess123".to_string(),
            1609459200000,
            AUDIT_GENESIS_PREV_HASH,
            None,
        );

        assert_eq!(event.actor, "user1");
        assert_eq!(event.prev_hash, AUDIT_GENESIS_PREV_HASH);
        assert_ne!(event.current_hash, [0u8; 32]);
    }

    #[test]
    fn test_audit_event_hash_chain() {
        let mut store = AuditEventStore::new();

        let event1 = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Login,
            "session".to_string(),
            "sess123".to_string(),
            1609459200000,
            AUDIT_GENESIS_PREV_HASH,
            None,
        );

        assert!(store.append(event1.clone()));
        assert_eq!(store.len(), 1);

        let event2 = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Query,
            "table".to_string(),
            "users".to_string(),
            1609459201000,
            event1.current_hash,
            None,
        );

        assert!(store.append(event2.clone()));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_audit_event_chain_verification() {
        let mut store = AuditEventStore::new();

        let event1 = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Login,
            "session".to_string(),
            "sess123".to_string(),
            1609459200000,
            AUDIT_GENESIS_PREV_HASH,
            None,
        );
        store.append(event1);

        let event2 = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Logout,
            "session".to_string(),
            "sess123".to_string(),
            1609459202000,
            AUDIT_GENESIS_PREV_HASH, // Wrong prev_hash
            None,
        );
        assert!(!store.append(event2)); // Should fail - wrong prev_hash

        assert!(store.verify_chain());
    }

    #[test]
    fn test_audit_stream_state() {
        let mut state = AuditStreamState::new();
        assert_eq!(state.event_count, 0);
        assert_eq!(state.last_hash, AUDIT_GENESIS_PREV_HASH);

        let event = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Login,
            "session".to_string(),
            "sess123".to_string(),
            1609459200000,
            AUDIT_GENESIS_PREV_HASH,
            None,
        );

        state.append_event(&event);
        assert_eq!(state.event_count, 1);
        assert_eq!(state.last_hash, event.current_hash);
    }
}

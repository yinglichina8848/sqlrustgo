//! Audit Verification Module
//!
//! Provides replay verification for audit chain integrity.

use super::event_stream::{AuditEvent, AuditEventStore, AUDIT_GENESIS_PREV_HASH};
use super::hash_chain::HashChain;

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub events_verified: u64,
    pub first_invalid_event: Option<u64>,
    pub error_message: Option<String>,
}

impl VerificationResult {
    pub fn success(events_verified: u64) -> Self {
        Self {
            is_valid: true,
            events_verified,
            first_invalid_event: None,
            error_message: None,
        }
    }

    pub fn failure(event_index: u64, message: &str) -> Self {
        Self {
            is_valid: false,
            events_verified: 0,
            first_invalid_event: Some(event_index),
            error_message: Some(message.to_string()),
        }
    }
}

pub struct AuditVerifier {
    expected_prev_hash: [u8; 32],
}

impl AuditVerifier {
    pub fn new() -> Self {
        Self {
            expected_prev_hash: AUDIT_GENESIS_PREV_HASH,
        }
    }

    pub fn verify_event(&mut self, event: &AuditEvent) -> bool {
        if !event.verify_chain_integrity(&self.expected_prev_hash) {
            return false;
        }
        self.expected_prev_hash = event.current_hash;
        true
    }

    pub fn verify_store(&mut self, store: &AuditEventStore) -> VerificationResult {
        if store.is_empty() {
            return VerificationResult::success(0);
        }

        for (i, event) in store.events().iter().enumerate() {
            if !event.verify_chain_integrity(&self.expected_prev_hash) {
                return VerificationResult::failure(
                    i as u64,
                    &format!("Chain broken at event {}", i),
                );
            }
            self.expected_prev_hash = event.current_hash;
        }

        VerificationResult::success(store.len() as u64)
    }

    pub fn verify_chain(&self, events: &[AuditEvent]) -> VerificationResult {
        if events.is_empty() {
            return VerificationResult::success(0);
        }

        let mut expected_prev = AUDIT_GENESIS_PREV_HASH;
        for (i, event) in events.iter().enumerate() {
            if !event.verify_chain_integrity(&expected_prev) {
                return VerificationResult::failure(
                    i as u64,
                    &format!("Chain broken at event {}", i),
                );
            }
            expected_prev = event.current_hash;
        }

        VerificationResult::success(events.len() as u64)
    }

    pub fn reset(&mut self) {
        self.expected_prev_hash = AUDIT_GENESIS_PREV_HASH;
    }
}

impl Default for AuditVerifier {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AuditRecoveryVerifier {
    chain: HashChain,
}

impl AuditRecoveryVerifier {
    pub fn new() -> Self {
        Self {
            chain: HashChain::new(),
        }
    }

    pub fn verify_recovery(&self, events: &[AuditEvent]) -> bool {
        if events.is_empty() {
            return true;
        }

        let verifier = AuditVerifier::new();
        let result = verifier.verify_chain(events);
        result.is_valid
    }

    pub fn chain(&self) -> &HashChain {
        &self.chain
    }
}

impl Default for AuditRecoveryVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::event_stream::AuditEventType;
    use super::*;

    #[test]
    fn test_audit_verifier_event() {
        let mut verifier = AuditVerifier::new();

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

        assert!(verifier.verify_event(&event1));

        let event2 = AuditEvent::new(
            "user1".to_string(),
            "device1".to_string(),
            AuditEventType::Logout,
            "session".to_string(),
            "sess123".to_string(),
            1609459201000,
            event1.current_hash,
            None,
        );

        assert!(verifier.verify_event(&event2));
    }

    #[test]
    fn test_audit_verifier_store() {
        let mut store = AuditEventStore::new();
        let mut verifier = AuditVerifier::new();

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

        let result = verifier.verify_store(&store);
        assert!(result.is_valid);
        assert_eq!(result.events_verified, 1);
    }

    #[test]
    fn test_verification_result() {
        let success = VerificationResult::success(10);
        assert!(success.is_valid);
        assert_eq!(success.events_verified, 10);
        assert!(success.first_invalid_event.is_none());

        let failure = VerificationResult::failure(5, "Chain broken");
        assert!(!failure.is_valid);
        assert_eq!(failure.first_invalid_event, Some(5));
        assert!(failure.error_message.is_some());
    }

    #[test]
    fn test_audit_recovery_verifier() {
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
            AuditEventType::Query,
            "table".to_string(),
            "users".to_string(),
            1609459201000,
            store.events()[0].current_hash,
            None,
        );
        store.append(event2);

        let verifier = AuditRecoveryVerifier::new();
        assert!(verifier.verify_recovery(store.events()));
    }
}

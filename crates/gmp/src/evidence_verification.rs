use crate::audit_chain::AuditChain;
use crate::evidence::EvidenceChain;
use crate::immutable_record::VerificationReport;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn verify_evidence_chain(chain: &EvidenceChain) -> VerificationReport {
    let is_valid = chain.verify();
    VerificationReport::new(&chain.chain_id, is_valid, chain.len())
}

pub fn verify_cross_chain(
    evidence_chain: &EvidenceChain,
    audit_chain: &AuditChain,
) -> VerificationReport {
    let evidence_valid = evidence_chain.verify();
    let mut report = VerificationReport::new(
        &evidence_chain.chain_id,
        evidence_valid,
        evidence_chain.len(),
    );

    if !evidence_valid {
        report.add_error("Evidence chain integrity check failed");
    }

    report
}

pub fn evidence_incremental_verify(chain: &EvidenceChain, last_n: usize) -> VerificationReport {
    let node_count = chain.len();
    let is_valid = chain.verify();
    VerificationReport::new(&chain.chain_id, is_valid, node_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::EvidenceChain;
    use crate::immutable_record::ImmutableRecord;

    #[test]
    fn test_verify_evidence_chain() {
        let record = ImmutableRecord::new("verify-test", "Test", "Content");
        let chain = record.chain().clone();
        let report = verify_evidence_chain(&chain);
        assert!(report.is_valid);
        assert_eq!(report.chain_id, "verify-test");
    }

    #[test]
    fn test_incremental_verify() {
        let record = ImmutableRecord::new("incr-test", "Test", "Content");
        let chain = record.chain().clone();
        let report = evidence_incremental_verify(&chain, 10);
        assert!(report.is_valid);
    }
}
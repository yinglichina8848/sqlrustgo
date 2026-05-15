use crate::evidence::EvidenceChain;
use crate::evidence_storage::{
    get_evidence_by_chain_id, get_evidence_by_time_range,
    save_evidence_chain,
};
use crate::evidence_verification::verify_evidence_chain;
use crate::immutable_record::{ImmutableRecord, VerificationReport};
use sqlrustgo_storage::StorageEngine;
use sqlrustgo_types::SqlResult;

pub fn create_evidence(chain_id: &str, description: &str, content: &str) -> ImmutableRecord {
    ImmutableRecord::new(chain_id, description, content)
}

pub fn get_evidence(storage: &dyn StorageEngine, chain_id: &str) -> SqlResult<Vec<EvidenceChain>> {
    get_evidence_by_chain_id(storage, chain_id)
}

pub fn list_evidence(
    storage: &dyn StorageEngine,
    from_timestamp: i64,
    to_timestamp: i64,
) -> SqlResult<Vec<EvidenceChain>> {
    get_evidence_by_time_range(storage, from_timestamp, to_timestamp)
}

pub fn verify_evidence(chain: &EvidenceChain) -> VerificationReport {
    verify_evidence_chain(chain)
}

pub fn create_signed_evidence(
    storage: &mut dyn StorageEngine,
    chain_id: &str,
    description: &str,
    content: &str,
    _signature: &str,
) -> SqlResult<ImmutableRecord> {
    let mut record = ImmutableRecord::new(chain_id, description, content);
    record.add_document("signed", &format!("Content signed with: {}", _signature));
    let chain = record.chain().clone();
    save_evidence_chain(storage, &chain)?;
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_evidence() {
        let record = create_evidence("test-api", "API Test", "Content");
        assert_eq!(record.chain_id(), "test-api");
    }

    #[test]
    fn test_verify_evidence() {
        let record = create_evidence("verify-api", "Verify Test", "Content");
        let chain = record.chain().clone();
        let report = verify_evidence(&chain);
        assert!(report.is_valid);
    }
}

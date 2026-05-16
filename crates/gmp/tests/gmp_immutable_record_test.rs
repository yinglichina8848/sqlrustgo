#[cfg(test)]
mod tests {
    use sqlrustgo_gmp::immutable_record::{ImmutableRecord, ImmutableRecordBuilder, VerificationReport};

    #[test]
    fn test_immutable_record_creation() {
        let record = ImmutableRecord::new("REC001", "test record", "initial content");
        assert_eq!(record.chain_id(), "REC001");
        assert!(!record.is_empty());
    }

    #[test]
    fn test_immutable_record_verify() {
        let record = ImmutableRecord::new("verify", "verification test", "content");
        assert!(record.verify());
    }

    #[test]
    fn test_immutable_record_builder() {
        let record = ImmutableRecordBuilder::new("builder-test", "Builder test")
            .with_creator("user123")
            .with_operation("CREATE")
            .build("Initial content");

        assert_eq!(record.chain_id(), "builder-test");
        assert!(record.verify());
    }

    #[test]
    fn test_immutable_record_add_document() {
        let mut record = ImmutableRecord::new("add-doc", "add document test", "initial");
        record.add_document("doc1", "new document content");
        assert_eq!(record.len(), 2);
    }

    #[test]
    fn test_verification_report() {
        let mut report = VerificationReport::new("test-chain", true, 5);
        assert!(report.is_valid);
        assert_eq!(report.node_count, 5);
        report.add_error("Test error");
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_immutable_record_default() {
        let record = ImmutableRecord::default();
        assert!(record.verify());
    }
}
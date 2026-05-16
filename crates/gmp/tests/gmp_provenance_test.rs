#[cfg(test)]
mod tests {
    use sqlrustgo_gmp::provenance::{OperationType, ProvenanceRecord, SourceType};
    use sqlrustgo_gmp::provenance_lineage::LineageGraph;

    #[test]
    fn test_provenance_record_creation() {
        let record = ProvenanceRecord::new(
            "record001".to_string(),
            SourceType::Derived,
            Some("source_record".to_string()),
            "creator1".to_string(),
            1234567890,
            OperationType::Create,
            vec![],
        );

        assert_eq!(record.record_id, "record001");
        assert_eq!(record.source_type, SourceType::Derived);
        assert!(record.verify_integrity());
    }

    #[test]
    fn test_lineage_graph_new() {
        let graph = LineageGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
    }

    #[test]
    fn test_lineage_graph_add_record() {
        let mut graph = LineageGraph::new();
        
        let record = ProvenanceRecord::new(
            "record001".to_string(),
            SourceType::Insert,
            None,
            "creator1".to_string(),
            1234567890,
            OperationType::Create,
            vec![],
        );
        
        graph.add_record(&record);
        assert!(!graph.is_empty());
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_lineage_graph_verify_integrity() {
        let mut graph = LineageGraph::new();
        
        let record = ProvenanceRecord::new(
            "record001".to_string(),
            SourceType::Insert,
            None,
            "creator1".to_string(),
            1234567890,
            OperationType::Create,
            vec![],
        );
        
        graph.add_record(&record);
        assert!(graph.verify_integrity());
    }
}

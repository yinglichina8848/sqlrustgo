//! Transaction Timeout Tests
//!
//! P2 tests for transaction timeout per TEST_PLAN.md
//! Tests transaction timeout and handling

#[cfg(test)]
mod tests {
    use sqlrustgo_transaction::deadlock::DeadlockDetector;
    use sqlrustgo_transaction::mvcc::TxId;
    use std::time::Duration;

    #[test]
    fn test_deadlock_timeout() {
        let detector = DeadlockDetector::with_timeout(Duration::from_secs(10));

        assert_eq!(detector.get_timeout(), Duration::from_secs(10));
    }

    #[test]
    fn test_deadlock_detection_with_wait() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx2, tx3);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none(), "No cycle yet");

        detector.add_edge(tx3, tx1);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_some(), "Cycle should be detected");
    }

    #[test]
    fn test_transaction_timeout_no_conflict() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);

        detector.add_edge(tx1, tx2);

        let cycle = detector.detect_cycle(tx1);

        assert!(cycle.is_none(), "No cycle with two transactions");
    }

    #[test]
    fn test_remove_edges_after_completion() {
        let mut detector = DeadlockDetector::new();

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        detector.add_edge(tx1, tx2);
        detector.add_edge(tx2, tx3);

        detector.remove_edges_for(tx2);

        let cycle = detector.detect_cycle(tx1);

        assert!(
            cycle.is_none(),
            "Cycle should be broken after removing edges"
        );
    }
}

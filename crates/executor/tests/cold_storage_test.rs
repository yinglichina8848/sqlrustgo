#[cfg(test)]
mod tests {
    use sqlrustgo_storage::storage_tier::StorageTier;

    #[test]
    fn test_storage_tier_order() {
        assert!(StorageTier::Hot < StorageTier::Warm);
        assert!(StorageTier::Warm < StorageTier::Cold);
        assert!(StorageTier::Cold < StorageTier::Archive);
    }

    #[test]
    fn test_tier_promotion() {
        assert!(StorageTier::Hot.can_promote_to(StorageTier::Warm));
        assert!(StorageTier::Warm.can_promote_to(StorageTier::Cold));
        assert!(StorageTier::Cold.can_promote_to(StorageTier::Archive));
        assert!(!StorageTier::Archive.can_promote_to(StorageTier::Hot));
    }

    #[test]
    fn test_tier_demotion() {
        assert!(StorageTier::Archive.can_demote_to(StorageTier::Cold));
        assert!(StorageTier::Cold.can_demote_to(StorageTier::Warm));
        assert!(StorageTier::Warm.can_demote_to(StorageTier::Hot));
        assert!(!StorageTier::Hot.can_demote_to(StorageTier::Archive));
    }

    #[test]
    fn test_tier_next_prev() {
        assert_eq!(StorageTier::Hot.next_tier(), Some(StorageTier::Warm));
        assert_eq!(StorageTier::Warm.next_tier(), Some(StorageTier::Cold));
        assert_eq!(StorageTier::Cold.next_tier(), Some(StorageTier::Archive));
        assert_eq!(StorageTier::Archive.next_tier(), None);

        assert_eq!(StorageTier::Hot.prev_tier(), None);
        assert_eq!(StorageTier::Warm.prev_tier(), Some(StorageTier::Hot));
        assert_eq!(StorageTier::Cold.prev_tier(), Some(StorageTier::Warm));
        assert_eq!(StorageTier::Archive.prev_tier(), Some(StorageTier::Cold));
    }

    #[test]
    fn test_tier_name_and_description() {
        assert_eq!(StorageTier::Hot.name(), "Hot");
        assert_eq!(StorageTier::Cold.name(), "Cold");
        assert_eq!(StorageTier::Hot.description(), "Memory BufferPool");
        assert_eq!(StorageTier::Cold.description(), "S3/Archive Storage");
    }
}

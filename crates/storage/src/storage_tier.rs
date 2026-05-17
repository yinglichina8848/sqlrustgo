use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum StorageTier {
    #[default]
    Hot,
    Warm,
    Cold,
    Archive,
}

impl StorageTier {
    pub fn name(&self) -> &'static str {
        match self {
            StorageTier::Hot => "Hot",
            StorageTier::Warm => "Warm",
            StorageTier::Cold => "Cold",
            StorageTier::Archive => "Archive",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            StorageTier::Hot => "Memory BufferPool",
            StorageTier::Warm => "SSD/Local Fast Storage",
            StorageTier::Cold => "S3/Archive Storage",
            StorageTier::Archive => "Glacier/Long-term Retention",
        }
    }

    pub fn can_promote_to(&self, target: StorageTier) -> bool {
        (*self as u8) < (target as u8)
    }

    pub fn can_demote_to(&self, target: StorageTier) -> bool {
        (*self as u8) > (target as u8)
    }

    pub fn next_tier(&self) -> Option<StorageTier> {
        match self {
            StorageTier::Hot => Some(StorageTier::Warm),
            StorageTier::Warm => Some(StorageTier::Cold),
            StorageTier::Cold => Some(StorageTier::Archive),
            StorageTier::Archive => None,
        }
    }

    pub fn prev_tier(&self) -> Option<StorageTier> {
        match self {
            StorageTier::Hot => None,
            StorageTier::Warm => Some(StorageTier::Hot),
            StorageTier::Cold => Some(StorageTier::Warm),
            StorageTier::Archive => Some(StorageTier::Cold),
        }
    }
}

impl std::fmt::Display for StorageTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_tier_navigation() {
        assert_eq!(StorageTier::Hot.next_tier(), Some(StorageTier::Warm));
        assert_eq!(StorageTier::Warm.prev_tier(), Some(StorageTier::Hot));
        assert_eq!(StorageTier::Archive.next_tier(), None);
        assert_eq!(StorageTier::Hot.prev_tier(), None);
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadCommitted,
    ReadUncommitted,
    SnapshotIsolation,
    #[default]
    Serializable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatement {
    Begin {
        work: bool,
        isolation_level: Option<IsolationLevel>,
    },
    Commit {
        work: bool,
    },
    Rollback {
        work: bool,
    },
    SetTransaction {
        isolation_level: IsolationLevel,
    },
    StartTransaction {
        isolation_level: Option<IsolationLevel>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_level_default() {
        let level = IsolationLevel::default();
        assert_eq!(level, IsolationLevel::Serializable);
    }

    #[test]
    fn test_isolation_level_variants() {
        assert_ne!(IsolationLevel::ReadCommitted, IsolationLevel::Serializable);
        assert_ne!(
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted
        );
        assert_ne!(
            IsolationLevel::SnapshotIsolation,
            IsolationLevel::Serializable
        );
    }
}

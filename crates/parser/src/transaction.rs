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

    #[test]
    fn test_transaction_statement_begin() {
        let stmt = TransactionStatement::Begin {
            work: false,
            isolation_level: Some(IsolationLevel::ReadCommitted),
        };
        match stmt {
            TransactionStatement::Begin {
                work,
                isolation_level,
            } => {
                assert!(!work);
                assert_eq!(isolation_level, Some(IsolationLevel::ReadCommitted));
            }
            _ => panic!("Expected Begin"),
        }
    }

    #[test]
    fn test_transaction_statement_commit() {
        let stmt = TransactionStatement::Commit { work: true };
        match stmt {
            TransactionStatement::Commit { work } => assert!(work),
            _ => panic!("Expected Commit"),
        }
    }

    #[test]
    fn test_transaction_statement_rollback() {
        let stmt = TransactionStatement::Rollback { work: false };
        match stmt {
            TransactionStatement::Rollback { work } => assert!(!work),
            _ => panic!("Expected Rollback"),
        }
    }

    #[test]
    fn test_transaction_statement_set_transaction() {
        let stmt = TransactionStatement::SetTransaction {
            isolation_level: IsolationLevel::SnapshotIsolation,
        };
        match stmt {
            TransactionStatement::SetTransaction { isolation_level } => {
                assert_eq!(isolation_level, IsolationLevel::SnapshotIsolation);
            }
            _ => panic!("Expected SetTransaction"),
        }
    }

    #[test]
    fn test_transaction_statement_start_transaction() {
        let stmt = TransactionStatement::StartTransaction {
            isolation_level: None,
        };
        match stmt {
            TransactionStatement::StartTransaction { isolation_level } => {
                assert!(isolation_level.is_none());
            }
            _ => panic!("Expected StartTransaction"),
        }
    }

    #[test]
    fn test_isolation_level_all_variants() {
        let levels = [
            IsolationLevel::ReadUncommitted,
            IsolationLevel::ReadCommitted,
            IsolationLevel::SnapshotIsolation,
            IsolationLevel::Serializable,
        ];
        for level in &levels {
            let stmt = TransactionStatement::Begin {
                work: false,
                isolation_level: Some(level.clone()),
            };
            match stmt {
                TransactionStatement::Begin {
                    work,
                    isolation_level: Some(l),
                } => {
                    assert!(!work);
                    assert_eq!(&l, level);
                }
                _ => panic!("Expected Begin with isolation level"),
            }
        }
    }

    #[test]
    fn test_transaction_statement_all_variants() {
        let stmts = [
            TransactionStatement::Begin {
                work: false,
                isolation_level: None,
            },
            TransactionStatement::Begin {
                work: false,
                isolation_level: Some(IsolationLevel::ReadCommitted),
            },
            TransactionStatement::Commit { work: false },
            TransactionStatement::Commit { work: true },
            TransactionStatement::Rollback { work: false },
            TransactionStatement::Rollback { work: true },
            TransactionStatement::SetTransaction {
                isolation_level: IsolationLevel::Serializable,
            },
            TransactionStatement::StartTransaction {
                isolation_level: None,
            },
            TransactionStatement::StartTransaction {
                isolation_level: Some(IsolationLevel::SnapshotIsolation),
            },
        ];
        for stmt in &stmts {
            match stmt {
                TransactionStatement::Begin { .. }
                | TransactionStatement::Commit { .. }
                | TransactionStatement::Rollback { .. }
                | TransactionStatement::SetTransaction { .. }
                | TransactionStatement::StartTransaction { .. } => {}
            }
        }
    }
}

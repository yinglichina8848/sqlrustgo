use rand::Rng;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IsolationLevel::ReadCommitted => write!(f, "READ COMMITTED"),
            IsolationLevel::RepeatableRead => write!(f, "REPEATABLE READ"),
            IsolationLevel::Serializable => write!(f, "SERIALIZABLE"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransactionOperation {
    Begin(Option<IsolationLevel>),
    Commit,
    Rollback,
    Savepoint(String),
    RollbackTo(String),
    Insert {
        table: String,
        values: Vec<String>,
    },
    Update {
        table: String,
        set: String,
        where_cond: String,
    },
    Delete {
        table: String,
        where_cond: String,
    },
    Select {
        table: String,
        where_cond: Option<String>,
    },
}

impl TransactionOperation {
    pub fn to_sql(&self) -> String {
        match self {
            TransactionOperation::Begin(level) => {
                if let Some(l) = level {
                    format!("BEGIN ISOLATION LEVEL {}", l)
                } else {
                    "BEGIN".to_string()
                }
            }
            TransactionOperation::Commit => "COMMIT".to_string(),
            TransactionOperation::Rollback => "ROLLBACK".to_string(),
            TransactionOperation::Savepoint(name) => format!("SAVEPOINT {}", name),
            TransactionOperation::RollbackTo(name) => format!("ROLLBACK TO {}", name),
            TransactionOperation::Insert { table, values } => {
                format!("INSERT INTO {} VALUES ({})", table, values.join(", "))
            }
            TransactionOperation::Update {
                table,
                set,
                where_cond,
            } => {
                format!("UPDATE {} SET {} WHERE {}", table, set, where_cond)
            }
            TransactionOperation::Delete { table, where_cond } => {
                format!("DELETE FROM {} WHERE {}", table, where_cond)
            }
            TransactionOperation::Select { table, where_cond } => {
                if let Some(cond) = where_cond {
                    format!("SELECT * FROM {} WHERE {}", table, cond)
                } else {
                    format!("SELECT * FROM {}", table)
                }
            }
        }
    }
}

pub struct TransactionGenerator {
    table_count: usize,
    active_savepoints: Vec<String>,
}

impl TransactionGenerator {
    pub fn new(table_count: usize) -> Self {
        Self {
            table_count,
            active_savepoints: Vec::new(),
        }
    }

    pub fn update_table_count(&mut self, count: usize) {
        self.table_count = count;
    }

    pub fn generate_begin(&self) -> TransactionOperation {
        let isolation = match rand::thread_rng().gen_range(0..3) {
            0 => Some(IsolationLevel::ReadCommitted),
            1 => Some(IsolationLevel::RepeatableRead),
            _ => Some(IsolationLevel::Serializable),
        };
        TransactionOperation::Begin(isolation)
    }

    pub fn generate_workload(&self, size: usize) -> Vec<TransactionOperation> {
        let mut ops = vec![self.generate_begin()];

        for _ in 0..size {
            let op = self.generate_dml();
            if let Some(op) = op {
                ops.push(op);
            }
        }

        if rand::thread_rng().gen_bool(0.8) {
            ops.push(TransactionOperation::Commit);
        } else {
            ops.push(TransactionOperation::Rollback);
        }

        ops
    }

    fn generate_dml(&self) -> Option<TransactionOperation> {
        if self.table_count == 0 {
            return None;
        }

        let table_idx = rand::thread_rng().gen_range(0..self.table_count);
        let table_name = format!("t{}", table_idx);

        match rand::thread_rng().gen_range(0..4) {
            0 => {
                let values = (0..3)
                    .map(|_| rand::thread_rng().gen_range(1..100).to_string())
                    .collect();
                Some(TransactionOperation::Insert {
                    table: table_name,
                    values,
                })
            }
            1 => {
                let col = rand::thread_rng().gen_range(0..3);
                let set = format!("c{} = {}", col, rand::thread_rng().gen_range(1..100));
                let where_cond = format!("c{} > {}", col, rand::thread_rng().gen_range(0..50));
                Some(TransactionOperation::Update {
                    table: table_name,
                    set,
                    where_cond,
                })
            }
            2 => {
                let col = rand::thread_rng().gen_range(0..3);
                let where_cond = format!("c{} > {}", col, rand::thread_rng().gen_range(0..50));
                Some(TransactionOperation::Delete {
                    table: table_name,
                    where_cond,
                })
            }
            _ => {
                let where_cond = if rand::thread_rng().gen_bool(0.5) {
                    let col = rand::thread_rng().gen_range(0..3);
                    Some(format!(
                        "c{} > {}",
                        col,
                        rand::thread_rng().gen_range(0..50)
                    ))
                } else {
                    None
                };
                Some(TransactionOperation::Select {
                    table: table_name,
                    where_cond,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin() {
        let gen = TransactionGenerator::new(1);
        let op = gen.generate_begin();
        assert!(op.to_sql().starts_with("BEGIN"));
    }

    #[test]
    fn test_workload() {
        let gen = TransactionGenerator::new(1);
        let ops = gen.generate_workload(3);
        assert!(!ops.is_empty());
        assert!(matches!(ops[0], TransactionOperation::Begin(_)));
    }
}

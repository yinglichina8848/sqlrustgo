use sqlrustgo_types::Value;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo::{ExecutorResult, MemoryStorage as MemStorage};
use sqllogictest::{DB, DBOutput, DefaultColumnType};

#[derive(Debug)]
pub struct DbError(pub String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DbError {}

impl From<String> for DbError {
    fn from(s: String) -> Self {
        DbError(s)
    }
}

impl From<&str> for DbError {
    fn from(s: &str) -> Self {
        DbError(s.to_string())
    }
}

pub struct EngineAdapter {
    storage: MemoryStorage,
}

impl EngineAdapter {
    pub fn new() -> Self {
        Self {
            storage: MemoryStorage::new(),
        }
    }
}

impl Default for EngineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

fn value_to_string(v: Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => if b { "TRUE".to_string() } else { "FALSE".to_string() },
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => {
            if f.is_nan() {
                "NaN".to_string()
            } else if f.is_infinite() {
                if f.is_sign_positive() { "Infinity".to_string() } else { "-Infinity".to_string() }
            } else {
                f.to_string()
            }
        }
        Value::Text(s) => s,
        Value::Blob(b) => format!("{:?}", b),
    }
}

fn execute_sql(storage: &mut MemoryStorage, sql: &str) -> Result<ExecutorResult, String> {
    let statement = parse(sql).map_err(|e| format!("Parse error: {:?}", e))?;

    match statement {
        Statement::CreateTable(create) => {
            let info = TableInfo {
                name: create.name.clone(),
                columns: create
                    .columns
                    .into_iter()
                    .map(|c| ColumnDefinition {
                        name: c.name,
                        data_type: c.data_type,
                        nullable: c.nullable,
                        primary_key: c.primary_key,
                    })
                    .collect(),
                foreign_keys: vec![],
                unique_constraints: vec![],
                check_constraints: vec![],
                partition_info: None,
            };
            storage
                .create_table(&info)
                .map_err(|e| format!("Create table error: {:?}", e))?;
            Ok(ExecutorResult::new(vec![], 0))
        }
        Statement::DropTable(drop) => {
            storage
                .drop_table(&drop.name)
                .map_err(|e| format!("Drop table error: {:?}", e))?;
            Ok(ExecutorResult::new(vec![], 0))
        }
        Statement::Insert(_) => {
            Ok(ExecutorResult::new(vec![], 0))
        }
        Statement::Select(_) => {
            Ok(ExecutorResult::new(vec![], 0))
        }
        _ => Ok(ExecutorResult::new(vec![], 0)),
    }
}

impl DB for EngineAdapter {
    type Error = DbError;
    type ColumnType = DefaultColumnType;

    fn run(&mut self, sql: &str) -> Result<DBOutput<Self::ColumnType>, Self::Error> {
        let result = execute_sql(&mut self.storage, sql).map_err(DbError)?;
        let num_rows = result.rows.len();
        let num_cols = if result.rows.is_empty() {
            0
        } else {
            result.rows[0].len()
        };

        let rows: Vec<Vec<String>> = result
            .rows
            .into_iter()
            .map(|row: Vec<Value>| row.into_iter().map(value_to_string).collect())
            .collect();

        if rows.is_empty() {
            Ok(DBOutput::StatementComplete(num_rows as u64))
        } else {
            Ok(DBOutput::Rows {
                rows,
                types: vec![DefaultColumnType::Any; num_cols],
            })
        }
    }

    fn engine_name(&self) -> &str {
        "sqlrustgo"
    }
}
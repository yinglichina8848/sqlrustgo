use rusqlite::Connection;
use sqlrustgo_parser::parse;
use sqlrustgo_parser::Statement;
use sqlrustgo_storage::{ColumnDefinition, MemoryStorage, StorageEngine, TableInfo};
use sqlrustgo::ExecutorResult;
use sqlrustgo_types::Value;

type Row = Vec<String>;

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => "NULL".to_string(),
        Value::Boolean(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
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
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("{:?}", b),
    }
}

pub struct SqliteEngine {
    conn: Connection,
}

impl SqliteEngine {
    pub fn new() -> Self {
        let conn = Connection::open_in_memory().unwrap();
        Self { conn }
    }

    pub fn execute(&self, sql: &str) -> Result<(), String> {
        self.conn
            .execute(sql, [])
            .map_err(|e| format!("SQLite error: {}", e))?;
        Ok(())
    }

    pub fn query(&self, sql: &str) -> Result<Vec<Row>, String> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| format!("Prepare error: {}", e))?;

        let column_count = stmt.column_count();
        let rows = stmt
            .query_map([], move |row| {
                let mut r = Vec::new();
                for i in 0..column_count {
                    let val: rusqlite::Result<rusqlite::types::Value> = row.get(i);
                    let s = match val {
                        Ok(rusqlite::types::Value::Null) => "NULL".to_string(),
                        Ok(rusqlite::types::Value::Integer(i)) => i.to_string(),
                        Ok(rusqlite::types::Value::Real(f)) => f.to_string(),
                        Ok(rusqlite::types::Value::Text(s)) => s,
                        Ok(rusqlite::types::Value::Blob(b)) => format!("{:?}", b),
                        Err(_) => "NULL".to_string(),
                    };
                    r.push(s);
                }
                Ok(r)
            })
            .map_err(|e| format!("Query error: {}", e))?;

        rows.map(|r| r.map_err(|e| format!("Row error: {}", e)))
            .collect()
    }
}

impl Default for SqliteEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RustEngine {
    storage: MemoryStorage,
}

impl RustEngine {
    pub fn new() -> Self {
        Self {
            storage: MemoryStorage::new(),
        }
    }

    pub fn execute(&mut self, sql: &str) -> Result<(), String> {
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
                self.storage
                    .create_table(&info)
                    .map_err(|e| format!("Create table error: {:?}", e))?;
                Ok(())
            }
            Statement::DropTable(drop) => {
                self.storage
                    .drop_table(&drop.name)
                    .map_err(|e| format!("Drop table error: {:?}", e))?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn query(&self, _sql: &str) -> Result<Vec<Row>, String> {
        Ok(vec![])
    }
}

impl Default for RustEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize(rows: &mut Vec<Row>) {
    for row in rows.iter_mut() {
        for val in row.iter_mut() {
            if val.is_empty() {
                *val = "NULL".to_string();
            }
        }
    }
    rows.sort();
}

pub fn assert_sql_eq(sql: &str, setup: &[&str]) -> Result<(), String> {
    let mut sqlite = SqliteEngine::new();
    let mut rust = RustEngine::new();

    for s in setup {
        sqlite.execute(s).map_err(|e| e.to_string())?;
        rust.execute(s).map_err(|e| e.to_string())?;
    }

    let mut left = sqlite.query(sql).map_err(|e| e.to_string())?;
    let mut right = rust.query(sql).map_err(|e| e.to_string())?;

    normalize(&mut left);
    normalize(&mut right);

    if left != right {
        return Err(format!(
            "SQL mismatch: {}\nSQLite: {:?}\nRust: {:?}",
            sql, left, right
        ));
    }
    Ok(())
}

pub fn assert_query_eq(left: Vec<Row>, right: Vec<Row>) -> Result<(), String> {
    let mut left_norm = left;
    let mut right_norm = right;

    normalize(&mut left_norm);
    normalize(&mut right_norm);

    if left_norm != right_norm {
        return Err(format!(
            "Query mismatch:\nExpected: {:?}\nGot: {:?}",
            left_norm, right_norm
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_basic_query() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT * FROM t").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_sqlite_count() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(result[0][0], "3");
    }

    #[test]
    fn test_sqlite_where() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let result = sqlite.query("SELECT a FROM t WHERE a > 1").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_sqlite_aggregate() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE orders(amount INT)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (100)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (200)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (150)").unwrap();

        let result = sqlite.query("SELECT SUM(amount) FROM orders").unwrap();
        assert_eq!(result[0][0], "450");
    }
}
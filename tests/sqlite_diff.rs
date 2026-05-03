use rusqlite::Connection;
use std::collections::HashMap;

type Row = Vec<String>;

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

pub fn assert_query_eq(sql: &str, left: Vec<Row>, right: Vec<Row>) -> Result<(), String> {
    let mut left_sorted = left;
    let mut right_sorted = right;

    left_sorted.sort();
    right_sorted.sort();

    if left_sorted != right_sorted {
        return Err(format!(
            "Query mismatch for: {}\nExpected: {:?}\nGot: {:?}",
            sql, left_sorted, right_sorted
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
        sqlite.execute("INSERT INTO t VALUES (1),(2),(3)").unwrap();

        let result = sqlite.query("SELECT * FROM t").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_sqlite_diff_aggregate() {
        let sqlite = SqliteEngine::new();
        sqlite.execute("CREATE TABLE orders(amount INT)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (100)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (200)").unwrap();
        sqlite.execute("INSERT INTO orders VALUES (150)").unwrap();

        let all = sqlite.query("SELECT amount FROM orders").unwrap();
        assert_eq!(all.len(), 3, "Should have 3 rows, got {:?}", all);

        let result = sqlite.query("SELECT SUM(amount) FROM orders").unwrap();
        assert_eq!(result[0][0], "450");
    }

    #[test]
    fn test_sqlite_count() {
        let sqlite = SqliteEngine::new();

        sqlite.execute("CREATE TABLE t(a INT)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (1)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (2)").unwrap();
        sqlite.execute("INSERT INTO t VALUES (3)").unwrap();

        let all = sqlite.query("SELECT a FROM t").unwrap();
        assert_eq!(all.len(), 3);

        let result = sqlite.query("SELECT COUNT(*) FROM t").unwrap();
        assert_eq!(result[0][0], "3");
    }
}
//! SQLite database adapter

use crate::db::Database;
use async_trait::async_trait;
use rusqlite::Connection;
use std::sync::Mutex;

/// SQLite database adapter
pub struct SqliteDB {
    conn: Mutex<Connection>,
}

impl SqliteDB {
    /// Create a new SQLite adapter
    pub async fn new(path: &str, scale: usize) -> anyhow::Result<Self> {
        // Enable WAL mode for better concurrent performance
        let conn = Connection::open(path)?;

        // Configure SQLite for benchmark
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=10000;
             PRAGMA temp_store=MEMORY;",
        )?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (id INTEGER PRIMARY KEY, balance INTEGER NOT NULL)",
            [],
        )?;

        // Insert initial data if table is empty
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        if count == 0 {
            let tx = conn.unchecked_transaction()?;
            for i in 0..scale as i64 {
                tx.execute(
                    "INSERT OR IGNORE INTO accounts (id, balance) VALUES (?1, 100)",
                    [i],
                )?;
            }
            tx.commit()?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

#[async_trait]
impl Database for SqliteDB {
    async fn read(&self, key: usize) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("SELECT * FROM accounts WHERE id = ?1", [key])?;
        Ok(())
    }

    async fn update(&self, key: usize) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE accounts SET balance = balance + 1 WHERE id = ?1",
            [key],
        )?;
        Ok(())
    }

    async fn insert(&self, key: usize) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT OR IGNORE INTO accounts VALUES (?1, 100)", [key])?;
        Ok(())
    }

    async fn scan(&self, start: usize, end: usize) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT * FROM accounts WHERE id BETWEEN ?1 AND ?2")?;
        let mut rows = stmt.query([start, end])?;
        while rows.next()?.is_some() {
            // Consume all rows
        }
        Ok(())
    }
}

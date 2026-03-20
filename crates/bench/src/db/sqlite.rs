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
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        // Enable WAL mode for better concurrent performance
        let conn = Connection::open(path)?;

        // Configure SQLite for benchmark
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=10000;
             PRAGMA temp_store=MEMORY;",
        )?;

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

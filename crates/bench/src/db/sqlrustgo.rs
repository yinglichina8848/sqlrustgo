//! SQLRustGo database adapter

use crate::db::Database;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// SQLRustGo TCP client adapter
pub struct SqlRustGoDB {
    addr: String,
}

impl SqlRustGoDB {
    /// Create a new SQLRustGo adapter
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        Ok(Self {
            addr: addr.to_string(),
        })
    }

    /// Execute a query via TCP
    async fn execute_query(&self, sql: &str) -> anyhow::Result<()> {
        let mut stream = TcpStream::connect(&self.addr).await?;

        // Send query
        stream.write_all(sql.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        // Read response (simple version)
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf).await?;

        Ok(())
    }
}

#[async_trait]
impl Database for SqlRustGoDB {
    async fn execute(&self, sql: &str) -> anyhow::Result<()> {
        self.execute_query(sql).await
    }

    async fn read(&self, key: usize) -> anyhow::Result<()> {
        self.execute_query(&format!("SELECT * FROM accounts WHERE id = {}", key))
            .await
    }

    async fn update(&self, key: usize) -> anyhow::Result<()> {
        self.execute_query(&format!(
            "UPDATE accounts SET balance = balance + 1 WHERE id = {}",
            key
        ))
        .await
    }

    async fn insert(&self, key: usize) -> anyhow::Result<()> {
        self.execute_query(&format!("INSERT INTO accounts VALUES ({}, 100)", key))
            .await
    }

    async fn scan(&self, start: usize, end: usize) -> anyhow::Result<()> {
        self.execute_query(&format!(
            "SELECT * FROM accounts WHERE id BETWEEN {} AND {}",
            start, end
        ))
        .await
    }
}

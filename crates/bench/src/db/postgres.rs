//! PostgreSQL database adapter

use crate::db::Database;
use async_trait::async_trait;
use tokio_postgres::{NoTls, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

/// PostgreSQL database adapter
pub struct PostgresDB {
    client: Arc<Mutex<Client>>,
}

impl PostgresDB {
    /// Create a new PostgreSQL adapter
    pub async fn new(conn_str: &str) -> anyhow::Result<Self> {
        let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;

        // Spawn connection driver
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }
}

#[async_trait]
impl Database for PostgresDB {
    async fn read(&self, key: usize) -> anyhow::Result<()> {
        let client = self.client.lock().await;
        client
            .query("SELECT * FROM accounts WHERE id = $1", &[&(key as i32)])
            .await?;
        Ok(())
    }

    async fn update(&self, key: usize) -> anyhow::Result<()> {
        let client = self.client.lock().await;
        client
            .execute(
                "UPDATE accounts SET balance = balance + 1 WHERE id = $1",
                &[&(key as i32)],
            )
            .await?;
        Ok(())
    }

    async fn insert(&self, key: usize) -> anyhow::Result<()> {
        let client = self.client.lock().await;
        client
            .execute(
                "INSERT INTO accounts (id, balance) VALUES ($1, 100) ON CONFLICT DO NOTHING",
                &[&(key as i32)],
            )
            .await?;
        Ok(())
    }

    async fn scan(&self, start: usize, end: usize) -> anyhow::Result<()> {
        let client = self.client.lock().await;
        client
            .query(
                "SELECT * FROM accounts WHERE id BETWEEN $1 AND $2",
                &[&(start as i32), &(end as i32)],
            )
            .await?;
        Ok(())
    }
}

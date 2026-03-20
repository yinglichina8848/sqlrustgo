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
    pub async fn new(conn_str: &str, scale: usize) -> anyhow::Result<Self> {
        let (mut client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;

        // Spawn connection driver
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("PostgreSQL connection error: {}", e);
            }
        });

        // Create table if not exists
        client
            .execute(
                "CREATE TABLE IF NOT EXISTS accounts (id SERIAL PRIMARY KEY, balance INTEGER NOT NULL)",
                &[],
            )
            .await?;

        // Insert initial data (use ON CONFLICT DO NOTHING for idempotency)
        let tx = client.transaction().await?;
        for i in 0..scale as i32 {
            tx.execute(
                "INSERT INTO accounts (id, balance) VALUES ($1, 100) ON CONFLICT DO NOTHING",
                &[&i],
            )
            .await?;
        }
        tx.commit().await?;

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

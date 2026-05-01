//! MySQL database adapter

use crate::db::Database;
use anyhow::Context;
use async_trait::async_trait;
use mysql::prelude::Queryable;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MySqlDB {
    pool: Arc<Mutex<mysql::Pool>>,
}

#[allow(dead_code)]
impl MySqlDB {
    pub async fn new(addr: &str) -> anyhow::Result<Self> {
        let addr = addr.to_string();
        let pool = tokio::task::spawn_blocking(move || {
            mysql::Pool::new(format!("mysql://{}", addr).as_str())
        })
        .await
        .context("Failed to create MySQL pool")??;

        Ok(Self {
            pool: Arc::new(Mutex::new(pool)),
        })
    }
}

#[async_trait]
impl Database for MySqlDB {
    async fn execute(&self, sql: &str) -> anyhow::Result<()> {
        let pool = self.pool.lock().await;
        let pool = pool.clone();
        let sql = sql.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get_conn()?;
            conn.query_drop(sql.as_str())?;
            Ok::<(), mysql::Error>(())
        })
        .await
        .context("MySQL execute failed")??;
        Ok(())
    }

    async fn read(&self, key: usize) -> anyhow::Result<()> {
        let pool = self.pool.lock().await;
        let pool = pool.clone();
        let key = key as i32;
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get_conn()?;
            let _: Vec<(i32, String, String, i32)> =
                conn.exec("SELECT * FROM accounts WHERE id = ?", (key,))?;
            Ok::<(), mysql::Error>(())
        })
        .await
        .context("MySQL read failed")??;
        Ok(())
    }

    async fn update(&self, key: usize) -> anyhow::Result<()> {
        let pool = self.pool.lock().await;
        let pool = pool.clone();
        let key = key as i32;
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get_conn()?;
            conn.exec_drop(
                "UPDATE accounts SET balance = balance + 1 WHERE id = ?",
                (key,),
            )?;
            Ok::<(), mysql::Error>(())
        })
        .await
        .context("MySQL update failed")??;
        Ok(())
    }

    async fn insert(&self, key: usize) -> anyhow::Result<()> {
        let pool = self.pool.lock().await;
        let pool = pool.clone();
        let key = key as i32;
        let name = format!("user_{}", key);
        let email = format!("user{}@example.com", key);
        let balance = 1000 + key;
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get_conn()?;
            conn.exec_drop(
                "INSERT INTO accounts (id, name, email, balance) VALUES (?, ?, ?, ?)",
                (key, name, email, balance),
            )?;
            Ok::<(), mysql::Error>(())
        })
        .await
        .context("MySQL insert failed")??;
        Ok(())
    }

    async fn scan(&self, start: usize, end: usize) -> anyhow::Result<()> {
        let pool = self.pool.lock().await;
        let pool = pool.clone();
        let start = start as i32;
        let end = end as i32;
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get_conn()?;
            let _: Vec<(i32, String, String, i32)> = conn.exec(
                "SELECT * FROM accounts WHERE id >= ? AND id < ?",
                (start, end),
            )?;
            Ok::<(), mysql::Error>(())
        })
        .await
        .context("MySQL scan failed")??;
        Ok(())
    }
}

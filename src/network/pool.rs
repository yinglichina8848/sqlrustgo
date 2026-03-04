//! Connection Pool and Session Management for SQLRustGo
//!
//! Provides connection pooling and session management for the network layer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: usize,
    pub min_idle: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_idle: 2,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(3600),
        }
    }
}

/// A pooled connection wrapper
#[derive(Debug, Clone)]
pub struct PooledConnection {
    pub id: u64,
    pub created_at: Instant,
    pub last_used: Instant,
    pub in_use: bool,
}

impl PooledConnection {
    pub fn new(id: u64) -> Self {
        let now = Instant::now();
        Self {
            id,
            created_at: now,
            last_used: now,
            in_use: false,
        }
    }

    pub fn is_expired(&self, max_lifetime: Duration) -> bool {
        self.created_at.elapsed() > max_lifetime
    }

    pub fn is_idle_timeout(&self, idle_timeout: Duration) -> bool {
        !self.in_use && self.last_used.elapsed() > idle_timeout
    }
}

/// Connection pool for managing database connections
#[derive(Debug)]
pub struct ConnectionPool {
    config: PoolConfig,
    connections: RwLock<Vec<PooledConnection>>,
    next_id: Mutex<u64>,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            connections: RwLock::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn get_connection(&self) -> Option<PooledConnection> {
        let mut connections = self.connections.write().unwrap();

        for conn in connections.iter_mut() {
            if !conn.in_use {
                conn.in_use = true;
                conn.last_used = Instant::now();
                return Some(PooledConnection {
                    id: conn.id,
                    created_at: conn.created_at,
                    last_used: conn.last_used,
                    in_use: true,
                });
            }
        }

        if connections.len() < self.config.max_connections {
            let id = {
                let mut next = self.next_id.lock().unwrap();
                let id = *next;
                *next += 1;
                id
            };
            let conn = PooledConnection::new(id);
            let result = conn.clone();
            connections.push(conn);
            return Some(result);
        }

        None
    }

    pub fn release_connection(&self, conn: PooledConnection) {
        let mut connections = self.connections.write().unwrap();
        if let Some(c) = connections.iter_mut().find(|c| c.id == conn.id) {
            c.in_use = false;
            c.last_used = Instant::now();
        }
    }

    pub fn cleanup(&self) {
        let mut connections = self.connections.write().unwrap();
        connections.retain(|c| {
            !c.is_expired(self.config.max_lifetime) && !c.is_idle_timeout(self.config.idle_timeout)
        });
    }

    pub fn stats(&self) -> PoolStats {
        let connections = self.connections.read().unwrap();
        let in_use = connections.iter().filter(|c| c.in_use).count();
        PoolStats {
            total: connections.len(),
            idle: connections.len() - in_use,
            in_use,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connections: RwLock::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total: usize,
    pub idle: usize,
    pub in_use: usize,
}

/// Session state
#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub variables: RwLock<HashMap<String, String>>,
    pub created_at: Instant,
    pub last_activity: RwLock<Instant>,
    pub database: RwLock<Option<String>>,
    pub connection_id: Option<u64>,
}

impl Session {
    pub fn new(id: u64) -> Self {
        let now = Instant::now();
        Self {
            id,
            variables: RwLock::new(HashMap::new()),
            created_at: now,
            last_activity: RwLock::new(now),
            database: RwLock::new(None),
            connection_id: None,
        }
    }

    pub fn set_variable(&self, key: &str, value: &str) {
        let mut vars = self.variables.write().unwrap();
        vars.insert(key.to_string(), value.to_string());
    }

    pub fn get_variable(&self, key: &str) -> Option<String> {
        let vars = self.variables.read().unwrap();
        vars.get(key).cloned()
    }

    pub fn set_database(&self, db: Option<String>) {
        let mut database = self.database.write().unwrap();
        *database = db;
    }

    pub fn get_database(&self) -> Option<String> {
        let database = self.database.read().unwrap();
        database.clone()
    }

    pub fn touch(&self) {
        let mut last = self.last_activity.write().unwrap();
        *last = Instant::now();
    }

    pub fn is_timeout(&self, timeout: Duration) -> bool {
        let last = self.last_activity.read().unwrap();
        last.elapsed() > timeout
    }
}

/// Session manager
#[derive(Debug)]
pub struct SessionManager {
    sessions: RwLock<HashMap<u64, Arc<Session>>>,
    next_id: Mutex<u64>,
    session_timeout: Duration,
}

impl SessionManager {
    pub fn new(session_timeout: Duration) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            next_id: Mutex::new(1),
            session_timeout,
        }
    }

    pub fn create_session(&self) -> Arc<Session> {
        let id = {
            let mut next = self.next_id.lock().unwrap();
            let id = *next;
            *next += 1;
            id
        };
        let session = Arc::new(Session::new(id));
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(id, session.clone());
        session
    }

    pub fn get_session(&self, id: u64) -> Option<Arc<Session>> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(&id).cloned()
    }

    pub fn remove_session(&self, id: u64) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(&id);
    }

    pub fn cleanup_timeout(&self) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, s| !s.is_timeout(self.session_timeout));
    }

    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(1800))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_pool() {
        let config = PoolConfig {
            max_connections: 5,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);

        let conn = pool.get_connection();
        assert!(conn.is_some());

        if let Some(c) = conn {
            pool.release_connection(c);
        }

        let stats = pool.stats();
        assert_eq!(stats.total, 1);
        assert_eq!(stats.idle, 1);
    }

    #[test]
    fn test_session() {
        let manager = SessionManager::new(Duration::from_secs(60));
        let session = manager.create_session();

        session.set_variable("key", "value");
        assert_eq!(session.get_variable("key"), Some("value".to_string()));

        session.set_database(Some("test_db".to_string()));
        assert_eq!(session.get_database(), Some("test_db".to_string()));

        assert_eq!(manager.session_count(), 1);
    }
}

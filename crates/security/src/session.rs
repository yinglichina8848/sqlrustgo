//! Session management
//!
//! Provides session tracking for audit and security purposes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Closing,
    Closed,
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: u64,
    pub user: String,
    pub ip: String,
    pub login_time: u64,
    pub last_activity: u64,
    pub status: SessionStatus,
    pub database: Option<String>,
    pub connection_id: u64,
}

impl Session {
    pub fn new(id: u64, user: String, ip: String) -> Self {
        let now = current_timestamp();
        Self {
            id,
            user,
            ip,
            login_time: now,
            last_activity: now,
            status: SessionStatus::Active,
            database: None,
            connection_id: 0,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = current_timestamp();
        if self.status == SessionStatus::Idle {
            self.status = SessionStatus::Active;
        }
    }

    pub fn set_idle(&mut self) {
        self.status = SessionStatus::Idle;
    }

    pub fn close(&mut self) {
        self.status = SessionStatus::Closed;
    }

    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active || self.status == SessionStatus::Idle
    }

    pub fn idle_time_seconds(&self) -> u64 {
        current_timestamp() - self.last_activity
    }
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u64, Session>>>,
    next_session_id: Arc<RwLock<u64>>,
    max_idle_seconds: u64,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_session_id: Arc::new(RwLock::new(1)),
            max_idle_seconds: 3600,
        }
    }

    pub fn with_max_idle(mut self, seconds: u64) -> Self {
        self.max_idle_seconds = seconds;
        self
    }

    pub fn create_session(&self, user: String, ip: String) -> u64 {
        let session_id = {
            let mut next_id = self.next_session_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let session = Session::new(session_id, user, ip);

        self.sessions.write().unwrap().insert(session_id, session);

        session_id
    }

    pub fn get_session(&self, session_id: u64) -> Option<Session> {
        self.sessions.read().unwrap().get(&session_id).cloned()
    }

    pub fn get_session_mut(&self, session_id: u64) -> Option<Session> {
        self.sessions.write().unwrap().get_mut(&session_id).cloned()
    }

    pub fn update_activity(&self, session_id: u64) {
        if let Some(mut session) = self.get_session_mut(session_id) {
            session.update_activity();
            let sessions = &mut self.sessions.write().unwrap();
            if let Some(s) = sessions.get_mut(&session_id) {
                s.last_activity = session.last_activity;
                s.status = session.status;
            }
        }
    }

    pub fn close_session(&self, session_id: u64) {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.close();
        }
    }

    pub fn remove_session(&self, session_id: u64) {
        self.sessions.write().unwrap().remove(&session_id);
    }

    pub fn get_active_sessions(&self) -> Vec<Session> {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.is_active())
            .cloned()
            .collect()
    }

    pub fn get_user_sessions(&self, user: &str) -> Vec<Session> {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.user == user && s.is_active())
            .cloned()
            .collect()
    }

    pub fn get_session_count(&self) -> usize {
        self.sessions.read().unwrap().len()
    }

    pub fn get_active_session_count(&self) -> usize {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.is_active())
            .count()
    }

    pub fn close_idle_sessions(&self) -> usize {
        let mut count = 0;
        let now = current_timestamp();

        let mut sessions = self.sessions.write().unwrap();
        for session in sessions.values_mut() {
            if session.status == SessionStatus::Idle
                && now - session.last_activity > self.max_idle_seconds
            {
                session.close();
                count += 1;
            }
        }
        count
    }

    pub fn cleanup_closed_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().unwrap();
        let initial = sessions.len();
        sessions.retain(|_, s| s.status != SessionStatus::Closed);
        initial - sessions.len()
    }

    pub fn set_session_database(&self, session_id: u64, database: &str) {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(&session_id) {
            session.database = Some(database.to_string());
        }
    }

    pub fn get_all_sessions(&self) -> Vec<Session> {
        self.sessions.read().unwrap().values().cloned().collect()
    }

    pub fn get_sessions_by_ip(&self, ip: &str) -> Vec<Session> {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.ip == ip && s.is_active())
            .cloned()
            .collect()
    }

    pub fn get_concurrent_user_count(&self, user: &str) -> usize {
        self.sessions
            .read()
            .unwrap()
            .values()
            .filter(|s| s.user == user && s.is_active())
            .count()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session("alice".to_string(), "127.0.0.1".to_string());

        assert_eq!(session_id, 1);
        assert_eq!(manager.get_session_count(), 1);
    }

    #[test]
    fn test_get_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session("alice".to_string(), "127.0.0.1".to_string());

        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.user, "alice");
    }

    #[test]
    fn test_close_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session("alice".to_string(), "127.0.0.1".to_string());

        manager.close_session(session_id);

        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.status, SessionStatus::Closed);
    }

    #[test]
    fn test_active_sessions() {
        let manager = SessionManager::new();
        let id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
        let _id2 = manager.create_session("bob".to_string(), "127.0.0.2".to_string());

        manager.close_session(id1);

        let active = manager.get_active_sessions();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].user, "bob");
    }

    #[test]
    fn test_user_sessions() {
        let manager = SessionManager::new();
        let _id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
        let _id2 = manager.create_session("alice".to_string(), "127.0.0.2".to_string());
        let _id3 = manager.create_session("bob".to_string(), "127.0.0.1".to_string());

        let alice_sessions = manager.get_user_sessions("alice");
        assert_eq!(alice_sessions.len(), 2);
    }

    #[test]
    fn test_session_activity() {
        let manager = SessionManager::new();
        let session_id = manager.create_session("alice".to_string(), "127.0.0.1".to_string());

        let before = manager.get_session(session_id).unwrap().last_activity;
        manager.update_activity(session_id);
        let after = manager.get_session(session_id).unwrap().last_activity;

        assert!(after >= before);
    }

    #[test]
    fn test_cleanup_closed() {
        let manager = SessionManager::new();
        let id1 = manager.create_session("alice".to_string(), "127.0.0.1".to_string());
        let _id2 = manager.create_session("bob".to_string(), "127.0.0.2".to_string());

        manager.close_session(id1);
        let removed = manager.cleanup_closed_sessions();

        assert_eq!(removed, 1);
        assert_eq!(manager.get_session_count(), 1);
    }
}

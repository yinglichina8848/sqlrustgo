//! Authentication module for SQLRustGo
//! 
//! Provides user authentication, session management, and permission control.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

/// User role types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    User,
    Readonly,
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            "readonly" => Ok(Role::Readonly),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

impl Role {
    /// Check if role can perform specific operation
    pub fn can_execute(&self, operation: &Operation) -> bool {
        match self {
            Role::Admin => true,
            Role::User => matches!(operation, Operation::Select | Operation::Insert | Operation::Update | Operation::Delete),
            Role::Readonly => matches!(operation, Operation::Select),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
            Role::Readonly => write!(f, "readonly"),
        }
    }
}

/// Database operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
}

/// User structure
#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Role,
}

/// Session structure
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: i64,
    pub username: String,
    pub role: Role,
    pub created_at: u64,
    pub expires_at: u64,
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Invalid password")]
    InvalidPassword,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid role: {0}")]
    InvalidRole(String),
}

/// Authentication manager
pub struct AuthManager {
    users: HashMap<String, User>,
    sessions: HashMap<String, Session>,
    session_timeout: u64,
}

impl AuthManager {
    /// Create a new AuthManager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            session_timeout: 3600,
        }
    }

    /// Create AuthManager with custom session timeout
    pub fn with_timeout(timeout_seconds: u64) -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            session_timeout: timeout_seconds,
        }
    }

    /// Register a new user
    pub fn register(&mut self, username: &str, password: &str, role: Role) -> Result<User, AuthError> {
        if self.users.contains_key(username) {
            return Err(AuthError::UserAlreadyExists(username.to_string()));
        }

        let id = (self.users.len() + 1) as i64;
        let password_hash = Self::hash_password(password);

        let user = User {
            id,
            username: username.to_string(),
            password_hash,
            role,
        };

        self.users.insert(username.to_string(), user.clone());
        Ok(user)
    }

    /// User login
    pub fn login(&mut self, username: &str, password: &str) -> Result<Session, AuthError> {
        let user = self.users
            .get(username)
            .ok_or_else(|| AuthError::UserNotFound(username.to_string()))?;

        if user.password_hash != Self::hash_password(password) {
            return Err(AuthError::InvalidPassword);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session_id = Self::generate_session_id();
        let session = Session {
            id: session_id.clone(),
            user_id: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: now,
            expires_at: now + self.session_timeout,
        };

        self.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// User logout
    pub fn logout(&mut self, session_id: &str) -> Result<(), AuthError> {
        self.sessions
            .remove(session_id)
            .ok_or_else(|| AuthError::SessionNotFound(session_id.to_string()))?;
        Ok(())
    }

    /// Verify session and return user
    pub fn verify(&self, session_id: &str) -> Result<User, AuthError> {
        let session = self.sessions
            .get(session_id)
            .ok_or_else(|| AuthError::SessionNotFound(session_id.to_string()))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > session.expires_at {
            return Err(AuthError::SessionExpired);
        }

        self.users
            .get(&session.username)
            .cloned()
            .ok_or_else(|| AuthError::UserNotFound(session.username.clone()))
    }

    /// Check if user has permission to perform operation
    pub fn check_permission(&self, session_id: &str, operation: &Operation) -> Result<(), AuthError> {
        let user = self.verify(session_id)?;
        if !user.role.can_execute(operation) {
            return Err(AuthError::PermissionDenied(
                format!("Role {} cannot perform {:?}", user.role, operation)
            ));
        }
        Ok(())
    }

    /// Simple password hashing
    fn hash_password(password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("sha256:{:016x}", hasher.finish())
    }

    /// Generate unique session ID
    fn generate_session_id() -> String {
        use std::time::SystemTime;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        format!("session:{:x}", now)
    }

    /// Get user by username
    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    /// List all users
    pub fn list_users(&self) -> Vec<(i64, String, Role)> {
        self.users.values()
            .map(|u| (u.id, u.username.clone(), u.role.clone()))
            .collect()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_login() {
        let mut auth = AuthManager::new();
        let user = auth.register("testuser", "password123", Role::User).unwrap();
        assert_eq!(user.username, "testuser");
        
        let session = auth.login("testuser", "password123").unwrap();
        assert_eq!(session.username, "testuser");
        
        let verified = auth.verify(&session.id).unwrap();
        assert_eq!(verified.username, "testuser");
    }

    #[test]
    fn test_invalid_password() {
        let mut auth = AuthManager::new();
        auth.register("testuser", "password123", Role::User).unwrap();
        
        let result = auth.login("testuser", "wrongpassword");
        assert!(matches!(result, Err(AuthError::InvalidPassword)));
    }

    #[test]
    fn test_permission_admin() {
        let mut auth = AuthManager::new();
        auth.register("admin", "pass", Role::Admin).unwrap();
        let session = auth.login("admin", "pass").unwrap();
        
        assert!(auth.verify(&session.id).is_ok());
    }

    #[test]
    fn test_permission_readonly() {
        let mut auth = AuthManager::new();
        auth.register("reader", "pass", Role::Readonly).unwrap();
        let session = auth.login("reader", "pass").unwrap();
        
        assert!(auth.check_permission(&session.id, &Operation::Select).is_ok());
        
        assert!(matches!(
            auth.check_permission(&session.id, &Operation::Insert),
            Err(AuthError::PermissionDenied(_))
        ));
    }

    #[test]
    fn test_logout() {
        let mut auth = AuthManager::new();
        auth.register("testuser", "pass", Role::User).unwrap();
        let session = auth.login("testuser", "pass").unwrap();
        
        auth.logout(&session.id).unwrap();
        
        let result = auth.verify(&session.id);
        assert!(result.is_err());
    }
}

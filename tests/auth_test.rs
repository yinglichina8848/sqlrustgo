//! Authentication module tests

use sqlrustgo::{AuthError, AuthManager, Operation, Role};
use std::str::FromStr;

#[test]
fn test_auth_manager_creation() {
    let auth = AuthManager::new();
    let users = auth.list_users();
    assert!(users.is_empty());
}

#[test]
fn test_auth_manager_with_timeout() {
    let auth = AuthManager::with_timeout(7200);
    let users = auth.list_users();
    assert!(users.is_empty());
}

#[test]
fn test_user_registration() {
    let mut auth = AuthManager::new();

    let user = auth.register("alice", "password123", Role::User).unwrap();
    assert_eq!(user.username, "alice");
    assert_eq!(user.id, 1);

    let users = auth.list_users();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].0, 1);
    assert_eq!(users[0].1, "alice");
    assert_eq!(users[0].2, Role::User);
}

#[test]
fn test_user_registration_admin() {
    let mut auth = AuthManager::new();

    let user = auth.register("admin", "adminpass", Role::Admin).unwrap();
    assert_eq!(user.username, "admin");

    let users = auth.list_users();
    assert_eq!(users[0].2, Role::Admin);
}

#[test]
fn test_user_registration_readonly() {
    let mut auth = AuthManager::new();

    let user = auth.register("reader", "readpass", Role::Readonly).unwrap();
    assert_eq!(user.username, "reader");

    let users = auth.list_users();
    assert_eq!(users[0].2, Role::Readonly);
}

#[test]
fn test_duplicate_user_registration() {
    let mut auth = AuthManager::new();

    auth.register("testuser", "password", Role::User).unwrap();
    let result = auth.register("testuser", "password", Role::User);

    assert!(matches!(result, Err(AuthError::UserAlreadyExists(_))));
}

#[test]
fn test_multiple_users() {
    let mut auth = AuthManager::new();

    auth.register("user1", "pass1", Role::User).unwrap();
    auth.register("user2", "pass2", Role::Admin).unwrap();
    auth.register("user3", "pass3", Role::Readonly).unwrap();

    let users = auth.list_users();
    assert_eq!(users.len(), 3);
}

#[test]
fn test_user_login_success() {
    let mut auth = AuthManager::new();
    auth.register("testuser", "password123", Role::User)
        .unwrap();

    let session = auth.login("testuser", "password123").unwrap();
    assert_eq!(session.username, "testuser");
    assert_eq!(session.user_id, 1);
    assert_eq!(session.role, Role::User);
    assert!(!session.id.is_empty());
    assert!(session.expires_at > session.created_at);
}

#[test]
fn test_user_login_wrong_password() {
    let mut auth = AuthManager::new();
    auth.register("testuser", "password123", Role::User)
        .unwrap();

    let result = auth.login("testuser", "wrongpassword");
    assert!(matches!(result, Err(AuthError::InvalidPassword)));
}

#[test]
fn test_user_login_nonexistent() {
    let mut auth = AuthManager::new();

    let result = auth.login("nonexistent", "password");
    assert!(matches!(result, Err(AuthError::UserNotFound(_))));
}

#[test]
fn test_session_verification() {
    let mut auth = AuthManager::new();
    auth.register("testuser", "password", Role::User).unwrap();

    let session = auth.login("testuser", "password").unwrap();
    let user = auth.verify(&session.id).unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.id, 1);
}

#[test]
fn test_session_verification_invalid() {
    let auth = AuthManager::new();

    let result = auth.verify("invalid_session_id");
    assert!(matches!(result, Err(AuthError::SessionNotFound(_))));
}

#[test]
fn test_user_logout() {
    let mut auth = AuthManager::new();
    auth.register("testuser", "password", Role::User).unwrap();

    let session = auth.login("testuser", "password").unwrap();
    auth.logout(&session.id).unwrap();

    let result = auth.verify(&session.id);
    assert!(result.is_err());
}

#[test]
fn test_logout_nonexistent_session() {
    let mut auth = AuthManager::new();

    let result = auth.logout("nonexistent");
    assert!(matches!(result, Err(AuthError::SessionNotFound(_))));
}

#[test]
fn test_get_user() {
    let mut auth = AuthManager::new();
    auth.register("testuser", "password", Role::Admin).unwrap();

    let user = auth.get_user("testuser");
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "testuser");

    let nonexistent = auth.get_user("nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_role_from_str() {
    assert_eq!(Role::from_str("admin"), Ok(Role::Admin));
    assert_eq!(Role::from_str("user"), Ok(Role::User));
    assert_eq!(Role::from_str("readonly"), Ok(Role::Readonly));
    assert_eq!(Role::from_str("ADMIN"), Ok(Role::Admin));
    assert!(Role::from_str("unknown").is_err());
}

#[test]
fn test_role_display() {
    assert_eq!(format!("{}", Role::Admin), "admin");
    assert_eq!(format!("{}", Role::User), "user");
    assert_eq!(format!("{}", Role::Readonly), "readonly");
}

#[test]
fn test_permission_admin_all_operations() {
    let mut auth = AuthManager::new();
    auth.register("admin", "pass", Role::Admin).unwrap();
    let session = auth.login("admin", "pass").unwrap();

    assert!(auth
        .check_permission(&session.id, &Operation::Select)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Insert)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Update)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Delete)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Create)
        .is_ok());
    assert!(auth.check_permission(&session.id, &Operation::Drop).is_ok());
}

#[test]
fn test_permission_user_crud_operations() {
    let mut auth = AuthManager::new();
    auth.register("user", "pass", Role::User).unwrap();
    let session = auth.login("user", "pass").unwrap();

    assert!(auth
        .check_permission(&session.id, &Operation::Select)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Insert)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Update)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Delete)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Create)
        .is_err());
    assert!(auth
        .check_permission(&session.id, &Operation::Drop)
        .is_err());
}

#[test]
fn test_permission_readonly_only_select() {
    let mut auth = AuthManager::new();
    auth.register("reader", "pass", Role::Readonly).unwrap();
    let session = auth.login("reader", "pass").unwrap();

    assert!(auth
        .check_permission(&session.id, &Operation::Select)
        .is_ok());
    assert!(auth
        .check_permission(&session.id, &Operation::Insert)
        .is_err());
    assert!(auth
        .check_permission(&session.id, &Operation::Update)
        .is_err());
    assert!(auth
        .check_permission(&session.id, &Operation::Delete)
        .is_err());
    assert!(auth
        .check_permission(&session.id, &Operation::Create)
        .is_err());
    assert!(auth
        .check_permission(&session.id, &Operation::Drop)
        .is_err());
}

#[test]
fn test_permission_denied_error() {
    let mut auth = AuthManager::new();
    auth.register("reader", "pass", Role::Readonly).unwrap();
    let session = auth.login("reader", "pass").unwrap();

    let result = auth.check_permission(&session.id, &Operation::Insert);
    assert!(matches!(result, Err(AuthError::PermissionDenied(_))));
}

#[test]
fn test_default_auth_manager() {
    let auth = AuthManager::default();
    let users = auth.list_users();
    assert!(users.is_empty());
}

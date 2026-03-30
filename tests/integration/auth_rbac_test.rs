use sqlrustgo_catalog::{
    AuthManager, MysqlDbTable, MysqlUserTable, ObjectRef, ObjectType, Privilege, ScramCredential,
    UserIdentity,
};

fn create_auth_manager() -> AuthManager {
    AuthManager::new()
}

fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[test]
fn test_auth_password_mismatch() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    let password_hash = hash_password("correct_password");
    auth_manager.create_user(&identity, &password_hash).unwrap();

    let result = auth_manager.authenticate(&identity, "wrong_password");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("Invalid") || err.message.contains("password"));
}

#[test]
fn test_auth_user_not_exists() {
    let auth_manager = create_auth_manager();
    let identity = UserIdentity::new("nonexistent", "localhost");

    let result = auth_manager.authenticate(&identity, "any_password");
    assert!(result.is_err());
}

#[test]
fn test_auth_host_mismatch() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    let password_hash = hash_password("password123");
    auth_manager.create_user(&identity, &password_hash).unwrap();

    let wrong_host_identity = UserIdentity::new("alice", "192.168.1.1");
    let result = auth_manager.authenticate(&wrong_host_identity, "password123");
    assert!(result.is_err());
}

#[test]
fn test_auth_wildcard_fallback() {
    let mut auth_manager = create_auth_manager();
    let wildcard_identity = UserIdentity::new("alice", "%");
    let password_hash = hash_password("password123");
    auth_manager
        .create_user(&wildcard_identity, &password_hash)
        .unwrap();

    let exact_identity = UserIdentity::new("alice", "localhost");
    let result = auth_manager.authenticate(&exact_identity, "password123");
    assert!(result.is_ok());
}

#[test]
fn test_user_cannot_access_other_user_data() {
    let mut auth_manager = create_auth_manager();
    let alice = UserIdentity::new("alice", "localhost");
    let bob = UserIdentity::new("bob", "localhost");
    auth_manager.create_user(&alice, "hash").unwrap();
    auth_manager.create_user(&bob, "hash").unwrap();

    auth_manager
        .grant_privilege(&bob, Privilege::Read, ObjectType::Table, "bob_data", 0)
        .unwrap();

    let result =
        auth_manager.check_privilege(&alice, &ObjectRef::table("bob_data"), Privilege::Read);
    assert!(result.is_err());
}

#[test]
fn test_default_no_privilege() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    let result =
        auth_manager.check_privilege(&identity, &ObjectRef::database("any_db"), Privilege::Read);
    assert!(result.is_err());
}

#[test]
fn test_schema_level_isolation() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Database, "db1", 0)
        .unwrap();

    let db1_result =
        auth_manager.check_privilege(&identity, &ObjectRef::database("db1"), Privilege::Read);
    assert!(db1_result.is_ok());

    let db2_result =
        auth_manager.check_privilege(&identity, &ObjectRef::database("db2"), Privilege::Read);
    assert!(db2_result.is_err());
}

#[test]
fn test_grant_then_can_access() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    let object = ObjectRef::table("users");
    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Table, "users", 0)
        .unwrap();

    let result = auth_manager.check_privilege(&identity, &object, Privilege::Read);
    assert!(result.is_ok());
}

#[test]
fn test_grant_allows_multiple_privileges() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    let object = ObjectRef::database("mydb");
    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Database, "mydb", 0)
        .unwrap();
    auth_manager
        .grant_privilege(
            &identity,
            Privilege::Insert,
            ObjectType::Database,
            "mydb",
            0,
        )
        .unwrap();

    assert!(auth_manager
        .check_privilege(&identity, &object, Privilege::Read)
        .is_ok());
    assert!(auth_manager
        .check_privilege(&identity, &object, Privilege::Insert)
        .is_ok());
    assert!(auth_manager
        .check_privilege(&identity, &object, Privilege::Update)
        .is_err());
}

#[test]
fn test_revoke_then_cannot_access() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Table, "users", 0)
        .unwrap();

    assert!(auth_manager
        .check_privilege(&identity, &ObjectRef::table("users"), Privilege::Read)
        .is_ok());

    auth_manager
        .revoke_privilege(&identity, Privilege::Read, ObjectType::Table, "users")
        .unwrap();

    assert!(auth_manager
        .check_privilege(&identity, &ObjectRef::table("users"), Privilege::Read)
        .is_err());
}

#[test]
fn test_revoke_idempotent() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Table, "users", 0)
        .unwrap();

    auth_manager
        .revoke_privilege(&identity, Privilege::Read, ObjectType::Table, "users")
        .unwrap();

    let result =
        auth_manager.revoke_privilege(&identity, Privilege::Read, ObjectType::Table, "users");
    assert!(result.is_ok());
}

#[test]
fn test_create_duplicate_user_fails() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash1").unwrap();

    let result = auth_manager.create_user(&identity, "hash2");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("already exists"));
}

#[test]
fn test_drop_user_auth_fails() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    let password_hash = hash_password("password123");
    auth_manager.create_user(&identity, &password_hash).unwrap();

    auth_manager.drop_user(&identity).unwrap();

    let result = auth_manager.authenticate(&identity, "password123");
    assert!(result.is_err());
}

#[test]
fn test_drop_user_host_isolation() {
    let mut auth_manager = create_auth_manager();
    let alice_localhost = UserIdentity::new("alice", "localhost");
    let alice_wildcard = UserIdentity::new("alice", "%");
    auth_manager.create_user(&alice_localhost, "hash").unwrap();
    auth_manager.create_user(&alice_wildcard, "hash").unwrap();

    auth_manager.drop_user(&alice_localhost).unwrap();

    assert!(auth_manager.find_exact_user(&alice_localhost).is_none());
    assert!(auth_manager.find_exact_user(&alice_wildcard).is_some());
}

#[test]
fn test_mysql_user_reflects_create() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    let rows = MysqlUserTable::rows(&auth_manager);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "alice");
    assert_eq!(rows[0][1], "localhost");
}

#[test]
fn test_mysql_user_reflects_drop() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    auth_manager.drop_user(&identity).unwrap();

    let rows = MysqlUserTable::rows(&auth_manager);
    assert!(rows.is_empty());
}

#[test]
fn test_mysql_db_reflects_grant() {
    let mut auth_manager = create_auth_manager();
    let identity = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&identity, "hash").unwrap();

    auth_manager
        .grant_privilege(&identity, Privilege::Read, ObjectType::Database, "mydb", 0)
        .unwrap();

    let rows = MysqlDbTable::rows(&auth_manager);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][2], "alice");
    assert_eq!(rows[0][1], "mydb");
    assert_eq!(rows[0][3], "Y");
}

#[test]
fn test_salt_randomness() {
    let cred1 = ScramCredential::new("password");
    let cred2 = ScramCredential::new("password");

    assert_ne!(
        cred1.salt, cred2.salt,
        "Salt should be random for same password"
    );
}

#[test]
fn test_iteration_count_correct() {
    let cred = ScramCredential::new("password");

    assert_eq!(cred.iterations, ScramCredential::DEFAULT_ITERATIONS);
    assert_eq!(cred.iterations, 32768);
}

#[test]
fn test_stored_key_not_password_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let password = "mysecretpassword";
    let cred = ScramCredential::new(password);

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    let password_hash = format!("{:x}", hasher.finish());

    assert_ne!(
        cred.stored_key,
        password_hash.as_bytes().to_vec(),
        "stored_key should not equal simple password hash"
    );
    assert_ne!(
        hex::encode(&cred.stored_key),
        password_hash,
        "stored_key hex should not equal simple password hash"
    );
}

#[test]
fn test_cannot_grant_privilege_not_owned() {
    let mut auth_manager = create_auth_manager();
    let alice = UserIdentity::new("alice", "localhost");
    let bob = UserIdentity::new("bob", "localhost");
    auth_manager.create_user(&alice, "hash").unwrap();
    auth_manager.create_user(&bob, "hash").unwrap();

    let has_grant_option =
        auth_manager.has_grant_option(&alice, Privilege::Grant, ObjectType::Database, "mydb");
    assert!(
        !has_grant_option,
        "Alice should not have grant option on mydb"
    );
}

#[test]
fn test_non_admin_cannot_create_user() {
    let mut auth_manager = create_auth_manager();
    let alice = UserIdentity::new("alice", "localhost");
    auth_manager.create_user(&alice, "hash").unwrap();

    let alice_has_admin =
        auth_manager.has_grant_option(&alice, Privilege::Grant, ObjectType::Database, "*");
    assert!(
        !alice_has_admin,
        "Regular user should not have admin privileges"
    );
}

#[test]
fn test_non_admin_cannot_grant() {
    let mut auth_manager = create_auth_manager();
    let alice = UserIdentity::new("alice", "localhost");
    let bob = UserIdentity::new("bob", "localhost");
    auth_manager.create_user(&alice, "hash").unwrap();
    auth_manager.create_user(&bob, "hash").unwrap();

    let alice_can_grant_to_bob =
        auth_manager.has_grant_option(&alice, Privilege::Read, ObjectType::Database, "mydb");
    assert!(
        !alice_can_grant_to_bob,
        "Alice should not be able to grant to Bob"
    );
}

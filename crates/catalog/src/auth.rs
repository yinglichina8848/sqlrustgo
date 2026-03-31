//! Authentication and Authorization Module
//!
//! Provides RBAC (Role-Based Access Control) functionality:
//! - User management and authentication
//! - Role management with inheritance
//! - Privilege grants and revocation
//! - Permission checking

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserIdentity {
    pub username: String,
    pub host: String,
}

impl UserIdentity {
    pub fn new(username: &str, host: &str) -> Self {
        Self {
            username: username.to_lowercase(),
            host: host.to_lowercase(),
        }
    }

    pub fn normalize(&self) -> Self {
        Self {
            username: self.username.to_lowercase(),
            host: self.host.to_lowercase(),
        }
    }
}

impl Serialize for UserIdentity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}@{}", self.username, self.host))
    }
}

impl<'de> Deserialize<'de> for UserIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() == 2 {
            Ok(UserIdentity::new(parts[0], parts[1]))
        } else {
            Ok(UserIdentity::new(&s, "%"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthInfo {
    pub identity: UserIdentity,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScramCredential {
    pub version: u8,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub stored_key: Vec<u8>,
    pub server_key: Vec<u8>,
}

impl ScramCredential {
    pub const CURRENT_VERSION: u8 = 1;
    pub const DEFAULT_ITERATIONS: u32 = 32768;

    pub fn new(password: &str) -> Self {
        let salt = generate_random_salt(32);
        let iterations = Self::DEFAULT_ITERATIONS;

        let salted_password = pbkdf2(password, &salt, iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = hash_sha256(&client_key);
        let server_key = hmac_sha256(&salted_password, b"Server Key");

        Self {
            version: Self::CURRENT_VERSION,
            salt,
            iterations,
            stored_key,
            server_key,
        }
    }

    pub fn verify(&self, password: &str) -> bool {
        let salted_password = pbkdf2(password, &self.salt, self.iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = hash_sha256(&client_key);
        constant_time_eq(&stored_key, &self.stored_key)
    }
}

fn generate_random_salt(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut salt = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

fn pbkdf2(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    use pbkdf2::pbkdf2_hmac_array;
    use sha2::Sha256;

    pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, iterations).to_vec()
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).unwrap();
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn hash_sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Privilege {
    Read,
    Insert,
    Update,
    Delete,
    Alter,
    Drop,
    Create,
    Grant,
    All,
}

#[allow(clippy::should_implement_trait)]
impl Privilege {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "READ" | "SELECT" => Some(Privilege::Read),
            "INSERT" | "WRITE" => Some(Privilege::Insert),
            "UPDATE" => Some(Privilege::Update),
            "DELETE" => Some(Privilege::Delete),
            "ALTER" => Some(Privilege::Alter),
            "DROP" => Some(Privilege::Drop),
            "CREATE" => Some(Privilege::Create),
            "GRANT" => Some(Privilege::Grant),
            "ALL" => Some(Privilege::All),
            _ => None,
        }
    }

    pub fn implies(&self, other: Privilege) -> bool {
        match self {
            Privilege::All => true,
            _ => self == &other,
        }
    }
}

impl std::fmt::Display for Privilege {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Privilege::Read => write!(f, "READ"),
            Privilege::Insert => write!(f, "INSERT"),
            Privilege::Update => write!(f, "UPDATE"),
            Privilege::Delete => write!(f, "DELETE"),
            Privilege::Alter => write!(f, "ALTER"),
            Privilege::Drop => write!(f, "DROP"),
            Privilege::Create => write!(f, "CREATE"),
            Privilege::Grant => write!(f, "GRANT"),
            Privilege::All => write!(f, "ALL"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Database,
    Table,
    Column,
}

#[allow(clippy::should_implement_trait)]
impl ObjectType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DATABASE" | "SCHEMA" => Some(ObjectType::Database),
            "TABLE" => Some(ObjectType::Table),
            "COLUMN" => Some(ObjectType::Column),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GranteeType {
    User,
    Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

impl User {
    pub fn new(id: u64, username: String, password_hash: String) -> Self {
        let now = current_timestamp();
        Self {
            id,
            username,
            password_hash,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: u64,
    pub name: String,
    pub parent_role_id: Option<u64>,
    pub created_at: u64,
    pub description: Option<String>,
}

impl Role {
    pub fn new(id: u64, name: String, parent_role_id: Option<u64>) -> Self {
        Self {
            id,
            name,
            parent_role_id,
            created_at: current_timestamp(),
            description: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
    pub granted_by: u64,
    pub granted_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeGrant {
    pub id: u64,
    pub grantee_type: GranteeType,
    pub grantee_id: u64,
    pub privilege: Privilege,
    pub object_type: ObjectType,
    pub object_name: String,
    pub column_name: Option<String>,
    pub granted_by: u64,
    pub granted_at: u64,
    pub with_grant_option: bool,
}

impl PrivilegeGrant {
    pub fn new(
        id: u64,
        grantee_type: GranteeType,
        grantee_id: u64,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: String,
        granted_by: u64,
    ) -> Self {
        Self {
            id,
            grantee_type,
            grantee_id,
            privilege,
            object_type,
            object_name,
            column_name: None,
            granted_by,
            granted_at: current_timestamp(),
            with_grant_option: false,
        }
    }

    pub fn matches(&self, required: Privilege, object: &ObjectRef) -> bool {
        if self.object_type != object.object_type {
            return false;
        }

        if self.object_name.eq_ignore_ascii_case(&object.object_name) {
            return match self.privilege {
                Privilege::All => true,
                p => p == required,
            };
        }

        if self.object_name == "*" {
            return match self.privilege {
                Privilege::All => true,
                p => p == required,
            };
        }

        if self.object_type == ObjectType::Database && self.object_name == "%" {
            return match self.privilege {
                Privilege::All => true,
                p => p == required,
            };
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct ObjectRef {
    pub object_type: ObjectType,
    pub object_name: String,
    pub column_name: Option<String>,
}

impl ObjectRef {
    pub fn table(name: &str) -> Self {
        Self {
            object_type: ObjectType::Table,
            object_name: name.to_string(),
            column_name: None,
        }
    }

    pub fn column(table: &str, column: &str) -> Self {
        Self {
            object_type: ObjectType::Column,
            object_name: table.to_string(),
            column_name: Some(column.to_string()),
        }
    }

    pub fn database(name: &str) -> Self {
        Self {
            object_type: ObjectType::Database,
            object_name: name.to_string(),
            column_name: None,
        }
    }

    pub fn matches(&self, other: &ObjectRef) -> bool {
        self.object_type == other.object_type && self.object_name == other.object_name
    }
}

#[derive(Debug, Clone)]
pub struct AuthError {
    pub code: AuthErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum AuthErrorCode {
    UserNotFound,
    RoleNotFound,
    DuplicateUser,
    DuplicateRole,
    PermissionDenied,
    AuthenticationFailed,
    InvalidGrant,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Auth error: {} - {}", self.code as u8, self.message)
    }
}

impl std::error::Error for AuthError {}

pub type AuthResult<T> = Result<T, AuthError>;

pub struct AuthManager {
    users: HashMap<UserIdentity, UserAuthInfo>,
    roles: HashMap<u64, Role>,
    roles_by_name: HashMap<String, u64>,
    user_roles: Vec<UserRole>,
    privileges: HashMap<UserIdentity, Vec<PrivilegeGrant>>,
    #[allow(dead_code)]
    next_user_id: u64,
    next_role_id: u64,
    next_grant_id: u64,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: Vec::new(),
            privileges: HashMap::new(),
            next_user_id: 1,
            next_role_id: 1,
            next_grant_id: 1,
        }
    }

    pub fn create_user(&mut self, identity: &UserIdentity, password_hash: &str) -> AuthResult<()> {
        if self.users.contains_key(identity) {
            return Err(AuthError {
                code: AuthErrorCode::DuplicateUser,
                message: format!(
                    "User '{}'@'{}' already exists",
                    identity.username, identity.host
                ),
            });
        }

        let user = UserAuthInfo {
            identity: identity.clone(),
            password_hash: password_hash.to_string(),
            is_active: true,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        self.users.insert(identity.clone(), user);
        Ok(())
    }

    pub fn find_exact_user(&self, identity: &UserIdentity) -> Option<&UserAuthInfo> {
        self.users.get(&identity.normalize())
    }

    pub fn find_wildcard_user(&self, username: &str) -> Option<&UserAuthInfo> {
        self.users.get(&UserIdentity::new(username, "%"))
    }

    pub fn authenticate(&self, identity: &UserIdentity, password: &str) -> AuthResult<u64> {
        if let Some(user) = self.users.get(identity) {
            return self.verify_and_return(identity, password, user);
        }

        let wildcard_identity = UserIdentity::new(&identity.username, "%");
        if let Some(user) = self.users.get(&wildcard_identity) {
            return self.verify_and_return(identity, password, user);
        }

        Err(AuthError {
            code: AuthErrorCode::AuthenticationFailed,
            message: "Invalid username or password".to_string(),
        })
    }

    fn verify_and_return(
        &self,
        _identity: &UserIdentity,
        password: &str,
        user: &UserAuthInfo,
    ) -> AuthResult<u64> {
        if !user.is_active {
            return Err(AuthError {
                code: AuthErrorCode::AuthenticationFailed,
                message: "User account is disabled".to_string(),
            });
        }

        if !verify_password(password, &user.password_hash) {
            return Err(AuthError {
                code: AuthErrorCode::AuthenticationFailed,
                message: "Invalid username or password".to_string(),
            });
        }

        Ok(0)
    }

    pub fn drop_user(&mut self, identity: &UserIdentity) -> AuthResult<()> {
        let normalized = identity.normalize();
        if self.users.remove(&normalized).is_some() {
            Ok(())
        } else {
            Err(AuthError {
                code: AuthErrorCode::UserNotFound,
                message: format!("User '{}'@'{}' not found", identity.username, identity.host),
            })
        }
    }

    pub fn create_role(&mut self, name: &str, parent_role_id: Option<u64>) -> AuthResult<u64> {
        if self.roles_by_name.contains_key(name) {
            return Err(AuthError {
                code: AuthErrorCode::DuplicateRole,
                message: format!("Role '{}' already exists", name),
            });
        }

        if let Some(parent_id) = parent_role_id {
            if !self.roles.contains_key(&parent_id) {
                return Err(AuthError {
                    code: AuthErrorCode::RoleNotFound,
                    message: format!("Parent role {} not found", parent_id),
                });
            }
        }

        let id = self.next_role_id;
        self.next_role_id += 1;

        let role = Role::new(id, name.to_string(), parent_role_id);
        self.roles.insert(id, role);
        self.roles_by_name.insert(name.to_string(), id);

        Ok(id)
    }

    pub fn get_role(&self, id: u64) -> Option<&Role> {
        self.roles.get(&id)
    }

    pub fn grant_role_to_user(
        &mut self,
        user_id: u64,
        role_id: u64,
        granted_by: u64,
    ) -> AuthResult<()> {
        if !self.roles.contains_key(&role_id) {
            return Err(AuthError {
                code: AuthErrorCode::RoleNotFound,
                message: format!("Role {} not found", role_id),
            });
        }

        let user_role = UserRole {
            user_id,
            role_id,
            granted_by,
            granted_at: current_timestamp(),
        };

        self.user_roles.push(user_role);
        Ok(())
    }

    pub fn get_user_roles(&self, user_id: u64) -> Vec<&Role> {
        self.user_roles
            .iter()
            .filter(|ur| ur.user_id == user_id)
            .filter_map(|ur| self.roles.get(&ur.role_id))
            .collect()
    }

    pub fn get_user_roles_recursive(&self, user_id: u64) -> Vec<u64> {
        let mut role_ids = Vec::new();
        self.collect_roles_recursive(user_id, &mut role_ids);
        role_ids
    }

    fn collect_roles_recursive(&self, user_id: u64, role_ids: &mut Vec<u64>) {
        for ur in &self.user_roles {
            if ur.user_id == user_id && !role_ids.contains(&ur.role_id) {
                role_ids.push(ur.role_id);
                if let Some(role) = self.roles.get(&ur.role_id) {
                    if let Some(parent_id) = role.parent_role_id {
                        self.collect_parent_roles(parent_id, role_ids);
                    }
                }
            }
        }
    }

    fn collect_parent_roles(&self, role_id: u64, role_ids: &mut Vec<u64>) {
        if !role_ids.contains(&role_id) {
            role_ids.push(role_id);
            if let Some(role) = self.roles.get(&role_id) {
                if let Some(parent_id) = role.parent_role_id {
                    self.collect_parent_roles(parent_id, role_ids);
                }
            }
        }
    }

    pub fn grant_privilege(
        &mut self,
        identity: &UserIdentity,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
        granted_by: u64,
    ) -> AuthResult<u64> {
        let id = self.next_grant_id;
        self.next_grant_id += 1;

        let grant = PrivilegeGrant {
            id,
            grantee_type: GranteeType::User,
            grantee_id: 0,
            privilege,
            object_type,
            object_name: object_name.to_string(),
            column_name: None,
            granted_by,
            granted_at: current_timestamp(),
            with_grant_option: false,
        };

        self.privileges
            .entry(identity.clone())
            .or_default()
            .push(grant);

        Ok(id)
    }

    pub fn grant_role_privilege(
        &mut self,
        role_id: u64,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
        granted_by: u64,
    ) -> AuthResult<u64> {
        let id = self.next_grant_id;
        self.next_grant_id += 1;

        let grant = PrivilegeGrant {
            id,
            grantee_type: GranteeType::Role,
            grantee_id: role_id,
            privilege,
            object_type,
            object_name: object_name.to_string(),
            column_name: None,
            granted_by,
            granted_at: current_timestamp(),
            with_grant_option: false,
        };

        let public_identity = UserIdentity::new("%", "%");
        self.privileges
            .entry(public_identity)
            .or_default()
            .push(grant);

        Ok(id)
    }

    pub fn revoke_privilege(
        &mut self,
        identity: &UserIdentity,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
    ) -> AuthResult<()> {
        if let Some(grants) = self.privileges.get_mut(identity) {
            grants.retain(|g| {
                !(g.privilege == privilege
                    && g.object_type == object_type
                    && g.object_name == object_name)
            });
        }
        Ok(())
    }

    pub fn check_privilege(
        &self,
        identity: &UserIdentity,
        object: &ObjectRef,
        required_privilege: Privilege,
    ) -> AuthResult<()> {
        let normalized_identity = identity.normalize();

        if let Some(grants) = self.privileges.get(&normalized_identity) {
            for grant in grants {
                if grant.matches(required_privilege, object) {
                    return Ok(());
                }
            }
        }

        let wildcard_identity = UserIdentity::new(&normalized_identity.username, "%");
        if let Some(grants) = self.privileges.get(&wildcard_identity) {
            for grant in grants {
                if grant.matches(required_privilege, object) {
                    return Ok(());
                }
            }
        }

        Err(AuthError {
            code: AuthErrorCode::PermissionDenied,
            message: format!(
                "User '{}'@'{}' does not have {} privilege on '{}'",
                identity.username, identity.host, required_privilege, object.object_name
            ),
        })
    }

    #[allow(dead_code)]
    fn has_direct_privilege(
        &self,
        identity: &UserIdentity,
        privilege: Privilege,
        object: &ObjectRef,
    ) -> bool {
        if let Some(grants) = self.privileges.get(identity) {
            return grants.iter().any(|g| {
                g.grantee_type == GranteeType::User
                    && g.privilege.implies(privilege)
                    && self.matches_object(&g.object_type, &g.object_name, object)
            });
        }
        false
    }

    #[allow(dead_code)]
    fn has_public_privilege(&self, privilege: Privilege, object: &ObjectRef) -> bool {
        let public_identity = UserIdentity::new("%", "%");
        if let Some(grants) = self.privileges.get(&public_identity) {
            return grants.iter().any(|g| {
                g.grantee_type == GranteeType::Role
                    && g.grantee_id == 0
                    && g.privilege.implies(privilege)
                    && self.matches_object(&g.object_type, &g.object_name, object)
            });
        }
        false
    }

    #[allow(dead_code)]
    fn matches_object(
        &self,
        grant_obj_type: &ObjectType,
        grant_obj_name: &str,
        check_obj: &ObjectRef,
    ) -> bool {
        if grant_obj_type != &check_obj.object_type {
            return false;
        }

        if grant_obj_name.eq_ignore_ascii_case(&check_obj.object_name) {
            return true;
        }

        if grant_obj_name == "*" {
            return true;
        }

        if grant_obj_type == &ObjectType::Database && grant_obj_name == "%" {
            return true;
        }

        false
    }

    pub fn get_user_privileges(&self, identity: &UserIdentity) -> Vec<&PrivilegeGrant> {
        self.privileges
            .get(identity)
            .map(|grants| grants.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_grant_option(
        &self,
        identity: &UserIdentity,
        privilege: Privilege,
        object_type: ObjectType,
        object_name: &str,
    ) -> bool {
        if let Some(grants) = self.privileges.get(identity) {
            return grants.iter().any(|g| {
                g.grantee_type == GranteeType::User
                    && g.privilege == privilege
                    && g.object_type == object_type
                    && g.object_name == object_name
                    && g.with_grant_option
            });
        }
        false
    }

    pub fn list_users(&self) -> Vec<&UserAuthInfo> {
        self.users.values().collect()
    }

    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn list_grants(&self) -> Vec<&PrivilegeGrant> {
        self.privileges.values().flat_map(|v| v.iter()).collect()
    }

    pub fn all_users(&self) -> impl Iterator<Item = &UserAuthInfo> {
        self.users.values()
    }

    pub fn all_privileges(&self) -> impl Iterator<Item = (&UserIdentity, &Vec<PrivilegeGrant>)> {
        self.privileges.iter()
    }

    pub fn drop_role(&mut self, role_id: u64) -> AuthResult<()> {
        let role = self.roles.remove(&role_id).ok_or(AuthError {
            code: AuthErrorCode::RoleNotFound,
            message: format!("Role {} not found", role_id),
        })?;

        self.roles_by_name.remove(&role.name);
        self.user_roles.retain(|ur| ur.role_id != role_id);

        for (_, grants) in self.privileges.iter_mut() {
            grants.retain(|g| !(g.grantee_type == GranteeType::Role && g.grantee_id == role_id));
        }

        for r in self.roles.values_mut() {
            if r.parent_role_id == Some(role_id) {
                r.parent_role_id = role.parent_role_id;
            }
        }

        Ok(())
    }
}

impl Default for AuthManager {
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

fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_identity_new_and_normalize() {
        let identity = UserIdentity::new("Alice", "Localhost");
        assert_eq!(identity.username, "alice");
        assert_eq!(identity.host, "localhost");

        let normalized = identity.normalize();
        assert_eq!(normalized.username, "alice");
        assert_eq!(normalized.host, "localhost");
    }

    #[test]
    fn test_user_identity_hash_and_eq() {
        let id1 = UserIdentity::new("alice", "localhost");
        let id2 = UserIdentity::new("Alice", "Localhost");
        let id3 = UserIdentity::new("alice", "127.0.0.1");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        let mut map: HashMap<UserIdentity, i32> = HashMap::new();
        map.insert(id1.clone(), 1);
        assert_eq!(map.get(&id2), Some(&1));
    }

    #[test]
    fn test_create_user() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("password123");

        auth.create_user(&identity, &password_hash).unwrap();

        let user = auth.find_exact_user(&identity).unwrap();
        assert_eq!(user.identity.username, "alice");
        assert_eq!(user.identity.host, "localhost");
        assert!(user.is_active);
    }

    #[test]
    fn test_authenticate() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("password123");

        auth.create_user(&identity, &password_hash).unwrap();

        let result = auth.authenticate(&identity, "password123");
        assert!(result.is_ok());

        let wrong_identity = UserIdentity::new("alice", "wronghost");
        assert!(auth.authenticate(&wrong_identity, "password123").is_err());
        assert!(auth.authenticate(&identity, "wrong").is_err());
    }

    #[test]
    fn test_authenticate_wildcard() {
        let mut auth = AuthManager::new();
        let wildcard_identity = UserIdentity::new("alice", "%");
        let password_hash = hash_password("password123");

        auth.create_user(&wildcard_identity, &password_hash)
            .unwrap();

        let exact_identity = UserIdentity::new("alice", "localhost");
        assert!(auth.authenticate(&exact_identity, "password123").is_ok());
    }

    #[test]
    fn test_create_role() {
        let mut auth = AuthManager::new();
        let role_id = auth.create_role("admin", None).unwrap();
        assert_eq!(role_id, 1);

        let child_id = auth.create_role("editor", Some(role_id)).unwrap();
        assert_eq!(child_id, 2);
    }

    #[test]
    fn test_find_exact_user_and_wildcard_user() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let wildcard_identity = UserIdentity::new("alice", "%");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();
        auth.create_user(&wildcard_identity, &password_hash)
            .unwrap();

        assert!(auth.find_exact_user(&identity).is_some());
        assert!(auth.find_exact_user(&wildcard_identity).is_some());
        assert!(auth.find_wildcard_user("alice").is_some());
        assert!(auth.find_wildcard_user("bob").is_none());
    }

    #[test]
    fn test_drop_user() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();
        assert!(auth.find_exact_user(&identity).is_some());

        auth.drop_user(&identity).unwrap();
        assert!(auth.find_exact_user(&identity).is_none());
    }

    #[test]
    fn test_drop_user_not_found() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");

        let result = auth.drop_user(&identity);
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::UserNotFound,
                ..
            })
        ));
    }

    #[test]
    fn test_duplicate_user_error() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        let result = auth.create_user(&identity, &password_hash);
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::DuplicateUser,
                ..
            })
        ));
    }

    #[test]
    fn test_grant_privilege() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(&identity, Privilege::Read, ObjectType::Table, "users", 0)
            .unwrap();

        assert!(auth
            .check_privilege(&identity, &ObjectRef::table("users"), Privilege::Read)
            .is_ok());
        assert!(auth
            .check_privilege(&identity, &ObjectRef::table("users"), Privilege::Insert)
            .is_err());
    }

    #[test]
    fn test_all_privilege_implies_all() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("admin", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(&identity, Privilege::All, ObjectType::Database, "*", 0)
            .unwrap();

        assert!(auth
            .check_privilege(&identity, &ObjectRef::database("anything"), Privilege::Read)
            .is_ok());
        assert!(auth
            .check_privilege(
                &identity,
                &ObjectRef::database("anything"),
                Privilege::Insert
            )
            .is_ok());
        assert!(auth
            .check_privilege(
                &identity,
                &ObjectRef::database("anything"),
                Privilege::Delete
            )
            .is_ok());
    }

    #[test]
    fn test_privilege_from_str() {
        assert_eq!(Privilege::from_str("READ"), Some(Privilege::Read));
        assert_eq!(Privilege::from_str("SELECT"), Some(Privilege::Read));
        assert_eq!(Privilege::from_str("INSERT"), Some(Privilege::Insert));
        assert_eq!(Privilege::from_str("ALL"), Some(Privilege::All));
        assert_eq!(Privilege::from_str("INVALID"), None);
    }

    #[test]
    fn test_scram_credential_constants() {
        assert_eq!(ScramCredential::CURRENT_VERSION, 1);
        assert_eq!(ScramCredential::DEFAULT_ITERATIONS, 32768);
    }

    #[test]
    fn test_scram_credential_new() {
        let cred = ScramCredential::new("password123");

        assert_eq!(cred.version, ScramCredential::CURRENT_VERSION);
        assert_eq!(cred.salt.len(), 32);
        assert_eq!(cred.iterations, ScramCredential::DEFAULT_ITERATIONS);
        assert_eq!(cred.stored_key.len(), 32);
        assert_eq!(cred.server_key.len(), 32);
    }

    #[test]
    fn test_scram_credential_verify_success() {
        let password = "mysecretpassword";
        let cred = ScramCredential::new(password);

        assert!(cred.verify(password));
    }

    #[test]
    fn test_scram_credential_verify_failure() {
        let cred = ScramCredential::new("correct_password");

        assert!(
            cred.verify("correct_password"),
            "verify with correct password should pass"
        );
        assert!(
            !cred.verify("wrong_password"),
            "verify with wrong password should fail"
        );
        assert!(!cred.verify(""));
        assert!(!cred.verify("correctpassword "));
    }

    #[test]
    fn test_scram_credential_different_passwords_different_keys() {
        let cred1 = ScramCredential::new("password1");
        let cred2 = ScramCredential::new("password2");

        assert_ne!(cred1.stored_key, cred2.stored_key);
        assert_ne!(cred1.server_key, cred2.server_key);
        assert_ne!(cred1.salt, cred2.salt);
    }

    #[test]
    fn test_scram_credential_clone_and_debug() {
        let cred = ScramCredential::new("test_password");
        let cloned = cred.clone();

        assert_eq!(cred.version, cloned.version);
        assert_eq!(cred.salt, cloned.salt);
        assert_eq!(cred.iterations, cloned.iterations);
        assert_eq!(cred.stored_key, cloned.stored_key);
        assert_eq!(cred.server_key, cloned.server_key);

        let debug_str = format!("{:?}", cred);
        assert!(debug_str.contains("ScramCredential"));
    }
}

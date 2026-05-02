//! Authentication and Authorization Module
//!
//! Provides RBAC (Role-Based Access Control) functionality:
//! - User management and authentication
//! - Role management with inheritance
//! - Privilege grants and revocation
//! - Permission checking

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlrustgo_types::SqlResult;
use std::collections::HashMap;

/// User identity (username@host)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserIdentity {
    /// Username
    pub username: String,
    /// Host (%, IP, or hostname)
    pub host: String,
}

impl UserIdentity {
    /// Create a new user identity
    pub fn new(username: &str, host: &str) -> Self {
        Self {
            username: username.to_lowercase(),
            host: host.to_lowercase(),
        }
    }

    /// Normalize the identity
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

/// User authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthInfo {
    /// User identity
    pub identity: UserIdentity,
    /// Password hash
    pub password_hash: String,
    /// Whether user is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
}

/// SCRAM-SHA-256 credential storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScramCredential {
    /// Credential version
    pub version: u8,
    /// Salt for PBKDF2
    pub salt: Vec<u8>,
    /// PBKDF2 iterations
    pub iterations: u32,
    /// Stored key (HMAC-SHA256 of salted password)
    pub stored_key: Vec<u8>,
    /// Server key
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

/// Database privilege types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Privilege {
    /// SELECT privilege
    Read,
    /// INSERT privilege
    Insert,
    /// UPDATE privilege
    Update,
    /// DELETE privilege
    Delete,
    /// ALTER TABLE privilege
    Alter,
    /// DROP privilege
    Drop,
    /// CREATE privilege
    Create,
    /// GRANT OPTION privilege
    Grant,
    /// ALL privileges
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

/// Object type for privileges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    /// Database or schema
    Database,
    /// Table
    Table,
    /// Column
    Column,
}

impl ObjectType {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DATABASE" | "SCHEMA" => Some(ObjectType::Database),
            "TABLE" => Some(ObjectType::Table),
            "COLUMN" => Some(ObjectType::Column),
            _ => None,
        }
    }
}

/// Type of grantee (user or role)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GranteeType {
    /// Individual user
    User,
    /// Role
    Role,
}

/// Database user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u64,
    /// Username
    pub username: String,
    /// Password hash
    pub password_hash: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Whether user is active
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
    pub grant_option: bool,
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
            grant_option: false,
        }
    }

    pub fn matches(&self, required: Privilege, object: &ObjectRef) -> bool {
        if self.object_type != object.object_type {
            return false;
        }

        if self.object_name.eq_ignore_ascii_case(&object.object_name) || self.object_name == "*" {
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

    pub fn matches_column(&self, required: Privilege, table: &str, column: &str) -> bool {
        if self.object_type != ObjectType::Column {
            return false;
        }

        if !self.object_name.eq_ignore_ascii_case(table) {
            return false;
        }

        if !self.privilege.implies(required) {
            return false;
        }

        if let Some(ref granted_column) = self.column_name {
            return granted_column.eq_ignore_ascii_case(column);
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
pub struct GrantInfo {
    pub user: UserIdentity,
    pub privilege: Privilege,
    pub object: ObjectRef,
    pub columns: Option<Vec<String>>,
    pub grant_option: bool,
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

#[derive(Debug, Clone)]
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
        let mut auth = Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: Vec::new(),
            privileges: HashMap::new(),
            next_user_id: 1,
            next_role_id: 1,
            next_grant_id: 1,
        };

        auth.roles
            .insert(0, Role::new(0, "PUBLIC".to_string(), None));
        auth.roles_by_name.insert("PUBLIC".to_string(), 0);

        auth
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
        grantor: &UserIdentity,
        grant_option: bool,
    ) -> AuthResult<u64> {
        let object = ObjectRef {
            object_type,
            object_name: object_name.to_string(),
            column_name: None,
        };

        if grant_option {
            let has_option = self.has_grant_option(grantor, privilege, &object);
            match has_option {
                Ok(true) => {}
                Ok(false) => {
                    return Err(AuthError {
                        code: AuthErrorCode::PermissionDenied,
                        message: "Access denied: you need GRANT OPTION".to_string(),
                    });
                }
                Err(e) => {
                    return Err(AuthError {
                        code: AuthErrorCode::PermissionDenied,
                        message: e.to_string(),
                    });
                }
            }
        }

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
            granted_by: 0,
            granted_at: current_timestamp(),
            grant_option,
        };

        self.privileges
            .entry(identity.clone())
            .or_default()
            .push(grant);

        Ok(id)
    }

    pub fn has_grant_option(
        &self,
        user: &UserIdentity,
        privilege: Privilege,
        object: &ObjectRef,
    ) -> SqlResult<bool> {
        let grants = self.get_user_privileges(user);
        for grant in grants {
            if grant.privilege == privilege
                && grant.object_type == object.object_type
                && grant.object_name == object.object_name
                && grant.grant_option
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn grant_column_privilege(
        &mut self,
        identity: &UserIdentity,
        privilege: Privilege,
        table_name: &str,
        column_name: &str,
        granted_by: u64,
    ) -> AuthResult<u64> {
        let id = self.next_grant_id;
        self.next_grant_id += 1;

        let grant = PrivilegeGrant {
            id,
            grantee_type: GranteeType::User,
            grantee_id: 0,
            privilege,
            object_type: ObjectType::Column,
            object_name: table_name.to_string(),
            column_name: Some(column_name.to_string()),
            granted_by,
            granted_at: current_timestamp(),
            grant_option: false,
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
            grant_option: false,
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
        let grants = self.effective_permissions(identity);

        for grant in grants {
            if grant.matches(required_privilege, object) {
                return Ok(());
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

    pub fn effective_permissions(&self, identity: &UserIdentity) -> Vec<PrivilegeGrant> {
        let mut grants = Vec::new();
        let normalized_identity = identity.normalize();

        if let Some(direct_grants) = self.privileges.get(&normalized_identity) {
            grants.extend(direct_grants.clone());
        }

        let wildcard_identity = UserIdentity::new(&normalized_identity.username, "%");
        if let Some(wildcard_grants) = self.privileges.get(&wildcard_identity) {
            grants.extend(wildcard_grants.clone());
        }

        let public_identity = UserIdentity::new("%", "%");
        if let Some(public_grants) = self.privileges.get(&public_identity) {
            for grant in public_grants {
                if grant.grantee_type == GranteeType::Role && grant.grantee_id == 0 {
                    grants.push(grant.clone());
                }
            }
        }

        grants
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

    pub fn get_all_grants_for_user(&self, user: &UserIdentity) -> Vec<GrantInfo> {
        let mut grants = Vec::new();
        if let Some(user_grants) = self.privileges.get(user) {
            for grant in user_grants {
                let object = ObjectRef {
                    object_type: grant.object_type,
                    object_name: grant.object_name.clone(),
                    column_name: grant.column_name.clone(),
                };
                let columns = grant.column_name.as_ref().map(|c| vec![c.clone()]);
                grants.push(GrantInfo {
                    user: user.clone(),
                    privilege: grant.privilege,
                    object,
                    columns,
                    grant_option: grant.grant_option,
                });
            }
        }
        grants
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

    pub fn get_authorized_columns(
        &self,
        identity: &UserIdentity,
        table_name: &str,
        privilege: Privilege,
    ) -> Vec<String> {
        let mut columns = Vec::new();
        let normalized_identity = identity.normalize();

        if let Some(grants) = self.privileges.get(&normalized_identity) {
            for grant in grants {
                if grant.object_type == ObjectType::Column
                    && grant.object_name.eq_ignore_ascii_case(table_name)
                    && grant.privilege.implies(privilege)
                {
                    if let Some(ref col_name) = grant.column_name {
                        columns.push(col_name.clone());
                    }
                }
            }
        }

        let wildcard_identity = UserIdentity::new(&normalized_identity.username, "%");
        if let Some(grants) = self.privileges.get(&wildcard_identity) {
            for grant in grants {
                if grant.object_type == ObjectType::Column
                    && grant.object_name.eq_ignore_ascii_case(table_name)
                    && grant.privilege.implies(privilege)
                {
                    if let Some(ref col_name) = grant.column_name {
                        if !columns.iter().any(|c| c.eq_ignore_ascii_case(col_name)) {
                            columns.push(col_name.clone());
                        }
                    }
                }
            }
        }

        columns
    }

    pub fn check_table_privilege(
        &self,
        user: &UserIdentity,
        object: &ObjectRef,
        privilege: Privilege,
    ) -> SqlResult<bool> {
        let grants = self.get_user_privileges(user);
        for grant in grants {
            if grant.matches(privilege, object) {
                return Ok(true);
            }
        }
        Ok(false)
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

        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &identity,
            false,
        )
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

        auth.grant_privilege(
            &identity,
            Privilege::All,
            ObjectType::Database,
            "*",
            &identity,
            false,
        )
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

    #[test]
    fn test_grant_role_to_user() {
        let mut auth = AuthManager::new();
        let role_id = auth.create_role("admin", None).unwrap();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        let result = auth.grant_role_to_user(0, role_id, 0);
        assert!(result.is_ok());

        let roles = auth.get_user_roles(0);
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin");
    }

    #[test]
    fn test_grant_role_to_nonexistent_role() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        let result = auth.grant_role_to_user(0, 999, 0);
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::RoleNotFound,
                ..
            })
        ));
    }

    #[test]
    fn test_get_user_roles_recursive() {
        let mut auth = AuthManager::new();
        let admin_id = auth.create_role("admin", None).unwrap();
        let editor_id = auth.create_role("editor", Some(admin_id)).unwrap();
        let viewer_id = auth.create_role("viewer", Some(editor_id)).unwrap();

        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();
        auth.grant_role_to_user(0, viewer_id, 0).unwrap();

        let roles = auth.get_user_roles_recursive(0);
        assert!(roles.contains(&viewer_id));
        assert!(roles.contains(&editor_id));
        assert!(roles.contains(&admin_id));
    }

    #[test]
    fn test_check_privilege_with_wildcard() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &identity,
            false,
        )
        .unwrap();

        let result = auth.check_privilege(&identity, &ObjectRef::table("users"), Privilege::Read);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_privilege_denied() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        let result = auth.check_privilege(&identity, &ObjectRef::table("secret"), Privilege::Read);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError {
                code: AuthErrorCode::PermissionDenied,
                ..
            }
        ));
    }

    #[test]
    fn test_revoke_privilege() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &identity,
            false,
        )
        .unwrap();

        let result = auth.revoke_privilege(&identity, Privilege::Read, ObjectType::Table, "users");
        assert!(result.is_ok());

        let check = auth.check_privilege(&identity, &ObjectRef::table("users"), Privilege::Read);
        assert!(check.is_err());
    }

    #[test]
    fn test_revoke_privilege_not_found() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        let result =
            auth.revoke_privilege(&identity, Privilege::Read, ObjectType::Table, "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_drop_role() {
        let mut auth = AuthManager::new();
        let role_id = auth.create_role("admin", None).unwrap();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();
        auth.grant_role_to_user(0, role_id, 0).unwrap();

        let result = auth.drop_role(role_id);
        assert!(result.is_ok());

        let roles = auth.get_role(role_id);
        assert!(roles.is_none());
    }

    #[test]
    fn test_drop_role_not_found() {
        let mut auth = AuthManager::new();

        let result = auth.drop_role(999);
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::RoleNotFound,
                ..
            })
        ));
    }

    #[test]
    fn test_has_grant_option() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &identity,
            false,
        )
        .unwrap();

        let result = auth.has_grant_option(&identity, Privilege::Read, &ObjectRef::table("users"));
        assert!(!result.unwrap());
    }

    #[test]
    fn test_list_users_and_roles() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();
        auth.create_role("admin", None).unwrap();

        let users = auth.list_users();
        assert_eq!(users.len(), 1);

        let roles = auth.list_roles();
        assert_eq!(roles.len(), 2);
        assert!(roles.iter().any(|r| r.name == "PUBLIC"));
        assert!(roles.iter().any(|r| r.name == "admin"));
    }

    #[test]
    fn test_list_grants() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        let password_hash = hash_password("pass");

        auth.create_user(&identity, &password_hash).unwrap();

        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &identity,
            false,
        )
        .unwrap();

        let grants = auth.list_grants();
        assert_eq!(grants.len(), 1);
    }

    #[test]
    fn test_duplicate_role_error() {
        let mut auth = AuthManager::new();
        auth.create_role("admin", None).unwrap();

        let result = auth.create_role("admin", None);
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::DuplicateRole,
                ..
            })
        ));
    }

    #[test]
    fn test_create_role_with_invalid_parent() {
        let mut auth = AuthManager::new();

        let result = auth.create_role("child", Some(999));
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::RoleNotFound,
                ..
            })
        ));
    }

    #[test]
    fn test_object_ref_table() {
        let table = ObjectRef::table("users");
        assert_eq!(table.object_type, ObjectType::Table);
        assert_eq!(table.object_name, "users");
        assert!(table.column_name.is_none());
    }

    #[test]
    fn test_object_ref_column() {
        let column = ObjectRef::column("users", "id");
        assert_eq!(column.object_type, ObjectType::Column);
        assert_eq!(column.object_name, "users");
        assert_eq!(column.column_name, Some("id".to_string()));
    }

    #[test]
    fn test_object_ref_database() {
        let db = ObjectRef::database("mydb");
        assert_eq!(db.object_type, ObjectType::Database);
        assert_eq!(db.object_name, "mydb");
        assert!(db.column_name.is_none());
    }

    #[test]
    fn test_object_ref_matches() {
        let table1 = ObjectRef::table("users");
        let table2 = ObjectRef::table("users");
        let table3 = ObjectRef::table("orders");

        assert!(table1.matches(&table2));
        assert!(!table1.matches(&table3));
    }

    #[test]
    fn test_privilege_grant_matches() {
        let grant = PrivilegeGrant::new(
            1,
            GranteeType::User,
            0,
            Privilege::Read,
            ObjectType::Table,
            "users".to_string(),
            0,
        );

        assert!(grant.matches(Privilege::Read, &ObjectRef::table("users")));
        assert!(!grant.matches(Privilege::Insert, &ObjectRef::table("users")));
    }

    #[test]
    fn test_privilege_grant_matches_all() {
        let grant = PrivilegeGrant::new(
            1,
            GranteeType::User,
            0,
            Privilege::All,
            ObjectType::Database,
            "*".to_string(),
            0,
        );

        assert!(grant.matches(Privilege::Read, &ObjectRef::database("anything")));
        assert!(grant.matches(Privilege::Insert, &ObjectRef::database("anything")));
    }

    #[test]
    fn test_privilege_grant_matches_wildcard_database() {
        let grant = PrivilegeGrant::new(
            1,
            GranteeType::User,
            0,
            Privilege::Read,
            ObjectType::Database,
            "%".to_string(),
            0,
        );

        assert!(grant.matches(Privilege::Read, &ObjectRef::database("anything")));
    }

    #[test]
    fn test_object_type_from_str() {
        assert_eq!(ObjectType::from_str("TABLE"), Some(ObjectType::Table));
        assert_eq!(ObjectType::from_str("DATABASE"), Some(ObjectType::Database));
        assert_eq!(ObjectType::from_str("SCHEMA"), Some(ObjectType::Database));
        assert_eq!(ObjectType::from_str("COLUMN"), Some(ObjectType::Column));
        assert_eq!(ObjectType::from_str("INVALID"), None);
    }

    #[test]
    fn test_grantee_type() {
        assert_eq!(format!("{:?}", GranteeType::User), "User");
        assert_eq!(format!("{:?}", GranteeType::Role), "Role");
    }

    #[test]
    fn test_auth_error_display() {
        let err = AuthError {
            code: AuthErrorCode::AuthenticationFailed,
            message: "test error".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Auth error"));
        assert!(display.contains("test error"));
    }

    #[test]
    fn test_privilege_display() {
        assert_eq!(format!("{}", Privilege::Read), "READ");
        assert_eq!(format!("{}", Privilege::Insert), "INSERT");
        assert_eq!(format!("{}", Privilege::All), "ALL");
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"test", b"test"));
        assert!(!constant_time_eq(b"test", b"Test"));
        assert!(!constant_time_eq(b"test", b"test1"));
        assert!(!constant_time_eq(b"", b"test"));
    }

    #[test]
    fn test_hash_password() {
        let hash1 = hash_password("password");
        let hash2 = hash_password("password");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_verify_password() {
        let hash = hash_password("secret");
        assert!(verify_password("secret", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_all_users_iterator() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        auth.create_user(&identity, "hash").unwrap();

        let users: Vec<_> = auth.all_users().collect();
        assert_eq!(users.len(), 1);
    }

    #[test]
    fn test_all_privileges_iterator() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        auth.create_user(&identity, "hash").unwrap();
        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "t1",
            &identity,
            false,
        )
        .unwrap();

        let privs: Vec<_> = auth.all_privileges().collect();
        assert_eq!(privs.len(), 1);
    }

    #[test]
    fn test_get_user_privileges() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        auth.create_user(&identity, "hash").unwrap();
        auth.grant_privilege(
            &identity,
            Privilege::Read,
            ObjectType::Table,
            "t1",
            &identity,
            false,
        )
        .unwrap();
        auth.grant_privilege(
            &identity,
            Privilege::Insert,
            ObjectType::Table,
            "t1",
            &identity,
            false,
        )
        .unwrap();

        let privs = auth.get_user_privileges(&identity);
        assert_eq!(privs.len(), 2);
    }

    #[test]
    fn test_grant_column_privilege() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        auth.create_user(&identity, "hash").unwrap();

        auth.grant_column_privilege(&identity, Privilege::Read, "users", "email", 0)
            .unwrap();
        auth.grant_column_privilege(&identity, Privilege::Read, "users", "name", 0)
            .unwrap();

        let authorized = auth.get_authorized_columns(&identity, "users", Privilege::Read);
        assert_eq!(authorized.len(), 2);
        assert!(authorized.contains(&"email".to_string()));
        assert!(authorized.contains(&"name".to_string()));
    }

    #[test]
    fn test_matches_column() {
        let grant = PrivilegeGrant::new(
            1,
            GranteeType::User,
            0,
            Privilege::Read,
            ObjectType::Column,
            "users".to_string(),
            0,
        );
        let grant_with_column = PrivilegeGrant {
            column_name: Some("email".to_string()),
            ..grant.clone()
        };

        assert!(grant_with_column.matches_column(Privilege::Read, "users", "email"));
        assert!(!grant_with_column.matches_column(Privilege::Read, "users", "password"));
        assert!(!grant_with_column.matches_column(Privilege::Insert, "users", "email"));
    }

    #[test]
    fn test_get_authorized_columns_wildcard() {
        let mut auth = AuthManager::new();
        let identity = UserIdentity::new("alice", "localhost");
        auth.create_user(&identity, "hash").unwrap();

        let wildcard_identity = UserIdentity::new("alice", "%");
        auth.grant_column_privilege(&wildcard_identity, Privilege::Read, "users", "email", 0)
            .unwrap();

        let authorized = auth.get_authorized_columns(&identity, "users", Privilege::Read);
        assert_eq!(authorized.len(), 1);
        assert!(authorized.contains(&"email".to_string()));
    }

    #[test]
    fn test_delete_without_privilege() {
        let mut auth = AuthManager::new();
        let alice = UserIdentity::new("alice", "localhost");
        auth.create_user(&alice, "hash").unwrap();

        let result =
            auth.check_table_privilege(&alice, &ObjectRef::table("users"), Privilege::Delete);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_show_grants() {
        let mut auth = AuthManager::new();
        let alice = UserIdentity::new("alice", "localhost");
        auth.create_user(&alice, "hash").unwrap();

        auth.grant_privilege(
            &alice,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &alice,
            false,
        )
        .unwrap();

        let grants = auth.get_all_grants_for_user(&alice);
        assert!(grants.iter().any(|g| g.privilege == Privilege::Read));
    }

    #[test]
    fn test_grant_option() {
        let mut auth = AuthManager::new();
        let alice = UserIdentity::new("alice", "localhost");
        let bob = UserIdentity::new("bob", "localhost");
        auth.create_user(&alice, "hash").unwrap();
        auth.create_user(&bob, "hash").unwrap();

        auth.grant_privilege(
            &bob,
            Privilege::Read,
            ObjectType::Table,
            "users",
            &alice,
            false,
        )
        .unwrap();

        let has_go = auth.has_grant_option(&bob, Privilege::Read, &ObjectRef::table("users"));
        assert!(has_go.is_ok());
        assert!(!has_go.unwrap());
    }

    #[test]
    fn test_public_role() {
        let mut auth = AuthManager::new();
        let alice = UserIdentity::new("alice", "localhost");
        auth.create_user(&alice, "hash").unwrap();

        auth.grant_role_privilege(0, Privilege::Read, ObjectType::Table, "users", 0)
            .unwrap();

        let result = auth.check_privilege(&alice, &ObjectRef::table("users"), Privilege::Read);
        assert!(result.is_ok());
    }
}

//! Authentication and Authorization Module
//!
//! Provides RBAC (Role-Based Access Control) functionality:
//! - User management and authentication
//! - Role management with inheritance
//! - Privilege grants and revocation
//! - Permission checking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    users: HashMap<u64, User>,
    users_by_name: HashMap<String, u64>,
    roles: HashMap<u64, Role>,
    roles_by_name: HashMap<String, u64>,
    user_roles: Vec<UserRole>,
    privileges: Vec<PrivilegeGrant>,
    next_user_id: u64,
    next_role_id: u64,
    next_grant_id: u64,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            users_by_name: HashMap::new(),
            roles: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: Vec::new(),
            privileges: Vec::new(),
            next_user_id: 1,
            next_role_id: 1,
            next_grant_id: 1,
        }
    }

    pub fn create_user(&mut self, username: &str, password: &str) -> AuthResult<u64> {
        if self.users_by_name.contains_key(username) {
            return Err(AuthError {
                code: AuthErrorCode::DuplicateUser,
                message: format!("User '{}' already exists", username),
            });
        }

        let id = self.next_user_id;
        self.next_user_id += 1;

        let user = User::new(id, username.to_string(), hash_password(password));
        self.users.insert(id, user);
        self.users_by_name.insert(username.to_string(), id);

        Ok(id)
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users_by_name
            .get(username)
            .and_then(|id| self.users.get(id))
    }

    pub fn authenticate(&self, username: &str, password: &str) -> AuthResult<u64> {
        let user = self.get_user_by_name(username).ok_or(AuthError {
            code: AuthErrorCode::AuthenticationFailed,
            message: "Invalid username or password".to_string(),
        })?;

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

        Ok(user.id)
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
        if !self.users.contains_key(&user_id) {
            return Err(AuthError {
                code: AuthErrorCode::UserNotFound,
                message: format!("User {} not found", user_id),
            });
        }

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

    pub fn grant_privilege(&mut self, grant: PrivilegeGrant) -> AuthResult<u64> {
        let id = self.next_grant_id;
        self.next_grant_id += 1;

        let mut g = grant;
        g.id = id;
        g.granted_at = current_timestamp();

        self.privileges.push(g);
        Ok(id)
    }

    pub fn revoke_privilege(&mut self, grant_id: u64) -> AuthResult<()> {
        let pos = self
            .privileges
            .iter()
            .position(|g| g.id == grant_id)
            .ok_or(AuthError {
                code: AuthErrorCode::InvalidGrant,
                message: format!("Grant {} not found", grant_id),
            })?;

        self.privileges.remove(pos);
        Ok(())
    }

    pub fn check_privilege(&self, user_id: u64, privilege: Privilege, object: &ObjectRef) -> bool {
        if self.has_direct_privilege(user_id, privilege, object) {
            return true;
        }

        let role_ids = self.get_user_roles_recursive(user_id);
        for role_id in role_ids {
            if self.has_role_privilege(role_id, privilege, object) {
                return true;
            }
        }

        if self.has_public_privilege(privilege, object) {
            return true;
        }

        false
    }

    fn has_direct_privilege(&self, user_id: u64, privilege: Privilege, object: &ObjectRef) -> bool {
        self.privileges.iter().any(|g| {
            (g.grantee_type == GranteeType::User
                && g.grantee_id == user_id
                && g.privilege.implies(privilege)
                && self.matches_object(&g.object_type, &g.object_name, object))
            || (g.privilege == Privilege::All
                && g.grantee_id == user_id
                && g.object_type == ObjectType::Database
                && g.object_name == "*"
                && g.privilege.implies(privilege))
        })
    }

    fn has_role_privilege(&self, role_id: u64, privilege: Privilege, object: &ObjectRef) -> bool {
        self.privileges.iter().any(|g| {
            (g.grantee_type == GranteeType::Role
                && g.grantee_id == role_id
                && g.privilege.implies(privilege)
                && self.matches_object(&g.object_type, &g.object_name, object))
            || (g.privilege == Privilege::All
                && g.grantee_id == role_id
                && g.object_type == ObjectType::Database
                && g.object_name == "*"
                && g.privilege.implies(privilege))
        })
    }

    fn has_public_privilege(&self, privilege: Privilege, object: &ObjectRef) -> bool {
        self.privileges.iter().any(|g| {
            (g.grantee_type == GranteeType::Role
                && g.grantee_id == 0
                && g.privilege.implies(privilege)
                && self.matches_object(&g.object_type, &g.object_name, object))
            || (g.privilege == Privilege::All
                && g.grantee_id == 0
                && g.object_type == ObjectType::Database
                && g.object_name == "*"
                && g.privilege.implies(privilege))
        })
    }

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

    pub fn get_user_privileges(&self, user_id: u64) -> Vec<&PrivilegeGrant> {
        self.privileges
            .iter()
            .filter(|g| {
                (g.grantee_type == GranteeType::User && g.grantee_id == user_id)
                    || (g.grantee_type == GranteeType::Role
                        && self.user_has_role(user_id, g.grantee_id))
            })
            .collect()
    }

    fn user_has_role(&self, user_id: u64, role_id: u64) -> bool {
        let role_ids = self.get_user_roles_recursive(user_id);
        role_ids.contains(&role_id)
    }

    pub fn has_grant_option(&self, user_id: u64, grant_id: u64) -> bool {
        if let Some(grant) = self.privileges.iter().find(|g| g.id == grant_id) {
            if grant.grantee_type == GranteeType::User && grant.grantee_id == user_id {
                return grant.with_grant_option;
            }
        }
        false
    }

    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn list_grants(&self) -> Vec<&PrivilegeGrant> {
        self.privileges.iter().collect()
    }

    pub fn drop_user(&mut self, user_id: u64) -> AuthResult<()> {
        let user = self.users.remove(&user_id).ok_or(AuthError {
            code: AuthErrorCode::UserNotFound,
            message: format!("User {} not found", user_id),
        })?;

        self.users_by_name.remove(&user.username);
        self.user_roles.retain(|ur| ur.user_id != user_id);
        self.privileges
            .retain(|g| !(g.grantee_type == GranteeType::User && g.grantee_id == user_id));

        Ok(())
    }

    pub fn drop_role(&mut self, role_id: u64) -> AuthResult<()> {
        let role = self.roles.remove(&role_id).ok_or(AuthError {
            code: AuthErrorCode::RoleNotFound,
            message: format!("Role {} not found", role_id),
        })?;

        self.roles_by_name.remove(&role.name);
        self.user_roles.retain(|ur| ur.role_id != role_id);
        self.privileges
            .retain(|g| !(g.grantee_type == GranteeType::Role && g.grantee_id == role_id));

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
    fn test_create_user() {
        let mut auth = AuthManager::new();
        let id = auth.create_user("alice", "password123").unwrap();
        assert_eq!(id, 1);

        let user = auth.get_user(id).unwrap();
        assert_eq!(user.username, "alice");
    }

    #[test]
    fn test_authenticate() {
        let mut auth = AuthManager::new();
        auth.create_user("alice", "password123").unwrap();

        let id = auth.authenticate("alice", "password123").unwrap();
        assert_eq!(id, 1);

        assert!(auth.authenticate("alice", "wrong").is_err());
        assert!(auth.authenticate("bob", "password").is_err());
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
    fn test_grant_role_to_user() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("alice", "pass").unwrap();
        let role_id = auth.create_role("admin", None).unwrap();

        auth.grant_role_to_user(user_id, role_id, 0).unwrap();

        let roles = auth.get_user_roles(user_id);
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "admin");
    }

    #[test]
    fn test_role_inheritance() {
        let mut auth = AuthManager::new();
        let parent_id = auth.create_role("parent", None).unwrap();
        let child_id = auth.create_role("child", Some(parent_id)).unwrap();

        auth.create_role("grandchild", Some(child_id)).unwrap();

        let mut user_id = auth.create_user("test", "pass").unwrap();
        auth.grant_role_to_user(user_id, 3, 0).unwrap(); // grandchild role

        let role_ids = auth.get_user_roles_recursive(user_id);
        assert!(role_ids.contains(&3)); // grandchild
        assert!(role_ids.contains(&2)); // child
        assert!(role_ids.contains(&1)); // parent
    }

    #[test]
    fn test_grant_privilege() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("alice", "pass").unwrap();

        let grant = PrivilegeGrant::new(
            0,
            GranteeType::User,
            user_id,
            Privilege::Read,
            ObjectType::Table,
            "users".to_string(),
            0,
        );

        let grant_id = auth.grant_privilege(grant).unwrap();
        assert_eq!(grant_id, 1);

        assert!(auth.check_privilege(user_id, Privilege::Read, &ObjectRef::table("users")));
        assert!(!auth.check_privilege(user_id, Privilege::Insert, &ObjectRef::table("users")));
    }

    #[test]
    fn test_privilege_via_role() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("alice", "pass").unwrap();
        let role_id = auth.create_role("reader", None).unwrap();

        auth.grant_role_to_user(user_id, role_id, 0).unwrap();

        let grant = PrivilegeGrant::new(
            0,
            GranteeType::Role,
            role_id,
            Privilege::Read,
            ObjectType::Table,
            "orders".to_string(),
            0,
        );
        auth.grant_privilege(grant).unwrap();

        assert!(auth.check_privilege(user_id, Privilege::Read, &ObjectRef::table("orders")));
    }

    #[test]
    fn test_all_privilege_implies_all() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("admin", "pass").unwrap();

        let grant = PrivilegeGrant::new(
            0,
            GranteeType::User,
            user_id,
            Privilege::All,
            ObjectType::Database,
            "*".to_string(),
            0,
        );
        auth.grant_privilege(grant).unwrap();

        assert!(auth.check_privilege(user_id, Privilege::Read, &ObjectRef::table("anything")));
        assert!(auth.check_privilege(user_id, Privilege::Insert, &ObjectRef::table("anything")));
        assert!(auth.check_privilege(user_id, Privilege::Delete, &ObjectRef::table("anything")));
    }

    #[test]
    fn test_revoke_privilege() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("alice", "pass").unwrap();

        let grant = PrivilegeGrant::new(
            0,
            GranteeType::User,
            user_id,
            Privilege::Read,
            ObjectType::Table,
            "users".to_string(),
            0,
        );

        let grant_id = auth.grant_privilege(grant).unwrap();
        assert!(auth.check_privilege(user_id, Privilege::Read, &ObjectRef::table("users")));

        auth.revoke_privilege(grant_id).unwrap();
        assert!(!auth.check_privilege(user_id, Privilege::Read, &ObjectRef::table("users")));
    }

    #[test]
    fn test_drop_user_removes_grants() {
        let mut auth = AuthManager::new();
        let user_id = auth.create_user("alice", "pass").unwrap();

        let grant = PrivilegeGrant::new(
            0,
            GranteeType::User,
            user_id,
            Privilege::Read,
            ObjectType::Table,
            "users".to_string(),
            0,
        );
        auth.grant_privilege(grant).unwrap();

        auth.drop_user(user_id).unwrap();
        assert!(auth.list_grants().is_empty());
    }

    #[test]
    fn test_duplicate_user_error() {
        let mut auth = AuthManager::new();
        auth.create_user("alice", "pass").unwrap();

        let result = auth.create_user("alice", "pass2");
        assert!(matches!(
            result,
            Err(AuthError {
                code: AuthErrorCode::DuplicateUser,
                ..
            })
        ));
    }

    #[test]
    fn test_privilege_from_str() {
        assert_eq!(Privilege::from_str("READ"), Some(Privilege::Read));
        assert_eq!(Privilege::from_str("SELECT"), Some(Privilege::Read));
        assert_eq!(Privilege::from_str("INSERT"), Some(Privilege::Insert));
        assert_eq!(Privilege::from_str("ALL"), Some(Privilege::All));
        assert_eq!(Privilege::from_str("INVALID"), None);
    }
}

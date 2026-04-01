# Issue #956: RBAC 权限系统详细设计

**Issue**: #956 RBAC 权限系统 - 用户/角色/GRANT
**创建日期**: 2026-03-28
**状态**: 设计完成

## 1. 背景与目标

现有代码已有基本的 GRANT/REVOKE 语法解析，但缺少：
- User/Role 数据模型存储
- Privilege 存储机制
- 权限检查执行逻辑

**目标**: 实现完整的 RBAC 权限系统

## 2. 数据模型

### 2.1 User

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}
```

### 2.2 Role

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: u64,
    pub name: String,
    pub parent_role_id: Option<u64>,  // 角色继承
    pub created_at: u64,
    pub description: Option<String>,
}
```

### 2.3 PrivilegeGrant

```rust
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GranteeType {
    User,
    Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObjectType {
    Database,
    Table,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
```

### 2.4 UserRole

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: u64,
    pub role_id: u64,
    pub granted_by: u64,
    pub granted_at: u64,
}
```

## 3. 核心组件

### 3.1 AuthStore

```rust
pub struct AuthStore {
    users: HashMap<u64, User>,
    roles: HashMap<u64, Role>,
    user_roles: Vec<UserRole>,
    privileges: Vec<PrivilegeGrant>,
    next_user_id: u64,
    next_role_id: u64,
    next_grant_id: u64,
}
```

### 3.2 AuthManager

```rust
impl AuthManager {
    pub fn new() -> Self;
    
    // 用户管理
    pub fn create_user(&mut self, username: &str, password: &str) -> Result<u64>;
    pub fn get_user(&self, id: u64) -> Option<&User>;
    pub fn authenticate(&self, username: &str, password: &str) -> Result<u64>;
    
    // 角色管理
    pub fn create_role(&mut self, name: &str, parent: Option<u64>) -> Result<u64>;
    pub fn grant_role_to_user(&mut self, user_id: u64, role_id: u64) -> Result<()>;
    
    // 权限管理
    pub fn grant(&mut self, grant: PrivilegeGrant) -> Result<()>;
    pub fn revoke(&mut self, grant_id: u64) -> Result<()>;
    pub fn check_privilege(&self, user_id: u64, priv: Privilege, object: &ObjectRef) -> bool;
    
    // 辅助
    pub fn get_user_roles(&self, user_id: u64) -> Vec<&Role>;
    pub fn get_user_privileges(&self, user_id: u64) -> Vec<&PrivilegeGrant>;
}
```

## 4. 权限检查算法

```rust
fn check_privilege(&self, user_id: u64, privilege: Privilege, object: &ObjectRef) -> bool {
    // 1. 直接检查用户权限
    if self.has_direct_privilege(user_id, privilege, object) {
        return true;
    }
    
    // 2. 检查用户角色权限（含继承）
    let roles = self.get_user_roles_recursive(user_id);
    for role in roles {
        if self.has_role_privilege(role.id, privilege, object) {
            return true;
        }
    }
    
    // 3. 检查 PUBLIC 权限
    if self.has_public_privilege(privilege, object) {
        return true;
    }
    
    false
}
```

## 5. GRANT/REVOKE 执行

### 5.1 GRANT 语法

```sql
GRANT <privilege> ON <object> TO <user|role> [WITH GRANT OPTION]
```

### 5.2 GRANT 执行流程

```rust
pub fn execute_grant(&mut self, stmt: GrantStatement, grantor_id: u64) -> Result<()> {
    // 1. 验证 grantor 有 GRANT OPTION
    if !self.check_privilege(grantor_id, Privilege::Grant, &object) {
        return Err(AuthError::PermissionDenied);
    }
    
    // 2. 查找被授权者
    let grantee = self.resolve_grantee(&stmt.to_user)?;
    
    // 3. 创建权限授予记录
    let grant = PrivilegeGrant {
        id: self.next_grant_id(),
        grantee_type: grantee.0,
        grantee_id: grantee.1,
        privilege: stmt.privilege,
        object_type: object.0,
        object_name: object.1,
        column_name: None,
        granted_by: grantor_id,
        granted_at: current_timestamp(),
    };
    
    // 4. 存储
    self.privileges.push(grant);
    Ok(())
}
```

## 6. 文件结构

```
crates/catalog/src/
├── lib.rs
├── auth.rs           # NEW: User, Role, PrivilegeGrant, AuthStore, AuthManager
└── ...
```

## 7. 与 Parser 集成

Parser 已有的 `GrantStatement`, `RevokeStatement`, `Privilege` 保持不变，仅在 executor 执行层使用 `AuthManager` 进行权限检查。

## 8. 实现计划

1. **Phase 1**: `auth.rs` - 数据模型和 AuthStore
2. **Phase 2**: `AuthManager` - 权限检查逻辑
3. **Phase 3**: GRANT/REVOKE 执行集成
4. **Phase 4**: 集成测试

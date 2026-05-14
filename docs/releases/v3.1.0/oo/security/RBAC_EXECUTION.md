# RBAC Execution Layer (DCL from Parse to Execution)

> **Document ID**: OO-SEC-RBAC-001
> **Version**: v3.1.0
> **Last Updated**: 2026-05-12
> **Parent Issue**: #619 (InnoDB Semantic Compatibility)
> **Milestone**: v3.1.0-beta

## 1. Overview

This document describes the RBAC (Role-Based Access Control) execution layer implementation for SQLRustGo v3.1.0. The RBAC system handles DCL (Data Control Language) operations including GRANT and REVOKE statements.

## 2. Architecture

### 2.1 Component Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                     ExecutionEngine                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  execute_grant() / execute_revoke()                  │   │
│  │  - RBAC permission checks                           │   │
│  │  - current_user context                              │   │
│  └─────────────────────────────────────────────────────┘   │
│                            │                               │
│                            ▼                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Catalog (AuthManager)                    │   │
│  │  - grant_privilege()                                │   │
│  │  - revoke_privilege()                               │   │
│  │  - has_grant_option()                               │   │
│  └─────────────────────────────────────────────────────┘   │
│                            │                               │
│                            ▼                               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Privilege Storage (in-memory)              │   │
│  │  - privileges HashMap                               │   │
│  │  - PrivilegeGrant records                            │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Key Components

| Component | Location | Responsibility |
|----------|----------|----------------|
| ExecutionEngine | `src/execution_engine.rs` | DCL statement execution, RBAC context |
| AuthManager | `crates/catalog/src/auth.rs` | Privilege management, permission checks |
| Catalog | `crates/catalog/src/catalog.rs` | Wraps AuthManager, provides catalog API |
| DdlExecutor | `crates/executor/src/ddl_executor.rs` | Low-level DDL operations |

## 3. GRANT Execution Flow

### 3.1 Permission Check Flow

```
User executes: GRANT SELECT ON db.tbl TO user WITH GRANT OPTION
                           │
                           ▼
              ┌────────────────────────┐
              │ Parse GRANT statement  │
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ Set current_user via   │
              │ set_current_user()      │
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ execute_grant() called │
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ Check: current_user exists?        │
              │ Error if not set                  │
              └────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ For each privilege to grant:       │
              │   Call catalog.grant_privilege()   │
              │   with current_user as grantor     │
              └────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ AuthManager.grant_privilege()     │
              │ - Check has_grant_option           │
              │ - If WITH GRANT OPTION: require    │
              │   current_user has GRANT OPTION     │
              │ - Store PrivilegeGrant record     │
              └────────────────────────────────────┘
```

### 3.2 Implementation Details

**ExecutionEngine Fields**:
```rust
pub struct ExecutionEngine<S: StorageEngine> {
    // ... other fields
    current_user: Option<UserIdentity>,
}
```

**Permission Check in execute_grant()**:
```rust
fn execute_grant(&mut self, grant: &GrantStatement) -> SqlResult<ExecutorResult> {
    // Get current user context
    let current_user = self.current_user.as_ref().ok_or_else(|| {
        SqlError::ExecutionError("No current user set for GRANT".to_string())
    })?;

    // ... privilege mapping ...

    // Call catalog.grant_privilege with current_user as grantor
    catalog.grant_privilege(
        &identity,      // grantee
        priv_obj,       // privilege
        obj_type,       // object type
        &grant.object_name,
        current_user,   // grantor (for GRANT OPTION check)
        grant.with_grant_option,
    )?
}
```

## 4. REVOKE Execution Flow

### 4.1 Permission Check Flow

```
User executes: REVOKE SELECT ON db.tbl FROM user
                           │
                           ▼
              ┌────────────────────────┐
              │ Parse REVOKE statement │
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────┐
              │ execute_revoke() called│
              └────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ Check: current_user exists?        │
              └────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ Build ObjectRef for target         │
              └────────────────────────────────────┘
                           │
                           ▼
              ┌────────────────────────────────────┐
              │ Check: current_user has            │
              │ has_grant_option on target?        │
              └────────────────────────────────────┘
                           │
                    ┌───────┴───────┐
                    │ YES           │ NO
                    ▼               ▼
        ┌──────────────────┐   ┌──────────────────┐
        │ Proceed with     │   │ Return Error:    │
        │ revoke_privilege │   │ "Access denied:  │
        │                  │   │ you need GRANT   │
        │                  │   │ OPTION"          │
        └──────────────────┘   └──────────────────┘
```

### 4.2 Implementation Details

**Permission Check in execute_revoke()**:
```rust
fn execute_revoke(&mut self, revoke: &RevokeStatement) -> SqlResult<ExecutorResult> {
    let current_user = self.current_user.as_ref().ok_or_else(|| {
        SqlError::ExecutionError("No current user set for REVOKE".to_string())
    })?;

    // Build ObjectRef
    let object_ref = sqlrustgo_catalog::auth::ObjectRef {
        object_type: obj_type,
        object_name: revoke.object_name.clone(),
        column_name: None,
    };

    // Check GRANT OPTION
    let has_grant_option = catalog
        .auth_manager()
        .has_grant_option(current_user, priv_obj, &object_ref)?;

    if !has_grant_option {
        return Err(SqlError::ExecutionError(
            "Access denied: you need GRANT OPTION to REVOKE".to_string(),
        ));
    }

    // Proceed with revoke
    catalog.revoke_privilege(&identity, priv_obj, obj_type, &revoke.object_name)?
}
```

## 5. Privilege Model

### 5.1 Privilege Types

| Privilege | Description | SQL Keywords |
|-----------|-------------|--------------|
| Read | SELECT privilege | SELECT, READ |
| Insert | INSERT privilege | INSERT, WRITE |
| Update | UPDATE privilege | UPDATE |
| Delete | DELETE privilege | DELETE |
| Alter | ALTER TABLE privilege | ALTER |
| Drop | DROP privilege | DROP |
| Create | CREATE privilege | CREATE |
| Grant | GRANT OPTION | GRANT |
| All | All privileges | ALL |

### 5.2 Object Types

| Object Type | Description |
|-------------|-------------|
| Database | Schema/database level |
| Table | Table level |
| Column | Column level |

### 5.3 PrivilegeGrant Structure

```rust
pub struct PrivilegeGrant {
    pub id: u64,
    pub grantee_type: GranteeType,      // User or Role
    pub grantee_id: u64,
    pub privilege: Privilege,
    pub object_type: ObjectType,
    pub object_name: String,
    pub column_name: Option<String>,
    pub granted_by: u64,
    pub granted_at: u64,
    pub grant_option: bool,            // Can grant this privilege
}
```

## 6. API Reference

### 6.1 ExecutionEngine Methods

#### set_current_user()
```rust
pub fn set_current_user(&mut self, user: UserIdentity)
```
Sets the current user context for RBAC permission checks.

#### get_current_user()
```rust
pub fn get_current_user(&self) -> Option<&UserIdentity>
```
Returns the current user identity.

#### clear_current_user()
```rust
pub fn clear_current_user(&mut self)
```
Clears the current user context (e.g., on logout).

### 6.2 AuthManager Methods

#### grant_privilege()
```rust
pub fn grant_privilege(
    &mut self,
    identity: &UserIdentity,     // grantee
    privilege: Privilege,
    object_type: ObjectType,
    object_name: &str,
    grantor: &UserIdentity,      // who is granting
    grant_option: bool,
) -> AuthResult<u64>
```

#### has_grant_option()
```rust
pub fn has_grant_option(
    &self,
    user: &UserIdentity,
    privilege: Privilege,
    object: &ObjectRef,
) -> SqlResult<bool>
```
Checks if user has GRANT OPTION on the specified object.

#### revoke_privilege()
```rust
pub fn revoke_privilege(
    &mut self,
    identity: &UserIdentity,
    privilege: Privilege,
    object_type: ObjectType,
    object_name: &str,
) -> AuthResult<()>
```

## 7. Acceptance Criteria

| ID | Criteria | Status |
|----|----------|--------|
| AC-1 | `GRANT SELECT ON db.tbl TO user;` actually grants the privilege | Implemented |
| AC-2 | `GRANT SELECT ON db.tbl TO user WITH GRANT OPTION;` grants both SELECT and GRANT OPTION | Implemented |
| AC-3 | User without GRANT OPTION cannot grant privileges | Implemented |
| AC-4 | `REVOKE SELECT ON db.tbl FROM user;` actually revokes the privilege | Implemented |
| AC-5 | User without GRANT OPTION cannot revoke privileges | Implemented |
| AC-6 | OO documentation completed | Pending |

## 8. Future Enhancements

### 8.1 Cascading REVOKE
Currently, REVOKE does not cascade to dependent grants. Future implementation should handle:
- When user A grants to user B WITH GRANT OPTION
- And user B grants to user C
- REVOKE from user A should cascade to user C

### 8.2 Audit Logging Integration
- Integration with AuditManager for GRANT/REVOKE events
- Tamper-evident audit trail

### 8.3 Role-based REVOKE CASCADE
- RESTRICT vs CASCADE behavior for role hierarchies

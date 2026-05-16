# DCL 权限链 + 冷存储实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 MySQL 兼容的 DCL 权限管理 (RLS谓词重写 + 角色嵌套) 和 S3 冷存储完善 (签名修复 + StorageTierManager)

**Architecture:**
- DCL: 在 `catalog/src/auth.rs` 新增 `RowLevelSecurity` 模块，在 planner 层注入 RLS 谓词
- 冷存储: 修复 `backup_storage.rs` 的 S3 签名，新增 `StorageTierManager` 处理冷热分层

**Tech Stack:** Rust, aws-config/手动签名, reqwest

---

## 一、DCL 权限链完善 (#994)

### Task 1: RowLevelSecurity 模块

**Files:**
- Create: `crates/catalog/src/auth_rls.rs`
- Modify: `crates/catalog/src/auth.rs` (添加 `use auth_rls::RowLevelSecurity`)
- Test: `crates/catalog/tests/auth_rls_test.rs`

**Step 1: Write failing test**

```rust
// crates/catalog/tests/auth_rls_test.rs
#[test]
fn test_rls_predicate_generation() {
    let rls = RowLevelSecurity::new();
    rls.add_policy(1, "orders", "region = '华北'");

    let predicate = rls.get_predicate(1, "orders");
    assert_eq!(predicate, Some("region = '华北'".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-catalog auth_rls -- --nocapture`
Expected: FAIL - auth_rls module not found

**Step 3: Write minimal RowLevelSecurity implementation**

```rust
// crates/catalog/src/auth_rls.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TablePolicy {
    pub table_name: String,
    pub predicate: String,
}

#[derive(Debug, Clone, Default)]
pub struct RowLevelSecurity {
    policies: HashMap<u64, Vec<TablePolicy>>,
}

impl RowLevelSecurity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_policy(&mut self, user_id: u64, table: &str, predicate: &str) {
        self.policies
            .entry(user_id)
            .or_default()
            .push(TablePolicy {
                table_name: table.to_string(),
                predicate: predicate.to_string(),
            });
    }

    pub fn get_predicate(&self, user_id: u64, table: &str) -> Option<String> {
        self.policies
            .get(&user_id)
            .and_then(|policies| {
                policies
                    .iter()
                    .find(|p| p.table_name == table)
                    .map(|p| p.predicate.clone())
            })
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-catalog auth_rls -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/catalog/src/auth_rls.rs crates/catalog/tests/auth_rls_test.rs
git commit -m "feat(catalog): add RowLevelSecurity module for RLS"
```

---

### Task 2: AuthManager 集成 RLS

**Files:**
- Modify: `crates/catalog/src/auth.rs` (在 AuthManager 添加 rls 字段和方法)
- Modify: `crates/catalog/tests/auth_rls_test.rs` (添加集成测试)

**Step 1: Write failing test**

```rust
#[test]
fn test_auth_manager_rls_integration() {
    let mut auth = AuthManager::new();
    auth.add_rls_policy(1, "orders", "region = '华北'");

    let predicate = auth.get_rls_predicate(1, "orders");
    assert_eq!(predicate, Some("region = '华北'".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-catalog auth_manager_rls -- --nocapture`
Expected: FAIL - method not found

**Step 3: Add RLS field and methods to AuthManager**

```rust
// 在 AuthManager struct 添加:
pub struct AuthManager {
    // ... existing fields ...
    rls: RowLevelSecurity,
}

// 添加方法:
impl AuthManager {
    pub fn add_rls_policy(&mut self, user_id: u64, table: &str, predicate: &str) {
        self.rls.add_policy(user_id, table, predicate);
    }

    pub fn get_rls_predicate(&self, user_id: u64, table: &str) -> Option<String> {
        self.rls.get_predicate(user_id, table)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-catalog auth_manager_rls -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/catalog/src/auth.rs crates/catalog/tests/auth_rls_test.rs
git commit -m "feat(catalog): integrate RLS into AuthManager"
```

---

### Task 3: 角色嵌套环形检测

**Files:**
- Modify: `crates/catalog/src/auth.rs` (在 create_role 添加环形检测)
- Modify: `crates/catalog/tests/auth_rls_test.rs` (添加环形检测测试)

**Step 1: Write failing test**

```rust
#[test]
fn test_role_cycle_detection() {
    let mut auth = AuthManager::new();

    let admin_id = auth.create_role("admin", None).unwrap();
    let manager_id = auth.create_role("manager", Some(admin_id)).unwrap();

    // 尝试创建形成环的角色 - 应该失败
    let result = auth.create_role("employee", Some(manager_id));
    // 此时不会形成环，因为 manager -> admin -> None
    assert!(result.is_ok());

    // 但如果尝试让 admin 的父角色指向 employee，形成环
    // 这需要额外的 set_role_parent 方法
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-catalog role_cycle -- --nocapture`
Expected: 可能 PASS 或 FAIL 取决于实现

**Step 3: 实现环形检测**

```rust
impl AuthManager {
    pub fn create_role(&mut self, name: &str, parent_role_id: Option<u64>) -> AuthResult<u64> {
        // 检查是否形成环
        if let Some(parent_id) = parent_role_id {
            if self.would_create_cycle(parent_id, id) {
                return Err(AuthError {
                    code: AuthErrorCode::RoleCycleDetected,
                    message: format!("Creating role '{}' with parent {} would create a cycle", name, parent_id),
                });
            }
        }
        // ... existing code ...
    }

    fn would_create_cycle(&self, new_parent_id: u64, role_id: u64) -> bool {
        let mut current = Some(new_parent_id);
        while let Some(id) = current {
            if id == role_id {
                return true;
            }
            current = self.roles.get(&id).and_then(|r| r.parent_role_id);
        }
        false
    }
}

// 添加新错误码
#[derive(Debug, Clone, Copy)]
pub enum AuthErrorCode {
    // ... existing ...
    RoleCycleDetected,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-catalog role_cycle -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git commit -m "feat(catalog): add role cycle detection"
```

---

### Task 4: 执行器集成 RLS 谓词重写

**Files:**
- Modify: `crates/executor/src/...` (在 SELECT 执行路径注入 RLS 谓词)
- Test: `crates/executor/tests/...`

**Step 1: Write failing test**

```rust
#[test]
fn test_select_with_rls_predicate() {
    // 准备: 创建表 + 用户 + RLS 策略
    // 执行: SELECT * FROM orders
    // 验证: 实际执行的是 SELECT * FROM orders WHERE region = '华北'
}
```

**Step 2: Run test to verify it fails**

**Step 3: 实现谓词注入**

在 executor 的 TableScan 或 Filter 阶段:
```rust
// 如果用户有 RLS 策略，添加额外过滤条件
if let Some(rls_pred) = auth.get_rls_predicate(user_id, "orders") {
    extra_filter = Some(rls_pred);
}
```

**Step 4: Run test to verify it passes**

**Step 5: Commit**

```bash
git commit -m "feat(executor): integrate RLS predicate injection"
```

---

## 二、冷存储完善 (#993)

### Task 5: 修复 S3 签名实现

**Files:**
- Modify: `crates/storage/src/backup_storage.rs` (修复 RemoteBackupStorage)

**Step 1: Write failing test (需要 mock 或实际 S3)**

```rust
#[test]
fn test_s3_upload_download() {
    // 需要 MinIO 或 mock
    // 先写一个签名验证测试
}
```

**Step 2: 分析当前签名代码的 bug**

当前代码问题:
```rust
// 错误 - 硬编码 {} 而没有实际签名计算
format!("AWS4-HMAC-SHA256 Credential={}/{{}}", self.config.access_key)
```

**Step 3: 实现正确的 AWS Signature V4**

```rust
impl RemoteBackupStorage {
    fn sign_request(&self, method: &str, path: &str, body: &[u8]) -> Result<HashMap<String, String>, S3Error> {
        use sha2::{Digest, Sha256};
        use hmac::{Hmac, Mac};

        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        let datetime = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

        // Credential: access_key/date/region/service/aws4_request
        let credential = format!(
            "{}/{}/{}/s3/aws4_request",
            self.config.access_key, date, self.config.region
        );

        // SignedHeaders
        let signed_headers = "host;x-amz-content-sha256;x-amz-date";

        // Canonical request
        let canonical_request = format!(
            "{}\n{}\n{}\nhost\n\neSha256\n{}",
            method,
            path,
            "",
            hex::encode(Sha256::digest(body))
        );

        // String to sign
        let canonical_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{}\n{}/{}/s3/aws4_request\n{}",
            datetime, date, self.config.region, canonical_hash
        );

        // Calculate signature
        let signature = self.calculate_signature(&string_to_sign, &date)?;

        let mut headers = HashMap::new();
        headers.insert("x-amz-date".to_string(), datetime);
        headers.insert("x-amz-content-sha256".to_string(), hex::encode(Sha256::digest(body)));
        headers.insert("Authorization".to_string(),
            format!("AWS4-HMAC-SHA256 Credential={}, SignedHeaders={}, Signature={}",
                credential, signed_headers, signature));

        Ok(headers)
    }

    fn calculate_signature(&self, string_to_sign: &str, date: &str) -> Result<String, S3Error> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let k_date = self.sign_sha256(&("AWS4".as_bytes()), date.as_bytes())?;
        let k_region = self.sign_sha256(&k_date, self.config.region.as_bytes())?;
        let k_service = self.sign_sha256(&k_region, b"s3")?;
        let k_signing = self.sign_sha256(&k_service, b"aws4_request")?;

        let mut mac = HmacSha256::new_from_slice(&k_signing).map_err(|_| S3Error::SigningError)?;
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize().into_bytes();

        Ok(hex::encode(result))
    }

    fn sign_sha256(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, S3Error> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(key).map_err(|_| S3Error::SigningError)?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}
```

**Step 4: 运行测试验证**

**Step 5: Commit**

```bash
git commit -m "fix(storage): implement correct AWS Signature V4 for S3"
```

---

### Task 6: StorageTierManager 冷热分层

**Files:**
- Create: `crates/storage/src/storage_tier.rs`
- Modify: `crates/storage/src/backup_storage.rs` (导出新模块)
- Test: `crates/storage/tests/storage_tier_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_tiering_policy_age() {
    let policy = TieringPolicy::default();
    assert!(policy.should_tier_to_cold(30, 0, 0)); // 30天

    assert!(!policy.should_tier_to_cold(5, 0, 0)); // 5天
}
```

**Step 2: Run test to verify it fails**

**Step 3: 实现 StorageTierManager**

```rust
// crates/storage/src/storage_tier.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageTier {
    Hot,
    Cold,
}

#[derive(Debug, Clone)]
pub struct TieringPolicy {
    pub age_threshold_days: u32,
    pub size_threshold_gb: u64,
    pub access_count_threshold: u32,
}

impl Default for TieringPolicy {
    fn default() -> Self {
        Self {
            age_threshold_days: 30,
            size_threshold_gb: 100,
            access_count_threshold: 10,
        }
    }
}

impl TieringPolicy {
    pub fn should_tier_to_cold(&self, age_days: u32, size_gb: u64, access_count: u32) -> bool {
        age_days >= self.age_threshold_days
            || size_gb >= self.size_threshold_gb
            || access_count < self.access_count_threshold
    }
}

pub struct StorageTierManager {
    hot: Arc<LocalBackupStorage>,
    cold: Arc<RemoteBackupStorage>,
    policy: TieringPolicy,
    metadata: Arc<RwLock<HashMap<String, StorageTier>>>,
}

impl StorageTierManager {
    pub fn new(hot: LocalBackupStorage, cold: RemoteBackupStorage, policy: TieringPolicy) -> Self {
        Self {
            hot: Arc::new(hot),
            cold: Arc::new(cold),
            policy,
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn save(&self, key: &str, data: &[u8], tier: StorageTier) -> SqlResult<()> {
        match tier {
            StorageTier::Hot => self.hot.save(key, data),
            StorageTier::Cold => self.cold.save(key, data),
        }?;

        self.metadata.write().unwrap().insert(key.to_string(), tier);
        Ok(())
    }

    pub fn load(&self, key: &str) -> SqlResult<Vec<u8>> {
        let tier = self.metadata.read().unwrap().get(key).copied();

        match tier {
            Some(StorageTier::Hot) | None => self.hot.load(key).or_else(|_| self.cold.load(key)),
            Some(StorageTier::Cold) => self.cold.load(key),
        }
    }

    pub fn run_tiering(&self) -> SqlResult<u32> {
        let mut migrated = 0;

        // 扫描热存储
        for key in self.hot.list("")? {
            let metadata = self.hot.load_metadata(&key)?; // 需要实现
            let age_days = metadata.age_days;
            let size_gb = metadata.size_bytes / (1024 * 1024 * 1024);
            let access_count = metadata.access_count;

            if self.policy.should_tier_to_cold(age_days, size_gb, access_count) {
                // 迁移到冷存储
                let data = self.hot.load(&key)?;
                self.cold.save(&key, &data)?;
                self.metadata.write().unwrap().insert(key.clone(), StorageTier::Cold);
                migrated += 1;
            }
        }

        Ok(migrated)
    }
}
```

**Step 4: Run test to verify it passes**

**Step 5: Commit**

```bash
git add crates/storage/src/storage_tier.rs crates/storage/tests/storage_tier_test.rs
git commit -m "feat(storage): add StorageTierManager for hot/cold tiering"
```

---

## 三、测试覆盖

### 运行所有测试

```bash
# DCL 测试
cargo test -p sqlrustgo-catalog --all-features

# 冷存储测试
cargo test -p sqlrustgo-storage --all-features

# 完整测试
cargo test --all-features
```

### 覆盖率检查

```bash
cargo test -p sqlrustgo-catalog --all-features -- --lucet-ratio 80
cargo test -p sqlrustgo-storage --all-features -- --lucet-ratio 75
```

---

## 四、验收清单

- [ ] Task 1-4: DCL RLS + 角色嵌套完成
- [ ] Task 5-6: S3 修复 + StorageTierManager 完成
- [ ] 所有测试 PASS
- [ ] 覆盖率 ≥75%
- [ ] Clippy 零警告
- [ ] Format 检查通过
- [ ] 提交 PR

---

*实现计划由 hermes-agent 创建 (2026-05-16)*

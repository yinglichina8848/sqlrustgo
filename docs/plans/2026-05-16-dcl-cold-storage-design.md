# DCL 权限链 + 冷存储完善设计

> **日期**: 2026-05-16
> **状态**: 已批准
> **维护人**: hermes-agent

---

## 概述

本设计涵盖 v3.2.0 的两个 P1 功能增强:
1. **DCL 权限链完善** - 实现 100% MySQL 兼容的权限管理
2. **冷存储集成** - S3/OSS 完善 + 冷热分层

---

## 一、DCL 权限链完善 (#994)

### 1.1 目标

实现 MySQL 兼容的 DCL 权限管理，包括:
- GRANT/REVOKE 完善
- 角色管理 (含嵌套)
- 行级安全 (RLS)

### 1.2 当前状态

`catalog/src/auth.rs` 已实现:
- ✅ 用户认证 (SCRAM-SHA-256)
- ✅ 角色管理 (含继承)
- ✅ 权限授予/撤销 (表级)
- ✅ 列级权限
- ✅ GRANT OPTION

### 1.3 缺失功能

| 功能 | 优先级 | 说明 |
|------|--------|------|
| 行级安全 (RLS) | P1 | MySQL 8.0 支持谓词重写实现 |
| 角色嵌套完善 | P2 | 环形检测、SET ROLE |
| 动态权限检查 | P2 | 执行时实时权限验证 |

### 1.4 RLS 实现方案

**选择: 谓词重写 (Predicate Rewriting)**

```
原始查询:
  SELECT * FROM orders WHERE amount > 1000

用户 joe@localhost 的 RLS 策略:
  orders.region = '华北'

重写后:
  SELECT * FROM orders WHERE amount > 1000 AND region = '华北'
```

**实现位置**: `catalog/src/auth.rs` 新增 `RowLevelSecurity` 模块

```rust
pub struct RowLevelSecurity {
    // user_id -> (table, predicate)
    policies: HashMap<u64, Vec<TablePolicy>>,
}

pub struct TablePolicy {
    table_name: String,
    predicate: String,  // e.g., "region = '华北'"
}
```

**执行器集成**:
- 在 `executor` 执行 SELECT 前调用 `AuthManager::apply_rls_predicate()`
- 返回重写后的查询条件

### 1.5 角色嵌套完善

```rust
// 当前: 线性继承
admin -> manager -> employee

// 完善: 图结构 + 环形检测
admin -> manager
  └── employee
  └── contractor
contractor -> employee (环形检测需拒绝)
```

### 1.6 验收条件

- [ ] RLS 谓词正确注入到 SELECT 查询
- [ ] 角色嵌套支持环形检测
- [ ] 权限验证集成到执行器
- [ ] 测试覆盖率 ≥80%

---

## 二、冷存储集成 (#993)

### 2.1 目标

实现 S3/OSS 对象存储集成，支持冷数据归档。

### 2.2 当前状态

`backup_storage.rs` 已实现:
- ✅ `BackupStorage` trait 抽象
- ✅ `LocalBackupStorage` 本地存储
- ✅ `RemoteBackupStorage` S3 兼容存储 (有 bug)
- ✅ `BackupStorageManager` 管理器

### 2.3 现有问题

**Bug 1: S3 签名格式错误**
```rust
// 当前 (错误):
format!("AWS4-HMAC-SHA256 Credential={}/{{}}", self.config.access_key)

// 正确应为: 动态计算签名或使用 AWS SDK
```

**Bug 2: 错误处理不完善**
- 网络超时未处理
- 重试逻辑缺失

### 2.4 完善方案

#### 阶段 1: 修复 S3 实现

```rust
// 使用 aws-config 或手动实现签名
impl RemoteBackupStorage {
    fn sign_request(&self, request: &mut Request) -> Result<(), S3Error> {
        // 实现 AWS Signature Version 4
    }
}
```

#### 阶段 2: StorageTierManager

```rust
pub enum StorageTier {
    Hot,    // SSD/本地存储
    Cold,   // S3/OSS
}

pub struct StorageTierManager {
    local: Arc<LocalBackupStorage>,
    remote: Arc<RemoteBackupStorage>,
    policy: TieringPolicy,
}

pub struct TieringPolicy {
    age_threshold_days: u32,      // 超过N天移至冷存储
    size_threshold_gb: u64,      // 超过N GB移至冷存储
    access_count_threshold: u32,  // N次访问后移至冷存储
}
```

#### 阶段 3: 自动迁移

```rust
impl StorageTierManager {
    pub fn run_tiering(&self) -> SqlResult<u32> {
        // 1. 扫描本地文件
        // 2. 检查访问时间/大小
        // 3. 符合条件则上传到 S3
        // 4. 本地保留元数据指针
    }
}
```

### 2.5 支持的存储后端

| 后端 | 状态 | 说明 |
|------|------|------|
| AWS S3 | 完善中 | 需修复签名 |
| 阿里云 OSS | 待实现 | 兼容 S3 协议 |
| 腾讯云 COS | 待实现 | 兼容 S3 协议 |
| MinIO | 待实现 | 本地测试 |

### 2.6 验收条件

- [ ] S3 签名修复，上传/下载正常工作
- [ ] StorageTierManager 实现
- [ ] 冷热分层策略可配置
- [ ] 归档/恢复功能测试通过
- [ ] 测试覆盖率 ≥75%

---

## 三、实现计划

### 分支策略

| 功能 | 分支 | PR 目标 |
|------|------|---------|
| DCL 权限链 | `feature/dcl-permission-chain` | 独立 PR |
| 冷存储 | `feature/cold-storage-tiering` | 独立 PR |

### 依赖关系

```
DCL (feature/dcl-permission-chain)
└── catalog/src/auth.rs 修改
    └── 可能有 executor 集成

冷存储 (feature/cold-storage-tiering)
└── backup_storage.rs 修改
    └── 新增 storage_tier.rs
```

---

## 四、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| RLS 性能开销 | 中 | 谓词下推优化 |
| S3 签名复杂性 | 高 | 考虑使用 aws-sdk 或简化实现 |
| 冷存储迁移一致性 | 高 | 使用 WAL + 两阶段提交 |

---

## 五、测试策略

### DCL 测试
```bash
# 单元测试
cargo test -p sqlrustgo-catalog auth

# 集成测试
# 1. GRANT/REVOKE 语法测试
# 2. RLS 谓词重写测试
# 3. 角色嵌套测试
```

### 冷存储测试
```bash
# 单元测试
cargo test -p sqlrustgo-storage backup

# 集成测试 (需 MinIO)
# 1. S3 上传/下载测试
# 2. 冷热分层测试
# 3. 归档恢复测试
```

---

*本文档由 hermes-agent 创建 (2026-05-16)*

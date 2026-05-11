# v3.1.0 GMP 合规性路线图

> **版本**: 1.0  
> **日期**: 2026-05-11  
> **状态**: 🟡 规划中  
> **目标**: 以 GMP（药品生产质量管理规范）为核心场景，建立完整审计链、可验证事务隔离和防篡改能力

---

## 一、GMP 核心需求映射

| GMP 要求 | 技术实现 | 当前状态 | v3.1.0 目标 |
|---------|---------|---------|-------------|
| **数据完整性** | 强 ACID，SERIALIZABLE，无脏写/幻读 | SSI 50%，Gap Locking ❌ | **Gap Locking + 完整 SERIALIZABLE 证明** |
| **全面审计** | 审计日志不可篡改（SHA-256 链） | audit trail 部分完成 | **Crash-safe 审计，WAL replay 一致，篡改报警** |
| **访问控制** | 细粒度权限，列级/行级安全 | RBAC 基础，列级 ❌ | **列级权限 + 行级安全基础** |
| **传输加密** | TLS | 已实现 | ✅ 保持 |
| **存储加密** | AES-256 页级加密 | ❌ | **实现 AES-256 加密** |
| **崩溃零丢失** | WAL + MVCC 完整验证 | BP2-2 未完成 | **混沌测试矩阵 5 场景全覆盖** |
| **高可用** | 组复制/自动故障转移 | Semi-sync 可用 | **P2（v3.2.0）** |
| **操作可追溯** | binlog，事件调度，触发器 | 基础 | **事件调度器 + 触发器完善** |

---

## 二、GMP P0 任务详解

### 2.1 BP2-1~BP2-6: 审计日志完整合规

**目标**: 审计日志满足 21 CFR Part 11（电子记录不可篡改性）

#### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    审计日志架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              AuditTrail (不可变日志链)                 │  │
│  │  ┌─────────────────────────────────────────────┐   │  │
│  │  │  Entry: {                                   │   │  │
│  │  │    seq: u64,                               │   │  │
│  │  │    prev_hash: SHA-256,                     │   │  │
│  │  │    timestamp: DateTime,                    │   │  │
│  │  │    user: String,                           │   │  │
│  │  │    action: DML|DDL|AUTH|...,               │   │  │
│  │  │    sql: String,                             │   │  │
│  │  │    session_id: u64,                         │   │  │
│  │  │    row_count: u64,                          │   │  │
│  │  │    hash: SHA-256 (self)                     │   │  │
│  │  │  }                                         │   │  │
│  │  └─────────────────────────────────────────────┘   │  │
│  └─────────────────────────────────────────────────────┘  │
│                            │                               │
│                            ▼                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              WAL Integration                         │  │
│  │  • 审计写 WAL（与其他数据 redo 共用）                 │  │
│  │  • checkpoint 时审计链一起刷盘                         │  │
│  │  • 崩溃恢复时 WAL replay 验证审计链完整性             │  │
│  └─────────────────────────────────────────────────────┘  │
│                            │                               │
│                            ▼                               │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              Tamper Detection                         │  │
│  │  • 启动时扫描历史审计链，检测断链                     │  │
│  │  • 实时监控：哈希链中断报警                           │  │
│  │  • 证据导出：JSON + 可验证签名                        │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

#### 崩溃注入矩阵（BP2-2）

| 场景 | 注入时机 | 预期恢复行为 |
|------|---------|-------------|
| S1 | WAL write 前 crash | 审计丢失，业务数据未变 → 事务回滚 |
| S2 | WAL write 后，未 commit | 审计存在，业务数据回滚 → 一致 |
| S3 | pre-commit crash | 审计存在，MVCC 标记未决 → recovery 回滚 |
| S4 | checkpoint 时 crash | 审计链 snapshot 完整 → 恢复后无丢失 |
| S5 | torn page (断电) | AES-NI 加密保证页级原子性 → 无部分写入 |

#### 核心文件

```
crates/gmp/src/
├── audit.rs              # 审计日志核心（不可变链）
├── evidence.rs          # 证据生成与验证
├── tamper_detector.rs    # 篡改检测
└── lib.rs               # 模块入口

crates/transaction/src/
├── audit_integration.rs  # 事务层审计 hook
└── recovery.rs          # WAL replay 审计验证
```

#### 验收条件

```
✓ 审计写入吞吐 ≥ 10,000 条/秒
✓ 100 并发下审计链无断链
✓ 崩溃注入 S1~S5 全部可恢复
✓ 篡改检测在 100ms 内报警
✓ 证据导出通过第三方验证
```

---

### 2.2 Gap Locking + SERIALIZABLE

**目标**: 完整实现 Next-Key Locking，消除幻读

#### 当前状态

```
crates/transaction/src/
├── ssi.rs           # SSI 已实现但压力测试未完成
├── lock_manager.rs  # 锁管理器（基础）
└── lock.rs          # 锁粒度（表级/行级）
```

#### 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│              Gap Locking 实现                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  LockManager                                                │
│  ├── table_locks: HashMap<TableId, TableLock>             │
│  ├── row_locks: BTreeMap<(PageId, SlotId), RowLock>      │
│  └── gap_locks: BTreeMap<(PageId, KeyRange), GapLock>     │
│                                                             │
│  GapLock 类型:                                               │
│  ├── RECORD_LOCK     — 记录锁（行级）                        │
│  ├── GAP_LOCK       — 间隙锁（区间）                        │
│  ├── NEXT_KEY_LOCK  — Next-Key（记录+间隙）                │
│  └── AUTO_INC_LOCK  — 自增锁                                │
│                                                             │
│  锁协议:                                                     │
│  1. 等值查询 → RECORD_LOCK 或 NEXT_KEY_LOCK                 │
│  2. 范围查询 → NEXT_KEY_LOCK 锁住整个区间                   │
│  3. 插入 → 检查插入点的 GAP_LOCK，无冲突则获 AUTO_INC_LOCK  │
│  4. SSI 检测 → 写入偏序（Write-Read, Write-Write）检测     │
└─────────────────────────────────────────────────────────────┘
```

#### SERIALIZABLE 验证矩阵

| 测试场景 | 期望行为 | 验证方法 |
|---------|---------|---------|
| T1: 并发 INSERT + SELECT 同一区间 | SELECT 看到或看不到插入，但不能 phantom | 100 并发，验证无幻读 |
| T2: 并发 UPDATE 同一行 | 只有一个成功，另一个等锁超时或回滚 | 锁顺序验证 |
| T3: 范围 UPDATE + 范围 SELECT | 快照隔离，UPDATE 不影响 SELECT | 读写偏序检测 |
| T4: SSI 死锁检测 | 检测到死锁后选择一个事务回滚 | 死锁超时 < 100ms |
| T5: Serializable SELECT + DML | SELECT 看到一致性快照 | 可序列化验证 |

#### 核心文件

```
crates/transaction/src/
├── gap_lock.rs          # GapLock 类型与逻辑
├── next_key_lock.rs     # Next-Key Lock 算法
├── ssi_detector.rs      # SSI 偏序检测增强
└── serializable_test.rs # 可序列化验证（新建）
```

#### 验收条件

```
✓ Gap Locking 覆盖等值查询、范围查询、插入场景
✓ SERIALIZABLE 隔离级别下 100 并发 T1~T5 全部无幻读/脏读
✓ SSI 死锁检测延迟 < 100ms
✓ Performance regression: Gap Locking 引入的 overhead ≤ 5%
```

---

### 2.3 聚簇索引

**目标**: 主键索引即数据，二级索引指向主键

#### 当前架构（B+Tree 无聚簇）

```
叶子节点: [key, value_ptr]  →  value 在独立页
```

#### 目标架构（Clustered B+Tree）

```
叶子节点: [key, primary_key, row_data]
           └─ 主键        └─ 行数据直接存储

非叶子节点: [key, child_page_ptr]

二级索引: [index_key, primary_key]  →  再查主键索引获取 row_data
```

#### ADR 决策点

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| 主键不存在时 | AUTO_INCREMENT 或 UUID | UUID 作为隐藏主键 | 避免应用侵入 |
| 行数据格式 | 行存储 vs 页存储 | 行存储在叶子节点 | 简化实现，查询只需一次 B+Tree 查找 |
| 二级索引更新 | 延迟更新 vs 同步更新 | 同步更新 | 避免主键索引损坏导致数据不一致 |
| 唯一约束实现 | 额外唯一索引 vs 主键约束 | 主键约束 | InnoDB 语义兼容 |

#### 核心文件

```
crates/storage/src/
├── bplus_tree/
│   ├── clustered_leaf.rs   # 聚簇叶子节点
│   ├── secondary_index.rs  # 二级索引指向主键
│   └── index.rs           # 重构
└── clustered_table.rs      # 表管理层（新建）

crates/storage/src/bplus_tree/mod.rs
```

#### 验收条件

```
✓ 主键查询减少 1 次 I/O（无需再查 value 页）
✓ 范围扫描性能提升 ≥ 20%（数据局部性好）
✓ 二级索引正确指向主键，无数据不一致
✓ 主键不存在时自动生成隐藏主键
✓ 聚簇索引建立后建二级索引，行为正确
```

---

### 2.4 AES-256 存储加密

**目标**: 页级 AES-256 加密，密钥管理与数据分离

#### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│              存储加密架构                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  KeyManager                                                 │
│  ├── master_key: [u8; 32]  ← 环境变量或文件               │
│  ├── key_version: u64                                       │
│  └── rotate() → 新 master_key                              │
│                                                             │
│  PageCipher                                                 │
│  ├── encrypt_page(page_data) → cipher_text + iv           │
│  ├── decrypt_page(cipher_text, iv) → page_data             │
│  └── algorithm: AES-256-GCM                                │
│                                                             │
│  Storage Layer                                              │
│  ├── 写: data_page → PageCipher.encrypt → encrypted_page  │
│  ├── 读: encrypted_page → PageCipher.decrypt → data_page  │
│  └── WAL: 明文 WAL（加密 WAL 需单独机制）                   │
└─────────────────────────────────────────────────────────────┘
```

#### 密钥管理策略

| 方式 | 适用场景 | 安全性 |
|------|---------|--------|
| 环境变量 `SQLRUSTGO_MASTER_KEY` | 开发/测试 | 中 |
| 文件存储（0600 权限） | 生产单节点 | 高 |
| KMIP/KMS 集成 | 未来扩展 | — |

#### 核心文件

```
crates/storage/src/
├── encryption/
│   ├── aes_cipher.rs      # AES-256-GCM 实现
│   ├── key_manager.rs     # 密钥管理
│   └── mod.rs
└── encrypted_storage.rs    # 加密存储层包装
```

#### 验收条件

```
✓ 页级加密 overhead ≤ 3%（AES-NI 硬件加速）
✓ 密钥轮转期间服务不中断
✓ 密钥错误时启动失败（非静默数据损坏）
✓ 加密存储通过 HIPAA/GMP 审计要求
```

---

### 2.5 细粒度权限（列级 + RBAC 执行）

**目标**: RBAC 不仅解析语法，要实际执行权限检查

#### 当前问题

```
现状: GRANT SELECT(col1, col2) ON t → 仅解析
目标: 执行时过滤未授权列，返回错误或空
```

#### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│              权限执行架构                                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  PrivilegeExecutor                                          │
│  ├── check_column_access(user, table, columns) → Vec<Col> │
│  ├── check_row_access(user, table, predicate) → Predicate │
│  └── filter_output(rows, allowed_columns) → rows           │
│                                                             │
│  RBAC 模型:                                                  │
│  ├── User → Roles → Privileges                              │
│  ├── Column-level: GRANT SELECT(col) ON t                  │
│  └── Row-level: CREATE POLICY ... WHERE ...                 │
│                                                             │
│  执行时机:                                                   │
│  ├── 解析后：验证用户是否有权限访问这些列                    │
│  ├── 执行前：行级策略过滤 WHERE 条件                         │
│  └── 返回前：删除未授权列（SELECT * 时）                     │
└─────────────────────────────────────────────────────────────┘
```

#### 核心文件

```
crates/security/src/
├── privilege_executor.rs   # 权限执行（新建）
├── column_acl.rs         # 列级访问控制
├── row_policy.rs         # 行级安全策略
└── lib.rs               # 重构整合

crates/planner/src/
└── privilege_check.rs    # Planner 权限检查 hook
```

#### 验收条件

```
✓ SELECT * 时只返回用户有权限的列（其他列为 NULL 或报错）
✓ 无权限列在 WHERE 中被引用时报错，而非静默忽略
✓ 行级策略：用户只能看到匹配 WHERE 条件的行
✓ 权限验证 overhead ≤ 1ms（单次查询）
```

---

## 三、GMP P1 任务

### 3.1 INFORMATION_SCHEMA 完善

**现状**: `crates/information-schema` 存在但不完整  
**目标**: ≥80% 覆盖率

| 表 | 当前状态 | v3.1.0 目标 |
|---|---------|-------------|
| SCHEMATA | ✅ 基础 | ✅ 完整 |
| TABLES | ⚠️ 部分 | ✅ 完整 |
| COLUMNS | ❌ 缺失 | ✅ 完整 |
| STATISTICS | ❌ 缺失 | ✅ 完整 |
| PARTITIONS | ❌ 缺失 | ✅ 基础 |
| EVENTS | ❌ 缺失 | ✅ 基础 |
| ROUTINES | ❌ 缺失 | ✅ 基础 |
| TRIGGERS | ⚠️ 部分 | ✅ 完整 |
| USER_PRIVILEGES | ❌ 缺失 | ✅ 完整 |
| SCHEMA_PRIVILEGES | ❌ 缺失 | ✅ 基础 |

### 3.2 Performance Schema 基础

**目标**: ≥50% (10+ 表)

| 表 | 说明 |
|---|------|
| setup_actors | 配置监控用户 |
| setup_instruments | 配置 instrument |
| events_statements_summary_by_digest | 语句聚合统计 |
| events_statements_history | 语句历史 |
| events_waits_summary_by_thread | 等待事件 |

### 3.3 MERGE 语句

**目标**: 实现完整的 MERGE INTO（MySQL 8.0 兼容）

```sql
MERGE INTO target AS t
USING source AS s
ON t.id = s.id
WHEN MATCHED THEN
  UPDATE SET t.col = s.col
WHEN NOT MATCHED THEN
  INSERT (id, col) VALUES (s.id, s.col);
```

---

## 四、GMP 测试矩阵

### 4.1 BP2-2 崩溃注入测试（5 场景）

| 测试 ID | 注入点 | 验证点 | 自动化 |
|---------|--------|--------|--------|
| BP2-2-S1 | WAL write 前 | 审计丢失=0 | ✅ chaos_crash_wal_before.rs |
| BP2-2-S2 | WAL write 后未 commit | 审计=已记录，数据=回滚 | ✅ chaos_crash_wal_after.rs |
| BP2-2-S3 | pre-commit | 审计存在，MVCC 标记未决 | ✅ chaos_crash_precommit.rs |
| BP2-2-S4 | checkpoint | 审计链 snapshot 完整 | ✅ chaos_crash_checkpoint.rs |
| BP2-2-S5 | torn page | 页级原子性，无部分写入 | ✅ chaos_crash_torn_page.rs |

### 4.2 审计完整性测试

| 测试 | 方法 | 通过标准 |
|------|------|---------|
| audit_hash_chain | 启动时验证 100 万条审计链 | 0 断链 |
| audit_concurrent_write | 100 并发写入审计 | 序列号连续无丢失 |
| audit_tamper | 修改历史审计条目的 hash | 启动时检测到并报警 |
| audit_evidence_export | 导出证据并第三方验证 | JSON 签名验证通过 |

### 4.3 SERIALIZABLE 隔离测试

| 测试 | 并发 | 验证 |
|------|------|------|
| serializable_phantom_read | 100x INSERT+SELECT | 0 幻读 |
| serializable_gap_lock | 100x 范围 UPDATE+SELECT | 0 幻读 |
| serializable_deadlock | SSI 死锁触发 | 延迟 < 100ms |
| serializable_write_skew | T1(T2) 读写偏序 | 检测并回滚 |

---

## 五、GMP 合规检查清单

> 独立于代码门禁，用于 GMP 预评估

### 5.1 审计完整性

- [ ] 审计日志不可变链已实现
- [ ] SHA-256 哈希链覆盖所有审计条目
- [ ] 崩溃恢复后审计链完整性验证通过
- [ ] 篡改检测功能已测试
- [ ] 证据导出格式符合 21 CFR Part 11

### 5.2 事务隔离

- [ ] Gap Locking 实现完整
- [ ] SERIALIZABLE 下无幻读证明（100 并发）
- [ ] SSI 死锁检测延迟 < 100ms
- [ ] 聚簇索引正确实现

### 5.3 访问控制

- [ ] 列级权限实际执行（非仅解析）
- [ ] 行级安全策略执行
- [ ] RBAC 执行层覆盖所有 DML

### 5.4 存储安全

- [ ] AES-256 页级加密实现
- [ ] 密钥管理（轮转/错误处理）
- [ ] 加密 overhead ≤ 3%

---

*本文档由 hermes agent 创建，基于 GMP 核心需求和 MySQL 对齐目标。*
*每次 Beta/RC/GA Gate 检查后更新状态。*

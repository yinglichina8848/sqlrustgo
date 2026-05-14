# 审计链集成分析

> WAL 协议与审计日志的深度集成分析
>
> **版本**: 1.0
> **日期**: 2026-05-12
> **目标**: 分析审计日志与 WAL 协议的集成方案，实现 crash-safe 审计链

## 1. 整体架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                    审计链完整架构                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐      │
│  │   审计日志    │ ──► │   WAL Log   │ ──► │   恢复引擎   │      │
│  │ AuditLogger  │      │   (wal.rs)  │      │ (recovery)   │      │
│  └──────┬───────┘      └──────┬───────┘      └──────┬───────┘      │
│         │                      │                      │              │
│         │  写入审计条目        │  写入 WAL Entry     │  Replay      │
│         ▼                      ▼                      ▼              │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │                    共享 WAL Buffer                          │      │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐          │      │
│  │  │ BEGIN  │ │ INSERT │ │ AUDIT   │ │ COMMIT  │          │      │
│  │  │ TXN    │ │ row    │ │ entry   │ │ TXN     │          │      │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘          │      │
│  └──────────────────────────────────────────────────────────┘      │
│                              │                                      │
│                              ▼                                      │
│                    ┌─────────────────┐                              │
│                    │   WAL File     │                              │
│                    │ (walfile.dat)  │                              │
│                    └────────┬────────┘                              │
│                             │                                       │
│         ┌───────────────────┼───────────────────┐                   │
│         ▼                   ▼                   ▼                   │
│  ┌───────────┐       ┌───────────┐       ┌───────────┐              │
│  │ Checkpoint │       │  Recovery  │       │ Replication │              │
│  └───────────┘       └───────────┘       └───────────┘              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 2. 当前实现状态

### 2.1 审计日志模块 (`crates/security/src/audit.rs`)

**已实现**:
- `AuditRecord` 结构：包含 id, timestamp, event_type, user, ip, details, session_id, duration_ms, rows
- `AuditEvent` 枚举：Login, Logout, ExecuteSql, DDL, DML, Grant, Revoke, Error, SessionStart, SessionEnd
- 内存缓冲 + 文件持久化

**缺失**:
- 与 WAL 的集成
- SHA-256 篡改检测链
- Crash-safe 审计写入

### 2.2 WAL 模块 (`crates/storage/src/wal.rs`)

**已实现**:
- `WalEntry` 结构：tx_id, entry_type, table_id, key, data, lsn, timestamp
- `WalEntryType` 枚举：BEGIN, INSERT, UPDATE, DELETE, COMMIT, ABORT, CHECKPOINT
- Log Sequence Number (LSN) 递增机制
- 检查点 (Checkpoint) 支持

**缺失**:
- 审计条目类型 (AUDIT)
- SHA-256 链式验证

## 3. 集成方案

### 3.1 审计条目作为 WAL Record

```
┌─────────────────────────────────────────────────────────────────────┐
│                 WAL Entry Type 扩展                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  现有类型:                                                          │
│  ┌────────┬────────┬────────┬────────┬────────┬────────┐             │
│  │ BEGIN  │ INSERT │ UPDATE │ DELETE │ COMMIT │ABORT  │             │
│  └────────┴────────┴────────┴────────┴────────┴────────┘             │
│                                                                      │
│  扩展类型:                                                          │
│  ┌────────┬────────┬────────┬────────┬────────┬────────┬────────┐       │
│  │ BEGIN  │ INSERT │ UPDATE │ DELETE │ COMMIT │ABORT  │ AUDIT │       │
│  └────────┴────────┴────────┴────────┴────────┴────────┴────────┘       │
│                                                                      │
│  AUDIT Entry 格式:                                                  │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │  LSN (8B) │ Timestamp (8B) │ TX_ID (8B) │ Type=AUDIT (1B) │    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │  User (32B) │ IP (16B) │ Session (8B) │ EventType (16B)     │    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │  Duration (8B) │ Rows (8B) │ DetailsHash (32B)            │    │
│  ├──────────────────────────────────────────────────────────────┤    │
│  │  Details (Variable)                                        │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.2 SHA-256 篡改检测链

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SHA-256 链式验证                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  每一 WAL Entry 包含上一 Entry 的 Hash:                             │
│                                                                      │
│  Entry N:                                                           │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │  prev_hash: SHA-256(Entry N-1)                             │    │
│  │  curr_hash: SHA-256(prev_hash || Entry N content)           │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  验证过程:                                                          │
│  1. 读取 Entry N，计算 curr_hash                                    │
│  2. 比对存储的 curr_hash                                           │
│  3. 读取 Entry N-1，验证 prev_hash                                 │
│  4. 递归验证整个链                                                   │
│                                                                      │
│  篡改检测:                                                          │
│  - 如果 Entry N 被修改，curr_hash 不匹配                            │
│  - 如果 Entry N-1 被删除，Entry N 的 prev_hash 不匹配                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 审计写入流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                    审计写入时序图                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Client          Executor          AuditLogger         WALManager     │
│    │               │                   │                   │            │
│    │─ Execute ───►│                   │                   │            │
│    │   SQL         │                   │                   │            │
│    │               │                   │                   │            │
│    │               │─ Log Event ─────►│                   │            │
│    │               │                   │                   │            │
│    │               │                   │─ Write Audit ────►│            │
│    │               │                   │   to WAL Buffer    │            │
│    │               │                   │                   │            │
│    │               │                   │◄── ACK ───────────│            │
│    │               │                   │                   │            │
│    │◄── Result ────│                   │                   │            │
│    │               │                   │                   │            │
│                                                                      │
│  WAL Buffer Flush:                                                 │
│    │               │                   │                   │            │
│    │               │                   │◄─ Flush ─────────│            │
│    │               │                   │   (fsync)         │            │
│    │               │                   │                   │            │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 4. 恢复链路分析

### 4.1 Crash Recovery 流程

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Crash Recovery 流程                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. 系统启动                                                          │
│     │                                                               │
│     ▼                                                               │
│  ┌─────────────────┐                                               │
│  │ 读取 WAL Header  │                                               │
│  │ 检查完整性        │                                               │
│  └────────┬────────┘                                               │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────┐                                               │
│  │ 验证 SHA-256 链 │  ◄── 如果链断裂，标记审计数据异常              │
│  └────────┬────────┘                                               │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────┐                                               │
│  │ 扫描 WAL Entries │                                               │
│  │ 重建审计记录     │                                               │
│  └────────┬────────┘                                               │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────┐                                               │
│  │ 重放未刷盘的     │                                               │
│  │ 审计条目        │                                               │
│  └────────┬────────┘                                               │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────┐                                               │
│  │ 恢复完成        │                                               │
│  └─────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 审计数据 vs WAL 数据

| 维度 | 审计日志 | WAL 数据 |
|------|----------|----------|
| **目的** | 安全合规、操作追溯 | 事务持久化、崩溃恢复 |
| **写入时机** | SQL 执行时同步写入 | 事务提交时批量写入 |
| **内容** | 用户操作元数据 | 数据页修改 |
| **生命周期** | 长期保留 (合规要求) | 可清理 (被 Checkpoint 截断) |
| **格式** | JSON/结构化文本 | 二进制序列化 |
| **清理策略** | 独立归档 | Checkpoint 后删除 |

## 5. 实现路径

### 5.1 Phase 1: 审计 WAL 集成

**任务**:
1. 扩展 `WalEntryType` 添加 `AUDIT` 类型
2. 在 `AuditLogger` 中实现 `write_to_wal()` 方法
3. 修改 `execute_insert/update/delete` 在事务提交前写入审计条目

**代码变更**:
```rust
// crates/storage/src/wal.rs
pub enum WalEntryType {
    BEGIN,
    INSERT,
    UPDATE,
    DELETE,
    COMMIT,
    ABORT,
    CHECKPOINT,
    AUDIT,  // 新增
}

// crates/security/src/audit.rs
impl AuditLogger {
    pub fn write_to_wal(&self, entry: &AuditRecord, wal: &mut WAL) -> Result<()> {
        let wal_entry = WalEntry {
            tx_id: entry.session_id,  // 复用 session_id 作为 tx_id
            entry_type: WalEntryType::AUDIT,
            table_id: 0,
            key: None,
            data: Some(entry.to_bytes()),
            lsn: 0,
            timestamp: entry.timestamp,
        };
        wal.append(wal_entry)
    }
}
```

### 5.2 Phase 2: SHA-256 链式验证

**任务**:
1. 在 `WalEntry` 中添加 `prev_hash` 和 `curr_hash` 字段
2. 实现 `compute_hash()` 方法计算 SHA-256
3. 实现 `verify_chain()` 方法验证完整性

**代码变更**:
```rust
// crates/storage/src/wal.rs
pub struct WalEntry {
    // ... existing fields
    pub prev_hash: Option<[u8; 32]>,  // 新增
    pub curr_hash: Option<[u8; 32]>,   // 新增
}

impl WalEntry {
    pub fn compute_hash(&self, prev_hash: &[u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(prev_hash);
        hasher.update(self.content_bytes());
        hasher.finalize().into()
    }

    pub fn verify_chain(&self, expected_prev: &[u8; 32]) -> bool {
        match (self.prev_hash, self.curr_hash) {
            (Some(prev), Some(curr)) => {
                prev == expected_prev && curr == self.compute_hash(&prev)
            }
            _ => false,
        }
    }
}
```

### 5.3 Phase 3: 恢复集成

**任务**:
1. 在 `RecoveryManager` 中添加审计恢复逻辑
2. 实现 `recover_audit_entries()` 方法
3. 添加完整性校验报告

## 6. 安全分析

### 6.1 威胁模型

| 威胁 | 防护措施 |
|------|----------|
| 审计日志被删除 | SHA-256 链断裂检测 |
| 审计日志被修改 | Hash 验证失败 |
| 事务日志被篡改 | WAL Entry Hash 链 |
| Crash 导致审计丢失 | WAL Buffer 同步写入 |

### 6.2 验证检查点

```
┌─────────────────────────────────────────────────────────────────────┐
│                    验证检查点清单                                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  □ 审计条目写入 WAL 后，Crush-Safe                                  │
│  □ SHA-256 链在 replay 时完整验证                                    │
│  □ 单个审计条目篡改可被检测                                          │
│  □ 连续多个审计条目篡改可被检测                                        │
│  □ 审计日志与数据修改的原子性保证                                      │
│  □ Checkpoint 不会截断未验证的审计链                                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 7. 测试用例

### 7.1 集成测试

```rust
#[test]
fn test_audit_wal_integration() {
    // 1. 创建审计条目
    let audit = AuditRecord::new("EXECUTE_SQL", "user1", "127.0.0.1", "SELECT * FROM t1");

    // 2. 写入 WAL
    let mut wal = WAL::new();
    audit_logger.write_to_wal(&audit, &mut wal).unwrap();

    // 3. Crash & Recovery
    let recovered = wal.recover_audit_entries();
    assert!(recovered.contains(&audit));
}

#[test]
fn test_sha256_chain_verification() {
    // 1. 写入多个审计条目
    // 2. 篡改中间条目
    // 3. 验证链检测到篡改
}
```

## 8. 相关文档

| 文档 | 说明 |
|------|------|
| `oo/wal/WAL_PROTOCOL.md` | WAL 协议详解 |
| `oo/transaction/MVCC_IMPLEMENTATION.md` | MVCC 可见性判断 |
| `oo/recovery/CRASH_RECOVERY.md` | 崩溃恢复链路 |
| `security/audit.rs` | 审计日志实现 |
| `crates/security/src/audit.rs` | 审计模块源码 |

## 9. 变更记录

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-05-12 | 1.0 | 初始版本，分析审计链与 WAL 集成的完整方案 |

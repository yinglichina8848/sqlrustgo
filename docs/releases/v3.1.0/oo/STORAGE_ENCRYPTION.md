# 存储加密执行链路文档

> **版本**: v3.1.0
> **日期**: 2026-05-12
> **Issue**: #607
> **状态**: 规划中

---

## 一、存储加密概述

### 1.1 什么是透明存储加密

透明存储加密（Transparent Storage Encryption）是一种在数据库存储层对数据进行加密的技术，对上层应用和 SQL 操作完全透明。用户无需修改任何 SQL 语句，加密和解密过程自动完成。

```
┌─────────────────────────────────────────────────────────────┐
│                    SQL 客户端                                 │
│  SELECT * FROM users WHERE id = 1;                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    SQL Engine                               │
│  Parser → Planner → Executor                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    存储加密层 (Transparent)                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │   加密引擎   │ ←→ │  Key Manager │ ←→ │   WAL 日志   │   │
│  └─────────────┘    └─────────────┘    └─────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    磁盘 (加密数据)                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  AES-256-GCM 加密的页面数据                          │   │
│  │  EncryptedPage { nonce, ciphertext, tag }           │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 加密算法选型

| 算法 | 优点 | 缺点 | 推荐场景 |
|------|------|------|----------|
| AES-128-CBC | 性能好 | 无认证 | 不推荐 |
| AES-256-CBC | 标准兼容 | 无认证 | 兼容场景 |
| AES-256-GCM | 认证加密 | 性能稍差 | **首选** |
| AES-256-XTS | 磁盘加密优化 | 需要 512-bit 块 | 存储设备 |

### 1.3 解决的问题

1. **数据泄露防护**：磁盘被盗时数据不可读
2. **合规要求**：满足 GDPR、PCI-DSS 等加密要求
3. **密钥管理**：支持密钥轮换而不重新加密所有数据
4. **审计支持**：加密操作可审计

---

## 二、当前实现状态

### 2.1 现有存储实现

根据 `crates/storage/src/`：

```rust
// 页面结构（当前未加密）
pub struct Page {
    pub page_id: u32,
    pub data: Vec<u8>,        // 原始数据
    pub is_dirty: bool,
}

// 文件存储
pub struct FileStorage {
    file: File,
    pages: HashMap<u32, Page>,
}
```

| 特性 | 状态 | 说明 |
|------|------|------|
| 页面缓存 | ✅ 已实现 | BufferPool 管理 |
| 文件持久化 | ✅ 已实现 | FileStorage |
| 数据压缩 | ❌ 未实现 | 可与加密配合 |
| 存储加密 | ❌ 未实现 | 需要规划 |
| 密钥管理 | ❌ 未实现 | 需要规划 |

### 2.2 现有 WAL 实现

```rust
// crates/transaction/src/wal.rs
pub struct WalEntry {
    pub tx_id: TxId,
    pub operation: WalOperation,
    pub page_id: u32,
    pub data: Vec<u8>,         // 原始数据
    pub lsn: u64,
}
```

---

## 三、加密架构设计

### 3.1 加密层次结构

```
┌─────────────────────────────────────────────────────────────┐
│                    密钥层次结构                              │
├─────────────────────────────────────────────────────────────┤
│  Master Key (KEK - Key Encryption Key)                     │
│  └→ 存储在外部密钥管理系统或硬件安全模块                      │
│  └→ 从不直接用于数据加密                                    │
│                                                             │
│  ├→ Tablespace Key (DEK - Data Encryption Key)            │
│  │   └→ 每个表空间一个 DEK                                 │
│  │   └→ 由 Master Key 加密存储                             │
│  │                                                          │
│  └→ Page Key (每页一个)                                     │
│      └→ 由 Tablespace Key 派生                              │
│      └→ 可选：定期轮换                                       │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 加密页面结构

```rust
// 加密页面格式
pub struct EncryptedPage {
    pub page_id: u32,              // 页面 ID（不加密）
    pub nonce: [u8; 12],           // GCM nonce (12 bytes)
    pub ciphertext: Vec<u8>,       // 加密后的数据
    pub tag: [u8; 16],            // GCM 认证标签 (16 bytes)
    pub key_version: u32,          // 密钥版本（用于密钥轮换）
}

// 解密后的页面
pub struct DecryptedPage {
    pub page_id: u32,
    pub data: Vec<u8>,             // 原始数据
}
```

### 3.3 密钥管理接口

```rust
// 密钥管理器 trait
pub trait KeyManager: Send + Sync {
    /// 获取当前活动的主密钥
    fn get_master_key(&self) -> Result<[u8; 32]>;

    /// 获取指定版本的 DEK
    fn get_dek(&self, tablespace_id: u32, version: u32) -> Result<[u8; 32]>;

    /// 生成新的 DEK 版本
    fn rotate_dek(&self, tablespace_id: u32) -> Result<u32>;

    /// 检查 DEK 是否需要轮换
    fn needs_rotation(&self, tablespace_id: u32) -> bool;
}
```

---

## 四、执行链路

### 4.1 页面读取链路（解密）

```
读取页面 page_id=100
  ↓
BufferPool::get_page(page_id=100)
  │
  ├→ 缓存命中
  │   └→ 返回缓存的解密页面
  │
  └→ 缓存未命中
      └→ FileStorage::read_page(page_id=100)
          └→ 从磁盘读取 EncryptedPage
          │
          └→ EncryptionManager::decrypt()
              ├→ 从 KeyManager 获取 DEK
              ├→ 提取 nonce 和 tag
              ├→ AES-256-GCM 解密
              │   └→ ciphertext → plaintext
              │
              └→ 验证 GCM tag
                  └→ 失败则报告安全错误
                  │
                  └→ 返回 DecryptedPage
                      └→ 存入 BufferPool
```

### 4.2 页面写入链路（加密）

```
写入页面 page_id=100 (data = "用户数据...")
  ↓
BufferPool::put_page(page_id=100, data)
  │
  ├→ 修改 BufferPool 中的页面
  │   └→ 标记 is_dirty = true
  │
  └→ 稍后刷新到磁盘
      └→ FileStorage::write_page()
          │
          └→ EncryptionManager::encrypt()
              ├→ 获取当前 DEK
              ├→ 生成随机 nonce
              ├→ AES-256-GCM 加密
              │   └→ plaintext + nonce + key → ciphertext + tag
              │
              └→ 构建 EncryptedPage
                  └→ 写入磁盘
```

### 4.3 WAL 写入链路（加密）

```
WAL::append(entry)
  │
  ├→ 页面 WAL 条目
  │   └→ 如果页面已加密
  │       └→ 使用页面加密 key 加密 WAL 数据
  │
  └→ 元数据 WAL 条目
      └→ 使用 Master Key 加密（高安全级别）

WAL::replay(entry)
  │
  └→ 解密并重放
      └→ EncryptionManager::decrypt_for_wal()
          └→ 根据 key_version 选择正确密钥解密
```

### 4.4 密钥轮换链路

```
ALTER INSTANCE ROTATE INNODB KEYRING
  ↓
KeyManager::rotate_master_key()
  │
  ├→ 生成新的 Master Key
  │
  ├→ 重新加密所有 DEK
  │   └→ 用新 Master Key 加密每个 DEK
  │
  ├→ 更新密钥版本元数据
  │
  └→ 记录密钥轮换事件到审计日志

后台任务：页面密钥轮换
  ↓
对于每个脏页面
  ├→ 用新 DEK 重新加密页面
  ├→ 更新页面 key_version
  └→ 标记为已轮换
```

---

## 五、关键设计决策

### 5.1 加密范围

| 选项 | 优点 | 缺点 | 推荐 |
|------|------|------|------|
| 全库加密 | 简单 | 性能开销大 | 初步实现 |
| 表空间加密 | 灵活 | 管理复杂 | 生产环境 |
| 页面级加密 | 最灵活 | 开销最大 | 高级场景 |

### 5.2 性能优化

```rust
// 加密上下文缓存
struct EncryptionContext {
    // DEK 缓存，避免每次解密都查询 KeyManager
    dek_cache: LruCache<(tablespace_id, version), [u8; 32]>,
    // 批量加密缓冲区
    encrypt_buffer: Vec<u8>,
}

// 并行页面刷新
async fn flush_pages_parallel(pages: Vec<DirtyPage>) {
    // 使用 rayon 并行加密多个页面
    pages.par_iter()
        .map(|page| encrypt_page(&page))
        .collect()
}
```

### 5.3 与压缩的配合

```
方案 A: 先压缩后加密
  Data → Compress → Encrypt → Store
  ✓ 压缩率更高（加密后数据无压缩性）
  ✗ 解密后需要先解压缩

方案 B: 先加密后压缩
  Data → Encrypt → Compress → Store
  ✓ 实现简单
  ✗ 压缩率降低

推荐: 方案 A（压缩后加密）
```

---

## 六、实现计划

### 6.1 第一阶段：核心加密

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 实现 EncryptionManager 结构 | P0 | 封装 AES-256-GCM 操作 |
| 实现 EncryptedPage / DecryptedPage | P0 | 页面加密格式 |
| 修改 FileStorage 支持加密页面 | P0 | 加密读写 |
| 实现 BasicKeyManager | P0 | 简单文件密钥管理 |
| 集成加密到 BufferPool | P0 | 缓存层加密 |

### 6.2 第二阶段：WAL 集成

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 加密 WAL 条目 | P1 | 页面数据加密 |
| 解密 WAL 回放 | P1 | 重放时解密 |
| 密钥版本跟踪 | P1 | 支持密钥轮换 |

### 6.3 第三阶段：密钥管理

| 任务 | 优先级 | 说明 |
|------|--------|------|
| KeyManager trait 完善 | P2 | 支持外部 KMS |
| 密钥轮换实现 | P2 | 在线密钥轮换 |
| 审计日志集成 | P2 | 加密操作审计 |

### 6.4 第四阶段：高级特性

| 任务 | 优先级 | 说明 |
|------|--------|------|
| 表空间级密钥 | P2 | 不同表不同密钥 |
| 内存加密 | P3 | 敏感数据内存加密 |
| 硬件加密支持 | P3 | AES-NI 加速 |

---

## 七、测试计划

### 7.1 单元测试

| 测试 | 验证点 |
|------|--------|
| encryption_round_trip_test | 加密解密后数据一致 |
| gcm_tag_verification_test | 篡改检测 |
| key_rotation_test | 密钥轮换正确 |

### 7.2 集成测试

| 测试 | 验证点 |
|------|--------|
| encrypted_crud_test | 加密表 CRUD 操作 |
| encrypted_wal_replay_test | WAL 加密重放 |
| encrypted_backup_restore_test | 加密备份恢复 |

### 7.3 性能测试

| 测试 | 目标 |
|------|------|
| encryption_overhead | 加密引入的延迟 |
| encrypted_throughput | 加密后 QPS |
| key_rotation_impact | 密钥轮换性能影响 |

---

## 八、相关文件

| 文件 | 作用 |
|------|------|
| `crates/storage/src/encryption.rs` | 加密核心实现（待创建） |
| `crates/storage/src/encrypted_file.rs` | 加密文件存储（待创建） |
| `crates/storage/src/key_manager.rs` | 密钥管理接口（待创建） |
| `crates/transaction/src/encrypted_wal.rs` | 加密 WAL（待创建） |
| `tests/integration/test_encryption.rs` | 集成测试（待创建） |
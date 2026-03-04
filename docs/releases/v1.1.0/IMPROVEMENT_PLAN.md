# v1.1.0-draft 改进计划

> **版本**: v1.1.0-draft  
> **创建日期**: 2026-03-05  
> **基于**: DeepSeek 评估报告  
> **当前分支**: draft/v1.1.0
> **工作分支**: fix/v1.1.0-*

---

## ⚠️ 分支规范

### 正确的分支流程

```
develop-v1.1.0      → 开发分支（功能开发）
      ↓
draft/v1.1.0        → 草稿分支（改进、修复）← 当前阶段
      ↓ [门禁通过后]
release/v1.1.0      → 发布分支（alpha → beta → rc → GA）
      ↓ [GA 验收后]
main                → 主分支（稳定版本）
```

### AI-CLI PR 提交规范

| AI 身份 | 工作分支 | PR 目标分支 |
|---------|----------|-------------|
| heartopen | fix/v1.1.0-* | **draft/v1.1.0** |
| openheart | feat/v1.1.0-* | **draft/v1.1.0** |
| maintainer | fix/v1.1.0-* | **draft/v1.1.0** |

**⚠️ 禁止事项**:
- 不要直接提交到 release/v1.1.0
- 不要直接提交到 main
- 所有 AI 的 PR 必须指向 draft/v1.1.0

---

## 一、改进概述

### 1.1 改进目标

根据 DeepSeek 评估报告，v1.1.0-draft 存在以下核心问题：

1. **门禁检查不一致**: 文档宣称达标，实际存在问题
2. **错误处理不完善**: 511 处 unwrap
3. **并发安全缺失**: 无锁保护
4. **功能实现不完整**: 与文档宣称不符

### 1.2 改进原则

| 原则 | 说明 |
|------|------|
| 优先级驱动 | 先解决 P0 问题 |
| 文档同步 | 改进后同步更新文档 |
| 测试覆盖 | 每个改进需有测试验证 |
| 向后兼容 | 不破坏现有 API |

---

## 二、改进任务清单

### 2.1 P0 - 立即修复

#### P0-1: 完成门禁检查

| 任务 | 说明 | 状态 |
|------|------|------|
| P0-1.1 | 修复 Clippy 错误 | ⏳ |
| P0-1.2 | 更新门禁清单状态 | ⏳ |
| P0-1.3 | 同步文档与实际状态 | ⏳ |

#### P0-2: 替换 unwrap

| 模块 | 文件 | unwrap 数量 | 状态 |
|------|------|-------------|------|
| executor | mod.rs | ~50 | ⏳ |
| storage | file_storage.rs | ~30 | ⏳ |
| storage | bplus_tree | ~20 | ⏳ |
| transaction | mod.rs | ~10 | ⏳ |

#### P0-3: 添加并发保护

| 任务 | 说明 | 状态 |
|------|------|------|
| P0-3.1 | Page 添加 RwLock | ⏳ |
| P0-3.2 | BPlusTree 添加锁保护 | ⏳ |
| P0-3.3 | TransactionManager 添加锁 | ⏳ |

### 2.2 P1 - 短期改进

#### P1-1: 完善核心功能

| 任务 | 说明 | 状态 |
|------|------|------|
| P1-1.1 | 实现火山模型执行器 | ⏳ |
| P1-1.2 | 连接网络层与执行引擎 | ⏳ |
| P1-1.3 | 实现简单锁机制 (2PL) | ⏳ |

#### P1-2: 优化性能

| 任务 | 说明 | 状态 |
|------|------|------|
| P1-2.1 | 实现 BufferPool LRU | ⏳ |
| P1-2.2 | 完善 B+ 树分裂逻辑 | ⏳ |
| P1-2.3 | 预编译表达式求值 | ⏳ |

#### P1-3: 重构架构

| 任务 | 说明 | 状态 |
|------|------|------|
| P1-3.1 | executor 与 storage 解耦 | ⏳ |
| P1-3.2 | 抽象 catalog 服务 | ⏳ |
| P1-3.3 | 完善 LogicalPlan/PhysicalPlan 转换 | ⏳ |

### 2.3 P2 - 长期规划

#### P2-1: 教学优化

| 任务 | 说明 | 状态 |
|------|------|------|
| P2-1.1 | 提供简化版指南 | ⏳ |
| P2-1.2 | 增加教学注释 | ⏳ |
| P2-1.3 | 创建渐进式练习 | ⏳ |

#### P2-2: 文档完善

| 任务 | 说明 | 状态 |
|------|------|------|
| P2-2.1 | 功能实现状态表 | ⏳ |
| P2-2.2 | 架构图更新 | ⏳ |
| P2-2.3 | API 文档完善 | ⏳ |

---

## 三、详细改进方案

### 3.1 P0-1: 完成门禁检查

#### 3.1.1 修复 Clippy 错误

```bash
# 运行 Clippy 检查
cargo clippy --all-features -- -D warnings 2>&1 | head -50
```

#### 3.1.2 更新门禁清单

修改 [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md):
- 将未完成项标记为 ❌
- 添加实际状态说明

### 3.2 P0-2: 替换 unwrap

#### 3.2.1 executor/mod.rs

```rust
// 修复前
let table = self.storage.get_table(&stmt.table).unwrap();

// 修复后
let table = self.storage.get_table(&stmt.table)
    .ok_or_else(|| SqlError::TableNotFound(stmt.table.clone()))?;
```

#### 3.2.2 storage/file_storage.rs

```rust
// 修复前
let mut tables = self.tables.lock().unwrap();

// 修复后
let mut tables = self.tables.lock()
    .map_err(|_| SqlError::LockError("Failed to acquire tables lock".to_string()))?;
```

### 3.3 P0-3: 添加并发保护

#### 3.3.1 Page 添加 RwLock

```rust
// 修复前
pub struct Page {
    pub data: Vec<u8>,
}

// 修复后
pub struct Page {
    data: RwLock<Vec<u8>>,
}

impl Page {
    pub fn read(&self) -> RwLockReadGuard<Vec<u8>> {
        self.data.read().unwrap()
    }
    
    pub fn write(&self) -> RwLockWriteGuard<Vec<u8>> {
        self.data.write().unwrap()
    }
}
```

#### 3.3.2 BPlusTree 添加锁保护

```rust
pub struct BPlusTree {
    root: RwLock<Option<Box<Node>>>,
    order: usize,
}

impl BPlusTree {
    pub fn insert(&self, key: i64, value: u32) -> Result<()> {
        let mut root = self.root.write()
            .map_err(|_| SqlError::LockError("Failed to acquire tree lock".to_string()))?;
        // ... 插入逻辑
    }
}
```

---

## 四、里程碑

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          改进里程碑                                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Week 1: P0 门禁检查                                                        │
│   ├── P0-1.1: 修复 Clippy 错误                                              │
│   ├── P0-1.2: 更新门禁清单状态                                              │
│   └── P0-1.3: 同步文档与实际状态                                            │
│                                                                              │
│   Week 2: P0 错误处理                                                        │
│   ├── P0-2.1: executor 模块 unwrap 替换                                     │
│   ├── P0-2.2: storage 模块 unwrap 替换                                      │
│   └── P0-2.3: 其他模块 unwrap 替换                                          │
│                                                                              │
│   Week 3: P0 并发安全                                                        │
│   ├── P0-3.1: Page 添加 RwLock                                              │
│   ├── P0-3.2: BPlusTree 添加锁保护                                          │
│   └── P0-3.3: TransactionManager 添加锁                                     │
│                                                                              │
│   Week 4: 验证与发布                                                         │
│   ├── 运行完整测试套件                                                       │
│   ├── 更新文档                                                               │
│   └── 发布 v1.1.0-draft-improved                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 五、验收标准

### 5.1 P0 验收标准

| 检查项 | 标准 |
|--------|------|
| Clippy | 零警告 |
| unwrap 数量 | 生产代码 < 10 处 |
| 并发测试 | 通过多线程测试 |
| 文档一致性 | 门禁清单与实际一致 |

### 5.2 测试验证

```bash
# 运行完整测试
cargo test --all-features

# 运行 Clippy
cargo clippy --all-features -- -D warnings

# 运行并发测试
cargo test --test concurrency_tests

# 检查 unwrap 数量
grep -r "unwrap()" src/ | wc -l
```

---

## 六、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 改动影响现有功能 | 高 | 中 | 完整测试覆盖 |
| 并发改动引入新问题 | 高 | 中 | 并发测试验证 |
| 文档更新遗漏 | 中 | 低 | 检查清单 |

---

## 七、相关文档

| 文档 | 说明 |
|------|------|
| [DeepSeek 评估报告](./DEEPSEEK_EVALUATION.md) | 问题详细分析 |
| [门禁检查清单](./RELEASE_GATE_CHECKLIST.md) | 门禁状态 |
| [代码质量审计](./CODE_QUALITY_AUDIT.md) | 质量问题 |

---

## 八、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-05 | 初始版本 |

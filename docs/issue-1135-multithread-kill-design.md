# Issue #1135 - 多线程服务器模式 KILL 实现设计方案

**Issue**: [v2.1.0][P0] MySQL 兼容性增强 - SHOW PROCESSLIST / KILL  
**子任务**: 多线程服务器模式 KILL 实现  
**日期**: 2026-03-30  
**状态**: 设计阶段 - 待审核

---

## 1. 背景与目标

### 1.1 Issue #1135 原始需求

| 功能 | 优先级 | 状态 |
|------|--------|------|
| SHOW PROCESSLIST | P1 | ✅ 已完成 |
| KILL CONNECTION | P1 | ⚠️ 部分完成 |
| KILL QUERY | P1 | ⚠️ 部分完成 |
| INFORMATION_SCHEMA.PROCESSLIST | P2 | ✅ 已完成 |
| **多线程服务器模式 KILL** | **P0** | ❌ 未实现 |

### 1.2 当前问题

在单线程 CLI 模式下，`execute_kill()` 可以正常工作。但在多线程服务器模式下存在以下问题：

1. **无线程追踪** - SessionManager 不知道哪个线程正在处理哪个 session
2. **无中断机制** - 无法向正在运行的查询发送终止信号
3. **无取消标志** - 查询执行循环不会检查 session 的取消状态
4. **KILL 未集成** - `Statement::Kill` 在 `ExecutionEngine::execute()` 中未被处理

---

## 1.3 业界标准：为什么数据库必须使用协作式取消？

**核心原因只有一个：强制终止线程会破坏数据库一致性**

数据库执行线程正在做的事情可能包括：

| 操作 | 风险 |
|------|------|
| 写 WAL | 日志损坏 |
| 更新 B+Tree | 索引损坏 |
| 更新 buffer pool | 脏页不一致 |
| 持有锁 | 死锁 |
| 事务中途终止 | ACID 破坏 |

**例如**：
```sql
UPDATE users SET balance = balance - 100;
```
如果线程被强杀，可能变成：`balance 已扣` 但 `WAL 未写`，数据库直接损坏。

**结论：数据库只能请求停止，不能强制停止**

---

## 1.4 业界标准取消机制参考

### 1.4.1 MySQL KILL 实现机制

**官方文档描述**：
> "When you use `KILL`, a **thread-specific kill flag is set** for the thread. In most cases, it might take some time for the thread to die because the kill flag is checked only at **specific intervals**"

**MySQL 内部模型**：
```cpp
THD {
    killed_state killed;
}

while (...) {
    check_killed();
}
```

**killed_state 定义**：

| killed_state | 含义 |
|--------------|------|
| NOT_KILLED | 正常 |
| KILL_QUERY | 终止当前查询 |
| KILL_CONNECTION | 终止 session |

### 1.4.2 PostgreSQL 取消机制（更先进）

PostgreSQL 使用独立标志：
```cpp
QueryCancelPending  // kill query
ProcDiePending      // kill session
```

**执行循环**：
```cpp
#define CHECK_FOR_INTERRUPTS() \
 if (InterruptPending) ProcessInterrupts()
```

### 1.4.3 不同操作类型的取消策略

数据库不会统一处理所有类型查询取消：

| 操作类型 | 取消检查点 | 说明 |
|----------|-----------|------|
| SELECT SeqScan | 每 N rows | 几乎即时响应 |
| ORDER BY / GROUP BY | chunk merge boundary | 排序不能随时中断 |
| UPDATE / DELETE | before next row | 不能 mid-row cancel |
| ALTER TABLE | copy boundary | metadata swap 不能中断 |

---

## 2. 当前架构分析

### 2.1 服务器线程模型 (`crates/server/src/main.rs`)

```
std::thread::spawn (per connection)
    └── handle_client()
        └── loop {
            ├── read query from stream
            ├── parse(query)
            ├── engine.execute(statement)  // 顺序执行
            └── write response
        }
```

- **每连接一个线程**: `std::thread::spawn` 为每个 TCP 连接创建新线程
- **顺序查询处理**: 每个线程内顺序处理查询，无并发
- **无 worker pool**: 未使用线程池

### 2.2 Session 管理 (`crates/security/src/session.rs`)

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u64, Session>>>,  // 线程安全
    next_session_id: Arc<RwLock<u64>>,
}
```

- **线程安全**: 使用 `Arc<RwLock<HashMap>>`
- **仅追踪 session ID**: 不追踪对应的线程

### 2.3 关键缺失

| 缺失项 | 影响 |
|--------|------|
| Session → Thread 映射 | 无法找到需要 kill 的线程 |
| 查询中断标志 | 无法通知正在运行的查询停止 |
| Cancellation 传播 | 查询执行期间无法检查取消状态 |

---

## 3. 设计方案

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         Server                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │ Connection 1│    │ Connection 2│    │ Connection N│         │
│  │ Thread #1   │    │ Thread #2   │    │ Thread #N   │         │
│  │ session_id=1│    │ session_id=2│    │ session_id=N│         │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘         │
│         │                   │                   │               │
│         └───────────────────┼───────────────────┘               │
│                             │                                   │
│                    ┌────────▼────────┐                          │
│                    │ SessionManager  │                          │
│                    │ ┌────────────┐  │                          │
│                    │ │ sessions   │  │                          │
│                    │ │ interrupt  │◄─┼── KILL signal           │
│                    │ │ thread_hndl│  │                          │
│                    │ └────────────┘  │                          │
│                    └─────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 核心数据结构变更

#### 3.2.1 CancelToken 和 CancelGuard

参考 PostgreSQL 设计的业界标准取消机制：

```rust
// crates/security/src/cancel.rs

use std::sync::atomic::{AtomicBool, Ordering};

/// CancelToken - 协作式取消标志
/// 参考 PostgreSQL QueryCancelPending 设计
pub struct CancelToken {
    /// 取消当前查询（KILL QUERY）
    query_cancelled: AtomicBool,
    /// 终止会话（KILL CONNECTION）
    connection_killed: AtomicBool,
}

impl CancelToken {
    pub fn new() -> Self {
        Self {
            query_cancelled: AtomicBool::new(false),
            connection_killed: AtomicBool::new(false),
        }
    }

    /// 取消当前查询（KILL QUERY）
    pub fn cancel_query(&self) {
        self.query_cancelled.store(true, Ordering::SeqCst);
    }

    /// 检查查询是否被取消
    pub fn is_query_cancelled(&self) -> bool {
        self.query_cancelled.load(Ordering::SeqCst)
    }

    /// 终止连接（KILL CONNECTION）
    pub fn kill_connection(&self) {
        self.connection_killed.store(true, Ordering::SeqCst);
    }

    /// 检查连接是否被终止
    pub fn is_connection_killed(&self) -> bool {
        self.connection_killed.load(Ordering::SeqCst)
    }

    /// 重置查询取消标志（用于新的查询）
    pub fn reset_query_cancelled(&self) {
        self.query_cancelled.store(false, Ordering::SeqCst);
    }
}

/// CancelGuard - 临界区取消保护
/// 参考 PostgreSQL CancelGuard 设计
/// 用于在关键操作（如 metadata swap）期间禁用取消
pub struct CancelGuard {
    token: Arc<CancelToken>,
    previous_state: bool,
}

impl CancelGuard {
    /// 禁用取消（进入临界区）
    pub fn disable(token: Arc<CancelToken>) -> Self {
        let previous_state = token.query_cancelled.swap(true, Ordering::SeqCst);
        Self { token, previous_state }
    }

    /// 重新启用取消（离开临界区）
    pub fn enable(self) {
        self.token.query_cancelled.store(self.previous_state, Ordering::SeqCst);
    }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        // 确保离开临界区时恢复取消状态
        self.token.query_cancelled.store(self.previous_state, Ordering::SeqCst);
    }
}
```

#### 3.2.2 KillType 枚举

```rust
// crates/parser/src/ast.rs 或新文件

/// KILL 语句类型
/// 参考 MySQL KILL_QUERY / KILL_CONNECTION
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillType {
    /// 终止当前查询，连接保持
    Query,
    /// 终止整个连接
    Connection,
}
```

#### 3.2.3 Session 增强

```rust
// crates/security/src/session.rs

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Session 状态追踪
pub struct SessionState {
    pub thread_id: AtomicU64,           // 当前处理线程 ID
    pub cancel_token: Arc<CancelToken>, // 取消标志
}

/// Session 扩展方法
impl Session {
    /// 标记 session 正在被线程处理
    pub fn set_thread_id(&self, thread_id: u64);
    
    /// 获取取消标志
    pub fn cancel_token(&self) -> Arc<CancelToken>;
    
    /// 检查是否已被取消查询
    pub fn is_query_cancelled(&self) -> bool {
        self.cancel_token.is_query_cancelled()
    }
    
    /// 标记为取消查询
    pub fn cancel_query(&self) {
        self.cancel_token.cancel_query();
    }
    
    /// 检查是否连接已被 kill
    pub fn is_connection_killed(&self) -> bool {
        self.cancel_token.is_connection_killed()
    }
    
    /// 标记为 kill 连接
    pub fn kill_connection(&self) {
        self.cancel_token.kill_connection();
    }
    
    /// 重置查询取消标志（用于新的查询）
    pub fn reset_query_cancelled(&self) {
        self.cancel_token.reset_query_cancelled();
    }
}
```

#### 3.2.4 SessionManager 增强

```rust
impl SessionManager {
    /// 新增: 获取活动的 session IDs
    pub fn get_active_session_ids(&self) -> Vec<u64>;
    
    /// 新增: 检查 session 是否可被当前线程 kill
    pub fn can_kill(&self, requester_id: u64, target_id: u64) -> bool;
    
    /// 新增: Kill session (CONNECTION)
    pub fn kill_session(&self, target_id: u64) -> Result<(), String>;
    
    /// 新增: Kill session 中的查询 (QUERY)
    pub fn kill_query(&self, target_id: u64) -> Result<(), String>;
}
```

### 3.3 KILL 执行流程

#### 3.3.1 KILL CONNECTION 流程

```
Client                        Server                          Target Thread
  │                             │                                  │
  │ KILL CONNECTION 12345       │                                  │
  │────────────────────────────>│                                  │
  │                             │                                  │
  │                             │ 1. Validate session 12345 exists │
  │                             │ 2. Check privilege (SUPER/kill own)│
  │                             │ 3. Set session.is_killed = true   │
  │                             │                                  │
  │                             │─────────── signal ──────────────>│
  │                             │          (Thread #N)              │
  │                             │                                  │
  │                             │     Thread sees is_killed=true    │
  │                             │     → breaks query loop           │
  │                             │     → closes connection           │
  │                             │     → returns "Connection killed" │
  │                             │                                  │
  │  KILL completed             │                                  │
  │<────────────────────────────│                                  │
```

#### 3.3.2 KILL QUERY 流程

```
Client                        Server                          Target Thread
  │                             │                                  │
  │ KILL QUERY 12345            │                                  │
  │────────────────────────────>│                                  │
  │                             │                                  │
  │                             │ 1. Validate session 12345 exists │
  │                             │ 2. Check privilege               │
  │                             │ 3. Set session.is_cancelled = true│
  │                             │                                  │
  │                             │─────────── signal ──────────────>│
  │                             │          (Thread #N)              │
  │                             │                                  │
  │                             │     Thread checks is_cancelled    │
  │                             │     during execution              │
  │                             │     → throws SqlError::Cancelled   │
  │                             │     → connection stays open        │
  │                             │                                  │
  │  KILL completed             │                                  │
  │<────────────────────────────│                                  │
```

### 3.4 查询执行集成

#### 3.4.1 ExecutionEngine 修改

```rust
// src/lib.rs

impl ExecutionEngine {
    pub fn execute(&mut self, statement: Statement) -> Result<ExecutorResult, SqlError> {
        // 执行前检查取消状态
        if self.session_state.map(|s| s.is_cancelled()).unwrap_or(false) {
            return Err(SqlError::ExecutionError("Query cancelled".to_string()));
        }
        
        match statement {
            Statement::Kill(kill) => {
                self.execute_kill(kill)
            }
            // ... 其他语句
        }
    }
}
```

#### 3.4.2 长期查询检查点

对于长时间运行的查询（如全表扫描），需要在执行循环中插入检查点：

```rust
// 在 scan() 等操作中
pub fn scan(&self, table: &str) -> Result<Vec<RowRef>, SqlError> {
    let mut results = Vec::new();
    for page in self.pages.iter() {
        // 检查点: 是否被取消
        if self.session_state.is_cancelled() {
            return Err(SqlError::ExecutionError("Query cancelled".to_string()));
        }
        results.extend(page.rows());
    }
    Ok(results)
}
```

### 3.5 线程间通信机制

#### 3.5.1 使用 Arc<AtomicBool> 标志

```rust
// SessionState 内部使用 AtomicBool
use std::sync::atomic::{AtomicBool, Ordering};

struct SessionState {
    is_cancelled: AtomicBool,
    is_killed: AtomicBool,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            is_cancelled: AtomicBool::new(false),
            is_killed: AtomicBool::new(false),
        }
    }
    
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }
}
```

#### 3.5.2 替代方案: 使用 Channel

```rust
use crossbeam_channel::{bounded, Sender};

struct SessionState {
    cancel_tx: Sender<()>,  // 发送取消信号
    // ...
}

// 目标线程
let cancel_rx = session.cancel_rx.clone();
loop {
    select! {
        recv(cancel_rx) -> _ => {
            return Err(SqlError::ExecutionError("Query cancelled".to_string()));
        }
        // 正常查询处理
    }
}
```

---

## 4. 实现计划

### 4.1 阶段一: Session 状态追踪 (P0)

**目标**: 为每个 Session 添加状态追踪能力

| 任务 | 文件 | 修改 |
|------|------|------|
| T1.1 添加 SessionState 结构体 | `session.rs` | 新增 `interrupt_flags` 字段 |
| T1.2 实现 `is_cancelled()`, `cancel()` | `session.rs` | 新增方法 |
| T1.3 实现 `is_killed()`, `kill()` | `session.rs` | 新增方法 |
| T1.4 SessionManager 添加 kill 方法 | `session.rs` | `kill_session()`, `kill_query()` |
| T1.5 添加单元测试 | `session.rs` | 测试取消/kill 流程 |

**验收标准**:
- [ ] Session 可被标记为 cancelled/killed
- [ ] SessionManager.kill_session() 可关闭 session
- [ ] 单元测试覆盖所有新方法

### 4.2 阶段二: ExecutionEngine KILL 集成 (P0)

**目标**: 在 ExecutionEngine 中处理 Statement::Kill

| 任务 | 文件 | 修改 |
|------|------|------|
| T2.1 添加 `execute_kill()` | `src/lib.rs` | 处理 KILL CONNECTION/QUERY |
| T2.2 添加 session_state 字段 | `ExecutionEngine` | 用于取消检查 |
| T2.3 解析 session context | `server/main.rs` | 传递 session_id 到 engine |
| T2.4 集成测试 | `tests/` | 测试 KILL 语句执行 |

**验收标准**:
- [ ] `KILL <id>` 可正确解析执行
- [ ] `KILL CONNECTION <id>` 关闭连接
- [ ] `KILL QUERY <id>` 取消查询但保持连接
- [ ] 权限检查: 只能 kill 自己或 SUPER 权限

### 4.3 阶段三: 查询取消传播 (P0)

**目标**: 在长时间运行查询中检查取消状态

| 任务 | 文件 | 修改 |
|------|------|------|
| T3.1 添加查询检查点接口 | `StorageEngine` trait | `fn check_cancelled()` |
| T3.2 MemoryStorage 实现检查 | `storage/memory.rs` | 实现检查点 |
| T3.3 BPlusTree scan 检查 | `storage/bplus.rs` | 扫描时检查 |
| T3.4 聚合操作检查 | `executor/` | COUNT/SUM 等检查 |

**验收标准**:
- [ ] 全表扫描可被取消
- [ ] 取消后返回 "Query cancelled" 错误
- [ ] 连接保持打开状态

### 4.4 阶段四: 服务器集成 (P0)

**目标**: 在 TCP 服务器中支持 KILL

| 任务 | 文件 | 修改 |
|------|------|------|
| T4.1 传递 session_context | `server/main.rs` | handle_client 接收 session_id |
| T4.2 传递 session_state | `ExecutionEngine` | 构造时传入 |
| T4.3 优雅关闭 | `server/main.rs` | is_killed 时关闭连接 |
| T4.4 压力测试 | `tests/server/` | 多连接 KILL 测试 |

**验收标准**:
- [ ] 多连接环境下 KILL 正常工作
- [ ] kill 自己创建的连接成功
- [ ] kill 他人连接需 SUPER 权限
- [ ] 无内存泄漏/死锁

---

## 5. 风险与缓解

### 5.1 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 线程安全问题 | 高 | 中 | 使用 Atomic 类型; 充分测试 |
| 死锁 | 高 | 低 | 避免在持有锁时调用外部代码 |
| 取消状态丢失 | 中 | 低 | AtomicBool 天然免疫 |
| 性能开销 | 低 | 高 | 检查点仅在循环迭代时执行 |

### 5.2 测试策略

```
单元测试 (T1-T3)
    ├── SessionState 测试
    ├── SessionManager.kill_* 测试
    └── ExecutionEngine KILL 测试

集成测试 (T4)
    ├── 单连接 KILL 测试
    ├── 多连接 KILL 测试
    └── 权限检查测试

压力测试
    └── 100 并发连接 + 随机 KILL
```

---

## 6. 资源估算

| 阶段 | 工作量 | 优先级 |
|------|--------|--------|
| 阶段一: Session 状态追踪 | 2-3 人天 | P0 |
| 阶段二: ExecutionEngine 集成 | 2-3 人天 | P0 |
| 阶段三: 查询取消传播 | 3-5 人天 | P0 |
| 阶段四: 服务器集成 | 2-3 人天 | P0 |
| **总计** | **9-14 人天** | |

---

## 7. 后续扩展

### 7.1 Thread Pool 模式

当前每连接一线程模式在大量连接时开销较大。后续可考虑：

```rust
// 未来: 使用线程池
let pool = ThreadPool::new(num_cpus * 2);
pool.execute(move || {
    handle_client(stream);
});
```

### 7.2 Async/Await

当前使用同步线程，未来可考虑 async 模式：

```rust
// 未来: async 模式
async fn handle_client(stream: TcpStream) {
    loop {
        let query = socket.read().await?;
        let result = engine.execute(query).await?;
        socket.write(result).await?;
    }
}
```

---

## 8. 结论

本设计方案为多线程服务器模式下的 KILL 实现提供了完整的解决方案：

1. **安全性**: 基于 SessionPrivilege 的权限检查
2. **可靠性**: 使用 Atomic 类型避免数据竞争
3. **可扩展性**: 模块化设计，易于后续改进
4. **向后兼容**: 不影响现有 CLI 模式

**建议**: 
- 批准此设计方案
- 分配优先级 P0
- 建议分 4 个阶段实施

---

## 9. 审核问题

请审核以下问题：

1. **架构设计**: Session → Thread 映射方案是否合理？
2. **线程安全**: AtomicBool 用于取消标志是否足够？
3. **性能**: 检查点开销是否可接受？
4. **替代方案**: Channel 方案 vs AtomicBool 方案，哪个更优？
5. **优先级**: 是否确认为 P0？

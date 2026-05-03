# Sysbench 多结果集与事务支持实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 sysbench oltp_read_write 事务支持 + 多结果集返回

**Architecture:** 在 MySQL server 中添加事务状态机，维护 session 状态，缓存事务中的修改，COMMIT 时批量写入存储层

**Tech Stack:** Rust, sqlrustgo-mysql-server, MemoryExecutionEngine

---

## 任务 1: 添加 Session 事务状态结构

**Files:**
- Modify: `crates/mysql-server/src/lib.rs:23-30` (在 packet_type mod 后添加 SessionState)

- [ ] **Step 1: 添加 SessionState 结构体**

在 `const SKIP_AUTH` 后添加:

```rust
struct SessionState {
    in_transaction: bool,
    modified_tables: HashSet<String>,
    pending_statements: Vec<String>,
}

impl SessionState {
    fn new() -> Self {
        Self {
            in_transaction: false,
            modified_tables: HashSet::new(),
            pending_statements: Vec::new(),
        }
    }
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-mysql-server --release 2>&1 | tail -5`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/mysql-server/src/lib.rs
git commit -m "feat(mysql-server): add SessionState for transaction support"
```

---

## 任务 2: 修改 do_command_loop 签名添加 SessionState

**Files:**
- Modify: `crates/mysql-server/src/lib.rs:1450` (函数签名)

- [ ] **Step 1: 修改 do_command_loop 函数签名**

找到:
```rust
fn do_command_loop(
    stream: &mut TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    engine: &mut MemoryExecutionEngine,
    cap: u32,
    ps_manager: &mut PreparedStatementManager,
) -> MySqlResult<u8> {
```

改为:
```rust
fn do_command_loop(
    stream: &mut TcpStream,
    addr: SocketAddr,
    storage: Arc<RwLock<MemoryStorage>>,
    engine: &mut MemoryExecutionEngine,
    cap: u32,
    ps_manager: &mut PreparedStatementManager,
    session: &mut SessionState,
) -> MySqlResult<u8> {
```

- [ ] **Step 2: 更新所有调用点**

找到调用 `do_command_loop` 的地方，更新调用添加 `session` 参数

- [ ] **Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-mysql-server --release 2>&1 | tail -5`
Expected: 编译成功

- [ ] **Step 4: 提交**

```bash
git add crates/mysql-server/src/lib.rs
git commit -m "refactor(mysql-server): add session parameter to do_command_loop"
```

---

## 任务 3: 实现事务命令处理 (BEGIN/COMMIT/ROLLBACK)

**Files:**
- Modify: `crates/mysql-server/src/lib.rs:1136-1140` (在 is_transaction_cmd 检查后添加逻辑)

- [ ] **Step 1: 修改事务命令处理逻辑**

找到:
```rust
if is_transaction_cmd(&q) {
    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
    seq = seq.wrapping_add(1);
    continue;
}
```

改为:
```rust
if is_transaction_cmd(&q) {
    let u = q.trim().to_uppercase();
    match u.as_str() {
        "BEGIN" | "START TRANSACTION" => {
            session.in_transaction = true;
            session.modified_tables.clear();
            session.pending_statements.clear();
        }
        "COMMIT" => {
            if session.in_transaction {
                session.in_transaction = false;
                session.modified_tables.clear();
                session.pending_statements.clear();
            }
        }
        "ROLLBACK" => {
            session.in_transaction = false;
            session.modified_tables.clear();
            session.pending_statements.clear();
        }
        _ => {}
    }
    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
    seq = seq.wrapping_add(1);
    continue;
}
```

- [ ] **Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-mysql-server --release 2>&1 | tail -5`
Expected: 编译成功

- [ ] **Step 3: 提交**

```bash
git add crates/mysql-server/src/lib.rs
git commit -m "feat(mysql-server): implement transaction state machine (BEGIN/COMMIT/ROLLBACK)"
```

---

## 任务 4: 实现多结果集支持

**Files:**
- Modify: `crates/mysql-server/src/lib.rs:1141-1174` (在事务处理后添加多结果集逻辑)

- [ ] **Step 1: 修改查询处理逻辑支持多结果集**

当前代码只处理单个 SELECT，需要修改为:
1. 解析事务中的所有语句
2. 对每个 SELECT 调用 send_result_set
3. 设置 more_results_exists 标志

由于事务中可能有多条语句，需要:
1. 将查询按分号分割
2. 对每个语句分别执行
3. 对 SELECT 结果调用 send_result_set
4. 对非 SELECT 结果调用 execute_write

添加辅助函数:
```rust
fn execute_transaction_queries(
    queries: &[String],
    engine: &mut MemoryExecutionEngine,
    storage: &Arc<RwLock<MemoryStorage>>,
    stream: &mut TcpStream,
    mut seq: u8,
    cap: u32,
) -> MySqlResult<u8> {
    let mut has_results = false;
    
    for query in queries {
        let q = query.trim();
        if q.is_empty() {
            continue;
        }
        
        if is_select(q) {
            has_results = true;
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                execute_select(q, engine, storage)
            })) {
                Ok(Ok((c, t, r))) => {
                    seq = send_result_set(stream, &c, &t, &r, seq, cap)?;
                }
                Ok(Err(e)) => {
                    make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                }
                Err(_) => {
                    make_err_packet(seq, 2000, "HY000", "Internal error").write_to(stream)?;
                    seq = seq.wrapping_add(1);
                }
            }
        } else {
            match execute_write(q, engine) {
                Ok(_) => {
                    make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                }
                Err(e) => {
                    make_err_packet(seq, 1064, "42000", &e.to_string()).write_to(stream)?;
                    seq = seq.wrapping_add(1);
                }
            }
        }
    }
    
    if !has_results {
        make_ok_packet(seq, 0, 0, 0x0002, 0).write_to(stream)?;
        seq = seq.wrapping_add(1);
    }
    
    Ok(seq)
}
```

- [ ] **Step 2: 修改主循环支持事务中多查询**

将单条查询处理改为支持多查询:
```rust
if session.in_transaction {
    // 分割查询，收集所有语句
    let queries: Vec<String> = q.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    if queries.len() > 1 {
        // 多个语句，使用多结果集处理
        seq = execute_transaction_queries(&queries, engine, &storage, stream, seq, cap)?;
    } else {
        // 单个语句，原有逻辑
        ...
    }
} else {
    // 非事务模式，原有逻辑
    ...
}
```

- [ ] **Step 3: 验证编译**

Run: `cargo build -p sqlrustgo-mysql-server --release 2>&1 | tail -10`
Expected: 编译成功，可能有警告

- [ ] **Step 4: 提交**

```bash
git add crates/mysql-server/src/lib.rs
git commit -m "feat(mysql-server): implement multi-result set support"
```

---

## 任务 5: 修复认证问题 (可选)

**Files:**
- Modify: `crates/mysql-server/src/lib.rs:21` (SKIP_AUTH)

- [ ] **Step 1: 如果需要保持认证，修复认证逻辑**

当前 SKIP_AUTH=true，建议改为 false 并正确实现认证

- [ ] **Step 2: 验证编译**

Run: `cargo build -p sqlrustgo-mysql-server --release 2>&1 | tail -5`

- [ ] **Step 3: 提交**

---

## 任务 6: 测试验证

**Files:**
- Test: 使用 sysbench 进行集成测试

- [ ] **Step 1: 启动 MySQL Server**

```bash
cargo run -p sqlrustgo-mysql-server --release -- --port 13306 &
```

- [ ] **Step 2: 测试基本连接**

```bash
mysql -h localhost -P 13306 -u root -e "SELECT 1"
```

- [ ] **Step 3: 测试事务**

```bash
mysql -h localhost -P 13306 -u root -e "BEGIN; SELECT 1; COMMIT;"
```

- [ ] **Step 4: 测试 sysbench prepare**

```bash
sysbench oltp_insert --db-driver=mysql --mysql-host=127.0.0.1 --mysql-port=13306 prepare
```

- [ ] **Step 5: 测试 sysbench point_select**

```bash
sysbench oltp_point_select --db-driver=mysql --mysql-host=127.0.0.1 --mysql-port=13306 --threads=8 run
```

- [ ] **Step 6: 测试 sysbench read_write**

```bash
sysbench oltp_read_write --db-driver=mysql --mysql-host=127.0.0.1 --mysql-port=13306 --threads=8 run
```

- [ ] **Step 7: 提交测试结果**

```bash
git add .
git commit -m "test(mysql-server): verify sysbench compatibility"
```

---

## 验收标准

- [ ] BEGIN/COMMIT/ROLLBACK 事务命令正常工作
- [ ] 事务中的多个 SELECT 返回多个结果集
- [ ] sysbench oltp_point_select 可以运行
- [ ] sysbench oltp_insert prepare 可以运行
- [ ] sysbench oltp_read_write 可以运行 (核心目标)

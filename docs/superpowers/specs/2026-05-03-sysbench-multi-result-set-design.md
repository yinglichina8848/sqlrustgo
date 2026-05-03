# Sysbench 多结果集与事务支持设计

> **Date**: 2026-05-03
> **Status**: Approved
> **Author**: OpenCode

## 1. 需求

满足 sysbench 测试要求：
- **场景 A**: sysbench prepare 阶段 (CREATE TABLE + INSERT)
- **场景 B**: sysbench oltp_read_write (事务中的多个查询)

## 2. 当前状态

| 功能 | 状态 |
|------|------|
| COM_STMT_PREPARE | ✅ 已实现 |
| COM_STMT_EXECUTE 参数绑定 | ✅ 已实现 |
| CREATE TABLE | ✅ 已实现 (execute_write) |
| INSERT | ✅ 已实现 (execute_write) |
| 事务状态机 | ❌ 未实现 |
| 多结果集 | ❌ 未实现 |

## 3. 架构设计

### 3.1 事务状态机

```
IDLE → (BEGIN) → IN_TRANSACTION → (COMMIT/ROLLBACK) → IDLE
                  ↓
            缓存修改到 local buffer
            COMMIT: 写入 storage
            ROLLBACK: 丢弃 buffer
```

### 3.2 Session 状态

为每个连接维护:
```rust
struct SessionState {
    in_transaction: bool,
    modified_tables: HashSet<String>,
    pending_changes: Vec<PendingChange>,
}
```

### 3.3 多结果集

一个事务中可能有多个 SELECT 返回结果：
- MySQL 协议需要为每个 SELECT 发送结果集
- 在结果集之间发送 OK packet (more results exists flag)

## 4. 实现要点

### 4.1 事务支持

1. 解析 BEGIN/COMMIT/ROLLBACK 命令
2. 维护事务状态
3. 缓存修改操作
4. COMMIT 时批量应用到 storage
5. ROLLBACK 时丢弃缓存

### 4.2 多结果集

1. 遍历事务中的所有查询
2. 对每个 SELECT 调用 `send_result_set`
3. 在结果集之间设置 `more_results_exists` flag
4. 最后一个结果集后发送 OK packet

## 5. 测试验证

- sysbench oltp_point_select
- sysbench oltp_insert
- sysbench oltp_read_write
- sysbench oltp_read_only

# SQL-2: Performance Schema Implementation Design

## Context

Performance Schema 提供数据库性能监控接口，收集和查询执行统计信息。

## Current State

已有基础结构：
- `crates/information-schema/src/performance_schema.rs`: PerformanceSchema 实现
- `crates/information-schema/src/lib.rs`: InformationSchema 集成

已有表：
- `events_statements_current`
- `events_statements_history`
- `events_transactions_current`
- `events_transactions_history`
- `events_waits_current`
- `events_waits_history`
- `lock_waits`
- `recovery_history`
- `wal_stats`

## Target Coverage

目标: ≥60% MySQL Performance Schema 功能覆盖

## Design

### 1. Setup Tables

```sql
CREATE TABLE setup_actors (
    TRIGGER_ID VARCHAR(64),
    FLAGS VARCHAR(64),
    ENABLED CHAR(1),
    HISTORY CHAR(1),
    PROPERTIES VARCHAR(64),
    FLAGS VARCHAR(64)
);

CREATE TABLE setup_instruments (
    NAME VARCHAR(128),
    ENABLED CHAR(1),
    TIMED CHAR(1),
    PROPERTIES VARCHAR(64),
    FLAGS VARCHAR(64),
    VOLATILITY INT
);
```

### 2. Events Statements

扩展 `events_statements_current` 和 `events_statements_history`:

```sql
CREATE TABLE events_statements_current (
    THREAD_ID BIGINT,
    EVENT_ID BIGINT,
    EVENT_NAME VARCHAR(128),
    SOURCE VARCHAR(64),
    TIMER_START BIGINT,
    TIMER_END BIGINT,
    TIMER_WAIT BIGINT,
    LOCK_TIME BIGINT,
    SQL_TEXT VARCHAR(2048),
    DIGEST VARCHAR(64),
    DIGEST_TEXT VARCHAR(256)
);
```

### 3. Events Waits with Ring Buffer

使用 ring buffer 实现固定大小的事件存储：

```rust
struct RingBuffer<T> {
    buffer: Vec<T>,
    size: usize,
    write_pos: usize,
}

impl<T> RingBuffer<T> {
    fn push(&mut self, item: T) {
        self.buffer[self.write_pos] = item;
        self.write_pos = (self.write_pos + 1) % self.size;
    }
}
```

### 4. Summary Tables

```sql
CREATE TABLE events_statements_summary_by_digest (
    SCHEMA_NAME VARCHAR(64),
    DIGEST VARCHAR(64),
    DIGEST_TEXT VARCHAR(256),
    COUNT_STAR BIGINT,
    SUM_TIMER_WAIT BIGINT,
    MIN_TIMER_WAIT BIGINT,
    AVG_TIMER_WAIT BIGINT,
    MAX_TIMER_WAIT BIGINT,
    SUM_LOCK_TIME BIGINT,
    SUM_ERRORS BIGINT,
    SUM_ROWS_AFFECTED BIGINT,
    SUM_ROWS_SENT BIGINT,
    SUM_ROWS_EXAMINED BIGINT
);
```

### 5. Global Events Aggregation

```rust
pub fn get_global_events() -> Vec<GlobalEventRow> {
    // Aggregate all thread events into global counters
}
```

## Implementation Plan

### Phase 1: Setup Tables
- 添加 `setup_actors` 表
- 添加 `setup_instruments` 表

### Phase 2: Events Statements
- 扩展 `events_statements_current/history` 字段
- 添加 SQL_TEXT 和 DIGEST 支持

### Phase 3: Events Waits
- 实现 ring buffer
- 添加 `events_waits_current/history_long`

### Phase 4: Summary Tables
- 实现 `events_statements_summary_by_digest`
- 实现 `events_statements_summary_by_thread`

## Files to Modify

- `crates/information-schema/src/performance_schema.rs`: 扩展 PerformanceSchema
- `crates/information-schema/src/lib.rs`: 添加新表到 InformationSchema

## Verification

```bash
cargo test -p sqlrustgo-information-schema -- performance_schema
```

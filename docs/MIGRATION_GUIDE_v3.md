# v2.9.0 → v3.0.0 API 迁移指南

> **版本**: 1.0
> **更新日期**: 2026-05-06

---

## 一、破坏性变更

### 1.1 `ExecutionEngine::new()` 标记为已弃用

**v2.9.0**:
```rust
let mut engine = ExecutionEngine::new(storage);
```

**v3.0.0**:
```rust
let mut engine = ExecutionEngine::new_with_config(storage, EngineConfig::default());
// 或使用 builder
let mut engine = ExecutionEngine::new_with_config(
    storage,
    EngineConfig::builder().cbo_enabled(true).build()
);
```

**迁移**: 替换 `::new()` → `::new_with_config(storage, EngineConfig::default())`

### 1.2 `ExecutionEngine::with_cbo()` 标记为已弃用

**v2.9.0**:
```rust
let mut engine = ExecutionEngine::with_cbo(storage, true);
```

**v3.0.0**:
```rust
let mut engine = ExecutionEngine::new_with_config(
    storage,
    EngineConfig::builder().cbo_enabled(true).build()
);
```

### 1.3 `EngineConfig` 新增

```rust
let config = EngineConfig {
    cbo_enabled: true,
    stats_enabled: true,
    default_isolation: TmIsolationLevel::SnapshotIsolation,
    cache_config: QueryCacheConfig::default(),
};
```

---

## 二、新功能

| 功能 | API | 版本 |
|------|-----|------|
| SHOW VARIABLES | SQL 命令 | v3.0.0 |
| SHOW TABLES | SQL 命令 | v3.0.0 |
| SHOW COLUMNS | SQL 命令 | v3.0.0 |
| INFORMATION_SCHEMA | SQL 查询 | v3.0.0 |
| CTE (WITH) | SQL 语法 | v3.0.0 |
| CBO 规则 | 自动启用 | v3.0.0 |

---

## 三、SQL 行为变更

| 行为 | v2.9.0 | v3.0.0 |
|------|--------|--------|
| CTE 语法 | 不支持 | 支持 `WITH ... SELECT` |
| SHOW VARIABLES | 不支持 | 返回系统变量 |
| CBO | 全表扫描 | 启用 Predicate Pushdown / Projection Pruning / Constant Folding |

---

## 四、存储格式

向下兼容：v2.9.0 的数据文件可直接在 v3.0.0 使用。

---

## 五、回退

如遇问题，可在配置中禁用新功能：

```bash
# 禁用 CBO（恢复到 v2.9.0 行为）
SQLRUSTGO_CBO_ENABLED=false sqlrustgo

# 禁用查询缓存
SQLRUSTGO_QUERY_CACHE_DISABLED=true sqlrustgo
```

---

*本文档由 SQLRustGo Team 维护*

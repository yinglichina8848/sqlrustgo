# SQLRustGo v2.5.0 Release Notes

> **版本**: v2.5.0
> **发布日期**: 2026-04-15
> **代号**: 全面集成 + GMP
> **当前阶段**: **GA (正式发布)**

---

## 一、版本概述

### 1.1 发布类型

| 项目 | 值 |
|------|---|
| 版本号 | v2.5.0 |
| 发布类型 | 全面集成 + GMP |
| 目标分支 | release/v2.5.0 |
| 开发分支 | develop/v2.5.0 |
| 前置版本 | v2.4.0 (GA) |

### 1.2 核心特性

v2.5.0 是 SQLRustGo 的**全面集成版本**，在 v2.4.0 基础上实现：

- **FOREIGN KEY**: 完整的 Parser + Executor 实现
- **Prepared Statement**: 参数化查询
- **子查询**: EXISTS/ANY/ALL/IN/相关子查询
- **连接池**: Connection Pool + 超时健康检查
- **Graph 持久化**: DiskGraphStore 实现
- **WAL Crash Recovery**: 崩溃恢复机制
- **TPC-H Benchmark**: SF=1 和 SF=10 性能测试
- **JOIN 增强**: LeftSemi/LeftAnti JOIN 支持

---

## 二、功能变更

### 2.1 FOREIGN KEY 约束 (P0) - Issue #1379

| 功能 | 说明 | PR |
|------|------|-----|
| Table-level FK | `CONSTRAINT name FOREIGN KEY (cols) REFERENCES table(cols)` | #1436 |
| Column-level FK | `col INTEGER REFERENCES table(col)` | #1436 |
| ON DELETE CASCADE | 级联删除 | #1436 |
| ON DELETE SET NULL | 删除置空 | #1436 |
| ON DELETE RESTRICT | 删除限制 | #1436 |
| ON UPDATE CASCADE | 级联更新 | #1436 |
| ON UPDATE SET NULL | 更新置空 | #1436 |
| ON UPDATE RESTRICT | 更新限制 | #1436 |
| Self-referencing FK | 自引用外键 | #1436 |
| Parser 修复 | Token::Cascade/Restrict 处理 | #1442 |

**Bug Fix**: 修复 parser 无法正确解析 `ON DELETE/UPDATE CASCADE/RESTRICT` 的问题

### 2.2 Prepared Statement (P1) - Issue #1384

| 功能 | 说明 | PR |
|------|------|-----|
| PREPARE | 预处理语句 | #1421 |
| EXECUTE | 执行预处理 | #1421 |
| DEALLOCATE | 释放预处理 | #1421 |
| 参数绑定 | 参数类型绑定 | #1421 |

### 2.3 子查询支持 (P1) - Issue #1382

| 功能 | 说明 | PR |
|------|------|-----|
| EXISTS | 存在性检查 | #1420 |
| ANY/ALL | 比较运算符 | #1420 |
| IN 子查询 | 列表匹配 | #1422 |
| 相关子查询 | 外部行上下文 | #1426 |

### 2.4 连接池 (P1) - Issue #1383

| 功能 | 说明 | PR |
|------|------|-----|
| 连接池配置 | 可配置池大小 | #1418 |
| 超时机制 | 连接超时控制 | #1418 |
| 健康检查 | 连接健康状态 | #1418 |
| 资源回收 | 自动回收空闲连接 | #1418 |

### 2.5 Graph 持久化 (P0) - Issue #1378

| 功能 | 说明 | PR |
|------|------|-----|
| DiskGraphStore | 图结构持久化 | #1413 |
| 图查询支持 | BFS/DFS/多跳 | #1413 |

### 2.6 WAL Crash Recovery - Issue #1388

| 功能 | 说明 | PR |
|------|------|-----|
| 崩溃恢复 | WAL 日志恢复 | #1406 |
| 写入原子性 | 事务原子性保证 | #1406 |

### 2.7 TPC-H Benchmark (P1) - Issue #1342

| 功能 | 说明 | PR |
|------|------|-----|
| SF=1 基准 | Q1-Q22 性能测试 | #1412 |
| SF=10 基准 | 大规模数据测试 | #1411 |
| Q13/Q22 修复 | 查询语法修复 | #1415 |

### 2.8 JOIN 增强 (P1) - Issue #1380

| 功能 | 说明 | PR |
|------|------|-----|
| LeftSemi JOIN | 左半连接 | #1435 |
| LeftAnti JOIN | 左反连接 | #1435 |
| RIGHT JOIN | 右连接支持 | - |

---

## 三、性能优化

### 3.1 BufferPool 优化

| 优化项 | PR |
|--------|-----|
| 死循环修复 | #1414 |

### 3.2 BloomFilter 优化

| 优化项 | PR |
|--------|-----|
| 布隆过滤器 | #1402, #1404 |

### 3.3 列式存储优化

| 优化项 | PR |
|--------|-----|
| 块级跳过 | #1398 |

### 3.4 查询优化

| 优化项 | PR |
|--------|-----|
| 统一查询 API | #1408 |
| TPC-H Q 优化 | #1407 |

---

## 四、测试验证

### 4.1 FOREIGN KEY 测试

```bash
cargo test -p sqlrustgo --test foreign_key_test
# test result: ok. 23 passed; 0 failed
```

| 测试 | 状态 |
|------|------|
| test_fk_insert_valid_reference | ✅ |
| test_fk_insert_invalid_reference | ✅ |
| test_fk_delete_cascade | ✅ |
| test_fk_delete_set_null | ✅ |
| test_fk_delete_restrict | ✅ |
| test_fk_update_cascade | ✅ |
| test_fk_update_set_null | ✅ |
| test_fk_update_restrict | ✅ |
| test_fk_self_reference_delete_cascade | ✅ |
| test_fk_combined_actions | ✅ |

### 4.2 Parser 测试

```bash
cargo test -p sqlrustgo-parser
# test result: ok. 316 passed; 0 failed
```

---

## 五、相关 Issue

| Issue | 说明 | 状态 |
|-------|------|------|
| #1379 | FOREIGN KEY 约束 | ✅ CLOSED |
| #1380 | JOIN 完整实现 | ⏳ 进行中 |
| #1382 | 子查询优化 | ✅ CLOSED |
| #1383 | 连接池 | ✅ CLOSED |
| #1384 | Prepared Statement | ✅ CLOSED |
| #1389 | MVCC 并发控制 | ⏳ 进行中 |
| #1390 | PITR 备份恢复 | 📋 待开发 |

---

## 六、里程碑

| 日期 | 里程碑 |
|------|---------|
| 2026-04-11 | v2.5.0 开发启动 |
| 2026-04-15 | GA 正式发布 |

---

## 七、已知问题

| 问题 | 说明 | 优先级 |
|------|------|--------|
| MVCC 快照隔离 | Phase 1/3 完成 | P0 |
| JOIN 完整实现 | LEFT/RIGHT JOIN | P1 |

---

## 八、下版本计划

- v2.6.0: MVCC 完整实现
- v2.7.0: JOIN 完整实现
- v2.8.0: CBO 优化器

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-15*
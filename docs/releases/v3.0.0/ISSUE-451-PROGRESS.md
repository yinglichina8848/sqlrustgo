# ISSUE-451 进展报告

> **日期**: 2026-05-08
> **分支**: feat/issue-451-sql-ops-v3
> **目标**: 提升 `test_sql_corpus_operations` 通过率 ≥ 80%

## 当前状态

### 测试结果

| 测试 | 通过率 | 目标 | 状态 |
|------|--------|------|------|
| `test_sql_corpus_all` | 94.1% | ≥80% (Beta) | ✅ 达标 |
| `test_sql_corpus_operations` | 12.7% (7/55) | ≥80% | ❌ 未达标 |

### 失败用例分析

`test_sql_corpus_operations` 失败的 48 个用例分类：

| 类别 | 用例数 | 失败原因 |
|------|--------|----------|
| BACKUP/RESTORE | 6 | 缺少 BACKUP/RESTORE 关键字 |
| CHECK TABLE | 3 | 缺少 Statement 变体和解析器入口 |
| OPTIMIZE/REPAIR TABLE | 4 | 缺少解析器入口 |
| VACUUM TABLE | 3 | 缺少解析器入口 |
| ISOLATION LEVEL | 5 | 事务解析不完整 |
| SAVEPOINT | 3 | 缺少解析器入口 |
| SHOW 命令 | 8 | parse_show 扩展不足 |
| INFORMATION_SCHEMA | 4 | executor 不支持 |
| EXPLAIN | 5 | executor 不支持 |
| 其他 | 7 | 各种原因 |

## 技术限制

### 1. Parser 支持不完整

以下语句需要完整的 Statement 变体和解析函数：

```rust
// 需要添加的 Statement 变体
Statement::Check(CheckStatement)
Statement::Optimize(OptimizeStatement)
Statement::Vacuum(VacuumStatement)
Statement::Repair(RepairStatement)
Statement::Backup(BackupStatement)
Statement::Restore(RestoreStatement)
Statement::Savepoint(SavepointStatement)
```

### 2. Executor 支持缺失

即使 Parser 能解析，executor 也需要支持：

- INFORMATION_SCHEMA 元数据查询
- BACKUP/RESTORE 文件操作
- 事务隔离级别完整支持

## 延期原因

1. **工作量评估**：完整支持所有失败用例需要：
   - 添加 7+ 个新的 Statement 变体
   - 实现 7+ 个新的 parse_xxx 函数
   - 实现对应的 executor 支持
   - 估计 2-4 人日工作量

2. **优先级权衡**：
   - `test_sql_corpus_all` 已达到 94.1%，满足 Beta Gate 要求
   - `test_sql_corpus_operations` 是 R8 的一部分，Beta 要求是 `test_sql_corpus_all` ≥ 80%
   - RC/GA 阶段可能需要更完整的支持

3. **建议**：
   - ISSUE-451 延期到 v3.1.0
   - v3.0.0 保持当前状态（test_sql_corpus_all 94.1%）

## 后续计划

v3.1.0 需要完成的工作：

1. 添加 CHECK/OPTIMIZE/REPAIR TABLE 语句支持
2. 添加 BACKUP/RESTORE 语句支持
3. 扩展 SHOW 命令支持
4. 完善事务隔离级别支持
5. 提升 `test_sql_corpus_operations` 至 ≥80%

## 结论

ISSUE-451 在 v3.0.0 阶段**部分完成**：
- ✅ `test_sql_corpus_all` 达到 94.1%（Beta 要求 ≥80%）
- ❌ `test_sql_corpus_operations` 仍为 12.7%

建议延期到 v3.1.0 继续完成。

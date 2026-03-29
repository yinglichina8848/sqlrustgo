# SQLRustGo v3.0 开发计划

> **版本**: 3.0
> **日期**: 2026-03-28
> **目标**: MySQL 5.6+ 兼容 - 触发器、分区表、全文索引、Auto Tuning
> **前置条件**: v2.2 GA 发布
> **预计周期**: 2 个月
> **Agent**: 多Agent并行开发

---

## 1. 版本目标

v3.0 是"MySQL 5.6 兼容版"，补齐 MySQL 标志性的高级特性，真正可以替代 MySQL 用于生产。

---

## 2. 任务分解

### 2.1 触发器与存储过程 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1301 | 触发器语法解析 (CREATE TRIGGER) | 12 | Claude A | P0 |
| #1302 | 触发器执行引擎 | 15 | Claude A | P0 |
| #1303 | 行级触发器 vs 语句级触发器 | 8 | Claude A | P0 |
| #1304 | 存储过程基础 (无事务) | 20 | Claude A | P0 |
| #1305 | 存储函数 | 12 | Claude A | P0 |

### 2.2 分区表 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1311 | 分区表语法解析 (RANGE/KEY/HASH) | 10 | OpenCode A | P0 |
| #1312 | 分区表物理存储设计 | 15 | OpenCode A | P0 |
| #1313 | 分区裁剪优化 | 12 | OpenCode A | P0 |
| #1314 | 分区表 DDL (ADD/DROP PARTITION) | 8 | OpenCode A | P0 |

### 2.3 全文索引 (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1321 | 全文索引语法 (FULLTEXT INDEX) | 8 | Claude B | P1 |
| #1322 | 倒排索引实现 | 15 | Claude B | P1 |
| #1323 | MATCH ... AGAINST 查询 | 10 | Claude B | P1 |
| #1324 | 中文分词 (结巴/RMM) | 8 | Claude B | P1 |

### 2.4 Prepared Statements (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1331 | PREPARE 语句解析 | 6 | OpenCode B | P1 |
| #1332 | EXECUTE 执行 | 8 | OpenCode B | P1 |
| #1333 | 参数绑定 (PreparedStatement 缓存) | 10 | OpenCode B | P1 |

### 2.5 高级复制 (P2)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1341 | GTID 复制 | 18 | OpenCode A | P2 |
| #1342 | 延迟复制 | 10 | OpenCode A | P2 |
| #1343 | 并行复制 (LOGICAL_CLOCK) | 12 | OpenCode A | P2 |

### 2.6 Auto Tuning (P2)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1351 | Buffer Pool 自动调参 | 12 | Claude B | P2 |
| #1352 | 慢查询自动分析 + 建议 | 10 | Claude B | P2 |
| #1353 | 索引推荐 | 15 | Claude B | P2 |

### 2.7 JSON 函数 (P2)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1361 | JSON 数据类型 | 8 | OpenCode B | P2 |
| #1362 | JSON_EXTRACT / JSON_SET | 10 | OpenCode B | P2 |
| #1363 | JSON_ARRAY / JSON_OBJECT | 6 | OpenCode B | P2 |

### 2.8 CTE 与递归 (P3)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1371 | WITH RECURSIVE 语法 | 15 | Claude A | P3 |
| #1372 | 递归执行引擎 | 12 | Claude A | P3 |

---

## 3. Issue 清单

```bash
# 创建所有 v3.0 Issue
# 触发器与存储过程
gh issue create --title "[v3.0][P0] 触发器语法解析" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 触发器执行引擎" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 行级/语句级触发器" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 存储过程基础" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 存储函数" --body "..." --label "enhancement"

# 分区表
gh issue create --title "[v3.0][P0] 分区表语法解析" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 分区表物理存储设计" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 分区裁剪优化" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P0] 分区表DDL" --body "..." --label "enhancement"

# 全文索引
gh issue create --title "[v3.0][P1] 全文索引语法" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P1] 倒排索引实现" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P1] MATCH查询" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P1] 中文分词" --body "..." --label "enhancement"

# Prepared Statements
gh issue create --title "[v3.0][P1] PREPARE语句解析" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P1] EXECUTE执行" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P1] PreparedStatement缓存" --body "..." --label "enhancement"

# 高级复制
gh issue create --title "[v3.0][P2] GTID复制" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] 延迟复制" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] 并行复制" --body "..." --label "enhancement"

# Auto Tuning
gh issue create --title "[v3.0][P2] Buffer Pool自动调参" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] 慢查询自动分析" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] 索引推荐" --body "..." --label "enhancement"

# JSON
gh issue create --title "[v3.0][P2] JSON数据类型" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] JSON_EXTRACT" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P2] JSON_ARRAY/JSON_OBJECT" --body "..." --label "enhancement"

# CTE
gh issue create --title "[v3.0][P3] WITH RECURSIVE语法" --body "..." --label "enhancement"
gh issue create --title "[v3.0][P3] 递归执行引擎" --body "..." --label "enhancement"
```

---

## 4. 开发顺序

```
Month 1: 触发器 + 分区表
Week 1:
  ├── #1301 触发器语法
  └── #1311 分区表语法

Week 2:
  ├── #1302 触发器引擎
  └── #1312 分区表存储

Week 3:
  ├── #1303 行级触发器
  ├── #1313 分区裁剪
  └── #1321 全文索引语法

Week 4:
  ├── #1304 存储过程基础
  ├── #1314 分区DDL
  └── #1322 倒排索引

Month 2: 全文索引 + Prepared + JSON + AutoTuning
Week 5:
  ├── #1323 MATCH查询
  ├── #1331 PREPARE解析
  └── #1361 JSON类型

Week 6:
  ├── #1324 中文分词
  ├── #1332 EXECUTE执行
  ├── #1333 缓存
  └── #1362 JSON函数

Week 7:
  ├── #1341 GTID
  ├── #1351 Buffer调参
  └── #1352 慢查询分析

Week 8:
  ├── #1371 CTE语法
  ├── #1372 递归引擎
  └── v3.0 GA 发布
```

---

## 5. 交付物

- [ ] 触发器 (行级/语句级)
- [ ] 存储过程 + 存储函数
- [ ] 分区表 (RANGE/KEY/HASH)
- [ ] 分区裁剪优化
- [ ] 全文索引 + 中文分词
- [ ] Prepared Statements + 缓存
- [ ] GTID 复制
- [ ] Auto Tuning (Buffer调参 + 索引推荐)
- [ ] JSON 数据类型 + 函数
- [ ] CTE RECURSIVE
- [ ] 性能基准: 50 并发 ≥ 5000 QPS (v2.2 +150%)
- [ ] MySQL 5.6 兼容性 ≥ 80%

---

## 6. 里程碑

| 日期 | 里程碑 |
|------|--------|
| Month 1 Week 4 | 触发器 + 分区表核心完成 |
| Month 2 Week 4 | v3.0 GA 发布 |

---

## 7. MySQL 5.6 兼容度目标

| 特性 | 状态 |
|------|------|
| 触发器 | ✅ |
| 存储过程/函数 | ✅ |
| 分区表 | ✅ |
| 全文索引 | ✅ |
| Prepared Statements | ✅ |
| GTID | ✅ |
| 延迟复制 | ✅ |
| 并行复制 | ✅ |
| JSON 函数 | ✅ |
| CTE RECURSIVE | ✅ |
| Auto Tuning | ✅ |

**MySQL 5.6 兼容度: 目标 ≥ 80%**

---

**状态**: 📋 规划完成，待创建Issue

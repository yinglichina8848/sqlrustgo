# v3.0.0 Agent 任务分工 & 提示词

> **发布位置**: Issue #353 (Gitea comment)
> **日期**: 2026-05-06
> **分支**: `develop/v3.0.0` @ `74c3abe9`
> **版本号**: Cargo.toml 仍为 `2.5.0`（建议每个 Agent 批量更新至 `3.0.0`）

---

## 一、总体状态

| 指标 | 目标 | 当前 |
|------|------|------|
| Point SELECT QPS | ≥20,000 | **7,312**（待 CBO 优化） |
| UPDATE QPS | ≥10,000 | **42,427** ✅ |
| DELETE QPS | ≥5,000 | **62,352** ✅ |
| SQL Corpus | ≥98% | **100%** ✅ |
| TPC-H SF=0.1 | 22/22 | **22/22** 🟡 (~10.9s) |
| 覆盖率 | ≥85% | 84.18%（旧数据） |
| Clippy | 零警告 | ✅ |
| Format | 零差异 | ✅ |

### 24 项已完成任务

优化器桥接 / 查询缓存 / 连接池 / Group Commit / INSERT...SELECT / 窗口函数 6 个 / CTE 执行 / INFORMATION_SCHEMA / EXPLAIN ANALYZE / SSL/TLS / 慢查询日志 / CI Gate / SHOW VARIABLES / 运维手册 / ADR / API 版本化 / 迁移指南 / 教学模式 / 在线 DDL / mysqldump 导出 / 性能调优指南 / PP-06 内存治理 / PROOF-026 Write Skew / SQL Corpus 100%

### 未完成

CBO 代价模型 / Sysbench OLTP / COM_MULTI / Prepared Statement 绑定 / 事务压力测试 / Optimizer 测试扩展 / Planner 测试扩展 / A-HYG 覆盖率/安全/文档链 / TPC-H SF=1 CI Gate

---

## 二、opencode — 提示词

### 工作目录

```bash
cd ~/workspace/dev/openheart/sqlrustgo
git checkout develop/v3.0.0
git pull origin develop/v3.0.0
git checkout -b feat/opencode-v3-cbo
```

### P0 任务 (必须完成)

#### 1. CBO 代价模型集成 (5-7d)

目标: `SimpleCostModel` 接入 planner，实现基于代价的索引选择和 Join 重排序。

```bash
# 文件
crates/optimizer/src/cost.rs    — SimpleCostModel (已实现)
crates/planner/src/planner.rs   — 需接入代价模型
crates/planner/src/lib.rs       — 需暴露优化接口

# 验证
cargo test -p sqlrustgo-planner
cargo test -p sqlrustgo-optimizer
```

**验收标准**:
- `EXPLAIN SELECT * FROM t WHERE id = 1` 选择索引扫描而非全表扫描
- TPC-H Q1 执行时间减少 ≥50%
- optimizer 测试全部通过

#### 2. Sysbench OLTP 适配 (#376, 3d)

```bash
# 分支
git checkout -b feat/opencode-sysbench

# 文件
crates/mysql-server/src/lib.rs   — 协议层
scripts/gate/check_sysbench.sh    — 门禁 (已创建)
tests/e2e/                        — 集成测试
```

**验收标准**:
- oltp_read_only QPS 可测量
- oltp_write_only (INSERT/UPDATE/DELETE) 通过
- oltp_read_write 通过
- 无 "No transaction in progress" 错误

#### 3. COM_MULTI 多语句执行 (#377, 2d)

```bash
# 文件
crates/mysql-server/src/lib.rs

# 需要
- 定义 0x11 常量
- split_sql_statements() 已存在 (line 1404)
- 多结果集协议 (SERVER_MORE_RESULTS_EXISTS 已定义)
```

**验收标准**: sysbench prepare 阶段通过

#### 4. Prepared Statement 参数绑定修复 (#378, 1d)

```bash
# 文件
crates/mysql-server/src/lib.rs (replace_placeholders, line ~1735)
```

**验收标准**: COM_STMT_EXECUTE 带参数能正确替换占位符

### 批量操作

完成后批量更新 Cargo.toml 版本号和 Issue 状态:
```bash
sed -i '' 's/^version = "2.5.0"/version = "3.0.0"/' Cargo.toml
# 关闭 Issue
curl -X PATCH http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues/376 \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  -H "Content-Type: application/json" \
  -d '{"state":"closed","body":"..."}'
```

---

## 三、claude — 提示词

### 工作目录

```bash
cd ~/workspace/dev/yinglichina163/sqlrustgo
git remote add gitea http://192.168.0.252:3000/openclaw/sqlrustgo.git
git fetch gitea develop/v3.0.0
git checkout -b feat/claude-v3-tx-stress gitea/develop/v3.0.0
```

### P0 任务

#### 1. 事务状态机压力测试 (#379, 2d)

```bash
# 文件
crates/transaction/tests/
crates/mysql-server/src/lib.rs
```

**验收标准**:
- 100 并发 BEGIN/COMMIT/ROLLBACK 无状态泄漏
- 嵌套事务正确回滚
- 长时间空闲连接后事务状态正确

### P1 任务

#### 2. Optimizer 测试扩展 (#380, 2d)

```bash
# 文件
crates/optimizer/tests/
crates/optimizer/src/rules.rs
```

**验收标准**:
- Predicate Pushdown 专用测试
- Projection Pruning 专用测试
- optimizer 覆盖率 ≥70%

#### 3. Planner 测试扩展 (#381, 2d)

```bash
# 文件
crates/planner/tests/
```

**验收标准**:
- SELECT/INSERT/UPDATE/DELETE plan 转换测试
- JOIN/子查询/CTE plan 转换测试
- planner 覆盖率 ≥80%

---

## 四、deepseek — 提示词 (我)

### 工作目录

```bash
cd ~/workspace/dev/openheart/sqlrustgo
git checkout develop/v3.0.0
```

### 当前状态

✅ Clippy 零警告, Format 零差异, Doc links 有效

### 待办

| 优先级 | 任务 | 说明 |
|--------|------|------|
| P0 | 文档同步 | DEVELOPMENT_PLAN.md 已同步 ✅ |
| P1 | ISSUE_CLOSURE_PLAN.md 更新 | 需标记 #376-#382 |
| P1 | Issue #353 body 更新 | 添加 Agent 分工表 |
| P1 | PR #387 合并状态跟踪 | — |

### 分工说明

- 无新增 P0 代码任务
- 聚焦文档同步和 Issue 管理
- 等待 opencode 完成 #376-#378 后做回归测试

---

## 五、Issue #376-#382 关闭条件

| Issue | 标题 | 关闭条件 | 负责人 |
|-------|------|---------|--------|
| #376 | Sysbench OLTP 适配 | 3 场景全部通过 + QPS 可测量 | opencode |
| #377 | COM_MULTI | sysbench prepare 通过 | opencode |
| #378 | Prepared Statement 绑定 | 参数化查询正确 | opencode |
| #379 | 事务状态机压力测试 | 100 并发无泄漏 | claude |
| #380 | Optimizer 测试扩展 | 覆盖率 ≥70% | claude |
| #381 | Planner 测试扩展 | 覆盖率 ≥80% | claude |
| #382 | TPC-H SF=1 CI Gate | check_tpch.sh --sf1 可运行 | — |

---

## 六、分支管理提醒

开发分支从 `develop/v3.0.0` 切出，命名规则 `feat/<agent>-<description>`。
完成后开 PR 到 `develop/v3.0.0`，PR 描述需包含验收标准检查清单。

Gitea API Token 供参考:
```
04bcda86dd601364a53eec33dc37aa6efa98a5b7
Gitea URL: http://192.168.0.252:3000/openclaw/sqlrustgo
```

---

*发布位置: Issue #353 评论 | docs/releases/v3.0.0/AGENT_TASKS.md*

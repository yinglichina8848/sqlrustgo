# SQLRustGo 版本演化计划

> **版本**: v6.0
> **更新日期**: 2026-03-21
> **战略定位**: 教学数据库产品（Teaching DBMS）
> **核心原则**: 替代 MySQL 教学，差异化超越

---

## 一、战略定位（关键转向）

### 从：数据库内核工程
### 到：教学数据库产品（Teaching DBMS）

```
💡 核心洞察：

❌ 不要试图完全兼容 MySQL（工程量爆炸）
✅ 正确策略：兼容"教材"，而不是兼容"数据库"

🎯 最终目标：

v1.9 = 可替代 MySQL（教学 + 上机）
v2.0 = 明显优于 MySQL（教学体验 + 分析能力）
```

---

## 二、版本语义化与技术范式

| 主版本 | 技术范式 | 成熟度等级 | 核心特征 | 对标系统 |
|--------|----------|------------|----------|----------|
| 1.x | 单机基础数据库 | L3 | 火山模型、SQL-92、MVCC、B+Tree | early PostgreSQL |
| 2.x | 高性能分析引擎 | L4 | 向量化、列存、CBO、并行执行 | DuckDB |

**演进路径**：1.x（教学可用）→ 2.x（超越 MySQL）

---

## 三、版本路线（教学 DBMS 战略）

### 战略总览

```
v1.6 ✅      v1.7        v1.8         v1.9         v2.0
 Benchmark   SQL+可观测   MySQL兼容    稳定落地     高性能分析
 修复       补完        ⭐核心        ⭐⭐⭐
           能讲清      可替代         课程使用    超越MySQL
           执行过程     MySQL教学
```

### 详细版本规划

| 版本 | 代号 | 核心目标 | 关键功能 |
|------|------|----------|----------|
| **v1.7** | **SQL+可观测补完** | 能讲清执行过程 | EXPLAIN ANALYZE, UNION, VIEW |
| **v1.8** | **MySQL 教学兼容** | 可替代 MySQL | FOREIGN KEY, MySQL 语法, CLI, MySQL 协议 |
| **v1.9** | **稳定+教学落地** | 正式课程使用 | 稳定性, ≥85%覆盖, 教学文档 |
| **v2.0** | **高性能分析** | 超越 MySQL | 向量化, 列存, 可视化执行 |

---

## 四、SQL-92 分阶段覆盖

```
v1.7 (60%)     v1.8 (90%)      v1.9 (100%)
┌─────────┐     ┌─────────┐      ┌─────────┐
│ 核心子集│ →   │ 高级扩展│  →   │ 全覆盖  │
│ +可观测 │     │ +兼容  │      │ +稳定   │
└─────────┘     └─────────┘      └─────────┘
```

### v1.7: SQL 核心补完

| 类别 | 功能 | 状态 |
|------|------|------|
| 查询 | SELECT, WHERE, GROUP BY, ORDER BY, LIMIT | ✅ |
| DML | INSERT, UPDATE, DELETE | ✅ |
| DDL | CREATE TABLE, DROP TABLE | ✅ |
| 事务 | BEGIN, COMMIT, ROLLBACK | ✅ |
| **新增** | **UNION, UNION ALL, VIEW** | **🔄** |
| **新增** | **EXPLAIN, EXPLAIN ANALYZE** | **🔄** |

### v1.8: MySQL 教学兼容

| 类别 | 功能 | 状态 |
|------|------|------|
| **新增** | **FOREIGN KEY** | **🔄** |
| **新增** | **AUTO_INCREMENT** | **🔄** |
| **新增** | **SHOW TABLES, DESCRIBE** | **🔄** |
| **新增** | **LIMIT offset, count** | **🔄** |
| **新增** | **NOW(), COUNT(), DATE_FORMAT()** | **🔄** |
| JOIN | INNER, LEFT, RIGHT, CROSS | 🔄 |

### v1.9: 全覆盖 + 稳定

| 类别 | 功能 | 状态 |
|------|------|------|
| 完整 | SQL-92 100% | 🔄 |
| 过程 | 存储过程基础 | 🔄 |
| 稳定 | WAL 恢复, Crash 安全 | 🔄 |
| 教学 | 实验文档, 标准数据集 | 🔄 |

---

## 五、v1.8 核心版本详解（MySQL 教学兼容）

### 🔥 硬性要求（必须完成）

#### 1️⃣ 外键（最重要）

```sql
FOREIGN KEY (user_id) REFERENCES users(id)
```

> 没这个 = 不能做数据库设计实验

#### 2️⃣ MySQL 兼容语法

```sql
-- AUTO_INCREMENT
CREATE TABLE users (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(255)
);

-- LIMIT offset, count
SELECT * FROM orders LIMIT 10, 20;

-- SHOW / DESCRIBE
SHOW TABLES;
DESCRIBE orders;
```

#### 3️⃣ CLI 工具

```bash
sqlrustgo

支持：
.tables
.schema orders
.indexes orders
```

#### 4️⃣ MySQL 协议（强烈建议）

```
支持 DBeaver, MySQL Workbench 连接
```

**完成后效果**：
> 💥 SQLRustGo 可以"假装 MySQL"被工具连接

---

## 六、v1.9 核心版本详解（稳定+教学落地）

### 🔥 稳定 + 教学支持

#### 1️⃣ 稳定性

- WAL 恢复强化
- Crash 安全
- 长时间运行测试

#### 2️⃣ 覆盖率

- ≥ 85%

#### 3️⃣ 教学模式（建议加🔥）

```bash
SQLRUSTGO_TEACHING_MODE=1
```

效果：
- 禁用优化器（便于教学）
- 强制 EXPLAIN 输出
- 更详细日志

#### 4️⃣ 教学支持

- 12 个标准实验
- MySQL → SQLRustGo 对照表
- Lab 文档

---

## 七、v2.0 核心版本详解（高性能分析）

### 🔥 超越 MySQL

#### 1️⃣ 向量化执行（核心革命）

```rust
// DataChunk 格式
struct DataChunk {
    columns: Vec<ColumnArray>,
    num_rows: usize,
}

// SIMD 加速
impl AggregateFunction for Sum<i32> {
    fn sum_batch(&self, chunk: &DataChunk) -> ScalarValue {
        // SIMD 加速聚合
    }
}
```

#### 2️⃣ 列存（分析能力）

- Columnar storage
- Projection pushdown
- Parquet 支持

#### 3️⃣ 教学增强（差异化）

> MySQL 做不到的：

- 可视化执行 pipeline
- 算子级 profiling
- vectorized trace

---

## 八、可观测性演进

| 版本 | 新增可观测能力 |
|------|----------------|
| v1.7 | **EXPLAIN ANALYZE**（核心亮点）, 算子级耗时 |
| v1.8 | MySQL 兼容 CLI, 索引信息 |
| v1.9 | 教学模式, 详细执行日志 |
| v2.0 | 向量化 trace, 可视化 pipeline |

---

## 九、成熟度演进

```
L1 (Toy)   →   L2 (Query Engine)   →   L3 (Mini DBMS)   →   L4 (Analytical DB)
                  v1.7                    v1.9                 v2.0
              能讲清执行             可替代MySQL教学        超越MySQL
```

---

## 十、风险与应对

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 试图完全兼容 MySQL | 极高 | ❌ 禁止！只兼容"教材" |
| MySQL 协议实现难度 | 高 | v1.8 先做 CLI，协议可选 |
| 教学文档工作量 | 中 | 参考现有 MySQL 教材 |
| 向量化开发难度 | 高 | v2.0 预留充足时间 |

---

## 十一、版本号与分支策略

| 策略 | 说明 |
|------|------|
| 开发分支 | develop/v1.7.0、develop/v1.8.0 等 |
| 发布分支 | release/v1.7.0、release/v1.8.0 等 |
| 主分支 | main 始终指向最新稳定版 |

---

## 十二、一句顶级结论

```
💥 你现在不是在做数据库
👉 而是在做"下一代数据库教学平台"
```

---

## 十三、关联文档

| 文档 | 说明 |
|------|------|
| `docs/plans/2026-03-21-v170-release-plan.md` | v1.7.0 详细计划 |
| `docs/releases/v1.6.1/RELEASE_GATE_CHECKLIST.md` | v1.6.1 门禁参考 |

---

## 十四、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-12 | 初始版本 |
| 2.0 | 2026-03-13 | v1.x/v2.x/v3.x 重构 |
| 3.0 | 2026-03-13 | 工程优化版 |
| 4.0 | 2026-03-18 | 整合 v1.x 版本 |
| 5.0 | 2026-03-21 | SQL-92 路线图 |
| **6.0** | **2026-03-21** | **教学 DBMS 战略定位** |

---

**文档状态**: 定稿
**制定日期**: 2026-03-21
**最后更新**: 2026-03-21
**制定人**: yinglichina8848
**战略定位**: 教学数据库产品（Teaching DBMS）
# SQLRustGo 版本演化计划

> **版本**: v7.0
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

v1.7 = 可替代 MySQL（教学 + 上机）← 合并 v1.8 + v1.9
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
v1.6 ✅      v1.7 (合并版)                    v2.0
  Benchmark   SQL+可观测+MySQL兼容+稳定+教学   高性能分析
  修复        ⭐⭐⭐ 全面替代 MySQL 教学         超越MySQL
```

### 详细版本规划

| 版本 | 代号 | 核心目标 | 关键功能 |
|------|------|----------|----------|
| **v1.7** | **MySQL 教学替代版** | 可替代 MySQL | UNION/VIEW, EXPLAIN ANALYZE, FOREIGN KEY, AUTO_INCREMENT, CLI, WAL, ≥85%覆盖 |
| **v2.0** | **高性能分析** | 超越 MySQL | 向量化, 列存, 可视化执行 |

### v1.7 Epics (合并版)

| Epic | 名称 | 来源版本 |
|------|------|----------|
| Epic-01 | SQL 补完 | 原 v1.7 |
| Epic-02 | 可观测性 | 原 v1.7 |
| Epic-03 | Benchmark 完善 | 原 v1.7 |
| Epic-04 | 错误系统 | 原 v1.7 |
| Epic-05 | 约束与外键 | 原 v1.8 |
| Epic-06 | MySQL 兼容语法 | 原 v1.8 |
| Epic-07 | CLI 工具完善 | 原 v1.8 |
| Epic-08 | 稳定性强化 | 原 v1.9 |
| Epic-09 | 覆盖率提升 | 原 v1.9 |
| Epic-10 | 教学支持 | 原 v1.9 |

---

## 四、v1.7 核心版本详解（MySQL 教学替代版）

### 🔥 一站式替代 MySQL 教学

#### Epic-01~04: SQL + 可观测性（原 v1.7）

| 功能 | 说明 |
|------|------|
| UNION, UNION ALL, INTERSECT, EXCEPT | 集合运算 |
| VIEW | 视图支持 |
| EXPLAIN, EXPLAIN ANALYZE | 执行计划可视化 |
| MySQL 风格错误 | Unknown column, Table not found, Duplicate key |

#### Epic-05: 约束与外键（原 v1.8）

```sql
FOREIGN KEY (user_id) REFERENCES users(id)
```

#### Epic-06: MySQL 兼容语法（原 v1.8）

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

-- 常用函数
NOW(), COUNT(), DATE_FORMAT()
```

#### Epic-07: CLI 工具完善（原 v1.8）

```bash
sqlrustgo

支持：
.tables
.schema orders
.indexes orders
```

#### Epic-08: 稳定性强化（原 v1.9）

- WAL 恢复强化
- Crash 安全
- 长时间运行测试

#### Epic-09: 覆盖率提升（原 v1.9）

- ≥ 85%

#### Epic-10: 教学支持（原 v1.9）

```bash
SQLRUSTGO_TEACHING_MODE=1
```

效果：
- 禁用优化器（便于教学）
- 强制 EXPLAIN 输出
- 更详细日志

教学资源：
- 12 个标准实验
- MySQL → SQLRustGo 对照表
- Lab 文档

---

## 五、v2.0 核心版本详解（高性能分析）

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

## 六、可观测性演进

| 版本 | 新增可观测能力 |
|------|----------------|
| v1.7 | **EXPLAIN ANALYZE**（核心亮点）, 算子级耗时, 教学模式 |

---

## 七、成熟度演进

```
L1 (Toy)   →   L2 (Query Engine)   →   L3 (Mini DBMS)   →   L4 (Analytical DB)
                   v1.7                    v2.0
               能替代MySQL教学          超越MySQL
```

---

## 八、风险与应对

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 试图完全兼容 MySQL | 极高 | ❌ 禁止！只兼容"教材" |
| MySQL 协议实现难度 | 高 | 先做 CLI，协议可选 |
| 教学文档工作量 | 中 | 参考现有 MySQL 教材 |
| 向量化开发难度 | 高 | v2.0 预留充足时间 |

---

## 九、版本号与分支策略

| 策略 | 说明 |
|------|------|
| 开发分支 | develop/v1.7.0, develop/v2.0 |
| 发布分支 | release/v1.7.0, release/v2.0 |
| 主分支 | main 始终指向最新稳定版 |

---

## 十、一句顶级结论

```
💥 你现在不是在做数据库
👉 而是在做"下一代数据库教学平台"
```

---

## 十一、关联文档

| 文档 | 说明 |
|------|------|
| `docs/plans/2026-03-21-v170-release-plan.md` | v1.7.0 详细计划 |
| `docs/releases/v1.6.1/RELEASE_GATE_CHECKLIST.md` | v1.6.1 门禁参考 |

---

## 十二、变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-12 | 初始版本 |
| 2.0 | 2026-03-13 | v1.x/v2.x/v3.x 重构 |
| 3.0 | 2026-03-13 | 工程优化版 |
| 4.0 | 2026-03-18 | 整合 v1.x 版本 |
| 5.0 | 2026-03-21 | SQL-92 路线图 |
| 6.0 | 2026-03-21 | 教学 DBMS 战略定位 |
| **7.0** | **2026-03-21** | **v1.7 合并 v1.8+v1.9，v2.0 独立** |

---

**文档状态**: 定稿
**制定日期**: 2026-03-21
**最后更新**: 2026-03-21
**制定人**: yinglichina8848
**战略定位**: 教学数据库产品（Teaching DBMS）
# SQLRustGo Sysbench OLTP 最小支持矩阵

> **目标**：让 SQLRustGo 从 connect → prepare → run 跑通 sysbench OLTP benchmark

---

## 一、sysbench OLTP 执行流程全景

sysbench 典型流程：

```
connect
→ create table
→ insert rows
→ create index
→ prepare finished
→ transaction workload
→ select/update/delete
```

对应数据库能力依赖：

| 阶段      | 必需能力                |
| ------- | ------------------- |
| connect | MySQL wire protocol |
| prepare | DDL                 |
| prepare | INSERT              |
| prepare | INDEX               |
| run     | SELECT              |
| run     | UPDATE              |
| run     | DELETE              |
| run     | transaction         |

---

## 二、Phase 1：通过 sysbench connect（最低要求）

目标：

```
sysbench oltp_common.lua prepare
```

至少需要支持：

```sql
SELECT 1
```

以及：

```sql
SHOW VARIABLES
```

sysbench 会调用：

```sql
SHOW VARIABLES LIKE 'tx_isolation'
```

否则直接失败。

---

## 三、Phase 2：通过 prepare 阶段（核心）

prepare 阶段执行：

### 1️⃣ CREATE TABLE

必须支持：

```sql
CREATE TABLE sbtest1 (
  id INTEGER NOT NULL,
  k INTEGER DEFAULT 0 NOT NULL,
  c CHAR(120) DEFAULT '' NOT NULL,
  pad CHAR(60) DEFAULT '' NOT NULL,
  PRIMARY KEY (id)
)
```

SQLRustGo 必须实现：

| feature      | required |
| ------------ | -------- |
| CREATE TABLE | ✅        |
| PRIMARY KEY  | ✅        |
| NOT NULL     | ✅        |
| DEFAULT      | ✅        |
| CHAR(n)      | ✅        |
| INTEGER      | ✅        |

最低实现方式：可以先忽略 DEFAULT 和 NOT NULL enforcement，只需 parser + catalog metadata 即可。

### 2️⃣ CREATE INDEX

prepare 阶段执行：

```sql
CREATE INDEX k_1 ON sbtest1(k)
```

必须支持：

| feature             | required |
| ------------------- | -------- |
| CREATE INDEX        | ✅        |
| single column index | ✅        |

最小实现：甚至可以 fake index metadata only，不需要真实索引结构（第一阶段）。

### 3️⃣ INSERT

prepare 阶段执行：

```sql
INSERT INTO sbtest1 VALUES (...)
```

必须支持：

| feature           | required |
| ----------------- | -------- |
| INSERT VALUES     | ✅        |
| multi-row insert  | 可选       |
| single-row insert | 必须       |

最低 executor：Vec<Row> 即可通过。

---

## 四、Phase 3：通过 run 阶段（OLTP workload）

sysbench run 会执行的核心 SQL：

### 1️⃣ Point Select

```sql
SELECT c FROM sbtest1 WHERE id=?
```

必须支持：

| feature            | required |
| ------------------ | -------- |
| SELECT             | ✅        |
| WHERE              | ✅        |
| equality predicate | ✅        |
| primary key lookup | 推荐       |

最低实现：table scan 即可运行（慢但可运行）。

### 2️⃣ Range Select

```sql
SELECT c FROM sbtest1 WHERE id BETWEEN ? AND ?
```

必须支持：

| feature | required |
| ------- | -------- |
| BETWEEN | ✅        |

可退化：scan + filter

### 3️⃣ UPDATE

```sql
UPDATE sbtest1 SET k=k+1 WHERE id=?
```

必须支持：

| feature               | required |
| --------------------- | -------- |
| UPDATE                | ✅        |
| arithmetic expression | 推荐       |

最低实现：甚至可以 SET k=constant，第一阶段允许。

### 4️⃣ DELETE

```sql
DELETE FROM sbtest1 WHERE id=?
```

必须支持：

| feature | required |
| ------- | -------- |
| DELETE  | ✅        |

实现：mark deleted flag 即可。

---

## 五、Phase 4：事务支持（关键）

sysbench 默认开启事务，执行：

```
BEGIN
SELECT
UPDATE
COMMIT
```

SQLRustGo 必须支持：

```sql
BEGIN
COMMIT
```

最低实现：fake transaction（no-op transaction manager）即可运行 benchmark：

```
BEGIN → ignore
COMMIT → ignore
ROLLBACK → ignore
```

即可通过第一阶段。

---

## 六、Phase 5：sysbench 隐藏依赖 SQL

### 查询 charset

```sql
SHOW VARIABLES LIKE 'character_set_client'
```

必须返回：`utf8`

### 查询 isolation level

```sql
SHOW VARIABLES LIKE 'tx_isolation'
```

返回：`REPEATABLE-READ` 即可。

### 查询 autocommit

```sql
SELECT @@autocommit
```

必须支持 session variable，最简单实现：always return 1。

---

## 七、最小 Catalog 设计（推荐实现）

SQLRustGo catalog 至少需要：

```
catalog
  ├── database
  │    ├── tables
  │    │    ├── columns
  │    │    └── indexes
```

Rust struct 推荐：

```rust
struct Table {
    name: String,
    columns: Vec<Column>,
    rows: Vec<Row>,
}
```

第一阶段够用。

---

## 八、Executor 最小实现模型（推荐）

推荐 executor pipeline：

```
parser → logical plan → simple interpreter executor
```

例如：

```
SelectPlan
InsertPlan
UpdatePlan
DeletePlan
```

执行：

```rust
match plan {
  Select => exec_select()
}
```

即可。

---

## 九、sysbench 支持能力优先级路线图（建议顺序）

| Step | SQL 能力                     | 说明                              |
| ---- | --------------------------- | --------------------------------- |
| 1    | `SELECT 1`                  | 基础连接验证                       |
| 2    | `SHOW VARIABLES`            | sysbench 连接必需                  |
| 3    | `CREATE TABLE`              | prepare 阶段必需                   |
| 4    | `INSERT`                    | prepare 阶段必需                   |
| 5    | `SELECT WHERE id=?`         | run 阶段必需                       |
| 6    | `UPDATE`                    | run 阶段必需                       |
| 7    | `DELETE`                    | run 阶段必需                       |
| 8    | `BEGIN / COMMIT`            | 事务支持                           |
| 9    | `CREATE INDEX metadata-only`| prepare 阶段索引                   |

完成后 `sysbench prepare` 即可成功。

---

## 十、真正跑通 sysbench 的成功标志

最终目标输出：

```
Preparing table 'sbtest1'...
Inserting 10000 records...
Creating index...
```

然后：

```
Running the test with following options:
```

最后：

```
SQL statistics:
queries performed:
```

说明 SQLRustGo 已成为可运行 OLTP benchmark 的数据库内核雏形。

---

**创建日期**: 2026-04-19
**来源**: ChatGPT sysbench OLTP 最小支持矩阵建议

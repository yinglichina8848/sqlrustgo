# v2.9.0 迁移指南

> **版本**: v2.9.0
> **从**: v2.8.0
> **代号**: Enterprise Resilience
> **日期**: 2026-05-05

---

## 一、升级路径

### 兼容性说明

v2.9.0 是 **Alpha → Beta 过渡**版本，与 v2.8.0 相比：

- ✅ 向后兼容：大多数 v2.8.0 SQL 语句无需修改即可运行
- ⚠️ 新增分布式功能需要额外配置
- ⚠️ 性能目标 (≥10K QPS) 尚未达成

---

## 二、主要变更

### 2.1 版本标识

| 变更项 | v2.8.0 | v2.9.0 |
|--------|--------|--------|
| 版本号 | v2.8.0 GA | v2.9.0 Alpha → Beta |
| 代号 | Production+Distributed+Secure | Enterprise Resilience |
| 状态 | GA | Alpha → Beta 过渡中 |

### 2.2 新增功能 (v2.9.0)

#### D 系列 - 分布式 (新增)

| 功能 | 说明 | 是否需要配置 |
|------|------|--------------|
| D-01 Semi-sync 复制 | 半同步复制确保数据同步 | 需要 |
| D-02 MTS 并行复制 | 多线程从库复制 | 需要 |
| D-03 Multi-source 复制 | 多主源复制 | 需要 |
| D-04 XA 事务 | 两阶段提交分布式事务 | 需要 |

#### C 系列 - SQL 兼容性 (增强)

| 功能 | 说明 | 兼容性 |
|------|------|--------|
| C-01 SQL Corpus | 96.9% 通过率 (原 85.4%) | ✅ 兼容 |
| C-02 CTE/WITH | 递归 CTE 支持 | ✅ 兼容 |
| C-03 JSON 操作 | JSON 提取和路径 | ✅ 兼容 |
| C-04 窗口函数 | ROW_NUMBER, RANK, DENSE_RANK | ✅ 兼容 |
| C-05 COUNT(DISTINCT) | 新增支持 (PR #256) | ✅ 兼容 |
| C-06 CASE/WHEN | 完整条件表达式 | ✅ 兼容 |

### 2.3 DDL/DML 命令新增

| 命令 | 状态 | 说明 |
|------|------|------|
| CREATE TABLE IF NOT EXISTS | ✅ | 新增 |
| DROP TABLE IF EXISTS | ✅ | 新增 |
| INSERT ON DUPLICATE KEY UPDATE | ✅ | 新增 |
| ALTER TABLE DROP/MODIFY COLUMN | ✅ | 新增 |
| CREATE VIEW / DROP VIEW | ✅ | 新增 |
| CREATE UNIQUE INDEX | ✅ | 新增 |
| DROP INDEX IF EXISTS | ✅ | 新增 |
| SHOW DATABASES / SHOW CREATE TABLE | ✅ | 新增 |

---

## 三、分布式功能配置

### 3.1 Semi-sync 复制配置

```toml
# config.toml
[replication]
type = "semi-sync"
source = "192.168.0.100:3306"
ack_timeout = 10  # 秒
```

### 3.2 MTS 并行复制配置

```toml
[replication]
type = "mts"
source = "192.168.0.100:3306"
mts_workers = 4
```

### 3.3 XA 事务配置

```toml
[transaction]
xa_enabled = true
xa_timeout = 60  # 秒
```

---

## 四、SQL 语法变化

### 4.1 CTE/WITH (增强)

```sql
-- v2.9.0 支持递归 CTE
WITH RECURSIVE cte AS (
    SELECT 1 AS n
    UNION ALL
    SELECT n + 1 FROM cte WHERE n < 10
)
SELECT * FROM cte;
```

### 4.2 JSON 操作 (新增)

```sql
-- JSON 提取
SELECT JSON_EXTRACT(data, '$.name') FROM t;

-- JSON 路径
SELECT data->>'$.name' FROM t;
```

### 4.3 窗口函数 (增强)

```sql
-- PARTITION BY 支持
SELECT
    department,
    name,
    RANK() OVER (PARTITION BY department ORDER BY salary DESC) as rank
FROM employees;
```

---

## 五、API 变更

### 5.1 Rust API

暂无破坏性 API 变更。v2.9.0 新增 API：

```rust
// 新增分布式 API
pub mod distributed {
    pub fn start_semi_sync_replication(config: &ReplicationConfig) -> Result<(), Error>;
    pub fn start_mts_replication(config: &MtsConfig) -> Result<(), Error>;
    pub fn start_xa_transaction(id: &str) -> Result<XaHandle, Error>;
}

// 新增 JSON API
pub mod json {
    pub fn json_extract(data: &Value, path: &str) -> Result<Value, Error>;
    pub fn json_path(data: &Value, path: &str) -> Result<String, Error>;
}
```

---

## 六、配置变更

### 6.1 新增配置项

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `replication.type` | string | "async" | 复制类型: async, semi-sync, mts |
| `replication.source` | string | "" | 主节点地址 |
| `replication.ack_timeout` | int | 10 | Semi-sync ACK 超时 (秒) |
| `mts_workers` | int | 4 | MTS 工作线程数 |
| `xa_enabled` | bool | false | 是否启用 XA 事务 |
| `xa_timeout` | int | 60 | XA 事务超时 (秒) |

### 6.2 弃用配置项

暂无。

---

## 七、升级步骤

### 7.1 从 v2.8.0 升级

1. **备份数据**
   ```bash
   # 备份数据库目录
   cp -r ./data ./data.backup
   ```

2. **停止服务**
   ```bash
   # 停止所有 sqlrustgo 实例
   pkill -f sqlrustgo
   ```

3. **更新代码**
   ```bash
   git fetch origin
   git checkout develop/v2.9.0
   cargo update
   ```

4. **构建**
   ```bash
   cargo build --release --all-features
   ```

5. **验证**
   ```bash
   cargo test --all-features
   ```

6. **启动服务**
   ```bash
   cargo run --release --bin sqlrustgo-mysql-server
   ```

### 7.2 分布式功能升级

如需启用分布式功能：

1. **配置主节点**
   ```bash
   sqlrustgo-server --role primary --port 3306
   ```

2. **配置从节点**
   ```bash
   sqlrustgo-server --role replica --port 3307 --source 127.0.0.1:3306
   ```

### 7.3 回滚步骤

如遇问题，回滚到 v2.8.0：

1. 停止 v2.9.0 服务
2. 恢复 v2.8.0 二进制
3. 从备份恢复数据：`cp -r ./data.backup ./data`
4. 报告问题：http://192.168.0.252:3000/openclaw/sqlrustgo/issues

---

## 八、行为变更

### 8.1 窗口函数

| 函数 | v2.8.0 | v2.9.0 |
|------|--------|--------|
| ROW_NUMBER | ✅ | ✅ |
| RANK | ✅ | ✅ |
| DENSE_RANK | ✅ | ✅ |
| PARTITION BY | ❌ | ✅ 新增 |
| LEAD/LAG | ⚠️ 部分 | ⚠️ 部分 |

### 8.2 聚合函数

| 函数 | v2.8.0 | v2.9.0 |
|------|--------|--------|
| COUNT(*) | ✅ | ✅ |
| COUNT(DISTINCT) | ❌ | ✅ 新增 (PR #256) |
| SUM/AVG | ✅ | ✅ |
| MIN/MAX | ✅ | ✅ |

---

## 九、已知问题

| Issue | 描述 | 状态 | 解决方式 |
|-------|------|------|----------|
| E-08 | 性能优化 (≥10K QPS) | ⚠️ 进行中 | 下一版本继续 |
| TPC-H | 9/22 → 18/22 查询 | ⚠️ 进行中 | 持续开发中 |

---

## 十、获取帮助

- **文档**: http://192.168.0.252:3000/openclaw/sqlrustgo/-/tree/develop/v2.9.0/docs
- **Issue**: http://192.168.0.252:3000/openclaw/sqlrustgo/issues
- **协作 Issue**: http://192.168.0.252:3000/openclaw/sqlrustgo/issues/11

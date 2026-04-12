# SQLRustGo vs PostgreSQL 对比测试指南

> **版本**: v1.6.1
> **状态**: 需手动执行

---

## 前提条件

### 安装 PostgreSQL

```bash
# macOS
brew install postgresql@15
brew services start postgresql@15

# Ubuntu
sudo apt install postgresql-15
sudo systemctl start postgresql
```

### 创建测试数据库

```bash
# 登录 PostgreSQL
psql -U postgres

# 创建测试数据库
CREATE DATABASE bench_test;

# 退出
\q
```

---

## 运行对比测试

### 方式一: 使用对比脚本

```bash
# 完整对比 (包含 PostgreSQL)
bash scripts/full_benchmark_compare.sh
```

### 方式二: 手动测试

```bash
# 1. 启动 PostgreSQL
pg_ctl -D /usr/local/var/postgresql@15 start

# 2. 创建测试表
psql -U postgres -d bench_test -c "
CREATE TABLE lineitem (
    l_orderkey INTEGER,
    l_partkey INTEGER,
    l_suppkey INTEGER,
    l_quantity REAL,
    l_extendedprice REAL,
    l_discount REAL,
    l_tax REAL,
    l_returnflag TEXT,
    l_shipdate INTEGER
);"

# 3. 插入测试数据
# ... (参考对比脚本)

# 4. 测试查询
time psql -U postgres -d bench_test -c "SELECT l_returnflag, SUM(l_quantity) FROM lineitem WHERE l_returnflag = 'N' GROUP BY l_returnflag;"
```

---

## 预期结果格式

```
| 系统      | Q1 (ms) | Q6 (ms) |
|-----------|----------|----------|
| SQLRustGo | 5.36     | 8.08     |
| SQLite    | 16       | 17       |
| PostgreSQL| [待测试] | [待测试] |
```

---

## 注意事项

1. **配置优化**: PostgreSQL 默认配置可能不是最优的，建议调整:
   ```sql
   SET shared_buffers = '1GB';
   SET effective_cache_size = '3GB';
   SET maintenance_work_mem = '256MB';
   SET random_page_cost = 1.1;
   ```

2. **数据一致性**: 确保 SQLRustGo 和 PostgreSQL 使用相同的数据

3. **预热**: 首次查询会被缓存，确保进行预热查询

4. **多次运行**: 建议运行多次取平均值

---

## 结果解读

### 合理范围

| 对比 | 预期 |
|------|------|
| SQLRustGo vs SQLite | SQLRustGo 快 1-5x |
| SQLRustGo vs PostgreSQL | 接近或略慢 (10% 内) |

### 异常情况

| 现象 | 原因 |
|------|------|
| PostgreSQL 比 SQLRustGo 快 10x+ | 配置问题或未预热 |
| 结果差异大 | 数据不一致 |

---

*指南版本: v1.6.1*
*更新日期: 2026-03-20*

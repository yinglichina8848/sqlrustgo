# SQLRustGo v2.0 迁移指南

> **版本**: 2.0
> **日期**: 2026-03-26
> **状态**: 规划中

---

## 1. 迁移概述

本文档帮助用户从 MySQL 迁移到 SQLRustGo。

### 1.1 迁移路径

```
MySQL → SQLRustGo
     ↓
  数据导出 (mysqldump)
     ↓
  格式转换 (可选)
     ↓
  数据导入 (sqlrustgo-cli)
     ↓
  应用适配
```

---

## 2. 兼容性检查

### 2.1 支持的数据类型

| MySQL | SQLRustGo | 兼容性 |
|-------|------------|--------|
| INT | INTEGER | ✅ 完全兼容 |
| BIGINT | BIGINT | ✅ 完全兼容 |
| FLOAT | FLOAT | ✅ 完全兼容 |
| DOUBLE | DOUBLE | ✅ 完全兼容 |
| DECIMAL | DECIMAL | ✅ 完全兼容 |
| CHAR | CHAR | ✅ 完全兼容 |
| VARCHAR | VARCHAR | ✅ 完全兼容 |
| TEXT | TEXT | ✅ 完全兼容 |
| DATE | DATE | ✅ 完全兼容 |
| TIME | TIME | ✅ 完全兼容 |
| DATETIME | DATETIME | ✅ 完全兼容 |
| TIMESTAMP | TIMESTAMP | ✅ 范围差异 |
| JSON | JSON | ✅ 部分支持 |
| BLOB | BLOB | ✅ 完全兼容 |

### 2.2 支持的 SQL 语法

| 功能 | MySQL | SQLRustGo | 状态 |
|------|-------|-----------|------|
| SELECT | ✅ | ✅ | 兼容 |
| INSERT | ✅ | ✅ | 兼容 |
| UPDATE | ✅ | ✅ | 兼容 |
| DELETE | ✅ | ✅ | 兼容 |
| JOIN | ✅ | ✅ | 兼容 |
| 子查询 | ✅ | ✅ | 兼容 |
| 聚合函数 | ✅ | ✅ | 兼容 |
| 窗口函数 | ✅ | 部分 | v2.0 完善中 |
| 存储过程 | ✅ | 有限 | v2.0 完善中 |
| 触发器 | ✅ | ❌ | 不支持 |
| 事件 | ✅ | ❌ | 不支持 |

### 2.3 不支持的功能

| 功能 | 说明 | 替代方案 |
|------|------|----------|
| 触发器 | 不支持 | 应用层实现 |
| 事件 | 不支持 | 外部调度 |
| 分区表 | 不支持 | 手动分表 |
| 存储引擎 | 仅 Memory/File | 选择合适的引擎 |

---

## 3. 数据迁移

### 3.1 导出 MySQL 数据

```bash
# 导出表结构
mysqldump -d -u root -p db_name > schema.sql

# 导出数据
mysqldump -t -u root -p db_name > data.sql

# 导出特定表
mysqldump -u root -p db_name table1 table2 > tables.sql
```

### 3.2 格式转换

```bash
# 使用转换工具
mysql-to-sqlrustgo convert \
    --input schema.sql \
    --output schema_converted.sql \
    --encoding utf8mb4
```

### 3.3 导入 SQLRustGo

```bash
# 使用 CLI 导入
sqlrustgo-cli -u root < schema.sql
sqlrustgo-cli -u root < data.sql

# 导入大文件
sqlrustgo-cli -u root -e "source /path/to/large_file.sql"
```

---

## 4. 应用适配

### 4.1 连接参数

```diff
# MySQL
- host=localhost
- port=3306
- database=myapp

# SQLRustGo
+ host=localhost
+ port=3306
+ database=myapp
```

### 4.2 JDBC 连接

```java
// MySQL
String url = "jdbc:mysql://localhost:3306/myapp";

// SQLRustGo (兼容 MySQL 协议)
String url = "jdbc:mysql://localhost:3306/myapp";
```

### 4.3 Python (mysql-connector)

```python
# MySQL
conn = mysql.connector.connect(
    host="localhost",
    port=3306,
    database="myapp",
    user="root",
    password="password"
)

# SQLRustGo (兼容)
conn = mysql.connector.connect(
    host="localhost",
    port=3306,
    database="myapp",
    user="root",
    password="password"
)
```

---

## 5. 迁移验证

### 5.1 数据验证

```sql
-- 对比行数
SELECT 'mysql' as src, COUNT(*) as cnt FROM users
UNION ALL
SELECT 'sqlrustgo' as src, COUNT(*) as cnt FROM users;

-- 对比校验和
SELECT SUM(id) FROM users;
SELECT COUNT(DISTINCT email) FROM users;
```

### 5.2 性能验证

```bash
# 执行相同查询对比
time mysql -e "SELECT COUNT(*) FROM orders WHERE status = 'pending'"
time sqlrustgo -e "SELECT COUNT(*) FROM orders WHERE status = 'pending'"
```

### 5.3 功能验证

```bash
# 运行应用测试
pytest tests/
# 或
npm test
```

---

## 6. 常见问题

### 6.1 字符集问题

**问题**: 乱码

**解决方案**:

```sql
-- 设置字符集
SET NAMES 'utf8mb4';
CREATE DATABASE db_name CHARACTER SET utf8mb4;
```

### 6.2 类型不匹配

**问题**: 字段类型不兼容

**解决方案**:

```sql
-- 修改字段类型
ALTER TABLE t MODIFY col VARCHAR(255) NOT NULL;
```

### 6.3 语法差异

**问题**: 语法不支持

**解决方案**:

```diff
- -- MySQL 特有语法
- SELECT * FROM t1 PROCEDURE ANALYSE();

+ -- SQLRustGo 等效
+ -- 无对应功能，使用应用层处理
```

### 6.4 性能问题

**问题**: 查询慢

**解决方案**:

1. 添加索引
```sql
CREATE INDEX idx_status ON orders(status);
```

2. 使用 EXPLAIN 分析
```sql
EXPLAIN SELECT * FROM orders WHERE status = 'pending';
```

---

## 7. 迁移检查清单

### 7.1 迁移前

- [ ] 评估 SQL 兼容性
- [ ] 识别不兼容功能
- [ ] 准备数据转换方案
- [ ] 备份源数据库
- [ ] 测试环境验证

### 7.2 迁移中

- [ ] 导出数据
- [ ] 转换格式（如需要）
- [ ] 导入数据
- [ ] 验证数据完整性

### 7.3 迁移后

- [ ] 功能测试
- [ ] 性能测试
- [ ] 监控配置
- [ ] 制定回滚方案

---

## 8. 回滚方案

### 8.1 回滚条件

- 数据丢失或损坏
- 性能显著下降
- 关键功能不可用

### 8.2 回滚步骤

```bash
# 1. 停止应用
systemctl stop myapp

# 2. 切换回 MySQL
# 修改应用配置指向 MySQL

# 3. 恢复数据
mysql -u root -p db_name < backup.sql

# 4. 重新启动应用
systemctl start myapp
```

---

**文档版本历史**

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-03-26 | 初始版本 |

**状态**: ✅ 规划完成

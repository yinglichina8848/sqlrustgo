# Mysqldump 导入工具

## 概述

SQLRustGo 提供 `mysqldump` 兼容的 SQL 文件导入工具，支持导入 MySQL/mysqldump 格式的 SQL 转储文件。

## 功能特性

- ✅ 解析 `CREATE TABLE` / `DROP TABLE` 语句
- ✅ 支持 `INSERT ... VALUES` 单行和多行语法
- ✅ 支持 `SET FOREIGN_KEY_CHECKS` 等 SET 语句
- ✅ 支持 `USE database` 语句
- ✅ 支持 `LOCK TABLES` / `UNLOCK TABLES`
- ✅ 支持事务语句 `BEGIN` / `COMMIT` / `ROLLBACK`
- ✅ 自动跳过 SQL 注释 (`--` 和 `/* */`)
- ✅ 流式处理大文件（不占用大量内存）

## 使用方法

### 基本用法

```bash
sqlrustgo-tools import -f dump.sql
```

### 详细输出

```bash
sqlrustgo-tools import -f dump.sql --verbose
```

## 支持的 SQL 语法

### CREATE TABLE

```sql
CREATE TABLE users (
  id INT NOT NULL AUTO_INCREMENT,
  name VARCHAR(255),
  email VARCHAR(255) UNIQUE,
  PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
```

### INSERT 语句

```sql
-- 单行 INSERT
INSERT INTO users (id, name) VALUES (1, 'Alice');

-- 多行 INSERT (推荐，效率更高)
INSERT INTO users (id, name) VALUES 
  (1, 'Alice'),
  (2, 'Bob'),
  (3, 'Charlie');
```

### SET 语句

```sql
SET FOREIGN_KEY_CHECKS = 0;
SET SQL_MODE = 'NO_AUTO_VALUE_ON_ZERO';
```

### 完整示例

```sql
-- MySQL dump 10.13
-- Host: localhost    Database: mydb

SET FOREIGN_KEY_CHECKS = 0;
SET SQL_MODE = 'NO_AUTO_VALUE_ON_ZERO';

CREATE TABLE `users` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

INSERT INTO `users` (`id`, `name`) VALUES
(1, 'Alice'),
(2, 'Bob'),
(3, 'Charlie');

SET FOREIGN_KEY_CHECKS = 1;
```

## 测试

运行导入工具测试：

```bash
cargo test -p sqlrustgo-tools
```

测试结果：

```
test result: ok. 36 passed; 0 failed
```

## 限制

- 断点续传功能仍在开发中
- 某些 MySQL 特有语法可能不完全兼容

## 相关 Issue

- Issue #1016: mysqldump 兼容导入工具

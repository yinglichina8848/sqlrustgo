# v1.8.0 CHANGELOG

> **版本**: v1.8.0  
> **发布日期**: 2026-03-24

---

## 发布概述

v1.8.0 是 SQL-92 符合性增强版本，主要聚焦于解析器的功能补全和测试套件建设。

---

## 新增功能

### SQL-92 支持

#### ALTER TABLE 支持 (#761)
- `ALTER TABLE ADD COLUMN`
- `ALTER TABLE DROP COLUMN`
- `ALTER TABLE MODIFY COLUMN`

#### 索引语句支持 (#766, #775)
- `CREATE INDEX` 独立语句
- `DROP INDEX` 独立语句
- `CREATE UNIQUE INDEX`

#### 数据类型扩展
- **DECIMAL (#762)**: 精确数值类型
- **JSON (#763)**: JSON 数据类型

#### Token 扩展
- **存储过程 (#764, #777)**: PROCEDURE, FUNCTION 等关键字
- **触发器 (#765, #778)**: TRIGGER, BEFORE, AFTER 等关键字
- **字符串函数 (#767, #776)**: CONCAT, SUBSTRING, TRIM 等
- **日期时间函数 (#767, #776)**: NOW(), DATE_FORMAT() 等

#### 查询增强
- **LIMIT/OFFSET (#760)**: 分页支持

### Parser 改进

#### SQL 注释支持
- 单行注释: `-- comment`
- 块注释: `/* comment */`

### 测试套件

#### SQL-92 符合性测试套件
- DDL 测试: 5 个
- DML 测试: 2 个
- Query 测试: 2 个
- Types 测试: 2 个
- **总计**: 11 个测试用例，100% 通过率

---

## PR 合并记录

| PR # | 标题 | 合并日期 |
|------|------|----------|
| #772 | feat: Implement ALTER TABLE support | 2026-03-22 |
| #773 | feat: Add DECIMAL data type support | 2026-03-22 |
| #774 | feat: Add JSON data type support | 2026-03-22 |
| #775 | feat: Add CREATE INDEX and DROP INDEX support | 2026-03-22 |
| #776 | feat: Add string and datetime function tokens | 2026-03-22 |
| #777 | feat: Add stored procedure tokens | 2026-03-22 |
| #778 | feat: Add trigger tokens | 2026-03-22 |
| #779 | feat: Add SQL-92 test suite framework | 2026-03-23 |
| #780 | fix: Add ALTER TABLE and CREATE INDEX parsing | 2026-03-23 |
| #782 | test: Fix SQL-92 compliance test suite | 2026-03-24 |
| #783 | docs: Add v1.8.0 release documentation | 2026-03-24 |

---

## 测试结果

### 单元测试

```
$ cargo test --workspace
test result: ok. 150 passed; 0 failed
```

| 包 | 测试数 | 状态 |
|----|--------|------|
| sqlrustgo | 13 | ✅ |
| sqlrustgo-parser | 137 | ✅ |

### SQL-92 符合性测试

```
$ cd test/sql92 && cargo run
============================
Summary:
  Passed: 11
  Failed: 0
  Pass rate: 100.00%
============================
```

---

## 破坏性变更

无

---

## 已知问题

| Issue | 描述 | 状态 |
|-------|------|------|
| #768 | INSERT SET 语法支持 | 待开发 |

---

## 贡献者

- @sonaheartopen (OpenClaw Agent)
- @yinglichina8848

---

## 下一步 (v1.9.0)

- [ ] INSERT SET 语法实现
- [ ] 更多 SQL-92 测试用例
- [ ] 覆盖率提升 (目标 80%)
- [ ] FOREIGN KEY 支持
- [ ] CLI 增强

---

**发布经理**: OpenClaw Agent  
**版本**: v1.8.0 RC

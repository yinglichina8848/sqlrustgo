# TPC-H Benchmark 测试报告

**Issue**: #1231 TPC-H 合规性测试缺失
**日期**: 2026-04-04
**状态**: ✅ 测试基础设施完成

---

## 📊 测试概览

### 测试文件

| 文件 | 测试数 | 数据规模 | 状态 |
|------|--------|---------|------|
| `tpch_test.rs` | 11 | 小规模 (内存) | ✅ 通过 |
| `tpch_compliance_test.rs` | 1 | SF=0.1 (600K 行) | ✅ 通过 |
| `tpch_sf03_test.rs` | 2 | SF=0.1 (600K 行) | ✅ 通过 |
| `tpch_sf1_test.rs` | 2 | SF=1 (6M 行) | ⚠️ 忽略 (需要 16GB+ RAM) |
| `tpch_index_test.rs` | 1 | 外部数据 | ✅ 通过 |
| `tpch_text_index_test.rs` | 1 | 外部数据 | ✅ 通过 |
| `tpch_qtest.rs` | 3 | 小规模 | ✅ 通过 |
| `tpch_full_test.rs` | 34+ | 外部数据 | ⚠️ 需验证 |
| `tpch_benchmark.rs` | 12+ | 外部数据 | ⚠️ 需验证 |
| `tpch_comparison_test.rs` | 5+ | 外部数据 | ⚠️ 需验证 |
| **总计** | **70+** | | |

---

## ✅ 已完成功能

### SQL 功能 (Issue #1231)

| 功能 | 状态 | 说明 |
|------|------|------|
| BETWEEN 操作符 | ✅ | `expr BETWEEN low AND high` |
| DATE 字面量 | ✅ | `DATE 'yyyy-mm-dd'` 语法 |
| IN value list | ✅ | `expr IN (value1, value2, ...)` |
| CASE WHEN | ✅ | `CASE WHEN ... THEN ... END` |
| COUNT(DISTINCT) | ✅ | 支持 DISTINCT 修饰符 |
| LIKE % 模式 | ✅ | 支持末尾 % 通配符 |
| NOT LIKE | ✅ | 逻辑NOT + LIKE |
| GROUP BY 排序 | ✅ | 结果按 GROUP BY 列排序 |
| SUM 聚合空集返回 NULL | ✅ | 符合 SQL 标准 |

### 索引系统

| 功能 | 状态 | 性能提升 |
|------|------|---------|
| INTEGER 列索引 | ✅ | 340-700x |
| TEXT 列索引 (=) | ✅ | 有效 |
| B+Tree 重复 key | ✅ | 支持 |
| HashJoin 键提取 | ✅ | 正确 |

---

## 📈 TPC-H SF=0.1 测试结果

### 数据规模

| 表 | 行数 |
|---|------|
| lineitem | 600,572 |
| orders | 150,000 |
| customer | 15,000 |
| part | 20,000 |
| partsupp | 80,000 |
| supplier | 1,000 |
| nation | 25 |
| region | 5 |
| **总计** | **866,602** |

### 合规性测试 (Q1)

```
SQLite returned 3 rows
SQLRustGo returned 3 rows
  [Text("N"), Integer(1167065)]
  [Text("R"), Integer(3785523)]
  [Text("A"), Integer(3774200)]
test test_tpch_q1_simple ... ok (4.94s)
```

### 性能测试 (SF=0.1)

| 测试 | 指标 | 结果 |
|------|------|------|
| COUNT(*) | 单次 | 847.7ms |
| COUNT(*) | 10次平均 | 625.1ms |
| SUM(filtered) | 单次 | 233.7ms |
| SUM(filtered) | 10次平均 | 172.9ms |
| 数据加载 | 总计 | ~5s |

---

## 🔧 测试运行方式

### 本地运行

```bash
# 基础 TPC-H 测试
cargo test --test tpch_test --release

# 合规性测试 (SF=0.1)
cargo test --test tpch_compliance_test --release

# SF=0.1 性能测试
cargo test --test tpch_sf03_test --release -- --nocapture

# 索引测试
cargo test --test tpch_index_test --release -- --nocapture

# 所有 TPC-H 测试
cargo test --test tpch --release
```

### 数据文件位置

- **SF=0.1**: `data/tpch-sf01/*.tbl` (未提交，需手动生成)
- **SF=0.3**: `data/tpch-sf03/*.tbl` (未提交)
- **SQLite 参考**: `data/tpch-sf01/tpch.db` (114MB，已提交)

### 生成测试数据

```bash
# 克隆 dbgen
cd /tmp && git clone https://github.com/electrum/tpch-dbgen.git

# 生成 SF=0.1 数据
cd /tmp/tpch-dbgen && ./dbgen -s 0.1 -f

# 复制到项目
cp *.tbl /path/to/sqlrustgo/data/tpch-sf01/
```

---

## ⚠️ 已知限制

### 1. 内存需求

- **SF=1**: 需要 ~5GB RAM，测试标记为 `#[ignore]`
- **SF=0.1**: 需要 ~500MB RAM，正常运行

### 2. 数据文件

- `.tbl` 数据文件未提交到仓库 (~104MB)
- 本地测试需要手动生成或从第三方获取

### 3. JOIN 行为

- 复杂 JOIN 查询 (Q7, Q18) 结果可能与 SQLite 不同
- 需要进一步调试 JOIN 实现

### 4. 浮点精度

- SUM 聚合可能存在浮点精度差异
- 使用 Decimal 类型缓解此问题

---

## 📋 Issue 状态

| Issue | 标题 | 状态 |
|-------|------|------|
| #1231 | TPC-H 合规性测试缺失 | ✅ 基础设施完成 |
| #1274 | TPC-H 性能问题 | 🔄 开发中 |
| #1271 | TPC-H SF=10 开发计划 | ⏳ 待做 |

---

## 🎯 下一步

1. **完善 Q2-Q22 合规测试** - 添加更多查询对比
2. **优化 JOIN 性能** - 改进 HashJoin 实现
3. **浮点精度改进** - 完善 Decimal 类型支持
4. **添加 PostgreSQL 对比** - 使用标准数据库验证

---

## 相关 PR

| PR | 描述 | 状态 |
|----|------|------|
| #1279 | WHERE 过滤器回退逻辑修复 | ✅ 已合并 |
| #1280 | performance_test 编译错误修复 | ✅ 已合并 |
| #1281 | 更新测试使用 SF=0.1 数据集 | ✅ 已合并 |

---

*报告生成时间: 2026-04-04*
# SQLRustGo — AI 编程行为准则

基于 Andrej Karpathy 的 karpathy-guidelines，针对 SQLRustGo 项目裁剪。

---

## 原则一：Think Before Coding

**模糊指令必须停下来问清楚，不猜。**

### SQLRustGo 场景

用户说"让 JOIN 更快"，可能指：
1. CBO join order 优化（耗时 2h）
2. Hash join → merge join 替换（耗时 4h）
3. SIMD 向量化（耗时 8h+）

遇到以下情况必须先问：
- 意图不明确的需求（"优化查询性能"）
- 存在多种合理解读的技术选型
- 涉及核心数据结构的改动
- 用户提到的关键词在项目中不存在

### 验证方式

每一项改动之前，回答：
> 这个改动基于什么假设？这个假设验证过吗？

---

## 原则二：Simplicity First

**只写解决当前问题所需的最少代码，不做预测性设计。**

### SQLRustGo 场景

用户要求"加一个聚合函数"：

- ❌ 引入 `AggregateFunction` trait + `RegisteredAggregate` 注册表 + YAML 配置
- ✅ 先写一个 `fn sum_int64(rows: &[Value]) -> Value`

判断标准：
```
如果一个高级工程师说这过度设计了，就重写到最小可用版本。
```

### 不做

- 单次使用的代码不抽象
- 没有人要求的功能不加
- 不可能出错的路径不写防御代码
- 不预留"以后可能用到"的配置

---

## 原则三：Surgical Changes

**只改必须改的，不碰原本就有的东西。改完的每一行都能回答"这是哪个需求导致的"。**

### SQLRustGo 场景

用户说"修复 RIGHT JOIN 的 test_right_join_basic"：

- ✅ 只改 `hash_inner_join_rows` 中的匹配逻辑
- ❌ 不要顺手改 `left_col_count`/`right_col_count` 的变量命名
- ❌ 不要顺便格式化其他文件
- ❌ 不要更新其他不相关的测试

判断标准：
```
git diff 里的每一行改动，都能说出对应的用户请求。
说不出来的不改。
```

### 格式纪律

- 引号风格不变（项目中已用单引号保持一致）
- 注释不变
- 不加 docstring（除非用户明确要求）
- 不改其他文件的格式

---

## 原则四：Goal-Driven Execution

**用"做什么 + 怎么验证"替代模糊指令。每个步骤完成后有明确的核对清单。**

### 标准格式

```
需求：[用户描述的问题或需求]

计划：
1. [具体步骤] → 验证：[成功标准]
2. [具体步骤] → 验证：[成功标准]

当前状态：[相关数据/错误信息]
```

### SQLRustGo 场景

用户说"JOIN 性能差"：

```
需求：某复杂查询的 JOIN 执行时间超过预期

计划：
1. 写测试：构造 3 表 JOIN 查询，重现代谢问题
   验证：cargo test 通过（复现性能问题）

2. 分析：CBO 是否输出最优 join order
   验证：日志中打印 join order = ["t1", "t2", "t3"]

3. 实现：修复 CBO cost model 中的 cardinality 估算
   验证：cargo test --package sqlrustgo-executor --test join_tests 9/9 通过

4. 回归检查：cargo test --lib 全部通过
   验证：测试输出 ok. X passed

当前 JOIN 测试状态：[数据]
```

---

## SQLRustGo 项目规范

### 编译与测试

```bash
# 快速验证
cargo build 2>&1 | grep error

# Join 执行器测试（核心）
cargo test --package sqlrustgo-executor --test test_join
cargo test --package sqlrustgo-executor --test join_tests

# 全量测试（需要时间）
cargo test --lib

# TPC-H 基准
cargo run --bin bench-cli -- tpch bench --queries Q1,Q3,Q6,Q10 --iterations 3 --sf 0.1
```

### 文件修改原则

| 场景 | 处理方式 |
|------|----------|
| 只改 bug | Surgical Changes，只改相关函数 |
| 新功能 | 先写测试，Simplicity First |
| 重构 | 明确说明影响范围，每步可回滚 |
| 跨 crate 改动 | 先确认 API 兼容性 |

### Git 远端

- 远端名：`gitea`（不是 `origin`）
- URL：`http://192.168.0.252:3000/openclaw/sqlrustgo.git`
- 当前分支：`feature/multi-table-join-planner`

### 测试数据

- TPC-H SF=0.1: `~/sqlrustgo/data/tpch-sf01/`
- GMP SOP: `~/gmp-qa/gmp.db` (1527 docs)

---

## 何时不适用这些原则

以下情况可以直接执行，不需要停下来问：

- `cargo test` 失败后的直接修复（原因已知）
- 简单的变量重命名
- 格式调整（仅当明确要求时）
- 明确的一次性任务（"把这个文件移到 src/ 下"）

当你不确定时，用 Principle 一（Think Before Coding）—— 问比猜更安全。

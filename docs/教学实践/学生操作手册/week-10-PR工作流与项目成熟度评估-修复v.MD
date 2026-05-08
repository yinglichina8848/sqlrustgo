# 第10周：Harness治理实战 —— DELETE/UPDATE性能优化

> 实验时间：2学时
> 实验类型：设计性+验证性
> 核心理念：**不是"写代码"，而是"让代码在约束下走向正确"**

---

## 一、实验目标

- [ ] 理解 Harness 治理的三层模型（提示词-上下文-Harness）
- [ ] 能够运行 QPS 基准测试并分析性能瓶颈
- [ ] 能够使用 AI 辅助定位性能问题根因
- [ ] 能够模拟 Gate 检查流程（BP1 → BP2 拦截 → 修复 → 通过）
- [ ] 能够绘制 Harness 治理闭环图

---

## 二、实验背景：一个真实的性能危机

### SQLRustGo v2.9.0 的性能基线

| 操作 | v2.9.0 QPS | v3.0.0 目标 | E-09 地板 |
|------|-----------|------------|----------|
| **DELETE** | **~206** | ≥10,000 | ≥10,000 |
| **UPDATE** | **~950** | ≥10,000 | ≥10,000 |
| INSERT | ~2,000 | ≥5,000 | — |
| SELECT | ~2,000 | ≥20,000 | — |

```
DELETE 只有 206 QPS —— 这是什么概念？
  每秒只能删除 206 行数据。
  如果有一个 10,000 行的表需要清空，
  要等 48 秒！这不可接受。

E-09 硬性地板：DELETE/UPDATE 必须 ≥ 10,000 QPS。
达不到 → 不能发布。
```

### v3.0.0 优化后的结果

| 操作 | v2.9.0 | v3.0.0 实际 | 提升倍数 |
|------|--------|------------|---------|
| DELETE | 206 | **64,896** | **315x** |
| UPDATE | 950 | **43,121** | **45x** |

> **本实验的核心问题：从 206 → 64,896，到底发生了什么？Harness 在这个过程中扮演了什么角色？**

---

## 三、实验环境

| 项目 | 要求 |
|------|------|
| Rust 工具链 | 1.75+ |
| 项目代码 | SQLRustGo（develop/v3.0.0 分支） |
| AI 工具 | Claude Code / OpenCode |

---

## 四、操作步骤

### 步骤0：切换到正确分支（5分钟）

```bash
cd ~/workspace/dev/openheart/sqlrustgo
git checkout develop/v3.0.0
git pull origin develop/v3.0.0
```

#### ✅ 检查点0：确认分支正确

```bash
git branch --show-current
# 应该输出: develop/v3.0.0
```

---

### 步骤1：运行 QPS 基准测试 —— 亲眼看"慢"（20分钟）

#### 1.1 运行 DELETE 基准测试

```bash
cargo test --test qps_benchmark_test test_qps_delete -- --ignored --nocapture
```

观察输出：
```
DELETE QPS: 10000 queries in X.XXs (XXXX.XX qps)
```

**记录你的实际 QPS 值**：`______ QPS`

#### 1.2 运行 UPDATE 基准测试

```bash
cargo test --test qps_benchmark_test test_qps_update -- --ignored --nocapture
```

**记录你的实际 QPS 值**：`______ QPS`

#### 1.3 运行 INSERT 和 SELECT 作为对照

```bash
cargo test --test qps_benchmark_test test_qps_insert -- --ignored --nocapture
cargo test --test qps_benchmark_test test_qps_simple_select -- --ignored --nocapture
```

#### 1.4 填写性能基线表

| 操作 | QPS | 与 E-09 地板 (10,000) 比较 |
|------|-----|--------------------------|
| DELETE | | ⬜ 达标 / ❌ 不达标 |
| UPDATE | | ⬜ 达标 / ❌ 不达标 |
| INSERT | | — |
| SELECT | | — |

#### ✅ 检查点1：保存基准测试截图和 QPS 数据

---

### 步骤2：分析根因 —— AI + 人工协作（25分钟）

#### 2.1 阅读基准测试代码，理解测试做了什么

打开 `tests/qps_benchmark_test.rs`，找到 `test_qps_delete` 函数：

```rust
// 关键代码段（第158-185行）
fn test_qps_delete() {
    let mut engine = create_engine();
    setup_tables(&mut engine);

    // 先插入 10000 行
    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!(
            "INSERT INTO users VALUES ({}, 'bench_{}', {})",
            i, i, 30
        ));
    }

    // 然后逐条 DELETE（测试的正是"逐条删除"的性能）
    let start = Instant::now();
    for i in 0..BENCHMARK_ITERATIONS {
        let _ = engine.execute(&format!("DELETE FROM users WHERE id = {}", i));
    }
    // ...
}
```

**思考**：这个测试为什么慢？DELETE 操作涉及哪些步骤？（提示：索引更新、WAL 写盘、MVCC 版本管理...）

#### 2.2 用 AI 辅助分析

```
提示词示例（向 AI 提问）：

"SQLRustGo 的 DELETE 操作只有 ~200 QPS，而 SELECT 有 ~400,000 QPS。
为什么 DELETE 比 SELECT 慢 2000 倍？可能的原因有哪些？

请从以下角度分析：
1. DELETE 操作在数据库内部需要做哪些额外工作（vs SELECT）？
2. 当前实现中可能有哪些性能瓶颈？
3. 给出 2-3 个可能的优化方向。"
```

#### 2.3 记录你的分析

| 可能原因 | 为什么影响 DELETE | 优化方向 |
|---------|------------------|---------|
| 1. | | |
| 2. | | |
| 3. | | |

#### ✅ 检查点2：保存 AI 分析结果和你的判断

---

### 步骤3：模拟 Gate 检查 —— 理解"约束"的作用（20分钟）

#### 3.1 场景设定

```
假设你是开发者，刚刚"优化"了 DELETE 的实现。
你提交了 PR #500。

现在 Harness Gate 系统启动——
```

#### 3.2 模拟 BP1 静态检查

在终端运行以下命令，模拟 BP1 Gate：

```bash
echo "=== BP1 静态检查 ==="
cargo build --all-features 2>&1 | tail -1
echo "---"
cargo fmt --check --all 2>&1 | tail -1
echo "---"
cargo clippy --all-features -- -D warnings 2>&1 | tail -1
```

| BP1 检查项 | 结果 | 
|-----------|------|
| 编译 | ✅ / ❌ |
| 格式 | ✅ / ❌ |
| Clippy | ✅ / ❌ |

#### 3.3 模拟 BP2 行为检查

```bash
echo "=== BP2 行为检查 ==="
cargo test --test qps_benchmark_test -- --ignored --nocapture 2>&1 | grep "QPS:"
```

**关键判断**：DELETE/UPDATE 的 QPS 是否 ≥ 10,000（E-09 硬性地板）？

| BP2 检查项 | 结果 | 是否通过 |
|-----------|------|---------|
| DELETE QPS | ______ | ≥10,000? |
| UPDATE QPS | ______ | ≥10,000? |

#### 3.4 理解"被 Gate 拦截"意味着什么

```
如果 DELETE QPS < 10,000：
  → BP2 Gate FAIL
  → PR #500 被 Blocked
  → 开发者收到通知："DELETE QPS 未达到 E-09 地板 (10,000)"
  → 必须修复后重新提交

这不是"你的代码不好"，
而是"客观指标告诉你：这个版本还不能发布"。

类比：体检报告说你血压偏高 —— 不是医生在骂你，是数据在提醒你。
```

#### ✅ 检查点3：保存 Gate 模拟结果

---

### 步骤4：绘制 Harness 治理闭环（15分钟）

#### 4.1 理解闭环结构

```
┌──────────────────────────────────────────┐
│         Harness 治理闭环                  │
├──────────────────────────────────────────┤
│                                          │
│  ① Issue: "DELETE QPS 需从 206 → 10K+"  │
│          │                               │
│          ▼                               │
│  ② AI/人写代码 → PR #500                 │
│          │                               │
│          ▼                               │
│  ③ Gate 检查                             │
│     BP1 ✅  BP2 ❌（QPS 不达标）          │
│          │                               │
│          ▼                               │
│  ④ 分析根因 → 设计优化方案               │
│     （批量提交 / 索引优化 / 缓存策略）    │
│          │                               │
│          ▼                               │
│  ⑤ 重新 PR → Gate 通过 → 合并            │
│          │                               │
│          ▼                               │
│  ⑥ 知识沉淀                              │
│     · 更新基准数据（QPS 从 206 → 64K）    │
│     · 记录 Pattern 到 GBrain             │
│     · 评估：Gate 阈值需要调整吗？          │
│          │                               │
│          └──→ 回到 ①（下一轮优化）       │
│                                          │
└──────────────────────────────────────────┘
```

#### 4.2 你的任务

在实验报告中，用 DELETE/UPDATE 优化案例画一个你自己的 Harness 治理闭环图。

要求：
- 每一步标注实际做了什么（不要泛泛而谈）
- 标注关键数据（QPS 从多少到多少）
- 标注"如果 Gate 不存在，会怎样"

#### ✅ 检查点4：保存治理闭环图

---

### 步骤5：三层模型反思（10分钟）

#### 5.1 对照 DELETE/UPDATE 案例

```
提示词层：
  "帮我优化 DELETE 性能" → 如果需求没说"目标 ≥ 10,000 QPS"
  AI 可能只是做了一些微小的调整，达不到目标
  
  → 治理改进：PR 模板必须写清楚量化目标

上下文层：
  AI 能看到：当前 DELETE 的实现代码
  AI 看不到：这个 DELETE 在真实生产环境（10万行表）下的表现
            B+Tree 索引删除的开销
            WAL 日志同步写盘的影响
  
  → 治理改进：向 AI 提供基准测试数据和性能 Profile

Harness 层：
  Gate 不管"AI 怎么实现的"，只管"QPS 达标了吗？"
  BP2: DELETE QPS 实测 → ≥10,000 ✅ / <10,000 ❌
  
  → Harness = 把"主观判断"变成"客观指标"
```

#### 5.2 填写你的反思

| 层级 | 本次实验中你注意到什么问题？ | 下次如何改进？ |
|------|--------------------------|--------------|
| 提示词层 | | |
| 上下文层 | | |
| Harness 层 | | |

#### ✅ 检查点5：保存三层分析

---

## 五、实验报告

### 5.1 报告内容

1. **QPS 基准数据**（步骤1的四组数据）
2. **根因分析**（步骤2的 AI 分析 + 你的判断）
3. **Gate 模拟结果**（步骤3的 BP1/BP2 检查表）
4. **Harness 治理闭环图**（步骤4）
5. **三层模型反思**（步骤5）

### 5.2 提交方式

```bash
git checkout -b experiment/week-10-你的学号
mkdir -p reports/week-10
# 将你的实验报告（Markdown 格式）放入 reports/week-10/
git add reports/week-10/
git commit -m "experiment: submit week-10 Harness governance lab"
git push origin experiment/week-10-你的学号
```

---

## 六、思考与反思

### 本周核心问题

1. **如果没有 Gate 系统**，DELETE 只有 206 QPS 的版本会被发现吗？什么时候才会被发现？

2. **AI 在这个优化过程中能做什么、不能做什么？**（AI 能写代码、但不能"知道" QPS 门槛是 10,000）

3. **为什么说"Harness 不是帮 AI 写代码，而是告诉 AI 哪里还不够好"？** 用本实验的经历说明。

---

## 七、评分标准

| 检查项 | 分值 |
|--------|------|
| QPS 基准测试数据（4组） | 15分 |
| 根因分析（AI辅助 + 人工判断） | 20分 |
| Gate 模拟结果 | 20分 |
| Harness 治理闭环图 | 25分 |
| 三层模型反思 | 10分 |
| 实验报告完整 | 10分 |

---

*最后更新: 2026-05-09*
*Harness 治理实战第1周 —— DELETE/UPDATE 性能优化案例*

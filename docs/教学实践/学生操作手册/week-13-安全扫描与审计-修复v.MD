# 第13周：代码智能与形式化思维

> 实验时间：2学时
> 实验类型：验证性+设计性
> 核心理念：**AI 不知道"改了这行代码会影响什么"——GitNexus 知道。AI 不习惯"明确逻辑"——IF/THEN 规则训练它。**

---

## 一、实验目标

- [ ] 能够运行 cargo audit 进行依赖安全扫描
- [ ] 能够使用 GitNexus 进行影响范围分析
- [ ] 理解 GitNexus 如何弥补 AI 的上下文盲区
- [ ] 能够用 IF/THEN 格式描述功能和不变式
- [ ] 理解"测试 ≠ 证明"的核心差异

---

## 二、实验背景

### GitNexus：代码的"搜索引擎"

```
传统 grep：搜索关键字 "execute_delete"
  → 返回 15 个文件，37 处匹配
  → 你需要逐个读，判断哪些是"调用"、哪些是"定义"、哪些是"注释"

GitNexus：查询 "谁调用了 execute_delete？"
  → storage.rs:145 → transaction.rs:89 → query_engine.rs:234
  → 3 个直接调用者，属于 "QueryExecution" 执行流
  → 风险等级：MEDIUM（不是核心 API）
```

### IF/THEN 规则：给 AI 明确的逻辑

```
模糊的描述（AI 容易理解错）：
  "DELETE 性能需要优化"

IF/THEN 格式（逻辑清晰）：
  ## IF 条件
  表中有 100,000 行数据，需要删除其中 10,000 行
  
  ## THEN 结果
  DELETE 操作在 1 秒内完成（QPS ≥ 10,000）
  
  ## INVARIANT
  删除后的数据完整（剩余 90,000 行正确）
  B+Tree 索引结构有效
  WAL 日志正确记录了删除操作
```

---

## 三、操作步骤

### 步骤1：依赖安全扫描（15分钟）

```bash
# 安装
cargo install cargo-audit 2>/dev/null || echo "已安装"

# 运行
cargo audit
```

#### 1.1 分析结果

```
如果发现漏洞，填写：

| 依赖包 | 版本 | CVE 编号 | 严重程度 | 修复方案 |
|--------|------|---------|---------|---------|
| | | | | |
```

#### 1.2 理解豁免机制

```
问题：如果 cargo audit 发现一个漏洞，但它的影响微乎其微怎么办？

  ❌ 错误做法：cargo audit || true（直接忽略）
  ✅ 正确做法：
    1. 分析漏洞影响范围
    2. 记录到 Issue："CVE-XXXX 影响评估"
    3. 如果风险可控 → E-09 豁免 + 设置复查日期
    4. 如果风险不可控 → 必须修复
```

#### ✅ 检查点1：保存 cargo audit 结果

---

### 步骤2：GitNexus 影响分析实践（25分钟）

#### 2.1 了解你的索引状态

```bash
# 检查 GitNexus 索引状态
npx gitnexus status 2>/dev/null || echo "GitNexus 未安装或索引未更新，使用模拟数据"
```

#### 2.2 分析 DELETE 相关的影响范围

如果 GitNexus 可用：
```bash
# 分析 DELETE 执行函数的影响范围
npx gitnexus context "execute_delete" 2>/dev/null

# 分析如果修改 DELETE 实现，会影响什么
npx gitnexus impact --target "execute_delete" --direction upstream 2>/dev/null
```

如果 GitNexus 不可用，手动搜索替代：
```bash
# 模拟：搜索 DELETE 相关的函数调用
grep -rn "delete\|DELETE" crates/executor/src/ crates/storage/src/ --include="*.rs" | head -20
```

#### 2.3 填写影响分析表

| 分析项 | 结果 |
|--------|------|
| DELETE 涉及的主要文件 | |
| DELETE 函数的直接调用者 | |
| 如果修改 DELETE 实现，影响几个模块？ | |
| 风险等级（你的判断） | LOW / MEDIUM / HIGH / CRITICAL |

#### 2.4 理解 GitNexus 如何弥补 AI 盲区

```
AI 写代码时（上下文限制）：
  ✅ 能看到：当前 DELETE 函数的实现
  ❌ 看不到：谁调用了这个 DELETE 函数（调用链）
  ❌ 看不到：DELETE 修改如何影响存储引擎的 B+Tree
  ❌ 看不到：并发场景下 DELETE 和 SELECT 的锁交互

GitNexus 补充了什么：
  ✅ 完整的调用链（谁调用了 DELETE）
  ✅ 影响范围（改了 DELETE 会影响哪些模块）
  ✅ 风险等级（这个改动有多"危险"）
```

#### ✅ 检查点2：保存影响分析结果

---

### 步骤3：编写 IF/THEN 规则（25分钟）

#### 3.1 理解 IF/THEN 的价值

```
测试（Testing）：
  DELETE 功能：删除 100 行 → 验证 100 行都不在了 ✅
  DELETE 性能：QPS = 64,896 ✅
  结论："DELETE 功能应该是正确的"

  问题：这只验证了"100行"的场景
       如果表有 100 万行？
       如果删除过程中断电？
       如果两个事务同时 DELETE？

形式化描述（IF/THEN + INVARIANT）：
  IF 执行 DELETE FROM users WHERE id = X
  AND IF X 存在
  THEN 该行从表中移除
  INVARIANT 其他行不受影响
  INVARIANT B+Tree 索引保持有效
  INVARIANT WAL 日志记录了此操作

  区别：IF/THEN 描述的不是"一个测试场景"
       而是"在所有可能情况下，这个规则都必须成立"
```

#### 3.2 为 DELETE 编写 IF/THEN 规则

```markdown
# DELETE 操作的 IF/THEN 规则

## 规则1：基本删除

## IF 条件
DELETE FROM users WHERE id = 42

## THEN 结果
id=42 的行从 users 表中移除

## INVARIANT
- 其他行的 id 不变
- 表的总行数减少 1


## 规则2：并发删除

## IF 条件
事务 T1 和 T2 同时 DELETE 同一行（id=42），
隔离级别为 Snapshot Isolation

## THEN 结果
只有一个事务成功提交，另一个收到 WriteConflict 错误

## INVARIANT
- 不会出现"一行被删除两次"的数据异常
- 不会出现"两个事务都没删掉"的遗漏


## 规则3：崩溃恢复

## IF 条件
DELETE 执行过程中，数据库进程崩溃

## THEN 结果
重启后，数据恢复到"最后一个已提交事务"的状态

## INVARIANT
- WAL 日志中记录了 DELETE 操作
- 恢复后不会出现"部分删除"的状态（行要么在，要么不在）
```

#### 3.3 你的任务：为 UPDATE 编写 IF/THEN 规则

选择 UPDATE 操作，编写至少两条 IF/THEN 规则（参考上面的 DELETE 规则）。

#### ✅ 检查点3：保存 IF/THEN 规则

---

### 步骤4：理解"测试 ≠ 证明"（15分钟）

#### 4.1 数学类比

```
测试（Testing）：
  已知 f(1)=2, f(2)=4, f(3)=6
  推测 f(x) = 2x
  验证 f(4)=8 ✅, f(5)=10 ✅
  结论："f(x)=2x 应该是对的"
  
  → 风险：f(100) 一定等于 200 吗？你不知道。

形式化证明（Formal Verification）：
  已知 f(1)=2, f(n+1)=f(n)+2
  通过数学归纳法证明：∀n, f(n)=2n
  
  → 保证：对所有 n 都成立，100% 确定
```

#### 4.2 数据库中的映射

```
测试能回答的问题：
  ✅ DELETE 100 行 → 结果正确吗？ → 测试可以验证
  ✅ 覆盖率 78% → 够了吗？ → 测试覆盖率可以衡量

测试不能回答的问题：
  ❓ 在所有可能的并发组合下，DELETE 行为正确吗？ → 需要形式化
  ❓ 删除过程中任何时刻崩溃，数据都能恢复吗？ → 需要 Crash 注入
  ❓ 审计记录能防篡改吗？ → 需要 Audit Integrity 验证

这就是 BP3（风险验证）要解决的问题 —— 
测试覆盖了"正常情况"，BP3 覆盖了"最坏情况"。
```

#### ✅ 检查点4：写出你的理解

---

## 四、实验报告

1. **cargo audit 结果和豁免分析**
2. **GitNexus 影响分析表**（或手动搜索的结果）
3. **DELETE 和 UPDATE 的 IF/THEN 规则**（至少各 2 条）
4. **"测试 ≠ 证明"的理解**（用你自己的话）

---

## 五、思考与反思

1. **GitNexus 和 grep 的区别是什么？** 为什么只说"代码搜索"低估了 GitNexus？

2. **IF/THEN 规则和普通的 PR 描述有什么区别？** 为什么前者对 AI 更友好？

3. **在你的项目中，有什么是"测试能覆盖的"、什么是"只有形式化才能保证的"？**

---

## 六、评分标准

| 检查项 | 分值 |
|--------|------|
| cargo audit 结果 + 豁免分析 | 15分 |
| GitNexus 影响分析 | 25分 |
| IF/THEN 规则（至少4条） | 30分 |
| "测试≠证明"理解 | 15分 |
| 实验报告完整 | 15分 |

---

*最后更新: 2026-05-09*

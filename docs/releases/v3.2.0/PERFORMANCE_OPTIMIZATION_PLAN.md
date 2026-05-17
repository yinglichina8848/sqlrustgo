# SQLRustGo v3.2.0 性能优化整改开发计划

> **版本**: v1.0
> **日期**: 2026-05-17
> **状态**: 规划中
> **基于**: PERFORMANCE_BOTTLENECK_ANALYSIS.md
> **类型**: Bugfix/性能优化

---

## 一、概述与目标

### 1.1 问题背景

v3.2.0 RC Gate 发现严重性能回归:

| 指标 | v3.1.0 基线 | v3.2.0 当前 | 变化 |
|------|-------------|-------------|------|
| UPDATE QPS | 532K | 58K | **-89%** |
| DELETE QPS | 706K | 62K | **-91%** |
| INSERT QPS | ~400K | 55K | **-86%** |
| aggregation | 1.27M | 768K | **-40%** |
| complex_where | ~5K | 368 | **-93%** |
| join | ~50K | 11K | **-78%** |

### 1.2 优化目标

| 指标 | 当前值 | 目标值 | 优先级 |
|------|--------|--------|--------|
| UPDATE QPS | 58K | ≥500K | P0 |
| DELETE QPS | 62K | ≥600K | P0 |
| INSERT QPS | 55K | ≥400K | P0 |
| aggregation | 768K | ≥1.2M | P0 |
| complex_where | 368 | ≥5K | P0 |
| join | 11K | ≥50K | P1 |

### 1.3 成功的定义

- [ ] UPDATE/DELETE/INSERT QPS 恢复到 400K+ 级别
- [ ] aggregation QPS 恢复到 1.2M+
- [ ] complex_where QPS ≥5K
- [ ] join QPS ≥50K
- [ ] RC Gate R9 Regression 检查通过

---

## 二、根因分析

### 2.1 性能回归时间线

```
2026-05-16 22:11  f561caa5  建立 baseline (aggregation=1.27M)
2026-05-16 22:23  f29ee983  PR #1098 clippy dead_code 修复
2026-05-17 01:31  (多个PR合并后)  RC Gate aggregation=768K

关键 PR:
- PR #1097 (f28bcddb): feat(storage): 冷存储分层 (Issue #993)
- PR #1091 (abd28897): feat(storage): 冷存储完善 (S3签名 + StorageTierManager)
- PR #1093 (5e10ff1f): fix(storage): AWS S3 SigV4 signing
```

### 2.2 假设根因

**假设 1: Storage tier 每次 DML 操作检查引入开销**
- `StorageTierManager` 可能在每次 write 时检查 tier 层级
- S3 签名计算 (`hmac_sha256`) 可能在关键路径上

**假设 2: cold-storage-tiering 改变了执行路径**
- 原有的直接 write 路径被修改为经过 tier 判断
- 可能引入不必要的分支预测失败

### 2.3 需要验证

```bash
# 1. 对比 PR #1097 前后的存储路径
git diff f561caa5..f28bcddb -- crates/storage/src/

# 2. 检查 StorageTierManager 是否在 write 路径上
grep -rn "StorageTierManager" crates/storage/src/
```

---

## 三、修复任务清单

### 3.1 P0 必须修复

#### PERF-FIX-1: 调查 aggregation -40% 回归

**Issue**: (新建)
**根因**: 未知,需调查
**步骤**:
1. 对比 baseline.json 建立前后的代码变更
2. 检查 storage tier 代码是否影响 aggregation 路径
3. 使用 `perf record` 或 `cargo flamegraph` 分析
4. 确定根因后修复

**验收条件**: aggregation QPS ≥1.2M

---

#### PERF-FIX-2: 恢复 UPDATE/DELETE/INSERT 性能

**Issue**: (新建)
**根因**: Storage tier 初始化引入开销
**步骤**:
1. 检查 `StorageTierManager::new()` 是否在每次操作时重新初始化
2. 检查 S3 签名是否在每次 write 时重新计算
3. 如果 `hmac_sha256` 在关键路径,添加缓存
4. 验证修复后 QPS 恢复到 400K+ 级别

**验收条件**:
- UPDATE QPS ≥500K
- DELETE QPS ≥600K
- INSERT QPS ≥400K

---

#### PERF-FIX-3: 激活 CBO 代价模型

**Issue**: #616 (继承自 v3.1.0)
**根因**: 43KB CBO 代价公式从未接入 `optimizer.rs`
**步骤**:
1. 研究 `optimizer/src/cost.rs` 中的代价模型
2. 在 `optimizer.rs` 中集成代价计算
3. 实现基于代价的执行计划选择
4. 验证 join/order_by/complex_where 性能提升

**验收条件**:
- join QPS ≥50K
- order_by QPS ≥50K
- complex_where QPS ≥5K

---

#### PERF-FIX-4: 优化 complex_where 查询

**Issue**: (新建)
**根因**: WHERE 子句无优化,全表扫描
**步骤**:
1. 分析 complex_where 的执行计划
2. 实现谓词下推优化
3. 添加表达式索引支持
4. 验证 QPS 从 368 提升到 5K+

**验收条件**: complex_where QPS ≥5K

---

### 3.2 P1 建议修复

#### PERF-FIX-5: 修复 Sysbench 握手问题

**Issue**: PERF-1 (os error 35)
**根因**: MySQL 协议层 handshake 并发问题
**步骤**:
1. 分析 `mysql-server` crate 的 handshake 代码
2. 检查 socket 设置 (Non-blocking vs blocking)
3. 修复 `os error 35` 资源暂时不可用问题
4. 验证 Sysbench 可以运行

**验收条件**: Sysbench handshake 成功

---

#### PERF-FIX-6: 优化 join QPS

**Issue**: (继承 PERF-FIX-3)
**根因**: CBO 未激活,错误执行计划
**步骤**:
1. 激活 CBO 后自然优化
2. 如仍不足,实现 Hash join 专门优化
3. 验证 QPS 从 11K 提升到 50K+

**验收条件**: join QPS ≥50K

---

## 四、技术方案

### 4.1 Storage Tier 修复方案

```rust
// 问题: 每次 write 操作都检查 tier 并计算 S3 签名
// 解决: 延迟初始化 + 签名缓存

pub struct StorageTierManager {
    remote: Option<RemoteBackupStorage>,  // 延迟初始化
    sign_cache: Arc<RwLock<SignatureCache>>,  // 签名缓存
}

impl StorageTierManager {
    pub fn write(&self, key: &str, data: &[u8]) -> Result<()> {
        // 检查是否需要写 remote (热数据不过 remote)
        if self.is_hot_data(key) {
            // 直接写 local,跳过 remote 检查
            return self.local_write(key, data);
        }
        // 冷数据才检查 remote
        self.write_remote_cold(key, data)
    }
}
```

### 4.2 CBO 激活方案

```rust
// cost.rs 中已有代价公式,需要接入 optimizer.rs

pub fn compute_hash_join_cost(
    build_size: f64,
    probe_size: f64,
    hash_probe_cost: f64,
) -> Cost {
    Cost {
        cpu_cost: build_size + probe_size * hash_probe_cost,
        io_cost: 0.0, // Hash join 主要是内存操作
    }
}

// 在 optimizer.rs 的 optimize 函数中:
let candidates = vec![
    (HashJoinPlan, compute_hash_join_cost(...)),
    (NestedLoopPlan, compute_nested_loop_cost(...)),
];
// 选择代价最低的
candidates.min_by_key(|(_, cost)| cost.total())
```

---

## 五、测试策略

### 5.1 验证测试

```bash
# 1. 运行 QPS 基准测试
cargo test qps_benchmark --release -- --nocapture

# 2. 对比修复前后
# 修复前: UPDATE=58K, DELETE=62K, INSERT=55K
# 修复后目标: UPDATE≥500K, DELETE≥600K, INSERT≥400K

# 3. RC Gate R9 回归检查
bash scripts/gate/check_rc_v320.sh
```

### 5.2 性能回归预防

| 测试 | 基线 | 回归阈值 | 告警阈值 |
|------|------|----------|----------|
| simple_select | 324K | ≤300K | ≤280K |
| update | 58K→500K | ≤450K | ≤400K |
| delete | 62K→600K | ≤550K | ≤500K |
| insert | 55K→400K | ≤360K | ≤320K |
| aggregation | 768K→1.2M | ≤1.1M | ≤1.0M |
| join | 11K→50K | ≤45K | ≤40K |
| complex_where | 368→5K | ≤4.5K | ≤4K |

---

## 六、里程碑

### 6.1 阶段目标

| 阶段 | 日期 | 任务 | 验收 |
|------|------|------|------|
| M1 | 2026-05-17 | 根因调查完成 | 确定 aggregation 回归原因 |
| M2 | 2026-05-18 | Storage tier 修复 | UPDATE/DELETE/INSERT 恢复 |
| M3 | 2026-05-19 | CBO 激活 | join/complex_where 提升 |
| M4 | 2026-05-20 | RC Gate 复查 | R9 通过 |

### 6.2 进度跟踪

- [ ] M1: 根因调查完成
- [ ] M2: Storage tier 修复完成
- [ ] M3: CBO 激活完成
- [ ] M4: RC Gate 复查通过

---

## 七、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| Storage tier 修复引人新问题 | 高 | 中 | 充分测试,保留 revert 能力 |
| CBO 激活改变执行计划 | 中 | 低 | 逐步激活,监控回归 |
| 修复时间超过预期 | 中 | 中 | 优先修复 P0,推迟 P1 |

---

## 八、附录

### A. 相关文档

- [PERFORMANCE_BOTTLENECK_ANALYSIS.md](./PERFORMANCE_BOTTLENECK_ANALYSIS.md)
- [v3.1.0 SYSTEM_BOTTLENECK_ANALYSIS.md](../v3.1.0/SYSTEM_BOTTLENECK_ANALYSIS.md)
- [DEVELOPMENT_PLAN.md](./DEVELOPMENT_PLAN.md)

### B. 相关 Issue

| Issue | 描述 | 状态 |
|-------|------|------|
| #616 | CBO 激活 | 待修复 |
| #619 | 多表 DML | 待修复 |
| #621 | 窗口函数补全 | 待修复 |
| #920 | PERF-1: Point SELECT QPS ≥1M | 待修复 |
| #993 | 冷存储分层 | 部分完成 |

### C. 命令参考

```bash
# 运行 QPS 基准测试
cargo test qps_benchmark --release

# 运行 RC Gate 检查
bash scripts/gate/check_rc_v320.sh

# 性能分析
cargo flamegraph --bin sqlrustgo -- qps_benchmark

# 对比代码变更
git diff f561caa5..HEAD -- crates/storage/src/
```

---

**维护人**: Sisyphus Orchestrator
**创建日期**: 2026-05-17
# 测试覆盖率深度分析与解决方案

## 当前状态

**覆盖率**: 76.14% (3909/5134 行)  
**目标**: 85%  
**差距**: 约 8.86% (约 455 行代码)

---

## 问题诊断

### 模块覆盖率分布

| 模块 | 覆盖率 | 状态 |
|------|--------|------|
| buffer_pool.rs | 109/109 (100%) | ✅ 完美 |
| file_storage.rs | 204/227 (90%) | ✅ 良好 |
| planner/optimizer | ~75% | ✅ 可接受 |
| **executor.rs** | **417/800 (52%)** | ❌ 关键瓶颈 |
| **http_server.rs** | **10/86 (12%)** | ❌ 关键瓶颈 |
| main.rs | 0/3 (0%) | ⚠️ CLI 难以测试 |

---

## 根本原因分析

### 1. 架构设计问题 (主要原因)

**executor.rs 的问题**:

```rust
// 难以测试的代码结构
pub struct SeqScanVolcanoExecutor {
    storage: Arc<dyn Storage>,  // 需要完整 mock Storage trait
    initialized: bool,         // 内部状态管理
    rows: Vec<Vec<Value>>,    // 大量边界条件分支
    current_idx: usize,
}
```

每个 executor 方法都有多个边界条件：
- `if !initialized { return Err(...) }`
- `if current_idx >= rows.len() { return Ok(None) }`
- `if self.initialized { return Ok(()); }` (重复 init)

### 2. 测试基础设施不足

- **缺少 Mock 框架**: 没有自动化 mock 工具
- **Storage trait 复杂**: 需要实现 5+ 方法
- **无依赖注入**: 硬编码依赖难以替换

### 3. 代码结构问题

- **800 行 executor.rs**: 单文件过大
- **私有字段**: 无法从外部设置测试状态
- **错误处理分支**: 难以在单元测试中触发

---

## 解决方案

### 方案 A: 引入 mockall (推荐)

**工作内容**:
1. 在 Cargo.toml 添加 `mockall` 依赖
2. 为关键 trait (Storage, PhysicalPlan) 生成 mock
3. 简化 executor 测试 setup

**预期效果**: +3-5% 覆盖率  
**工作量**: 2-3 小时

### 方案 B: 重构 executor 代码

**工作内容**:
1. 拆分 executor.rs 为多个文件
2. 添加依赖注入支持
3. 提取接口 trait

**预期效果**: +8-10% 覆盖率  
**工作量**: 2 周

### 方案 C: 接受现状

当前 76% 已高于行业平均 (60-70%)，可继续开发新功能。

---

## 行动计划

1. [ ] 引入 mockall 依赖
2. [ ] 为 Storage trait 生成 mock
3. [ ] 重写 executor 测试使用 mock
4. [ ] 验证覆盖率提升

---

## 结论

**覆盖率低的根本原因是架构设计问题，不是测试受限制。**

通过引入 mockall 可以快速改善测试覆盖情况。

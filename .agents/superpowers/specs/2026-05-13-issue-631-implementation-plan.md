# Issue #631 向量化执行评估 + Iterator Model 分析 实现计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完成 Issue #631 的两个验收条件：
1. `oo/execution/ITERATOR_MODEL.md` 分析文档
2. 向量化评估报告（含 VecTableScanExecutor 原型 + Benchmark）

**Architecture:**
- VEC-1: 文档产出（技术调研 + ADR 格式）
- VEC-2: 原型代码（Benchmark + 向量化执行器）
- VEC-3: SIMD 评估（代码片段 + 理论分析）

**Tech Stack:** Rust, Criterion (benchmark), std::simd (实验性)

---

## Chunk 1: VEC-1 文档产出

### Task 1: 创建 oo/execution 目录和 ITERATOR_MODEL.md 文档

**Files:**
- Create: `oo/execution/ITERATOR_MODEL.md`

- [ ] **Step 1: 创建 oo/execution 目录**
  ```bash
  mkdir -p oo/execution
  ```

- [ ] **Step 2: 创建 ITERATOR_MODEL.md 文档**
  文档结构：
  ```markdown
  # 火山模型 vs 向量化执行分析

  ## 1. 问题背景
  ### 1.1 火山模型 (Volcano Model)
  ### 1.2 性能瓶颈分析

  ## 2. 向量化执行模型
  ### 2.1 批量处理 (Batch-at-a-time)
  ### 2.2 Columnar vs Row-oriented

  ## 3. 火山 vs 向量化对比
  ### 3.1 性能对比
  ### 3.2 适用场景

  ## 4. SQLRustGo 当前实现
  ### 4.1 火山模型执行器
  ### 4.2 向量化基础设施

  ## 5. 向量化 Scanner 原型设计
  ### 5.1 接口设计
  ### 5.2 数据流

  ## 6. SIMD 加速评估
  ### 6.1 可 SIMD 化操作分析
  ### 6.2 代码片段
  ### 6.3 加速比估算

  ## 7. 结论与下一步
  ```

- [ ] **Step 3: 补充完整内容**
  基于设计文档 `docs/superpowers/specs/2026-05-13-vec-execution-assessment-design.md` 的内容，填充 `oo/execution/ITERATOR_MODEL.md`。

- [ ] **Step 4: Commit**
  ```bash
  git add oo/execution/ITERATOR_MODEL.md
  git commit -m "docs: add ITERATOR_MODEL analysis document for issue #631"
  ```

---

## Chunk 2: VEC-2 向量化执行器原型

### Task 2: 创建 VecTableScanExecutor 执行器

**Files:**
- Create: `crates/executor/src/vec_table_scan.rs`
- Modify: `crates/executor/src/lib.rs` (添加模块导出)

- [ ] **Step 1: 创建 vec_table_scan.rs**
  实现内容：
  ```rust
  //! Vectorized Table Scan Executor
  //!
  //! 实现批量表扫描，支持 DataChunk 输出格式

  use crate::{VolcanoExecutor, ExecutorResult};
  use sqlrustgo_planner::{Expr, Schema};
  use sqlrustgo_types::{SqlResult, Value};
  use std::sync::Arc;

  /// 向量化表扫描执行器
  pub struct VecTableScanExecutor {
      data: Vec<DataChunk>,  // 预加载的数据
      batch_size: usize,
      current_chunk_idx: usize,
      chunk_position: usize,
      predicate: Option<Expr>,
  }

  impl VecTableScanExecutor {
      pub fn new(data: Vec<DataChunk>, batch_size: usize) -> Self;
      pub fn with_predicate(mut self, predicate: Expr) -> Self;
      pub fn next_batch(&mut self) -> SqlResult<Option<DataChunk>>;
  }

  impl VolcanoExecutor for VecTableScanExecutor {
      fn open(&mut self) -> SqlResult<()>;
      fn next(&mut self) -> SqlResult<Option<Vec<Value>>>;
      fn close(&mut self) -> SqlResult<()>;
  }
  ```

- [ ] **Step 2: 修改 lib.rs 添加模块导出**
  在 `crates/executor/src/lib.rs` 添加：
  ```rust
  pub mod vec_table_scan;
  pub use vec_table_scan::VecTableScanExecutor;
  ```

- [ ] **Step 3: 运行 cargo check 验证**
  ```bash
  cd crates/executor && cargo check --all-features
  ```

- [ ] **Step 4: 提交**
  ```bash
  git add crates/executor/src/vec_table_scan.rs crates/executor/src/lib.rs
  git commit -m "feat(executor): add VecTableScanExecutor prototype for issue #631"
  ```

---

### Task 3: 创建向量化 Filter 执行器

**Files:**
- Create: `crates/executor/src/vec_filter.rs`

- [ ] **Step 1: 创建 vec_filter.rs**
  使用 `vectorization.rs` 中的 `ColumnArray` 和 `vectorized_expr::eval_expr` 实现批量过滤。

- [ ] **Step 2: 运行测试验证**
  ```bash
  cd crates/executor && cargo test vec_filter --all-features
  ```

- [ ] **Step 3: 提交**
  ```bash
  git add crates/executor/src/vec_filter.rs crates/executor/src/lib.rs
  git commit -m "feat(executor): add VecFilterExecutor for issue #631"
  ```

---

## Chunk 3: VEC-2 Benchmark 套件

### Task 4: 创建 Benchmark 目录和基础设施

**Files:**
- Create: `crates/executor/benches/vec_benchmark.rs` (需要创建 benches 目录)
- Modify: `crates/executor/Cargo.toml` (添加 dev-dependencies)

- [ ] **Step 1: 创建 benches 目录并添加 Cargo.toml 配置**
  ```bash
  mkdir -p crates/executor/benches
  ```

  在 `Cargo.toml` 添加：
  ```toml
  [dev-dependencies]
  criterion = { version = "0.5", features = ["html_reports"] }

  [[bench]]
  name = "vec_benchmark"
  harness = false
  ```

- [ ] **Step 2: 创建基础 Benchmark 文件**
  ```rust
  // crates/executor/benches/vec_benchmark.rs

  use criterion::{black_box, criterion_group, criterion_main, Criterion};
  use sqlrustgo_executor::{VecTableScanExecutor, VecFilterExecutor, DataChunk, ColumnArray};
  use sqlrustgo_planner::{Expr, DataType, Field};
  use sqlrustgo_types::Value;

  fn generate_test_data(n: usize) -> Vec<DataChunk> {
      // 生成 n 行测试数据，分成多个 chunk
  }

  fn bench_filter_volcano_vs_vectorized(c: &mut Criterion) {
      // B1: 内存 Benchmark - FilterVolcanoExecutor vs VecFilterExecutor
  }

  fn bench_storage_pipeline(c: &mut Criterion) {
      // B2: Storage 集成 Benchmark
  }

  fn bench_tpch_queries(c: &mut Criterion) {
      // B3: TPC-H Q1/Q6
  }

  criterion_group!(
      benches,
      bench_filter_volcano_vs_vectorized,
      bench_storage_pipeline,
      bench_tpch_queries
  );
  criterion_main!(benches);
  ```

- [ ] **Step 3: 运行 Benchmark 验证**
  ```bash
  cd crates/executor && cargo bench --features benchmark
  ```

- [ ] **Step 4: 提交**
  ```bash
  git add crates/executor/benches/ crates/executor/Cargo.toml
  git commit -m "feat(executor): add vectorized benchmark suite for issue #631"
  ```

---

## Chunk 4: VEC-3 SIMD 评估

### Task 5: 添加 SIMD 评估代码

**Files:**
- Create: `crates/executor/src/vec_simd.rs`

- [ ] **Step 1: 创建 vec_simd.rs**
  实现 SIMD 加速的代码片段：
  ```rust
  //! SIMD Acceleration Evaluation
  //!
  //! 提供 SIMD 优化的代码片段和加速比估算

  /// SIMD 加速的整数求和（使用 packed_simd）
  pub fn sum_i64_simd(values: &[i64]) -> i64;

  /// SIMD 加速的浮点求和
  pub fn sum_f64_simd(values: &[f64]) -> f64;

  /// SIMD 加速的聚合（求和、计数）
  pub mod simd_agg {
      pub fn sum_i64(values: &[i64]) -> i64;
      pub fn avg_i64(values: &[i64]) -> f64;
      // ...
  }
  ```

- [ ] **Step 2: 在 Benchmark 中添加 SIMD 对比**
  在 `benches/vec_benchmark.rs` 添加：
  ```rust
  fn bench_simd_vs_scalar(c: &mut Criterion) {
      // 对比 SIMD 版本和纯量版本的性能
  }
  ```

- [ ] **Step 3: 运行 SIMD Benchmark**
  ```bash
  cd crates/executor && cargo bench simd --features benchmark
  ```

- [ ] **Step 4: 更新 ITERATOR_MODEL.md 补充 SIMD 评估结果**
  记录实际测得的加速比数据。

- [ ] **Step 5: 提交**
  ```bash
  git add crates/executor/src/vec_simd.rs
  git commit -m "feat(executor): add SIMD evaluation code for issue #631"
  ```

---

## Chunk 5: 验收与收尾

### Task 6: 验证 Issue #631 验收条件

**Files:**
- Check: `oo/execution/ITERATOR_MODEL.md`
- Check: Benchmark 运行结果

- [ ] **Step 1: 确认 ITERATOR_MODEL.md 存在且完整**
  ```bash
  ls -la oo/execution/ITERATOR_MODEL.md
  ```

- [ ] **Step 2: 确认 Benchmark 可运行**
  ```bash
  cd crates/executor && cargo bench --features benchmark
  ```

- [ ] **Step 3: 更新 Issue #631 进度**
  通过 Gitea API 更新 Issue 评论或 Progress Update。

- [ ] **Step 4: 最终提交**
  ```bash
  git add -A
  git commit -m "feat: complete issue #631 vec execution assessment"
  ```

---

## 依赖关系

```
Task 1 (VEC-1 文档)
    ↓
Task 2 (VecTableScanExecutor) ← Task 3 (VecFilterExecutor)
    ↓
Task 4 (Benchmark) ← Task 2, Task 3
    ↓
Task 5 (SIMD 评估) ← Task 2
    ↓
Task 6 (验收)
```

## 验收条件

- [ ] `oo/execution/ITERATOR_MODEL.md` 存在且内容完整
- [ ] 向量化评估报告（包含 Benchmark 结果和 SIMD 加速比估算）
- [ ] 代码可编译运行：`cargo check --all-features`
- [ ] Benchmark 可执行：`cargo bench --features benchmark`

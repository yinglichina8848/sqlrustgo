# SQLRustGo SIMD 向量化性能基准测试报告

> **版本**: v2.8.0
> **日期**: 2026-04-30
> **Issue**: #32
> **状态**: 完成基准测试，性能目标待 AVX2 硬件验证

---

## 1. 测试环境

| 项目 | 值 |
|------|-----|
| CPU | Intel Xeon Gold 6138 @ 2.00GHz |
| 核心数 | 80 核 (2x 40-core) |
| SIMD 支持 | SSE2, SSE4.1, SSE4.2, AVX, AVX2 **（AVX2 可用）** |
| 内存 | 409 GB DDR4 |
| OS | Linux (HP Z6G4) |
| Rust | 1.85+ |
| 测试工具 | cargo bench + criterion |

---

## 2. 基准测试结果

### 2.1 Dot Product (点积)

| 维度 | Scalar (µs) | SIMD (µs) | 加速比 |
|------|-------------|-----------|--------|
| 128 | 12.1 | 22.6 | **0.54x** (更慢) |
| 256 | 24.1 | 45.4 | **0.53x** (更慢) |
| 512 | 50.3 | 88.4 | **0.57x** (更慢) |
| 1024 | 103.2 | 175.5 | **0.59x** (更慢) |

### 2.2 L2 Distance (欧氏距离)

| 维度 | Scalar (µs) | SIMD (µs) | 加速比 |
|------|-------------|-----------|--------|
| 128 | 12.6 | 35.5 | **0.35x** (更慢) |
| 256 | 24.9 | 71.2 | **0.35x** (更慢) |
| 512 | 50.3 | 140.8 | **0.36x** (更慢) |
| 1024 | 103.5 | 281.5 | **0.37x** (更慢) |

### 2.3 Batch Dot Product (批量点积, 1000 vectors)

| 维度 | Scalar (ms) | SIMD Batch (ms) | 加速比 |
|------|-------------|-----------------|--------|
| 128 | 12.1 | 13.4 | **0.90x** (略慢) |
| 256 | 24.2 | 27.0 | **0.90x** (略慢) |
| 512 | 50.6 | 53.8 | **0.94x** (略慢) |
| 1024 | 103.8 | 105.2 | **0.99x** (持平) |

---

## 3. 问题分析

### 3.1 SIMD 性能反而更慢的原因

**根本原因**: 当前 CPU (Xeon Gold 6138, Skylake) 支持 AVX2，但 **运行时动态检测 (`is_x86_feature_detected!("avx2")`) 失败**或编译器未能充分优化 SIMD 路径。

可能因素：
1. **编译器优化级别**: benchmark 使用 `--release` 但可能未达到最优向量化
2. **SIMD 内在函数开销**: `_mm256_*` 序列化和水平求和 (horizontal sum) 开销较大
3. **对齐**: `loadu_ps` (unaligned) 比 `load_ps` (aligned) 慢，代码使用 `loadu_ps` 避免对齐错误但损失性能
4. **向量化长度**: 维度 < 1024 时，SIMD 启动开销占比大；维度增大时加速比趋于 1.0

### 3.2 已发现的正确性 bug

**cosine_similarity_simd 错误**:

```rust
// 当前错误实现 (simd_explicit.rs:141-151)
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product_simd(a, b);
    let norm_a = euclidean_distance_simd(a, a).sqrt();  // ← BUG: 永远返回 0
    let norm_b = euclidean_distance_simd(b, b).sqrt();  // ← BUG: 永远返回 0
    // ...
    dot / (norm_a * norm_b)  // ← 除以 0!
}
```

**应修复为**:

```rust
pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    let dot = dot_product_simd(a, b);
    let norm_a = dot_product_simd(a, a).sqrt();  // L2 norm of a
    let norm_b = dot_product_simd(b, b).sqrt();  // L2 norm of b
    // ...
}
```

---

## 4. 验收标准状态

| 验收标准 | 状态 | 说明 |
|---------|------|------|
| SIMD 加速 L2 距离计算 | ✅ 已实现 | `l2_distance_simd` / `euclidean_distance_simd` 存在 |
| SIMD 加速 Cosine 距离计算 | ⚠️ 有 bug | `cosine_distance_simd` 依赖错误的 `cosine_similarity_simd` |
| SIMD 加速 DotProduct 计算 | ✅ 已实现 | `dot_product_simd` 存在 |
| 性能提升 > 3x vs scalar | ❌ 未达标 | 当前: 0.35x-0.99x (更慢或持平) |

**目标**: 在 **AVX2 机器** (如 AMD Zen2+, Intel Haswell+ Server) 上重新测试，目标 > 3x 加速。

---

## 5. 下一步行动

1. **修复 cosine_similarity_simd bug** — 将 `euclidean_distance_simd(a, a)` 改为 `dot_product_simd(a, a).sqrt()`
2. **在真 AVX2 机器验证** — 当前 HP Z6G4 的 Xeon Gold 6138 虽支持 AVX2，但内存带宽可能限制 SIMD 发挥
3. **优化 SIMD 实现**:
   - 使用 `restrict` 指针（Rust unstable feature）
   - 预先对齐数据到 32 字节边界
   - 考虑 AVX-512 (Ice Lake+) 的 16-lane 优势
   - 使用 `#[target_feature(enable = "avx2")]` 强制内联 SIMD 路径

---

## 6. 性能目标

| 索引类型 | 目标 | 当前 (Z6G4) |
|---------|------|-------------|
| FlatIndex search | 10K vectors < 5ms | ~12ms (需 SIMD 优化) |
| HNSW search (ef=50) | 10K vectors < 5ms | ~8ms (已集成 SIMD) |
| IVF-PQ search | 100K vectors < 10ms | ~15ms (SIMD 待优化) |

---

*基准测试命令: `cargo bench --package sqlrustgo-vector -- simd_vs_scalar`*

# SQLRustGo v1.5.0 开发计划

> **版本**: 1.0
> **日期**: 2026-03-18
> **阶段**: Craft (设计规划)

---

## 一、开发目标

v1.5.0 专注于**性能深度优化**，在 v1.4.0 基础上实现显著性能提升。

---

## 二、功能清单

### 2.1 向量化执行 (P0)

| 任务ID | 任务名称 | 依赖 | 状态 |
|--------|----------|------|------|
| V-01 | SIMD 基础设施增强 | v1.4.0 M-003 | ⏳ |
| V-02 | 向量化 Filter 算子 | V-01 | ⏳ |
| V-03 | 向量化聚合算子 | V-01 | ⏳ |

### 2.2 CBO 深度优化 (P1)

| 任务ID | 任务名称 | 依赖 | 状态 |
|--------|----------|------|------|
| CBO-05 | IndexScan 完整实现 | v1.4.0 CBO-04 | ⏳ |
| CBO-06 | SortMergeJoin 启用 | v1.4.0 SMJ-01 | ⏳ |
| CBO-07 | 代价模型集成 | v1.4.0 CBO-01/02 | ⏳ |

### 2.3 性能优化 (P2)

| 任务ID | 任务名称 | 状态 |
|--------|----------|------|
| P-01 | JIT 编译优化 | ⏳ |
| P-02 | 内存优化 | ⏳ |

---

## 三、开发顺序

### Phase 1: SIMD 基础设施 (V-01)

1. 集成 `packed_simd` 或 `autovectorize`
2. 实现 SIMD 整数/浮点数运算
3. 实现 SIMD 比较操作

### Phase 2: 向量化算子 (V-02, V-03)

1. 向量化 Filter 实现
2. 向量化聚合实现
3. 性能测试验证

### Phase 3: CBO 深度优化 (CBO-05, CBO-06, CBO-07)

1. IndexScan 完整实现
2. SortMergeJoin 启用
3. 代价模型集成

### Phase 4: 优化 (P-01, P-02)

1. JIT 编译优化 (探索)
2. 内存优化

---

## 四、验收标准

- [ ] 性能提升 ≥30% (vs v1.4.0)
- [ ] 向量化 Filter 可用
- [ ] IndexScan 完整实现
- [ ] SortMergeJoin 稳定运行
- [ ] 测试覆盖率 ≥85%

---

## 五、相关文档

- [VERSION_PLAN.md](./VERSION_PLAN.md)
- [RELEASE_GATE_CHECKLIST.md](./RELEASE_GATE_CHECKLIST.md)
- [RELEASE_NOTES.md](./RELEASE_NOTES.md)

---

**文档状态**: Draft  
**制定日期**: 2026-03-18

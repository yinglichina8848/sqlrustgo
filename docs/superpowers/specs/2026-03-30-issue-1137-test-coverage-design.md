# Issue #1137 测试覆盖率提升 - 设计文档

## 目标

将 SQLRustGo 项目测试覆盖率从当前 ~12% 提升到 80%。

## 当前状态

| 模块 | 当前覆盖率 | 目标 |
|-----|----------|------|
| 总体 | ~12% (1440/12313) | 80% |
| executor | ~30% | 80% |
| storage | ~15% | 80% |
| parser | ~50% | 80% |
| transaction | ~40% | 80% |
| distributed | ~5% | 80% |

## 策略选择

### 测试编写策略: 混合策略 (Hybrid)
- **核心模块** (executor, storage, transaction): 采用 TDD - 测试先行
- **遗留代码**: 采用覆盖率优先 - 针对未覆盖代码补充测试

### 模块顺序: 依赖顺序 (Dependency Order)
按依赖关系从底层到上层构建:
```
types → parser → catalog → storage → transaction → executor → distributed
```

### CI 门槛: 渐进式 (Progressive)
| 周 | 目标 | 检查命令 |
|---|------|---------|
| 1 | 40% | `cargo tarpaulin --fail-under 40` |
| 2 | 50% | `cargo tarpaulin --fail-under 50` |
| 3 | 65% | `cargo tarpaulin --fail-under 65` |
| 4 | 80% | `cargo tarpaulin --fail-under 80` |

### 测试文件组织: 混合 (Mixed)
- **单元测试**: `#[cfg(test)]` 模块紧邻源文件
- **集成测试**: `crates/{module}/tests/*.rs`
- **跨模块测试**: `tests/integration/*.rs`

## 实施计划

### Phase 1 (Week 1): types + parser
**目标**: 40%

| 模块 | 起点 | 目标 | 测试重点 |
|-----|-----|------|---------|
| types | ~12% | 60% | Value 运算、类型转换、序列化/反序列化 |
| parser | ~50% | 70% | 完整 SQL92 语法解析、边界条件 |

### Phase 2 (Week 2): catalog + storage
**目标**: 50%

| 模块 | 起点 | 目标 | 测试重点 |
|-----|-----|------|---------|
| catalog | ~20% | 70% | Schema 管理、Table/Column 元数据操作 |
| storage | ~15% | 60% | BufferPool、Page 读写、B+Tree 操作、WAL |

### Phase 3 (Week 3): transaction + network
**目标**: 65%

| 模块 | 起点 | 目标 | 测试重点 |
|-----|-----|------|---------|
| transaction | ~40% | 75% | 2PC 流程、WAL 回放、锁管理、并发事务 |
| network | ~10% | 50% | MySQL 协议解析、连接管理、异常处理 |

### Phase 4 (Week 4): executor + distributed + CI
**目标**: 80%

| 模块 | 起点 | 目标 | 测试重点 |
|-----|-----|------|---------|
| executor | ~30% | 80% | JOIN、聚合、子查询、排序分页 |
| distributed | ~5% | 60% | 分片、路由、故障转移 |

### CI 集成
- 添加 `cargo tarpaulin` 到 CI pipeline
- 每个 PR 必须通过渐进式覆盖率检查
- 生成 HTML 覆盖率报告

## PR 流程

```
每完成一个模块:
1. git checkout -b feature/test-{module}-coverage
2. 编写测试 (TDD 或覆盖率优先)
3. 运行: cargo tarpaulin --fail-under {current_threshold}
4. 创建 PR → review → 合并到 develop/v2.1.0
```

## 验收标准

- [ ] 总体覆盖率 ≥ 80%
- [ ] 核心模块 (executor, storage, transaction) ≥ 85%
- [ ] 新代码覆盖率 ≥ 90%
- [ ] 所有 PR 必须通过渐进式覆盖率检查
- [ ] 覆盖率报告可生成 (HTML/JSON)

## 关键命令

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --all-features --lib --out html

# 检查覆盖率 (不阻断)
cargo tarpaulin --all-features --out json

# CI 模式 (低于门槛则失败)
cargo tarpaulin --fail-under 80

# 检查特定模块
cargo tarpaulin -p sqlrustgo-executor --out json
```

## 风险与应对

| 风险 | 应对 |
|-----|------|
| 测试覆盖率数字误导 (只测不证) | 确保测试验证实际行为，而非仅仅执行代码 |
| 遗留代码难以测试 | 重构时添加测试，或使用 mock 隔离 |
| 并发测试不稳定 | 使用适当的同步机制，确保测试隔离 |
| 依赖外部资源 (网络、磁盘) | 使用 mock 或临时目录 |

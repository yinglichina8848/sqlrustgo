# v2.5.0 门禁检查清单

**版本**: v2.5.0  
**分支**: develop/v2.5.0  
**更新日期**: 2026-04-15  
**Issue 追踪**: #1080  

---

## 1. 功能完成状态

### ✅ 已完成功能

| 功能 | Issue | PR | 状态 |
|------|-------|-----|------|
| Prepared Statement | #1384 | #1421 | ✅ MERGED |
| 子查询 EXISTS/ANY/ALL | #1382 | #1420 | ✅ MERGED |
| 子查询 Phase 2 | #1382 | #1422 | ✅ MERGED |
| 子查询 Phase 3 (外部行上下文) | #1382 | #1426 | ✅ MERGED |
| 连接池 + 超时健康检查 | #1383 | #1418 | ✅ MERGED |
| 派生表/Inline View | - | #1416 | ✅ MERGED |
| Graph 持久化 | #1378 | #1413 | ✅ MERGED |
| WAL Crash Recovery | #1388 | #1406 | ✅ MERGED |
| TPC-H SF=1 基准测试 | #1342 | #1412 | ✅ MERGED |
| TPC-H SF=10 基准测试 | #1342 | #1411 | ✅ MERGED |
| BufferPool 死循环修复 | - | #1414 | ✅ MERGED |
| BloomFilter 优化 | - | #1402, #1404 | ✅ MERGED |
| 列式存储块级跳过 | - | #1398 | ✅ MERGED |
| TransactionalExecutor | - | #1401 | ✅ MERGED |
| 统一查询 API | #1326 | #1408 | ✅ MERGED |
| TPC-H 性能优化 | #1342 | #1407 | ✅ MERGED |
| 语义嵌入 API | - | #1410 | ✅ MERGED |
| FOREIGN KEY (Parser) | #1379 | #1427 | ✅ MERGED |
| FK 集成测试 | #1379 | #1428 | ✅ MERGED |
| 回归测试集成 | - | #1419 | ✅ MERGED |
| TPC-H Q13/Q22 修复 | - | #1415 | ✅ MERGED |

### ⏳ 进行中

| 功能 | Issue | 状态 | 负责人 |
|------|-------|------|--------|
| MVCC 并发控制 | #1389 | ⏳ Phase 1/3 | @sonaheartopen |
| 完整 JOIN 实现 | #1380 | ⏳ LEFT/RIGHT | - |

### 📋 待开发

| 功能 | Issue | 优先级 |
|------|-------|--------|
| MVCC 快照隔离 | #1389 | P0 |
| JOIN 完整实现 | #1380 | P1 |
| 子查询优化 EXISTS/IN | #1382 | P1 |
| PITR 备份恢复 | #1390 | P2 |
| CBO 优化器 | #1385 | P2 |
| 分布式存储 | #1386 | P2 |

---

## 2. 门禁检查项

### 2.1 编译检查

```bash
cargo build --workspace --all-features
cargo build --release --workspace --all-features
```

**通过标准**: 无错误

---

### 2.2 测试检查

```bash
cargo test --workspace --all-features
```

**通过标准**: 所有测试通过 (0 failures)

---

### 2.3 代码规范检查

```bash
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all -- --check
```

**通过标准**: 无警告、无格式错误

---

### 2.4 覆盖率检查

```bash
cargo tarpaulin --workspace --all-features --out Xml
```

| 阶段 | 目标 | 当前 |
|------|------|------|
| Alpha | ≥ 60% | - |
| Beta | ≥ 70% | - |
| RC | ≥ 75% | - |
| GA | ≥ 80% | - |

---

## 3. 功能门禁检查

### 3.1 Prepared Statement (#1384)

- [ ] `PREPARE stmt AS SELECT ...`
- [ ] `EXECUTE stmt USING @param`
- [ ] `DEALLOCATE PREPARE stmt`
- [ ] 参数类型绑定
- [ ] 集成测试通过 (#1425)

### 3.2 子查询支持 (#1382)

- [ ] EXISTS 子查询
- [ ] ANY/ALL 比较
- [ ] IN 子查询
- [ ] 相关子查询 (外部行上下文)
- [ ] Phase 2 执行 (#1422)
- [ ] Phase 3 外部行传递 (#1426)

### 3.3 FOREIGN KEY (#1379)

- [ ] Parser 支持 FK 语法
- [ ] 表级约束定义
- [ ] 列级约束定义
- [ ] 引用完整性检查
- [ ] CASCADE/SET NULL 动作
- [ ] 集成测试 (#1428)

### 3.4 连接池 (#1383)

- [ ] 连接池配置
- [ ] 超时机制
- [ ] 健康检查
- [ ] 资源回收

### 3.5 TPC-H 基准测试 (#1342)

| Query | SF=0.1 | SF=1 | SF=10 |
|-------|--------|------|-------|
| Q1 | < 100ms | < 500ms | < 5s |
| Q2 | < 50ms | < 200ms | < 2s |
| Q3 | < 100ms | < 500ms | < 5s |
| Q4 | < 50ms | < 200ms | < 2s |
| Q5 | < 100ms | < 500ms | < 5s |
| Q6 | < 50ms | < 200ms | < 2s |
| Q7 | < 100ms | < 500ms | < 5s |
| Q8 | < 100ms | < 500ms | < 5s |
| Q9 | < 100ms | < 500ms | < 5s |
| Q10 | < 100ms | < 500ms | < 5s |
| Q11 | < 50ms | < 200ms | < 2s |
| Q12 | < 100ms | < 500ms | < 5s |
| Q13 | < 100ms | < 500ms | < 5s |
| Q14 | < 50ms | < 200ms | < 2s |
| Q15 | < 100ms | < 500ms | < 5s |
| Q16 | < 100ms | < 500ms | < 5s |
| Q17 | < 100ms | < 500ms | < 5s |
| Q18 | < 100ms | < 500ms | < 5s |
| Q19 | < 100ms | < 500ms | < 5s |
| Q20 | < 100ms | < 500ms | < 5s |
| Q21 | < 100ms | < 500ms | < 5s |
| Q22 | < 100ms | < 500ms | < 5s |

---

## 4. 性能门禁 (参考 #1423, #1424)

### 4.1 OLTP 性能目标

| 场景 | 并发 | 目标 TPS | 目标 P99 |
|------|------|----------|----------|
| Point Select | 32 | > 50,000 | < 5ms |
| Index Scan | 32 | > 10,000 | < 20ms |
| Insert | 16 | > 20,000 | < 10ms |
| Update | 16 | > 15,000 | < 15ms |
| Mixed OLTP | 32 | > 10,000 | < 30ms |

### 4.2 OLAP 性能目标

| 场景 | 目标 | MySQL 对比 |
|------|------|-----------|
| TPC-H Q1 (SF=1) | < 500ms | > 5x faster |
| TPC-H All (SF=1) | < 10s | > 3x faster |
| Vector Search 10K | < 100ms | N/A |

---

## 5. CI/CD 门禁

- [ ] CI 所有 workflow 通过
- [ ] Regression Tests 通过
- [ ] SQL-92 Compliance Tests 通过
- [ ] Benchmark PR 验证通过

### 常见 CI 问题

| 问题 | 解决方案 |
|------|----------|
| startup_failure | 检查测试环境初始化 |
| timeout | 增加测试超时时间 |
| compilation error | 更新依赖或修复代码 |

---

## 6. 发布检查清单

### 6.1 文档更新

- [ ] CHANGELOG 更新
- [ ] README 更新
- [ ] API 文档生成
- [ ] 版本说明 (RELEASE_NOTES.md)

### 6.2 版本标签

```bash
git tag -a v2.5.0 -m "v2.5.0 release"
git push origin v2.5.0
```

### 6.3 发布前确认

- [ ] 所有 P0 功能完成
- [ ] 所有测试通过
- [ ] 覆盖率达标
- [ ] 性能达标
- [ ] 文档完整
- [ ] Code Review 完成

---

## 7. 相关 Issue

- #1080: v2.1-v2.5 开发总控
- #1378: Graph 持久化
- #1379: FOREIGN KEY
- #1380: JOIN 完整实现
- #1382: 子查询优化
- #1383: 连接池
- #1384: Prepared Statement
- #1385: CBO
- #1386: 分布式存储
- #1388: WAL Crash Recovery
- #1389: MVCC
- #1390: PITR
- #1423: 替代 MySQL 差距追踪
- #1424: Benchmark 测试计划

---

**门禁状态**: 🚧 进行中  
**完成度**: ~75% (主要功能已完成，MVCC 和 JOIN 待完成)  
**预计 RC**: 2026-04-21  
**预计 GA**: 2026-04-28

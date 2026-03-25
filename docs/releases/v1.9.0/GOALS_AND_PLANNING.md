# v1.9.0 开发目标与规划

## 愿景
**目标**: 成为生产级嵌入式数据库 + 教学实验平台
- 简单生产场景（非金融）替代 MySQL
- 教学实验场景展示 SQL 特性

## 核心目标

### Phase 1: 数据完整性 & SQL 扩展 (Week 1-2)
- [ ] 索引支持优化
  - B+ Tree 索引
  - Hash 索引
  - WHERE 条件加速
- [ ] 外键约束实现 (ISSUE #901)
- [ ] 事务增强
  - SAVEPOINT / ROLLBACK TO SAVEPOINT
  - 多语句事务支持

### Phase 2: SQL 特性增强 (Week 3-4)
- [ ] 窗口函数基础支持 (ISSUE #905)
  - PARTITION BY / ORDER BY
  - AVG / SUM / ROW_NUMBER 等
- [ ] 子查询增强 (ISSUE #907)
  - Scalar / IN / EXISTS / ANY / ALL
- [ ] JOIN 优化 (ISSUE #905)

### Phase 3: 性能优化 (Week 5-6)
- [ ] 批量写入优化 (ISSUE #904)
- [ ] 查询缓存 (ISSUE #909)
- [ ] 连接池实现 (ISSUE #908)

### Phase 4: 运维与安全 (Week 7-8)
- [ ] 日志与监控 (NEW)
  - SQL 日志
  - 错误日志
  - 执行计划展示
- [ ] 自动恢复测试脚本 (ISSUE #913)
- [ ] 用户权限管理 (NEW)
  - READ / WRITE 权限

## 门禁检查
- [ ] 编译检查 (cargo build)
- [ ] 单元测试 (cargo test)
- [ ] 集成测试 (cargo test --workspace)
- [ ] SQL-92 测试 (100% 通过)
- [ ] 格式化 (cargo fmt)
- [ ] Clippy (无 error)
- [ ] 覆盖率 (≥75%)
- [ ] 压力测试 (50+ 并发)
- [ ] 崩溃恢复测试

## 测试计划

### 功能测试
- [ ] SQL 语法覆盖测试
- [ ] 子查询测试
- [ ] JOIN 测试 (INNER/LEFT/RIGHT/CROSS)
- [ ] 窗口函数测试
- [ ] 事务测试 (并发/ROLLBACK/COMMIT/SAVEPOINT)

### 性能 & 稳定性测试
- [ ] 压力测试 (50~100 并发)
- [ ] 崩溃恢复测试
- [ ] 生产场景测试
- [ ] 导入导出测试 (CSV/JSON)

### 教学场景测试
- [ ] 实验 SQL 脚本
- [ ] 可重复实验结果验证

## 发布时间
- Alpha: 待定
- Beta: 待定  
- GA: 待定

## 团队
- Agent: openclaw-refactor
- 目标: 教学 + 低强度生产环境

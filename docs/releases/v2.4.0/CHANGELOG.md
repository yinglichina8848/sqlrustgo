# SQLRustGo v2.4.0 Changelog

## [v2.4.0] - 2026-04-09

### GA 正式发布

#### 发布说明
- **版本**: v2.4.0 GA
- **日期**: 2026-04-09
- **代号**: Graph Intelligence

#### GA 门禁状态
| 检查项 | 状态 |
|--------|------|
| 编译检查 | ✅ |
| 测试检查 | ✅ |
| 性能检查 | ✅ |
| 安全检查 | ✅ |
| 文档检查 | ✅ |

---

## [v2.4.0-rc1] - 2026-04-09

### RC1 阶段完成

#### 新功能合并
- **Graph Engine** 完全合并到 `develop/v2.4.0`
- **OpenClaw API** REST 端点实现完成
- **Columnar Compression** LZ4/Zstd 压缩支持
- **CBO Index Selection** 基于成本的索引选择优化器

#### 已知问题
- 无阻塞性问题

### RC1 测试统计

| 测试类别 | 通过 | 总数 | 状态 |
|----------|------|------|------|
| 单元测试 | 35 | 35 | ✅ 100% |
| 集成测试 | 1040 | 1042 | ✅ 99.8% |
| TPC-H SF=1 | 11 | 11 | ✅ 100% |
| OpenClaw API | 11 | 11 | ✅ 100% |

---

## [v2.4.0-beta] - 2026-04-05

### Beta 阶段完成

### Added

#### Graph Engine (Issue #1077)
- **#1077** Graph Engine 核心架构
- **#1077** GQL Parser - Graph Query Language 解析器
- **#1077** Graph Planning - 图查询规划器
- **#1077** Graph Execution - 图遍历执行器
- **#1077** 独立 crate `graph-engine`

#### OpenClaw API (Issue #1078)
- **#1078** OpenClaw HTTP Server
- **#1078** `/query` 端点 - SQL 查询执行
- **#1078** `/nl_query` 端点 - 自然语言查询
- **#1078** `/schema` 端点 - 数据库 schema introspection
- **#1078** `/stats` 端点 - 执行统计
- **#1078** `/memory/*` 端点 - 记忆管理 API

#### Columnar Storage Compression (Issue #1302)
- **#1302** LZ4 压缩支持
- **#1302** Zstd 压缩支持
- **#1302** `compression` 字段添加到 ColumnDefinition

#### CBO-based Index Selection (Issue #1303)
- **#1303** Cost-based optimizer 索引选择
- **#1303** Index selection heuristics
- **#1303** Query planning optimization

#### TPC-H SF=1 Performance (Issue #1304)
- **#1304** TPC-H SF=1 完整测试报告
- **#1304** 性能基准测试文档
- **#1304** Q1, Q6, Q12 等关键查询优化

### Changed

- Graph Engine 独立为 `crates/graph-engine`
- 内部 ID 使用 u64/u32 (UUID 仅作为 property 字段)
- API 返回 Iterator 而非 Vec，支持 lazy evaluation

### Fixed

- **#1324** 修复 `compression` 字段缺失问题
- **#1323** 移除二进制文件 (third_party/*.o, dbgen, qgen)
- **#1325** CBO Index Selection 修复

---

## [v2.3.0] - 2026-04-03

### v2.3.0 合并到 v2.4.0

所有 v2.3.0 功能已合并到 `develop/v2.4.0`：

#### 可观测性 (从 v2.1.0 升级)
- Prometheus metrics 端点
- Grafana Dashboard
- 慢查询日志

#### SQL Firewall
- SQL 注入检测
- KILL/PROCESSLIST 支持

#### 工具链
- Physical Backup CLI
- mysqldump 导入导出

---

## 版本历史

| 版本 | 日期 | 成熟度 | 说明 |
|------|------|--------|------|
| **v2.4.0** | 2026-04-09 | **RC1** | **Graph Engine, OpenClaw, Compression** |
| v2.3.0 | 2026-04-03 | RC | 功能合并到 v2.4.0 |
| v2.2.0 | 2026-04-01 | RC | 架构优化 |
| v2.1.0 | 2026-04-03 | RC | 企业可观测性 |
| v2.0.0 | 2026-03-29 | GA | 向量引擎、Cascades CBO |

---

*此变更日志由 SQLRustGo Team 维护*

# SQLRustGo v1.1.0 Release Notes

**发布日期**: 2026-03-03
**发布类型**: Draft (草稿版本)
**目标成熟度**: L3 产品级

---

## 概述

SQLRustGo v1.1.0 是一个重要的架构升级版本，实现了从 L2 (原型级) 到 L3 (产品级) 的成熟度提升。本版本引入了 LogicalPlan/PhysicalPlan 分离架构、ExecutionEngine 插件化、Client-Server 架构和 HashJoin 支持，显著提升了系统的可扩展性和性能。

---

## 新功能

### 🏗️ 架构升级

#### LogicalPlan/PhysicalPlan 分离

- **LogicalPlan**: 独立的逻辑计划模块，支持查询优化
- **PhysicalPlan**: 可扩展的物理计划接口
- **查询优化器基础**: 为后续优化器实现奠定基础

#### ExecutionEngine 插件化

- **ExecutionEngine trait**: 统一的执行器接口
- **插件化架构**: 支持自定义执行器实现
- **HashJoin 执行器**: 高效的哈希连接实现

### 🌐 Client-Server 架构

#### 网络层 Phase 1

- 基础 C/S 架构实现
- MySQL 协议兼容
- 连接管理

#### 网络层 Phase 2

- 异步服务器 (Tokio)
- 连接池支持
- 多客户端并发

### 🔧 HashJoin 实现

- Inner Join 支持
- Left Join 支持
- Hash trait 实现 (支持 Float NaN 特殊处理)

---

## 变更说明

### API 变更

| 模块 | 变更 | 说明 |
|------|------|------|
| `planner` | 新增 | LogicalPlan/PhysicalPlan 模块 |
| `executor` | 重构 | ExecutionEngine trait 插件化 |
| `network` | 新增 | Client-Server 网络层 |
| `types::value` | 扩展 | Hash trait 实现 |

### 配置变更

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `server.bind` | "0.0.0.0:3306" | 服务器绑定地址 |
| `server.max_connections` | 100 | 最大连接数 |

### 依赖变更

| 依赖 | 版本 | 说明 |
|------|------|------|
| tokio | 1.x | 异步运行时 |
| criterion | 0.5 | 性能基准测试 |

---

## 性能改进

### 测试覆盖率

| 指标 | v1.0.0 | v1.1.0 | 改进 |
|------|--------|--------|------|
| 行覆盖率 | 84% | **94.18%** | +10.18% |
| 函数覆盖率 | 85% | **93.81%** | +8.81% |
| 区域覆盖率 | 85% | **93.54%** | +8.54% |

### 基准测试

新增 Criterion 基准测试框架，覆盖：
- Lexer 性能测试
- Parser 性能测试
- Executor 性能测试
- Storage 性能测试
- Network 性能测试

---

## 质量保证

### 代码质量门禁

| 检查项 | 状态 |
|--------|------|
| 编译通过 | ✅ |
| 测试通过 | ✅ |
| Clippy 检查 | ✅ |
| 格式检查 | ✅ |
| 安全审计 | ✅ |

### 安全审计

- ✅ 依赖审计通过
- ✅ 无高危安全问题
- ✅ 无敏感信息泄露
- ✅ SQL 注入防护

---

## 已知问题

| 问题 | 影响 | 状态 |
|------|------|------|
| LIKE 语句性能 | 大数据量下性能待优化 | 后续版本 |
| 子查询支持有限 | 部分复杂子查询不支持 | 后续版本 |
| 事务隔离级别 | 仅支持 READ COMMITTED | 后续版本 |

---

## 升级指南

### 从 v1.0.0 升级

1. **备份数据**: 升级前备份所有数据文件
2. **更新依赖**: 运行 `cargo update`
3. **重新编译**: 运行 `cargo build --release`
4. **迁移配置**: 更新配置文件格式

### 破坏性变更

- `Executor` trait 重命名为 `ExecutionEngine`
- 部分内部 API 签名变更

---

## 贡献者

感谢以下贡献者对本版本的贡献：

- @yinglichina8848
- @openheart-opencode (TRAE AI Assistant)

---

## 下一步计划

### v1.2.0 计划

- 查询优化器增强
- 更多 JOIN 类型支持
- 事务隔离级别完善

### v1.3.0 计划

- 分布式架构支持
- 存储引擎优化
- 监控指标完善

---

## 反馈

如有问题或建议，请通过以下方式反馈：

- GitHub Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- GitHub Discussions: https://github.com/minzuuniversity/sqlrustgo/discussions

---

*本 Release Notes 由 TRAE (GLM-5.0) 生成*

# SQLRustGo v1.1.0 发布说明

## 发布信息
- **版本**: v1.1.0
- **类型**: 正式版本 (GA)
- **发布日期**: 2026-03-10
- **Git Tag**: v1.1.0
- **分支**: release/v1.1.0

## 新功能

### 核心功能
- **查询规划器 (Query Planner)**: 实现 LogicalPlan/PhysicalPlan 架构
- **执行引擎抽象**: 定义 ExecutionEngine trait，支持多引擎注册
- **聚合函数增强**: 支持 COUNT/SUM/AVG/MIN/MAX 完整聚合
- **网络层 Phase 2**: 异步服务器、连接池和配置支持
- **认证与授权**: 基于角色的访问控制 (RBAC)
- **B+ Tree 索引**: 高效的索引支持

### 质量保证
- 测试覆盖率提升至 90%+
- 完整的单元测试套件
- 集成测试覆盖
- 代码质量检查 (Clippy)
- 安全扫描

### 工程系统
- 完整的 CI/CD 流程
- 分支保护规则
- 版本管理规范
- 门禁验收流程
- Release 工作流自动化

## 修复内容

### Bug 修复
- 修复了聚合函数中的 NULL 值处理
- 修复了执行器中的边界情况
- 修复了网络协议解析问题
- 修复了存储引擎的并发问题

### 性能优化
- 优化了查询执行计划生成
- 改进了内存使用效率
- 优化了网络 I/O 性能

## 变更记录

### 架构变更
- 新增 Planner 模块: Analyzer、LogicalPlan、PhysicalPlan
- 新增执行引擎抽象层 (ExecutionEngine trait)
- 网络层支持异步操作和连接池

### API 变更
- `ExecutionEngine` trait 定义
- `EngineRegistry` 引擎注册表
- `LogicalPlan` 逻辑计划节点
- `PhysicalPlan` 物理计划节点

### 依赖更新
- thiserror: 1.0 → 2.0
- criterion: 0.5 → 0.8
- env_logger: 0.10 → 0.11

## 安全信息

### 安全扫描
- 无高危漏洞
- 依赖库安全检查通过
- License 合规检查通过

### 安全特性
- 基于角色的访问控制
- 密码安全哈希
- Session 管理

## 安装信息

### 安装方法
```bash
# 通过 cargo 安装
cargo install --git https://github.com/minzuuniversity/sqlrustgo.git

# 从源码构建
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo
cargo build --release
```

### 系统要求
- Rust 1.75.0 或更高版本
- 支持 macOS、Linux、Windows

## 贡献者

- @yinglichina8848 - 项目负责人
- 所有参与代码审查和测试的团队成员
- AI 协作开发团队

## 相关文档

- **架构文档**: docs/ARCHITECTURE.md
- **工程自动化**: docs/ENGINEERING_AUTOMATION.md
- **分支治理**: docs/BRANCH_GOVERNANCE.md
- **发布流程**: docs/VERSION_PROMOTION_SOP.md
- **升级指南**: docs/releases/v1.1.0/03-upgrade-guide.md

## 后续规划

### v1.2.0 开发计划
- Storage trait 化 (多后端支持)
- Optimizer 规则系统 (谓词下推、投影裁剪)
- 向量化执行 (批量处理)
- 技术债务清理

### 长期目标
- 成为生产级 SQL 引擎
- 支持更多 SQL 特性
- 提供完整的生态系统

## 支持

如有问题或建议，请通过以下方式联系：
- GitHub Issues: https://github.com/minzuuniversity/sqlrustgo/issues
- 项目讨论组

---

**SQLRustGo 团队**
2026-03-10

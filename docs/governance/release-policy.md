# SQLRustGo 发布政策

## 文档信息
- **版本**: v1.0
- **创建日期**: 2026-02-21
- **生效日期**: 2026-02-21
- **适用范围**: SQLRustGo 项目所有版本发布

## 核心原则

本政策定义了 SQLRustGo 项目的发布流程和不可违反的规则，确保发布过程的可审计性、可追溯性和安全性。

## 不可违反规则

### 1. 发布 Tag 永不删除
- **规则**: 禁止删除已发布的 GA Tag
- **禁止行为**:
  - 删除任何形式的发布 Tag
  - 重建同名 Tag
  - 修改已发布 Tag 的指向
- **错误处理**:
  - 若 Tag 创建错误，必须 bump patch 版本号
  - 例如: v1.0.0 错误 → 发布 v1.0.1

### 2. main 永远只接受 release 合并
- **规则**: main 分支只能接受来自 release 分支的合并
- **禁止行为**:
  - 直接 push 到 main
  - 绕过 PR 流程
  - 强制推送 (force push)
- **要求**:
  - 必须通过 PR
  - 必须通过 CI 检查
  - 必须获得批准

### 3. release 分支冻结后不可提交
- **规则**: release 分支在发布后进入冻结状态
- **允许行为**:
  - 只能 cherry-pick 修复性变更
  - 必须通过 PR
  - 必须通过 CI 检查
- **禁止行为**:
  - 直接提交
  - 功能新增
  - 重构

### 4. RC 分支必须通过完整门禁
- **规则**: RC 分支必须通过所有门禁检查才能进入发布流程
- **门禁要求**:
  - CI 100% 通过
  - 测试覆盖率达标 (≥ 80%)
  - 安全扫描通过 (无高危漏洞)
  - 依赖审计通过
  - 文档完整性检查通过

### 5. 发布证据必须归档
- **规则**: 发布证据必须完整归档且不可修改
- **要求**:
  - 版本目录 (`docs/releases/vX.X.X/`) 不可修改
  - 包含完整的发布证据链
  - 包含审计所需的所有文档

## 分支模型

### 分支类型及职责
| 分支类型 | 命名规范 | 职责 | 保护状态 |
|---------|---------|------|---------|
| main | main | 稳定版本分支，只接受 release 合并 | 严格保护 |
| develop | develop | 主要开发分支，新功能开发 | 建议保护 |
| release | release/vX.X.X | 正式稳定版本分支 | 严格保护 |
| RC | rc/vX.X.X | 发布候选分支，门禁验收 | 严格保护 |
| feature | feature/xxx | 功能开发分支 | 可选保护 |
| hotfix | hotfix/xxx | 紧急修复分支 | 可选保护 |

### 正确的分支推进顺序
```
develop → rc/vX.X.X → release/vX.X.X → tag → main
```

## 发布流程

### 1. RC 阶段
- **目标**: 门禁验收
- **流程**:
  1. 从 develop 创建 RC 分支
  2. 执行完整门禁检查
  3. 修复发现的问题
  4. 完成验收报告
  5. 准备发布证据

### 2. GA 发布
- **目标**: 正式版本发布
- **流程**:
  1. 从 RC 分支创建 release 分支
  2. 执行最终门禁检查
  3. 创建不可变 Tag
  4. 发布 GitHub Release
  5. 合并到 main 分支

### 3. 后续管理
- **目标**: 版本维护
- **流程**:
  1. 冻结 release 分支
  2. 冻结 RC 分支
  3. 从 main 创建 develop 分支
  4. 开始下一版本规划

## 强制约束

### GitHub 分支保护
- **main 分支**:
  - Require pull request
  - Require status checks
  - Require approvals
  - Do NOT allow force push
  - Do NOT allow deletion
  - Include administrators

- **release/* 分支**:
  - 与 main 分支相同的保护规则

### Tag 保护
- **规则**:
  - Pattern: v*
  - Do NOT allow deletion
  - Include administrators

### CI 强制检查
- **发布时检查**:
  - 重新运行完整测试套件
  - 校验版本号一致性
  - 校验 CHANGELOG 包含版本信息
  - 校验发布证据目录存在且完整
  - 校验安全扫描结果

## 发布证据链

### 必须包含的文档
- **发布摘要**: `docs/releases/vX.X.X/00-release-summary.md`
- **发布说明**: `docs/releases/vX.X.X/01-release-notes.md`
- **测试报告**: `docs/releases/vX.X.X/03-test-report.md`
- **覆盖率报告**: `docs/releases/vX.X.X/04-coverage-report.md`
- **安全扫描报告**: `docs/releases/vX.X.X/05-security-scan-report.md`
- **性能报告**: `docs/releases/vX.X.X/06-performance-report.md`
- **依赖审计**: `docs/releases/vX.X.X/07-dependency-audit.md`
- **License 合规**: `docs/releases/vX.X.X/08-license-compliance.md`
- **CI 构建日志**: `docs/releases/vX.X.X/09-ci-build-log.md`
- **审批记录**: `docs/releases/vX.X.X/10-approval-record.md`

### 文档要求
- 所有文档必须完整
- 所有文档必须真实反映发布状态
- 所有文档必须可审计

## 违规处理

### 轻微违规
- **定义**: 流程执行不严格但未影响发布质量
- **处理**: 记录在发布回顾中，要求改进

### 严重违规
- **定义**: 违反核心原则，影响发布质量或审计
- **处理**:
  - 立即停止发布流程
  - 进行全面审查
  - 采取纠正措施
  - 记录在案

### 特别严重违规
- **定义**: 违反不可篡改原则，破坏审计链
- **处理**:
  - 撤销发布
  - 重新发布新版本
  - 团队内部通报
  - 修订流程防止再次发生

## 版本管理

### 版本号规范
- **格式**: vX.Y.Z
- **X**: 主版本号，重大变更
- **Y**: 次版本号，功能新增
- **Z**: 补丁版本号，bug 修复

### Tag 命名规范
- **格式**: vX.Y.Z
- **示例**: v1.0.0, v1.0.1, v1.1.0

### 发布频率
- **主版本**: 重大架构变更时
- **次版本**: 功能显著增加时
- **补丁版本**: 发现 bug 需要修复时

## 审批流程

### 发布审批
- **RC 审批**:
  - QA Lead
  - Technical Lead
  - Release Manager

- **GA 审批**:
  - QA Lead
  - Technical Lead
  - Product Owner
  - Release Manager

### 审批要求
- 所有审批必须基于完整的发布证据
- 所有审批必须在 GitHub 上记录
- 所有审批必须明确表示同意或拒绝

## 培训要求

### 团队培训
- **发布管理培训**:
  - Tag 管理
  - 分支保护
  - 流程执行

- **工程治理培训**:
  - 审计准备
  - 合规要求
  - 风险控制

- **自动化技能培训**:
  - CI/CD 配置
  - 脚本开发
  - 工具使用

### 认证要求
- 所有参与发布流程的团队成员必须通过发布管理培训
- Release Manager 必须具备发布流程认证

## 工具与自动化

### 推荐工具
- **版本管理**: Git
- **CI/CD**: GitHub Actions
- **代码质量**: Clippy, Rustfmt
- **测试覆盖率**: Tarpaulin
- **安全扫描**: cargo-audit
- **依赖管理**: Dependabot, Renovate
- **文档构建**: mdbook

### 自动化目标
- **版本推进自动化**
- **测试自动化**
- **部署自动化**
- **发布证据生成自动化**

## 持续改进

### 发布回顾
- **每次发布后**:
  - 进行发布回顾
  - 识别问题
  - 制定改进措施
  - 更新发布政策

### 政策修订
- **修订频率**: 每季度至少一次
- **修订流程**:
  - 收集反馈
  - 分析问题
  - 提出修订方案
  - 团队讨论
  - 批准实施

### 最佳实践分享
- 定期分享发布管理最佳实践
- 学习行业标准
- 持续优化流程

## 责任分工

### 角色与职责
| 角色 | 职责 |
|------|------|
| **Release Manager** | 负责发布流程执行，协调各方，确保流程合规 |
| **QA Lead** | 负责门禁验收，测试执行，质量保证 |
| **Technical Lead** | 负责技术审核，架构评估，风险控制 |
| **Product Owner** | 负责产品功能审核，发布内容确认 |
| **DevOps Engineer** | 负责 CI/CD 维护，自动化工具开发 |
| **Documentation Lead** | 负责发布证据管理，文档完整性 |

### 责任追究
- **Release Manager**: 对整个发布流程负责
- **QA Lead**: 对测试结果负责
- **Technical Lead**: 对技术质量负责
- **Product Owner**: 对产品功能负责
- **DevOps Engineer**: 对 CI/CD 可靠性负责
- **Documentation Lead**: 对文档完整性负责

## 生效与实施

### 生效日期
- **本政策自发布之日起生效**
- **适用于所有未来版本发布**

### 实施计划
1. **政策发布**: 2026-02-21
2. **培训实施**: 2026-02-22 至 2026-02-28
3. **工具配置**: 2026-02-22 至 2026-02-28
4. **流程试运行**: 2026-03-01 至 2026-03-15
5. **全面实施**: 2026-03-16 起

### 监督与执行
- **监督机构**: 项目管理委员会
- **执行机构**: Release Manager
- **投诉渠道**: GitHub Issues

## 附录

### 参考文档
- **分支治理**: docs/BRANCH_GOVERNANCE.md
- **版本推进**: docs/VERSION_PROMOTION_SOP.md
- **工程自动化**: docs/ENGINEERING_AUTOMATION.md
- **发布门禁**: docs/RELEASE_GATES_COMPREHENSIVE.md
- **发布回顾**: docs/governance/v1.0-release-retrospective.md

### 工具清单
- **版本状态检查**: scripts/version/status.sh
- **门禁检查**: scripts/gate/gate.sh
- **发布准备**: scripts/release/prepare.sh
- **审计工具**: scripts/audit/check.sh

### 联系人
- **Release Manager**: @yinglichina8848
- **QA Lead**: @sonaheartopen
- **Technical Lead**: @yinglichina8848
- **Product Owner**: @yinglichina8848
- **DevOps Engineer**: @yinglichina8848
- **Documentation Lead**: @yinglichina8848

---

**政策审核**

| 角色 | 姓名 | 审核状态 | 日期 |
|------|------|----------|------|
| 技术负责人 | @yinglichina8848 | ✅ 已审核 | 2026-02-21 |
| 产品负责人 | @yinglichina8848 | ✅ 已审核 | 2026-02-21 |
| QA 负责人 | @yinglichina8848 | ✅ 已审核 | 2026-02-21 |
| 项目负责人 | @yinglichina8848 | ✅ 已审核 | 2026-02-21 |

**SQLRustGo 团队**
2026-02-21
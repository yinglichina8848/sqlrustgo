# SQLRustGo v1.0.0 GA 发布时间表

## 📅 发布日期
- **计划发布日期**: 2026-02-21
- **发布类型**: 首次正式 GA 发布
- **版本号**: v1.0.0

## ⏰ 小时级操作时间表

### 🕘 09:00 — 冻结确认

**目标**: 确认可以进入最终发布流程

**检查项**:
- [ ] 所有 RC Gate 项已勾选
- [ ] baseline 分支 CI 全绿
- [ ] 无未合并 PR
- [ ] 无 open blocker issue
- [ ] 覆盖率达到标准 (≥ 80%)
- [ ] 安全扫描 0 critical

**执行命令**:
```bash
git checkout baseline
git pull origin baseline
```

**记录**:
- Commit hash: 
- CI 状态: 
- 覆盖率: 
- 安全扫描: 

### 🕙 10:00 — 最终版本号确认

**确认版本号**: v1.0.0

**更新文件**:
- [ ] VERSION 文件
- [ ] CURRENT_VERSION.md
- [ ] CHANGELOG.md

**提交命令**:
```bash
git commit -am "chore: release v1.0.0"
git push origin baseline
```

**等待**: CI 全部通过

### 🕚 11:00 — 打 Tag

**执行命令**:
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

**检查**:
- [ ] Tag 是否出现在 GitHub
- [ ] Tag commit 是否正确

### 🕛 12:00 — 构建正式发布产物

**构建内容**:
- [ ] 构建 release binary
- [ ] 构建 docker image
- [ ] 生成 checksum
- [ ] 保存 build log

**记录**:
- 构建环境: 
- 构建时间: 
- commit hash: 
- 构建机器: 

### 🕐 13:00 — 生成发布证据目录

**创建目录**:
```bash
mkdir -p docs/releases/v1.0.0/
```

**归档文件**:
- [ ] 测试报告
- [ ] 覆盖率报告
- [ ] 安全扫描
- [ ] CI 日志
- [ ] 审批记录

**提交命令**:
```bash
git add docs/releases/v1.0.0
git commit -m "docs: archive release evidence for v1.0.0"
git push origin baseline
```

### 🕑 14:00 — 创建 GitHub Release

**步骤**:
1. 选择 Tag: v1.0.0
2. 填写 Release Notes
3. 上传构建产物
4. 标记为 Latest
5. 不要勾选 Pre-release

### 🕒 15:00 — 验证发布

**验证步骤**:
- [ ] 从 Release 页面下载构建产物
- [ ] 本地重新安装
- [ ] 运行 smoke test
- [ ] 校验 checksum

### 🕓 16:00 — 宣布 GA

**更新内容**:
- [ ] 更新 README Badge
- [ ] 更新 CURRENT_VERSION
- [ ] 发布公告
- [ ] 通知团队

### 🕔 17:00 — 锁定发布

**保护措施**:
- [ ] baseline 设置保护规则
- [ ] 禁止 force push
- [ ] 禁止删除 tag
- [ ] 发布分支进入维护模式

## 📝 GitHub Release 页面模板

### 标题
```
🚀 SQLRustGo v1.0.0
```

### 基本信息
```
Release Date: 2026-02-21
Commit: abcdef123456
Type: GA (General Availability)
```

### 🎯 Overview
```
This is the first stable General Availability release of SQLRustGo.

This release completes:
- Core engine implementation
- Governance system
- CI/CD integration
- RC validation cycle
```

### ✨ New Features
```
- SQL 解析引擎
- 查询执行引擎
- 优化器
- 存储引擎
```

### 🐛 Bug 修复
```
- Fix issue #123
- Fix performance regression in X
```

### 🔒 Security
```
- Dependency audit passed
- No critical vulnerabilities detected
```

### 📊 Quality Metrics
```
- Test Coverage: 84%
- All CI checks passed
- Security scan: 0 Critical / 0 High
```

### 📦 Artifacts
```
- sqlrustgo-v1.0.0-darwin-amd64.tar.gz
- sqlrustgo-v1.0.0-linux-amd64.tar.gz
- docker image: sqlrustgo:1.0.0
```

### 📝 Upgrade Notes
```
- No breaking changes
- Compatible with previous RC1
```

### ⚠ Known Issues
```
- Minor logging inconsistency (non-blocking)
```

## 🧰 门禁自动化脚本结构

### 建议目录结构
```
scripts/
  gate/
    gate.sh              # 总入口脚本
    check_tests.sh       # 测试检查
    check_coverage.sh    # 覆盖率检查
    check_security.sh    # 安全检查
    check_docs.sh        # 文档检查
    check_version.sh     # 版本检查
```

### 1. 总入口脚本 (gate.sh)
```bash
#!/usr/bin/env bash

set -e

echo "Running Release Gate..."

./scripts/gate/check_tests.sh
./scripts/gate/check_coverage.sh
./scripts/gate/check_security.sh
./scripts/gate/check_docs.sh
./scripts/gate/check_version.sh

echo "All gates passed."
```

### 2. 覆盖率门禁 (check_coverage.sh)
```bash
#!/usr/bin/env bash

COVERAGE=$(cat coverage.txt)
REQUIRED=80

if [ "$COVERAGE" -lt "$REQUIRED" ]; then
  echo "Coverage too low!"
  exit 1
fi

echo "Coverage check passed: $COVERAGE%"
```

### 3. 版本一致性检查 (check_version.sh)
```bash
#!/usr/bin/env bash

TAG=$(git describe --tags --exact-match 2>/dev/null || echo "")

if [ -z "$TAG" ]; then
  echo "Not on a release tag!"
  exit 1
fi

echo "Version check passed: $TAG"
```

### 4. CI 集成

**GitHub Actions 配置**:
```yaml
- name: Release Gate
  run: bash scripts/gate/gate.sh
```

## 🎯 执行优先级

### 当前阶段
1. 用时间表发布 v1.0.0
2. 用 Release 模板写页面
3. 准备发布证据目录

### 下个版本
1. 接入自动 Gate 脚本
2. 完善 CI/CD 集成
3. 实现全自动化发布流程

## 📋 发布准备清单

### 文档准备
- [ ] GA 发布时间表文档
- [ ] GitHub Release 页面内容
- [ ] 发布证据目录结构
- [ ] 审批记录文档

### 技术准备
- [ ] 版本号确认和更新
- [ ] Tag 准备
- [ ] 构建产物准备
- [ ] 门禁检查脚本

### 发布执行
- [ ] 冻结确认
- [ ] 版本更新
- [ ] Tag 创建
- [ ] 构建发布
- [ ] 证据归档
- [ ] GitHub Release 创建
- [ ] 发布验证
- [ ] GA 宣布
- [ ] 发布锁定

## 🟢 发布状态

| 阶段 | 状态 | 开始时间 | 完成时间 | 负责人 |
|------|------|----------|----------|--------|
| 准备阶段 | ⏳ | | | yinglichina8848 |
| 冻结确认 | ⏳ | | | yinglichina8848 |
| 版本确认 | ⏳ | | | yinglichina8848 |
| 打 Tag | ⏳ | | | yinglichina8848 |
| 构建发布 | ⏳ | | | yinglichina8848 |
| 证据归档 | ⏳ | | | yinglichina8848 |
| GitHub Release | ⏳ | | | yinglichina8848 |
| 验证发布 | ⏳ | | | yinglichina8848 |
| 宣布 GA | ⏳ | | | yinglichina8848 |
| 锁定发布 | ⏳ | | | yinglichina8848 |

---

*文档版本: v1.0.0*
*最后更新: 2026-02-21*
*负责人: @yinglichina8848*

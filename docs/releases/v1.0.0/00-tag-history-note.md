# v1.0.0 Tag 历史记录

## 基本信息

- **版本**: v1.0.0
- **文档创建日期**: 2026-02-21
- **记录目的**: 透明化Tag管理历史，确保审计链完整

## Tag 历史

### 初始创建
- **时间**: 2026-02-21
- **Commit**: 2d191aaaaffa3799865ef6989e40cdc9137598a7
- **操作**: 从 rc/v1.0.0-1 分支创建正式 GA Tag
- **状态**: 当前有效

### 历史变更

#### 2026-02-21 Tag 重建
- **原因**: 解决与 alpha 阶段 Tag 冲突问题
- **操作**: 
  - 删除旧 Tag: `git tag -d v1.0.0`
  - 删除远程 Tag: `git push origin --delete v1.0.0`
  - 重建 Tag: `git tag -a v1.0.0 -m "Release v1.0.0"`
  - 推送 Tag: `git push origin v1.0.0`
- **影响**: 
  - Tag 指向新的 commit
  - GitHub Release 页面自动更新
  - 构建产物重新生成

## 审计信息

### 当前 Tag 状态
- **Tag**: v1.0.0
- **Commit**: 2d191aaaaffa3799865ef6989e40cdc9137598a7
- **分支**: release/v1.0.0
- **GitHub Release**: https://github.com/minzuuniversity/sqlrustgo/releases/tag/v1.0.0

### 相关分支
- **release/v1.0.0**: 正式稳定版本分支
- **rc/v1.0.0-1**: 已完成历史使命，可冻结
- **main**: 主分支，需要从 v1.0.0 新建 develop 分支

## 修复措施

### 已执行
1. ✅ 确认 v1.0.0 Tag 指向最终 commit
2. ✅ 创建此历史记录文档，透明化过程
3. ✅ 验证 GitHub Release 页面绑定正确 commit
4. ✅ 确认构建产物完整

### 计划执行
1. ⏳ 冻结 release/v1.0.0 分支，禁止 force push 和删除
2. ⏳ 从 v1.0.0 创建 develop 分支，作为下一版本开发分支
3. ⏳ 明确 main 分支定位为稳定版本分支

## 经验教训

1. **Tag 管理**:
   - Tag 应视为不可变发布证据
   - 避免删除已发布的 Tag
   - 如需要修正，应创建新的版本号

2. **分支保护**:
   - 严格遵守 PR 流程，不绕过分支保护
   - 确保 CI 完全通过后再合并
   - 保持分支历史清晰可追溯

3. **版本同步**:
   - 建立 RC → release → main 的 fast-forward 流程
   - 避免 main 分支在冻结期间产生独立历史
   - 定期进行版本状态体检

## 参考命令

### 验证当前状态
```bash
# 查看 Tag 信息
git show v1.0.0

# 查看分支 commit
git rev-parse release/v1.0.0

# 验证一致性
git rev-parse v1.0.0 == git rev-parse release/v1.0.0
```

### 分支管理
```bash
# 从 v1.0.0 创建 develop 分支
git checkout v1.0.0
git checkout -b develop
git push origin develop

# 设置 main 为 stable
git checkout main
git reset --hard v1.0.0
git push origin main --force-with-lease
```

## 责任人

- **记录创建**: @yinglichina8848
- **审核**: 团队
- **日期**: 2026-02-21
# SQLRustGo 并行开发协作指南

## 分支架构

```
                    PR 交叉评审
                        │
    ┌───────────────────┼───────────────────┐
    │                   │                   │
    ▼                   ▼                   ▼
┌─────────┐      ┌─────────────┐      ┌─────────────┐
│ index-  │      │ network-    │      │ docs-       │
│ executor│      │ protocol    │      │ testing     │
│ (你)    │      │ (MacBookPro)      │ (另一实例)  │
└────┬────┘      └──────┬──────┘      └──────┬──────┘
     │                   │                   │
     └───────────────────┼───────────────────┘
                        ▼
                 ┌─────────────┐
                 │  baseline   │
                 │  (合并点)   │
                 └──────┬──────┘
                        │
                        ▼ PR(maintainer 审批)
                 ┌─────────────┐
                 │    main    │ (锁定)
                 └─────────────┘
```

## 分支说明

| 分支 | 用途 | 开发者 |
|------|------|--------|
| `main` | 主分支，锁定，只能通过 PR 合并 | - |
| `baseline` | 基线版本，所有功能分支基于此创建 | - |
| `feature/index-executor` | 索引优化 + 执行引擎 | 本地 Claude (你) |
| `feature/network-protocol` | 网络协议 + MySQL 兼容 | MacBookPro Claude |
| `feature/docs-testing` | 测试补全 + 文档完善 | 本地另一个 Claude |

## 开发流程

### 1. 开始新任务
```bash
# 确保基于 baseline
git checkout baseline
git pull origin baseline

# 创建或切换到你的功能分支
git checkout feature/xxx
```

### 2. 开发完成后
```bash
# 提交你的更改
git add .
git commit -m "feat: 完成 xxx 功能"

# 推送到远程
git push origin feature/xxx
```

### 3. 创建 PR
- 目标: `baseline`
- 源: `feature/xxx`
- 需要至少 1 个其他分支的评审

## PR 评审规则

### 评审要求
| PR 来源 | 评审者要求 |
|---------|------------|
| feature/index-executor | 需要 network-protocol **或** docs-testing 评审 |
| feature/network-protocol | 需要 index-executor **或** docs-testing 评审 |
| feature/docs-testing | 需要 index-executor **或** network-protocol 评审 |

### 评审要点
- [ ] 代码逻辑正确性
- [ ] 是否有测试覆盖
- [ ] 是否符合项目规范
- [ ] 是否有冲突需要解决

### 合并条件
- [ ] 所有 CI 检查通过
- [ ] 至少 1 个其他分支的评审通过
- [ ] 无冲突

## 任务分配

### feature/index-executor (本地 Claude)
- B+ Tree 索引完善
- Volcano 执行引擎优化
- 查询优化

### feature/network-protocol (MacBookPro)
- MySQL 协议解析
- TCP 服务器/客户端
- 连接管理

### feature/docs-testing (本地另一个 Claude)
- 测试覆盖补全 (目标 80%+)
- 文档完善
- README 更新

## 沟通协调

1. **每日同步**: 各分支开发者通过微信/消息沟通进度
2. **冲突解决**: 如有冲突，先在本地解决后再推送
3. **问题讨论**: 通过 GitHub Issue 或 PR 评论

## 相关链接

- 仓库: https://github.com/yinglichina8848/sqlrustgo
- baseline: https://github.com/yinglichina8848/sqlrustgo/tree/baseline
- index-executor: https://github.com/yinglichina8848/sqlrustgo/tree/feature/index-executor
- network-protocol: https://github.com/yinglichina8848/sqlrustgo/tree/feature/network-protocol
- docs-testing: https://github.com/yinglichina8848/sqlrustgo/tree/feature/docs-testing

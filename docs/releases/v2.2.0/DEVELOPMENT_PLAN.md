# SQLRustGo v2.2 开发计划

> **版本**: 2.2
> **日期**: 2026-03-28
> **目标**: 故障自动化 - 自动切换、读写分离、Web运维面板
> **前置条件**: v2.1 GA 发布
> **预计周期**: 1 个月
> **Agent**: 多Agent并行开发

---

## 1. 版本目标

v2.2 是"省心线中级版"，实现故障自动检测和切换，1个人能值守，不用人工干预。

---

## 2. 任务分解

### 2.1 高可用自动化 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1201 | 自动故障检测 (心跳 + 超时) | 12 | OpenCode A | P0 |
| #1202 | 自动主从切换 (promote slave) | 15 | OpenCode A | P0 |
| #1203 | 脑裂防护 (fencing机制) | 8 | OpenCode A | P0 |
| #1204 | 故障恢复通知 (webhook/邮件) | 6 | OpenCode A | P0 |

### 2.2 读写分离 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1211 | 读写分离代理 (proxy层) | 18 | Claude A | P0 |
| #1212 | 备库延迟检测 | 8 | Claude A | P0 |
| #1213 | 延迟阈值自动降级 | 6 | Claude A | P0 |

### 2.3 运维界面 (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1221 | 运维 Web 面板 (状态监控) | 20 | OpenCode B | P1 |
| #1222 | SQL 执行界面 | 10 | OpenCode B | P1 |
| #1223 | 备份管理界面 | 8 | OpenCode B | P1 |

### 2.4 运维增强 (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1231 | SHOW PROCESSLIST 实现 | 10 | OpenCode A | P1 |
| #1232 | KILL 命令实现 | 6 | OpenCode A | P1 |
| #1233 | SHOW STATUS 完善 | 8 | OpenCode A | P1 |

---

## 3. Issue 清单

```bash
# 创建所有 v2.2 Issue
gh issue create --title "[v2.2][P0] 自动故障检测" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 自动主从切换" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 脑裂防护机制" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 故障恢复通知" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 读写分离代理" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 备库延迟检测" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P0] 延迟阈值自动降级" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] 运维 Web 面板" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] SQL 执行界面" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] 备份管理界面" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] SHOW PROCESSLIST" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] KILL 命令" --body "..." --label "enhancement"
gh issue create --title "[v2.2][P1] SHOW STATUS 完善" --body "..." --label "enhancement"
```

---

## 4. 开发顺序

```
Week 1: 高可用核心
  ├── #1201 自动故障检测
  ├── #1202 自动主从切换
  └── #1203 脑裂防护

Week 2: 读写分离 + 通知
  ├── #1211 读写分离代理
  ├── #1204 故障通知
  └── #1212 备库延迟检测

Week 3: 运维界面
  ├── #1221 Web 面板核心
  ├── #1222 SQL 执行界面
  └── #1231 SHOW PROCESSLIST

Week 4: 收尾
  ├── #1223 备份管理界面
  ├── #1232 KILL 命令
  ├── #1233 SHOW STATUS
  └── #1213 延迟降级
```

---

## 5. 交付物

- [ ] 自动故障检测 (心跳 < 5s)
- [ ] 自动主从切换 (< 30s 完成)
- [ ] 脑裂防护机制
- [ ] 故障通知 webhook
- [ ] 读写分离代理 (读性能 +100%)
- [ ] 备库延迟检测 + 自动降级
- [ ] 运维 Web 面板
- [ ] SHOW PROCESSLIST / KILL
- [ ] 性能基准: 50 并发 ≥ 2000 QPS (v2.1 +33%)

---

## 6. 故障切换 SLA

| 指标 | 目标 |
|------|------|
| 故障检测时间 | < 5 秒 |
| 自动切换时间 | < 30 秒 |
| 数据丢失 | 0 (同步复制模式) |
| 脑裂防护 | 100% |

---

## 7. 里程碑

| 日期 | 里程碑 |
|------|--------|
| Week 1 | 自动故障切换核心完成 |
| Week 2 | 读写分离上线 |
| Week 3 | Web 面板上线 |
| Week 4 | v2.2 GA 发布 |

---

**状态**: 📋 规划完成，待创建Issue

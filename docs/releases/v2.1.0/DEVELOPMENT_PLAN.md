# SQLRustGo v2.1 开发计划

> **版本**: 2.1
> **日期**: 2026-03-28
> **目标**: 运维自动化 - 监控、告警、备份CLI、健康检查
> **前置条件**: v2.0 GA 发布
> **预计周期**: 1 个月
> **Agent**: 多Agent并行开发

---

## 1. 版本目标

v2.1 是"省心线初级版"，让 DBA 能通过 Prometheus/Grafana 观察数据库状态，有慢查询日志可分析，有一键备份工具。

---

## 2. 任务分解

### 2.1 监控与可观测性 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1101 | Prometheus 监控端点 (QPS/连接数/缓存命中率/锁等待) | 12 | OpenCode A | P0 |
| #1102 | 慢查询日志 (记录 >阈值的SQL，支持分析工具) | 15 | OpenCode A | P0 |
| #1103 | 健康检查端点 (/health, /ready, /metrics) | 8 | OpenCode A | P0 |

### 2.2 备份与恢复 (P0)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1111 | mysqldump 兼容导入工具 | 10 | OpenCode B | P0 |
| #1112 | 物理备份 CLI (基于文件快照) | 12 | OpenCode B | P0 |
| #1113 | 增量备份工具 | 8 | OpenCode B | P0 |
| #1114 | 备份恢复验证工具 | 6 | OpenCode B | P0 |

### 2.3 配置与运维 (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1121 | 配置热更新能力 (不改配置不重启) | 12 | Claude A | P1 |
| #1122 | 版本升级脚本 (不停服滚动升级) | 8 | Claude A | P1 |
| #1123 | 日志轮转 + 错误分级 | 6 | OpenCode A | P1 |

### 2.4 性能调优 (P1)

| Issue | 任务 | PR估算 | Agent | 优先级 |
|-------|------|--------|-------|--------|
| #1131 | 查询缓存优化 (基于表修改的失效策略) | 10 | Claude B | P1 |
| #1132 | 连接池实现 (executor 复用) | 8 | Claude B | P1 |

---

## 3. Issue 清单

```bash
# 创建所有 v2.1 Issue
gh issue create --title "[v2.1][P0] Prometheus 监控端点" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] 慢查询日志系统" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] 健康检查端点" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] mysqldump 兼容导入工具" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] 物理备份 CLI" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] 增量备份工具" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P0] 备份恢复验证工具" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P1] 配置热更新能力" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P1] 版本升级脚本" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P1] 日志轮转与错误分级" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P1] 查询缓存优化" --body "..." --label "enhancement"
gh issue create --title "[v2.1][P1] 连接池实现" --body "..." --label "enhancement"
```

---

## 4. 开发顺序

```
Week 1: 监控端点 + 健康检查
  ├── #1101 Prometheus 端点
  └── #1103 健康检查

Week 2: 慢查询 + 备份CLI
  ├── #1102 慢查询日志
  ├── #1111 mysqldump 导入
  └── #1112 物理备份CLI

Week 3: 备份恢复 + 配置热更新
  ├── #1113 增量备份
  ├── #1114 恢复验证
  └── #1121 配置热更新

Week 4: 收尾 + 调优
  ├── #1122 升级脚本
  ├── #1123 日志轮转
  ├── #1131 查询缓存
  └── #1132 连接池
```

---

## 5. 交付物

- [ ] Prometheus 监控端点可访问
- [ ] Grafana Dashboard 模板
- [ ] 慢查询日志可分析
- [ ] 一键物理备份 CLI
- [ ] 增量备份工具
- [ ] 备份恢复验证通过
- [ ] 配置热更新不重启
- [ ] 版本升级脚本可用
- [ ] 性能基准: 50 并发 ≥ 1500 QPS (v2.0 +30%)

---

## 6. 里程碑

| 日期 | 里程碑 |
|------|--------|
| Week 1 | 监控 + 健康检查上线 |
| Week 2 | 备份工具链完成 |
| Week 3 | 配置热更新完成 |
| Week 4 | v2.1 GA 发布 |

---

**状态**: 📋 规划完成，待创建Issue

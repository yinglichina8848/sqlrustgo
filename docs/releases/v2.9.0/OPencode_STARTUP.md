# OpenCode 启动指令

## Z6G4 OpenCode (Phase G + S)
- 分工: Issue #116 (可信任治理) + #117 (SQL可证明性)
- 工作目录: ~/workspace/dev/yinglichina163/sqlrustgo
- 启动:
  git fetch origin develop/v2.9.0
  git checkout develop/v2.9.0
  git pull origin develop/v2.9.0
  cat docs/releases/v2.9.0/DEVELOPMENT_PLAN.md
  git checkout -b feature/g01-r-gate-extend

## Z440 OpenCode (Phase C + D)
- 分工: Issue #118 (SQL兼容性) + #119 (分布式增强)
- 工作目录: ~/workspace/dev/sqlrustgo
- 启动:
  git fetch origin develop/v2.9.0
  git checkout develop/v2.9.0
  git pull origin develop/v2.9.0
  git checkout -b feature/c01-corpus-80pct

## OpenCode 3 (Phase E)
- 分工: Issue #120 (生产就绪)
- 工作目录: ~/workspace/dev/openheart/sqlrustgo
- 启动:
  git fetch origin develop/v2.9.0
  git checkout develop/v2.9.0
  git pull origin develop/v2.9.0
  git checkout -b feature/e01-sysbench

## 报告规则
1. 每次提交 PR 必须在对应 ISSUE 中评论，注明进度和变更内容
2. PR 描述引用 ISSUE: Closes #116 等
3. 每周至少报告一次进度
4. 阻塞问题及时在 ISSUE 中提出
5. Hermes 负责跟踪 ISSUE 和 PR 状态

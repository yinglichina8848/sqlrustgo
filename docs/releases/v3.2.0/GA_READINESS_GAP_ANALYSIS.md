# v3.2.0 GA Readiness 差距分析

> 日期: 2026-05-17
> 分支: develop/v3.2.0 (HEAD=4752d9977)
> 规范: GATE_SPEC_MASTER.md §5 (G-Gate)

---

## GA 入口条件检查

| # | 入口条件 | 状态 | 证据 |
|---|----------|------|------|
| 1 | R-Gate 已通过 | ⚠️ | 核心项✅ R1✅ R5⏳ R6⏳ R9⏳ R10⏳ |
| 2 | Point Select QPS ≥10,000 | ⚠️ | baseline 324K ✅ 但需刷新对比 |
| 3 | UPDATE QPS ≥5,000 | ⚠️ | baseline 58K ✅ 但需刷新对比 |
| 4 | DELETE QPS ≥2,000 | ⚠️ | baseline 62K ✅ 但需刷新对比 |
| 5 | TPC-H SF=1 22/22 无OOM | ❌ | 无 tpch_data/ 数据目录 |
| 6 | SQL Corpus ≥98% | ✅ | 10/10=100% (1 ignored) |
| 7 | 所有已知问题已关闭 | ⚠️ | 需审查 Gitea Issues |

---

## G-Gate 逐项检查

| # | 检查项 | 标准 | 结果 | 阻塞? |
|---|--------|------|------|-------|
| G1 | Release Build | 无错误 | ✅ PASS | — |
| G2 | Test (all-features) | 100% | ⚠️ 核心✅ MySQL 集成3fail | 🔴 G13 |
| G3 | Clippy | 零警告 | ✅ PASS | — |
| G4 | Format | 无错误 | ✅ PASS | — |
| G5 | Coverage L1 | ≥85% | ⏳ 测量中 | 🟡 |
| G6 | Security | 无高危漏洞 | ⏳ GitHub不可达 | 🟡 |
| G7 | Point Select QPS | ≥10,000 | ⚠️ baseline 324K，待刷新 | 🟡 |
| G8 | UPDATE QPS | ≥5,000 | ⚠️ baseline 58K，待刷新 | 🟡 |
| G9 | DELETE QPS | ≥2,000 | ⚠️ baseline 62K，待刷新 | 🟡 |
| G10 | TPC-H SF=1 | 22/22 无OOM | ❌ 无数据 | 🔴 |
| G11 | SQL Corpus | ≥98% | ✅ 10/10=100% | — |
| G12 | B-S稳定性 | 全部PASS | ✅ 13/13=100% | — |
| G13 | MySQL Protocol | 连接成功 | ❌ "Could not setup connection" | 🔴 |

---

## 🔴 阻塞项 (必须修复才能 GA)

### 1. G13: MySQL Protocol 握手失败
- **症状**: `mysql::Conn::new()` 返回 `DriverError { Could not setup connection }`
- **根因**: sqlrustgo-mysql-server 的 MySQL 协议握手实现不完整
- **修复方向**: 
  - 调试 handshake 包交换 (ClientHandshake → ServerGreeting → AuthSwitch)
  - 检查 auth plugin 兼容性
  - 参考 `docs/releases/v3.1.0/MYSQL_PROTOCOL_OPTIMIZATION.md`

### 2. G10: TPC-H SF=1 数据缺失
- **症状**: `tpch_data/` 目录不存在
- **修复**: 
  - 生成 TPC-H SF=1 数据: `bash scripts/gate/setup_tpch_env.sh --sf 1`
  - 运行: `bash scripts/gate/check_tpch.sh --sf1`

---

## 🟡 待验证项

### G5: 覆盖率
- llvm-cov 后台运行中
- 目标: L1 整体 ≥85%

### G6: 安全审计
- cargo audit 无法连接 GitHub advisory-db (网络限制)
- 需要在 CI 或有网环境中运行
- 可暂时接受 (网络不可达 ≠ 有漏洞)

### G7/G8/G9: 性能基准刷新
- v3.2.0 baseline 已存在 (2026-05-16)
- 需要: `bash scripts/gate/check_perf_baseline.sh` (约需 15-30min)
- 对比 baseline 退化 ≤5% = PASS

---

## 结论

**当前状态: ❌ 不能进入 GA**

3个阻塞项必须解决:
1. G13 MySQL 协议握手 — 代码 bug，需 debug 修复
2. G10 TPC-H 数据 — 需生成数据文件
3. G5 覆盖率 — 等待测量完成

性能基准、安全审计可在修复阻塞项的同时并行完成。

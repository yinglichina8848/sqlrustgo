# SQLRustGo v2.6.0 72 小时长稳测试部署与监控指南

> **版本**: v2.6.0 GA
> **发布日期**: 2026-04-21
> **测试类型**: 72 小时长稳测试

---

## 一、概述

本文档描述如何在服务器上部署和执行 SQLRustGo v2.6.0 的 72 小时长稳测试，包括环境准备、部署步骤、监控方法和结果收集。

### 1.1 测试目标

| 目标 | 说明 |
|------|------|
| 内存泄漏检测 | 监控内存使用，确保无内存泄漏 |
| 崩溃检测 | 监控进程状态，确保无崩溃 |
| 死锁检测 | 监控线程状态，确保无死锁 |
| 性能稳定性 | 监控 QPS，确保稳定 ≥1000 |
| 资源使用 | 监控 CPU、磁盘 I/O、网络 |

### 1.2 测试内容

根据 `tests/regression_test.rs`，长稳测试包含：

| 测试项 | 说明 |
|--------|------|
| `long_run_stability_test` | 长时间运行稳定性测试 |
| `qps_benchmark_test` | QPS 基准测试 |

---

## 二、服务器要求

### 2.1 硬件要求

| 资源 | 最低配置 | 推荐配置 |
|------|----------|----------|
| CPU | 4 核 | 8 核+ |
| 内存 | 8GB | 16GB+ |
| 磁盘 | 20GB | 50GB+ SSD |
| 网络 | 100Mbps | 1Gbps |

### 2.2 软件要求

| 软件 | 版本要求 |
|------|----------|
| 操作系统 | Ubuntu 20.04+ / CentOS 8+ / macOS 12+ |
| Rust | 1.85+ |
| Cargo | 最新版 |
| tmux | 最新版 (用于保持会话) |
| htop | 最新版 (用于监控) |

### 2.3 网络要求

| 要求 | 说明 |
|------|------|
| SSH 访问 | 需要 SSH 访问服务器 |
| 端口开放 | 22 (SSH), 可选 3306 (MySQL 协议) |
| 网络稳定 | 需要稳定的网络连接 |

---

## 三、部署步骤

### 3.1 环境准备

```bash
# 1. 更新系统
sudo apt update && sudo apt upgrade -y  # Ubuntu
# sudo yum update -y  # CentOS

# 2. 安装必要软件
sudo apt install -y build-essential git curl tmux htop sysstat

# 3. 安装 Rust (如果没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 4. 验证安装
rustc --version
cargo --version
```

### 3.2 项目部署

```bash
# 1. 克隆项目
git clone https://github.com/minzuuniversity/sqlrustgo.git
cd sqlrustgo

# 2. 切换到 release/v2.6.0 分支
git checkout release/v2.6.0

# 3. 构建项目 (Release 模式，推荐)
cargo build --release

# 4. 验证构建
cargo build --release --verify
```

### 3.3 使用自动化脚本部署

```bash
# 使用提供的部署脚本
bash scripts/test/deploy_stability_test.sh

# 或者手动执行
bash scripts/test/deploy_stability_test.sh --branch release/v2.6.0 --hours 72
```

---

## 四、测试执行

### 4.1 手动执行

```bash
# 1. 进入项目目录
cd /path/to/sqlrustgo

# 2. 创建测试会话 (使用 tmux)
tmux new -s stability_test

# 3. 在 tmux 会话中执行测试
cargo test --test regression_test -- --test-threads=1 --ignored

# 4. 等待 72 小时，或使用后台执行
nohup cargo test --test regression_test -- --test-threads=1 --ignored > test.log 2>&1 &
```

### 4.2 使用脚本执行

```bash
# 执行 72 小时测试
bash scripts/test/deploy_stability_test.sh --hours 72

# 执行自定义时长测试
bash scripts/test/deploy_stability_test.sh --hours 24
```

### 4.3 测试参数说明

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--hours` | 测试时长 (小时) | 72 |
| `--branch` | Git 分支 | release/v2.6.0 |
| `--test-type` | 测试类型 (long_run/qps/all) | all |
| `--output-dir` | 输出目录 | ./test_results |

---

## 五、监控方法

### 5.1 使用监控脚本

```bash
# 1. 启动监控 (在新终端)
bash scripts/test/monitor_stability_test.sh

# 2. 或者指定输出目录
bash scripts/test/monitor_stability_test.sh --output-dir /path/to/results

# 3. 监控特定指标
bash scripts/test/monitor_stability_test.sh --metrics cpu,mem,disk,network
```

### 5.2 监控指标说明

| 指标 | 监控内容 | 告警阈值 |
|------|----------|----------|
| CPU | CPU 使用率 | > 90% 持续 5 分钟 |
| 内存 | 内存使用率、泄漏检测 | > 85% 或持续增长 |
| 磁盘 | 磁盘 I/O、使用空间 | > 80% |
| 网络 | 网络带宽、连接数 | 异常流量 |
| 进程 | 进程状态、崩溃重启 | 任何崩溃 |
| 线程 | 线程数、死锁检测 | 线程数异常 |

### 5.3 实时监控命令

```bash
# 监控 CPU 和内存
htop

# 监控磁盘 I/O
iostat -x 1

# 监控网络
iftop

# 监控进程
ps aux | grep cargo

# 监控日志
tail -f test.log
```

### 5.4 自动监控配置

监控脚本会自动执行以下操作：

1. **每分钟**: 记录 CPU、内存、磁盘使用率
2. **每5分钟**: 记录进程状态、线程数
3. **每小时**: 生成资源使用报告
4. **检测到异常**: 发送告警 (可选)

---

## 六、结果收集

### 6.1 使用结果收集脚本

```bash
# 1. 测试完成后收集结果
bash scripts/test/collect_stability_results.sh

# 2. 指定结果目录
bash scripts/test/collect_stability_results.sh --input-dir /path/to/test_results

# 3. 生成报告
bash scripts/test/collect_stability_results.sh --generate-report
```

### 6.2 结果文件说明

| 文件 | 说明 |
|------|------|
| `test_output.log` | 测试完整输出 |
| `monitor_metrics.csv` | 监控指标数据 |
| `memory_usage.png` | 内存使用图表 |
| `cpu_usage.png` | CPU 使用图表 |
| `summary.txt` | 测试摘要 |
| `report.md` | 测试报告 (Markdown) |

### 6.3 报告内容

结果收集脚本会生成包含以下内容的报告：

1. **测试基本信息**: 开始时间、结束时间、时长、分支、提交
2. **测试结果**: 通过/失败数量、错误信息
3. **资源使用统计**: CPU、内存、磁盘、网络平均值/峰值
4. **异常事件**: 崩溃、内存泄漏、性能下降记录
5. **结论**: 测试是否通过

---

## 七、测试日志分析

### 7.1 常见日志模式

```bash
# 查看测试输出
cat test_output.log

# 查看错误
grep -i error test_output.log

# 查看警告
grep -i warning test_output.log

# 查看崩溃
grep -i "panicked\|aborted\|segmentation" test_output.log
```

### 7.2 性能分析

```bash
# 查看 QPS 变化
grep "QPS" test_output.log

# 查看测试阶段
grep "test" test_output.log | head -20
```

---

## 八、回滚演练

### 8.1 回滚步骤

```bash
# 1. 创建当前版本备份
git tag backup/v2.6.0-ga-$(date +%Y%m%d)

# 2. 切换到上一稳定版本
git checkout release/v2.5.0

# 3. 验证功能正常
cargo test --lib

# 4. 如果需要，恢复到 v2.6.0
git checkout release/v2.6.0
```

### 8.2 回滚条件

满足以下任一条件需要回滚：

| 条件 | 说明 |
|------|------|
| 崩溃 | 发生任何崩溃 |
| 内存泄漏 | 内存持续增长超过 50% |
| 性能下降 | QPS 持续低于 500 |
| 功能错误 | 关键功能测试失败 |

---

## 九、常见问题

### 9.1 测试启动失败

**问题**: `cargo test` 报错

**解决方案**:
```bash
# 清理并重新构建
cargo clean
cargo build --release

# 重新运行测试
cargo test --test regression_test -- --test-threads=1 --ignored
```

### 9.2 内存使用过高

**问题**: 内存使用超过 85%

**解决方案**:
1. 检查是否有内存泄漏
2. 减少测试并发数
3. 增加系统内存

### 9.3 测试卡住

**问题**: 测试长时间无响应

**解决方案**:
```bash
# 查看进程状态
ps aux | grep cargo

# 查看线程
pstack <pid>

# 强制终止 (谨慎使用)
kill -9 <pid>
```

---

## 十、联系支持

### 10.1 测试报告

测试完成后，请将以下信息发送到开发团队：

1. `report.md` - 测试报告
2. `monitor_metrics.csv` - 监控数据
3. `test_output.log` - 测试日志
4. `summary.txt` - 测试摘要

### 10.2 问题反馈

如遇到问题，请提交 Issue：
- GitHub: https://github.com/minzuuniversity/sqlrustgo/issues

---

## 附录

### A. 快速命令参考

```bash
# 部署测试
bash scripts/test/deploy_stability_test.sh --hours 72

# 监控测试
bash scripts/test/monitor_stability_test.sh

# 收集结果
bash scripts/test/collect_stability_results.sh
```

### B. 测试流程图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        72 小时长稳测试流程                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│  │ 环境准备  │ -> │ 部署项目  │ -> │ 执行测试  │ -> │ 监控测试  │        │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘        │
│                                                                     │    │
│       │                                                           │    │
│       v                                                           v    │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│  │ 收集结果  │ <- │ 分析日志  │ <- │ 测试完成  │ <- │ 72 小时  │        │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘        │
│                                                                          │
│  ┌──────────┐    ┌──────────┐                                            │
│  │ 生成报告  │ -> │ 提交审核  │                                            │
│  └──────────┘    └──────────┘                                            │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### C. 相关文件

| 文件 | 说明 |
|------|------|
| `scripts/test/deploy_stability_test.sh` | 部署脚本 |
| `scripts/test/monitor_stability_test.sh` | 监控脚本 |
| `scripts/test/collect_stability_results.sh` | 结果收集脚本 |
| `tests/regression_test.rs` | 测试源码 |

---

*本文档由 SQLRustGo Team 维护*
*更新日期: 2026-04-21*

# 项目分析报告生成指南

> **版本**: v1.0
> **日期**: 2026-04-24
> **维护人**: yinglichina8848
> **目的**: 规范项目分析报告的生成流程，确保每个版本发布时能系统性地更新报告

---

## 一、概述

项目分析报告通过两个核心数据源——**Minimax Coding Plan 用量数据**和**GitHub PR/版本数据**——对 SQLRustGo 项目的开发强度、资源消耗和版本演进进行量化分析与趋势预测。

### 分析维度

| 维度 | 数据源 | 分析内容 |
|------|--------|----------|
| 资源消耗 | Minimax CSV | Token 用量、调用次数、模型/接口分布 |
| 开发强度 | Minimax CSV | 日均/周均/月均用量、峰谷波动、输出消费增长 |
| 版本演进 | Git 分支/PR | 版本分支列表、PR 数量、版本时间跨度 |
| 趋势预测 | 两者结合 | 用量趋势、月度预估、资源规划建议 |

---

## 二、数据源说明

### 2.1 Minimax 用量数据（CSV）

**获取方式**：从 Minimax 开放平台导出消费账单

**文件命名规范**：`export_YYYY_MM_DD.csv`（导出日期）

**存放位置**：`artifacts/usage/`

**CSV 字段说明**：

| 字段 | 类型 | 说明 |
|------|------|------|
| 消费账号 | string | 通常为"主账号" |
| 接口密钥名称 | string | 通常为"coding_plan" |
| 消费接口 | string | API 接口名称（见下方接口类型） |
| 消费模型 | string | 模型名称（见下方模型类型） |
| 消费金额 | float | 本次消费金额（Coding Plan 内为 0） |
| 代金券后消费金额 | float | 代金券抵扣后金额 |
| 输入消费数 | int/string | 输入 Token 数（可能为 `<nil>`） |
| 输出消费数 | int/string | 输出 Token 数（可能为 `<nil>`） |
| 总消费数 | int | 总 Token 数 |
| 消费时间 | string | 格式 `YYYY-MM-DD HH:00-HH:00` |
| 消费结果 | string | SUCCESS / FAIL |

**接口类型**：

| 接口 | 说明 | 特点 |
|------|------|------|
| `chatcompletion-v2(Text API)` | 对话补全 | 产生输入+输出 Token |
| `cache-read(Text API)` | 缓存读取 | 仅产生输入 Token，单次消费高 |
| `cache-create(Text API)` | 缓存创建 | 仅产生输入 Token |
| `code_plan_resource_package` | 资源包 | Coding Plan 搜索 |
| `code_plan_purchase` | 订阅购买 | 套餐购买记录 |

**模型类型**：

| 模型 | 定位 | 特点 |
|------|------|------|
| `MiniMax-M2.7-highspeed` | 高速模型 | 主力开发模型，速度快 |
| `MiniMax-M2.7` | 标准模型 | 平衡速度与质量 |
| `MiniMax-M2.1` | 轻量模型 | 适合简单任务 |

### 2.2 GitHub PR/版本数据

**获取方式**：通过 Git 命令从远程仓库获取

**关键命令**：

```bash
# 列出所有版本分支
git branch -r --list 'origin/develop/v*' | sort -V

# 查看分支最后更新日期
for branch in $(git branch -r --list 'origin/develop/v*' | sort -V); do
    echo "$branch $(git log -1 --format='%cs' $branch)"
done

# 统计版本间 PR 数量
git log origin/develop/v2.6.0..origin/develop/v2.8.0 --oneline | wc -l

# 查看最近 PR 活动
git log origin/develop/v2.8.0 --oneline -20

# 查看归档分支（已发布版本）
git branch -r --list 'origin/archive/*'

# 查看 release 分支
git branch -r --list 'origin/release/*'
```

---

## 三、分析脚本工具指南

### 3.1 脚本清单

| 脚本 | 路径 | 用途 |
|------|------|------|
| `analyze_april_bill.py` | `scripts/usage/` | **主分析脚本**，综合分析用量数据，生成完整报告和图表 |
| `update_trend_chart.py` | 项目根目录 | 更新趋势图（4子图+最近2周高亮） |
| `analyze_minimax_bill.py` | `scripts/usage/` | 早期分析脚本，按日/周/月汇总 |
| `analyze_march_bill.py` | `scripts/usage/` | 3月专项分析 |
| `analyze_new_bill.py` | `scripts/usage/` | 新版账单分析（3子图） |
| `analyze_output_usage.py` | `scripts/usage/` | 输出消费专项分析 |
| `comprehensive_statistics.py` | `scripts/usage/` | 综合统计（Markdown 表格输出） |
| `calculate_averages.py` | `scripts/usage/` | 日均/周均计算 |

### 3.2 主分析脚本使用方法

**脚本**：`scripts/usage/analyze_april_bill.py`

**使用步骤**：

1. **更新 CSV 文件路径**：修改脚本中的 `CSV_FILE` 变量指向最新的导出文件

```python
CSV_FILE = 'artifacts/usage/export_YYYY_MM_DD.csv'  # 替换为实际文件名
```

2. **更新日期参数**：修改 `today` 变量为报告日期

```python
today = datetime(YYYY, MM, DD)  # 替换为实际报告日期
```

3. **运行脚本**：

```bash
cd /path/to/sqlrustgo
python3 scripts/usage/analyze_april_bill.py
```

4. **输出内容**：
   - 终端输出：完整的文本格式分析报告
   - `scripts/usage/minimax_usage_trend.png`：总体趋势图（4子图）
   - `scripts/usage/minimax_2weeks_detail.png`：最近2周详细柱状图
   - `scripts/usage/minimax_usage_summary.xlsx`：汇总数据 Excel（6个 Sheet）

### 3.3 趋势图更新脚本使用方法

**脚本**：`update_trend_chart.py`

**使用步骤**：

1. **更新 CSV 文件路径和日期**（同上）
2. **运行脚本**：

```bash
python3 update_trend_chart.py
```

3. **输出**：`scripts/usage/minimax_usage_trend.png`

### 3.4 依赖环境

```bash
pip3 install pandas matplotlib openpyxl
```

---

## 四、报告生成完整流程

### 4.1 流程总览

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│  Step 1: 数据准备    │ ──▶ │  Step 2: 数据分析    │ ──▶ │  Step 3: 报告编写    │
│  - 导出 Minimax CSV  │     │  - 运行分析脚本      │     │  - 更新项目分析报告   │
│  - 拉取 Git 分支信息 │     │  - 收集 Git 版本数据 │     │  - 归档历史报告      │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
```

### 4.2 Step 1：数据准备

#### 1.1 导出 Minimax 用量数据

1. 登录 Minimax 开放平台
2. 进入「账单管理」→「消费明细」
3. 选择时间范围（建议从项目开始日期至当前）
4. 导出 CSV 文件
5. 将文件复制到项目目录：

```bash
cp ~/Downloads/export_YYYY_MM_DD.csv artifacts/usage/
```

#### 1.2 拉取最新 Git 数据

```bash
git fetch --all
```

### 4.3 Step 2：数据分析

#### 2.1 运行用量分析脚本

1. 更新 `scripts/usage/analyze_april_bill.py` 中的 `CSV_FILE` 和 `today` 参数
2. 运行脚本并保存输出：

```bash
python3 scripts/usage/analyze_april_bill.py 2>&1 | tee scripts/usage/analysis_output.txt
```

3. 更新趋势图：

```bash
# 更新 update_trend_chart.py 中的 CSV_FILE 和 today
python3 update_trend_chart.py
```

#### 2.2 收集 Git 版本数据

执行以下命令并记录结果：

```bash
# 版本分支列表及日期
for branch in $(git branch -r --list 'origin/develop/v*' | sort -V); do
    echo "$branch $(git log -1 --format='%cs' $branch)"
done

# 版本间 PR 数量（逐对计算）
git log origin/develop/v2.6.0..origin/develop/v2.8.0 --oneline | wc -l

# 最近 PR 活动
git log origin/develop/v2.8.0 --oneline -10

# 归档分支（已发布版本）
git branch -r --list 'origin/archive/develop*'

# Release 分支
git branch -r --list 'origin/release/*'
```

### 4.4 Step 3：报告编写

#### 3.1 更新项目分析报告

编辑 `项目分析报告.md`，按以下顺序更新各章节：

| 章节 | 更新内容 | 数据来源 |
|------|----------|----------|
| 1.1 数据概览 | 日期范围、天数、记录数、峰值日 | 分析脚本输出 |
| 1.2 用量趋势分析 | 更新趋势图引用 | 脚本生成的 PNG |
| 1.3 月度用量对比 | 各月总用量、日均 | 分析脚本输出 |
| 1.4 最近2周用量分析 | 每日详细数据 | 分析脚本输出 |
| 1.5 最近2周 vs 前2周 | 对比数据 | 分析脚本输出 |
| 1.6 模型使用分布 | 模型占比 | 分析脚本输出 |
| 1.7 接口使用分布 | 接口占比 | 分析脚本输出 |
| 1.8 用量波动分析 | 标准差、变异系数、峰谷比 | 分析脚本输出 |
| 2.1 版本分支列表 | 新增版本分支 | Git 命令 |
| 2.2 版本时间跨度 | 新版本时间跨度 | Git 命令 |
| 2.3 版本 PR 数量 | 新增 PR 统计 | Git 命令 |
| 2.4 最近 PR 活动 | 最新 PR 列表 | Git 命令 |
| 2.5 分支策略分析 | 当前活跃分支 | Git 命令 |
| 3. 趋势洞察 | 更新分析结论 | 综合判断 |
| 4. 结论与建议 | 更新结论和建议 | 综合判断 |
| 5. 附件 | 更新文件列表 | 实际文件 |

#### 3.2 归档历史报告

将历史报告归档到 `docs/governance/reports/`：

```bash
cp 项目分析报告.md docs/governance/reports/project_analysis_YYYY_MM_DD.md
```

#### 3.3 更新变更记录

在报告底部添加变更记录：

```markdown
*变更记录*
- YYYY-MM-DD：初始报告，数据范围 XX-XX 至 XX-XX
- YYYY-MM-DD：更新报告，[变更内容摘要]
```

---

## 五、版本发布时的更新检查清单

每个版本发布时，按以下检查清单执行：

### 5.1 数据更新

- [ ] 从 Minimax 平台导出最新 CSV 文件
- [ ] 将 CSV 文件复制到 `artifacts/usage/`
- [ ] 执行 `git fetch --all` 拉取最新分支信息

### 5.2 脚本运行

- [ ] 更新 `analyze_april_bill.py` 中的 `CSV_FILE` 和 `today`
- [ ] 运行 `python3 scripts/usage/analyze_april_bill.py`
- [ ] 更新 `update_trend_chart.py` 中的 `CSV_FILE` 和 `today`
- [ ] 运行 `python3 update_trend_chart.py`

### 5.3 Git 数据收集

- [ ] 执行版本分支列表命令
- [ ] 执行版本间 PR 数量统计
- [ ] 执行最近 PR 活动查询
- [ ] 检查归档分支和 release 分支

### 5.4 报告更新

- [ ] 更新 §1 用量分析各小节
- [ ] 更新 §2 版本分析各小节
- [ ] 更新 §3 趋势洞察
- [ ] 更新 §4 结论与建议
- [ ] 更新 §5 附件
- [ ] 添加变更记录
- [ ] 归档历史报告到 `docs/governance/reports/`

### 5.5 验证

- [ ] 检查趋势图是否正确生成
- [ ] 检查 Excel 汇总数据是否完整
- [ ] 检查报告中数据与分析脚本输出一致
- [ ] 检查版本分支信息与 Git 仓库一致

---

## 六、关键分析指标说明

### 6.1 用量指标

| 指标 | 计算方法 | 意义 |
|------|----------|------|
| 日均用量 | 总消费数 ÷ 总天数 | 衡量日常消耗水平 |
| 周均用量 | 总消费数 ÷ 周数 | 消除日度波动 |
| 月均用量 | 总消费数 ÷ 月数 | 月度基准 |
| 输出消费占比 | 输出消费 ÷ 总消费 × 100% | 衡量代码生成量 |
| 峰谷比 | 最高日用量 ÷ 最低日用量 | 衡量用量波动程度 |
| 变异系数 | 标准差 ÷ 均值 × 100% | 衡量用量稳定性 |

### 6.2 版本指标

| 指标 | 计算方法 | 意义 |
|------|----------|------|
| 版本时间跨度 | 当前版本日期 - 上一版本日期 | 开发速度 |
| 版本 PR 密度 | PR 数量 ÷ 时间跨度 | 开发强度 |
| 累计 PR 数量 | 版本间 commit 数量 | 版本规模 |

### 6.3 预测指标

| 指标 | 计算方法 | 用途 |
|------|----------|------|
| 月度预估用量 | 最近2周日均 × 30 | 资源规划 |
| 月度预估调用 | 最近2周日均调用 × 30 | 套餐评估 |
| 短期趋势 | 最近3日均值 vs 最近2周均值 | 趋势判断 |

---

## 七、分析脚本开发规范

### 7.1 新建脚本时

1. 脚本放在 `scripts/usage/` 目录下
2. 文件命名：`analyze_<描述>_<月份/版本>.py`
3. CSV 文件路径使用 `artifacts/usage/` 前缀
4. 输出文件路径使用 `scripts/usage/` 前缀
5. 处理 `<nil>` 值：`df['字段'].replace('<nil>', '0')`

### 7.2 脚本模板

```python
#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime, timedelta

plt.rcParams['font.sans-serif'] = ['Arial Unicode MS', 'SimHei', 'PingFang SC']
plt.rcParams['axes.unicode_minus'] = False

CSV_FILE = 'artifacts/usage/export_YYYY_MM_DD.csv'
OUTPUT_DIR = 'scripts/usage/'

df = pd.read_csv(CSV_FILE)

df['输入消费数'] = pd.to_numeric(df['输入消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)
df['输出消费数'] = pd.to_numeric(df['输出消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)
df['总消费数'] = pd.to_numeric(df['总消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)

df['日期'] = df['消费时间'].apply(lambda x: datetime.strptime(x.split(' ')[0], '%Y-%m-%d'))

today = datetime(YYYY, MM, DD)
two_weeks_ago = today - timedelta(days=14)

# ... 分析逻辑 ...
```

### 7.3 注意事项

- **中文字体**：macOS 使用 `Arial Unicode MS`，Linux 使用 `SimHei`，Windows 使用 `Microsoft YaHei`
- **日期解析**：消费时间格式为 `YYYY-MM-DD HH:00-HH:00`，取空格前的日期部分
- **数值处理**：`<nil>` 值需替换为 `0` 后再转数值类型
- **图表保存**：使用 `dpi=150, bbox_inches='tight'` 确保图表清晰且无裁切

---

## 八、文件结构

```
sqlrustgo/
├── 项目分析报告.md                          # 主报告（每次版本发布时更新）
├── update_trend_chart.py                    # 趋势图更新脚本
├── artifacts/usage/                         # 原始数据存放
│   ├── export_YYYY_MM_DD.csv               # Minimax 导出文件
│   └── ...
├── scripts/usage/                           # 分析脚本和输出
│   ├── analyze_april_bill.py               # 主分析脚本
│   ├── minimax_usage_trend.png             # 趋势图
│   ├── minimax_2weeks_detail.png           # 2周详细图
│   ├── minimax_usage_summary.xlsx          # 汇总 Excel
│   └── ...
└── docs/governance/
    ├── PROJECT_ANALYSIS_GUIDE.md            # 本文档
    └── reports/                             # 历史报告归档
        ├── usage_report_2026_02_03.md
        └── project_analysis_YYYY_MM_DD.md  # 归档报告
```

---

## 九、常见问题

### Q1: CSV 文件中有 `<nil>` 值导致分析失败？

**解决**：在读取 CSV 后，对数值列进行替换：

```python
df['输入消费数'] = pd.to_numeric(df['输入消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)
```

### Q2: 图表中文显示为方块？

**解决**：设置中文字体：

```python
plt.rcParams['font.sans-serif'] = ['Arial Unicode MS', 'SimHei', 'PingFang SC']
plt.rcParams['axes.unicode_minus'] = False
```

### Q3: 如何判断版本是否已发布 GA？

**解决**：检查是否存在对应的 release 分支和归档分支：

```bash
git branch -r --list 'origin/release/vX.Y.Z*'
git branch -r --list 'origin/archive/develop-vX.Y.Z'
```

如果存在 `release/vX.Y.Z-ga` 和 `origin/archive/develop-vX.Y.Z`，则该版本已发布 GA。

### Q4: 如何处理4月19日无数据的情况？

**解决**：Minimax 导出数据按小时聚合，如果某天无任何调用则不会产生记录。分析脚本按实际有数据的日期计算日均，不会因缺失日期导致偏差。

### Q5: 预估月用量准确吗？

**解决**：预估基于最近2周日均用量 × 30，适用于用量稳定的场景。如果项目处于开发冲刺期，用量波动较大（变异系数 > 50%），预估可能偏高或偏低。建议结合最近3日均值和2周均值综合判断趋势方向。

---

*文档创建日期*: 2026-04-24
*最后更新*: 2026-04-24

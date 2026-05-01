# SQLRustGo 仓库目录与文档审计报告（2026-04-19）

## 1. 审计范围与基线

- 审计时间：2026-04-19
- 审计分支：`develop/v2.6.0`
- 基线提交：`d6aa9ddf`
- 审计对象：仓库顶层目录、`docs/` 文档体系、代码目录（`src/` + `crates/`）边界、文档索引有效性

## 2. 结论摘要

- 当前仓库“可用但不整洁”：工程代码主干明确，但文档与产物沉积导致目录噪声偏高。
- 主要问题不在“缺文档”，而在“文档分层、命名、索引一致性”。
- 建议分三阶段整改：先止损（索引修复+归档规则），再重构（目录归一），最后建立门禁（CI 校验）。

## 3. 关键发现（按优先级）

### P0（立即处理）

1. 根目录 `README.md` 存在错误相对链接
- 现象：`已发布版本` 链接使用 `../releases/...`，从仓库根路径解析会越级到仓库外，链接不可达。
- 影响：新成员与外部协作者无法从主入口正确跳转历史版本文档。
- 建议：统一改为 `docs/releases/...`（文档路径）或 `releases/...`（制品路径），避免混用。

2. 文档索引与实际文件存在不一致
- 现象：`docs/README.md` 引用了不存在文件，如：
  - `docs/v2.0/WHITEPAPER_V3.md`
  - `docs/v2.0/GIT_PERMISSION_MODEL.md`
  - `docs/v2.0/GIT_PERMISSION_MODEL_V3.md`
  - `docs/v2.0/PRESENTATION_MATERIALS.md`
  - `docs/2026-02-13-sqlcc-rust-redesign-impl-plan.md`
- 影响：文档信任度下降，检索成本提高。
- 建议：修复为真实路径或删除无效入口，建立链接校验。

### P1（本周处理）

3. `docs/` 存在重复目录，且内容完全重复
- 现象：`docs/AI增强软件工程` 与 `docs/ai_collaboration` 中核心文件内容一致（逐文件 `cmp` 一致）。
- 影响：双写维护、后续漂移风险、检索混淆。
- 建议：保留单一 canonical 目录，另一目录迁移为索引或软链接策略（跨平台可用“索引跳转”替代软链）。

4. 版本目录命名规范不统一
- 现象：同时存在 `v2.0` 与 `v2.0.0`；`OO` 与 `oo` 大小写混用。
- 影响：脚本与人工检索均容易误判。
- 建议：统一语义：
  - `docs/releases/vX.Y.Z` 仅存发布文档
  - `docs/releases/vX.Y` 禁止新增（历史迁移后只留索引说明）
  - 子目录统一小写

5. 仓库根目录沉积大量非工程产物
- 现象：顶层有 CSV/PNG/XLSX/TAR/覆盖率 HTML 等运行产物。
- 影响：根目录可读性差，误提交风险高。
- 建议：迁移至 `artifacts/` 或 `analysis/`，并补 `.gitignore`/留存策略。

### P2（两周内处理）

6. `docs/` 规模大且层级职责交叉
- 现象：`docs` 文件约 780 个，其中 markdown 约 665 个；教学、治理、计划、报告、版本、白皮书等并列且存在交叉。
- 影响：入门与维护成本高，文档“查得到但找不准”。
- 建议：按“读者与生命周期”重构文档 IA（Information Architecture）。

7. 工程代码目录处于“迁移中”，但说明滞后
- 现象：`src/` 与 `crates/` 并存，`crates/README.md` 仍描述“正在迁移”，与当前规模（30+ crate）不匹配。
- 影响：新人难判断 canonical 代码入口。
- 建议：更新 `crates/README.md` 与根 `README.md` 的“代码组织”章节，明确主入口与遗留边界。

## 4. 目标目录与文档架构（建议）

建议形成如下稳定结构（示意）：

```text
/
├── src/                        # 顶层二进制/兼容入口（若保留）
├── crates/                     # 核心业务与基础设施 crate（唯一主实现）
├── tests/                      # 集成与端到端测试
├── docs/
│   ├── index/                  # 总索引与导航（README + 导航页）
│   ├── architecture/           # 架构与设计
│   ├── releases/               # 版本发布文档（vX.Y.Z）
│   ├── governance/             # 流程与治理
│   ├── guides/                 # 教程/操作指南
│   ├── plans/                  # 实施计划（有时效）
│   ├── reports/                # 验证/评估报告（可归档）
│   └── archive/                # 历史冻结资料
├── artifacts/                  # 非源码运行产物（报告图、导出数据等）
└── scripts/
```

## 5. 整改计划（可执行）

### 阶段 A：止损（1-2 天）

1. 修复 P0 链接错误
- 修复根 `README.md` 与 `docs/README.md` 的失效链接。

2. 建立文档链接校验
- 增加脚本：`scripts/gate/check_docs_links.sh`
- 在 CI 增加 gate：PR 必须通过文档链接检查。

3. 规范新增文档入口
- 新文档必须挂在 `docs/README.md` 或对应子目录 `README.md`。

### 阶段 B：结构归一（3-5 天）

1. 去重与合并目录
- 选定 `docs/ai_collaboration` 或 `docs/AI增强软件工程` 作为唯一目录。
- 另一目录改为“迁移说明页”并在两周后移除。

2. 版本目录标准化
- 制定并执行 rename/迁移脚本，统一到 `vX.Y.Z` 与小写子目录。

3. 根目录清理
- 迁移 CSV/PNG/XLSX/tarpaulin 报告到 `artifacts/`，并更新 `.gitignore`。

### 阶段 C：治理闭环（1 周）

1. 文档生命周期治理
- `plans/` 与 `reports/` 增加状态字段（`active`/`archived`）。
- 超过 N 个月的计划文档自动归档到 `docs/archive/`。

2. 目录守护规则
- 增加 `docs/DIRECTORY_POLICY.md`，定义命名、放置、索引、归档规则。
- 在 PR 模板中增加“文档路径与索引更新”检查项。

3. 可观测指标
- 每周自动输出：
  - 失效链接数
  - 未索引文档数
  - 重复文件对数（基于哈希）

## 6. 验收标准（Definition of Done）

- 根 `README.md` 与 `docs/README.md` 0 失效链接。
- 文档重复目录收敛到单一 canonical 目录。
- 版本目录命名符合统一规范（无 `vX.Y`/`vX.Y.Z` 混用）。
- 根目录仅保留工程入口文件（README、Cargo、LICENSE 等），运行产物归档完成。
- CI 中文档门禁稳定通过。

## 7. 风险与回滚

- 风险：批量重命名会导致历史链接失效。
- 缓解：
  - 先新增跳转页再删除旧路径；
  - 用一次性映射表（old->new）批量替换链接；
  - 迁移分批 PR（索引修复、目录迁移、CI 门禁分开）。


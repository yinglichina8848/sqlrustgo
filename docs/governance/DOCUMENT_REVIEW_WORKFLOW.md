# 文档审查流程规范

> **版本**: v1.0.0
> **创建日期**: 2026-04-22
> **适用范围**: SQLRustGo 版本发布文档审查
> **基于实践**: v2.7.0 GA 文档审查经验

---

## 1. 概述

### 1.1 目的

本文档定义文档审查的标准化流程，确保每个版本发布前文档完整、一致、可审计。

### 1.2 适用范围

- 版本发布前的文档自查
- RC → GA 门禁检查
- 版本文档对比与整改

### 1.3 审查原则

| 原则 | 说明 |
|------|------|
| 版本一致性 | 所有文档版本号、状态必须与发布状态一致 |
| 引用正确性 | 文档内部引用必须正确，无悬空链接 |
| 内容完整性 | 必选文档必须存在且内容完整 |
| 格式规范性 | Markdown 格式、目录结构必须规范 |

---

## 2. 文档分类体系

### 2.1 文档分类

| 类别 | 说明 | 示例 |
|------|------|------|
| **核心发布文档** | 版本入口、发布说明、变更日志 | README.md, RELEASE_NOTES.md, CHANGELOG.md |
| **用户文档** | 用户操作指南、API 文档 | USER_MANUAL.md, API_DOCUMENTATION.md, *USER_GUIDE.md |
| **开发文档** | 开发环境、代码规范、PR 流程 | DEVELOPMENT_GUIDE.md, INSTALL.md |
| **测试文档** | 测试计划、手册、报告 | TEST_PLAN.md, TEST_MANUAL.md |
| **运维文档** | 部署、备份恢复、安全 | DEPLOYMENT_GUIDE.md, BACKUP_RESTORE.md |
| **架构文档** | OO 架构、模块设计 | oo/architecture/*.md, oo/modules/*/ |
| **报告文档** | 性能、安全、覆盖率报告 | PERFORMANCE_REPORT.md, SECURITY_ANALYSIS.md |

### 2.2 用户文档要求

**重要**: v2.7.0 起，用户指南必须包含详细的功能使用说明：

| 文档 | 说明 | 路径 |
|------|------|------|
| **主用户手册** | SQL 基础操作、配置说明 | `oo/user-guide/USER_MANUAL.md` |
| **GMP 用户指南** | GMP 审计功能使用 | `oo/user-guide/GMP_USER_GUIDE.md` |
| **图检索用户指南** | 图引擎、Cypher 查询 | `oo/user-guide/GRAPH_SEARCH_USER_GUIDE.md` |
| **向量检索用户指南** | 向量索引、混合检索 | `oo/user-guide/VECTOR_SEARCH_USER_GUIDE.md` |

---

## 3. 审查流程

### 3.1 审查阶段

```
┌─────────────────────────────────────────────────────────────┐
│                    文档审查流程                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 1: 目录结构检查                                       │
│  ├── 检查版本目录是否存在                                    │
│  ├── 检查 OO 目录结构                                        │
│  └── 检查模块目录结构                                        │
│                                                             │
│  Phase 2: 必选文档检查                                       │
│  ├── 检查核心发布文档 (17 项)                                │
│  ├── 检查用户文档 (4 项)                                     │
│  ├── 检查 OO 架构文档 (5 项)                                │
│  └── 检查模块设计文档 (14 项)                                │
│                                                             │
│  Phase 3: 内容一致性检查                                    │
│  ├── 版本号一致性                                           │
│  ├── 版本状态一致性 (alpha/beta/RC/GA)                      │
│  ├── 日期更新检查                                           │
│  └── 内部引用检查                                           │
│                                                             │
│  Phase 4: 质量检查                                          │
│  ├── 链接有效性检查                                          │
│  ├── 格式规范性检查                                         │
│  └── 内容完整性检查                                          │
│                                                             │
│  Phase 5: 版本对比                                          │
│  ├── 与上一版本对比文档结构                                  │
│  ├── 检查新增功能文档                                        │
│  └── 确认废弃功能已标注                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Phase 1: 目录结构检查

```bash
# 检查版本目录
VERSION=v2.7.0
BASE_DIR="docs/releases/$VERSION"

# 检查目录存在
ls -la "$BASE_DIR/" || echo "ERROR: Version directory missing"

# 检查 OO 目录结构
ls -la "$BASE_DIR/oo/" || echo "ERROR: OO directory missing"
ls -la "$BASE_DIR/oo/architecture/"
ls -la "$BASE_DIR/oo/modules/"
ls -la "$BASE_DIR/oo/user-guide/"
ls -la "$BASE_DIR/oo/reports/"

# 检查子目录结构
for module in mvcc wal executor parser graph vector storage optimizer catalog planner transaction server bench unified-query; do
    if [ ! -d "$BASE_DIR/oo/modules/$module" ]; then
        echo "ERROR: Missing module directory: $module"
    fi
done
```

### 3.3 Phase 2: 必选文档检查

#### 3.3.1 核心发布文档 (17 项)

| 文档 | 必选 | 检查脚本 |
|------|------|----------|
| README.md | ✅ | 文件存在检查 |
| CHANGELOG.md | ✅ | 文件存在 + 版本号检查 |
| RELEASE_NOTES.md | ✅ | 文件存在 + 日期检查 |
| MIGRATION_GUIDE.md | ✅ | 文件存在 |
| DEPLOYMENT_GUIDE.md | ✅ | 文件存在 |
| DEVELOPMENT_GUIDE.md | ✅ | 文件存在 |
| TEST_PLAN.md | ✅ | 文件存在 |
| TEST_MANUAL.md | ✅ | 文件存在 |
| EVALUATION_REPORT.md | ✅ | 文件存在 |
| DOCUMENT_AUDIT.md | ✅ | 文件存在 |
| FEATURE_MATRIX.md | ✅ | 文件存在 + 状态检查 |
| COVERAGE_REPORT.md | ✅ | 文件存在 |
| SECURITY_ANALYSIS.md | ✅ | 文件存在 |
| PERFORMANCE_TARGETS.md | ✅ | 文件存在 |
| QUICK_START.md | ✅ | 版本状态检查 |
| INSTALL.md | ✅ | 版本状态检查 |
| API_DOCUMENTATION.md | ✅ | 文件存在 |

#### 3.3.2 用户文档 (4 项)

| 文档 | 必选 | 说明 |
|------|------|------|
| USER_MANUAL.md | ✅ | 主用户手册 |
| GMP_USER_GUIDE.md | ✅ | GMP 审计用户指南 |
| GRAPH_SEARCH_USER_GUIDE.md | ✅ | 图检索用户指南 |
| VECTOR_SEARCH_USER_GUIDE.md | ✅ | 向量检索用户指南 |

#### 3.3.3 OO 架构文档 (5 项)

| 文档 | 必选 | 检查 |
|------|------|------|
| oo/README.md | ✅ | OO 文档索引 |
| oo/architecture/ARCHITECTURE_VX.Y.md | ✅ | 版本架构设计 |
| oo/user-guide/USER_MANUAL.md | ✅ | 用户手册 |
| oo/user-guide/GMP_USER_GUIDE.md | ✅ | GMP 用户指南 |
| oo/user-guide/GRAPH_SEARCH_USER_GUIDE.md | ✅ | 图检索用户指南 |
| oo/user-guide/VECTOR_SEARCH_USER_GUIDE.md | ✅ | 向量检索用户指南 |
| oo/reports/PERFORMANCE_ANALYSIS.md | ✅ | 性能分析 |
| oo/reports/SQL92_COMPLIANCE.md | ✅ | SQL 合规报告 |

#### 3.3.4 模块设计文档 (14 项)

| 模块 | 必选 | 文档 |
|------|------|------|
| MVCC | ✅ | mvcc/MVCC_DESIGN.md |
| WAL | ✅ | wal/WAL_DESIGN.md |
| Executor | ✅ | executor/EXECUTOR_DESIGN.md |
| Parser | ✅ | parser/PARSER_DESIGN.md |
| Graph | ✅ | graph/GRAPH_DESIGN.md |
| Vector | ✅ | vector/VECTOR_DESIGN.md |
| Storage | ✅ | storage/STORAGE_DESIGN.md |
| Optimizer | ✅ | optimizer/OPTIMIZER_DESIGN.md |
| Catalog | ✅ | catalog/CATALOG_DESIGN.md |
| Planner | ✅ | planner/PLANNER_DESIGN.md |
| Transaction | ✅ | transaction/TRANSACTION_DESIGN.md |
| Server | ✅ | server/SERVER_DESIGN.md |
| Bench | ✅ | bench/BENCH_DESIGN.md |
| Unified Query | ✅ | unified-query/UNIFIED_QUERY_DESIGN.md |

### 3.4 Phase 3: 内容一致性检查

#### 3.4.1 版本号一致性

检查所有文档中的版本号是否统一为当前版本：

```bash
# 搜索所有文档中的版本号
grep -r "v2.7.0\|alpha/v2.7.0\|beta/v2.7.0" "$BASE_DIR/" --include="*.md"
```

**常见问题**:
- 文档标题或正文显示 `alpha/v2.7.0` 但实际已 GA
- 版本号写错 (如 v2.6.0 而非 v2.7.0)

**整改方法**:
```bash
# 批量替换 alpha/v2.7.0 为 v2.7.0
sed -i '' 's/alpha\/v2.7.0/v2.7.0/g' "$BASE_DIR"/*.md
sed -i '' 's/alpha\/v2.7.0/v2.7.0/g' "$BASE_DIR"/oo/**/*.md
```

#### 3.4.2 版本状态一致性

| 文档类型 | Alpha 阶段 | Beta 阶段 | RC 阶段 | GA 阶段 |
|----------|------------|-----------|---------|---------|
| README.md | alpha/vX.Y.Z | beta/vX.Y.Z | rc/vX.Y.Z | vX.Y.Z |
| RELEASE_NOTES.md | alpha | beta | RC | GA |
| VERSION_PLAN.md | 所有任务 ⏳ | 进行中 ✅ | 完成 ✅ | 完成 ✅ |
| FEATURE_MATRIX.md | ⏳ | 部分 ✅ | 大部分 ✅ | 全部 ✅ |

#### 3.4.3 日期检查

检查文档中的日期是否为最近更新：

```bash
# 检查文档最后修改日期
for f in $(find "$BASE_DIR" -name "*.md"); do
    echo "$(stat -f '%Sm' -t '%Y-%m-%d' "$f") $f"
done | sort
```

#### 3.4.4 内部引用检查

检查文档内部引用是否正确：

```bash
# 检查引用格式
grep -E '\[.+\]\(\./|\.\.\/' "$BASE_DIR"/*.md

# 运行链接检查脚本
bash scripts/gate/check_docs_links.sh --all
```

**常见引用问题**:
- 引用不存在的文件: `[UPGRADE_GUIDE_v2.7.0.md](./UPGRADE_GUIDE.md)`
- 引用路径错误: `[文档](../v2.7.0/UPGRADE_GUIDE.md)` 而非 `[文档](./UPGRADE_GUIDE.md)`

### 3.5 Phase 4: 质量检查

#### 3.5.1 链接有效性

```bash
# 使用项目提供的检查脚本
bash scripts/gate/check_docs_links.sh

# 或手动检查
for f in $(find "$BASE_DIR" -name "*.md"); do
    # 检查相对链接
    grep -oE '\]\([^)]+\)' "$f" | while read -r link; do
        path=$(echo "$link" | sed 's/\]//g' | sed 's/(//g')
        if [[ "$path" == *.md ]]; then
            # 解析相对路径
            dir=$(dirname "$f")
            full_path="$dir/$path"
            if [ ! -f "$full_path" ]; then
                echo "BROKEN LINK: $f -> $path"
            fi
        fi
    done
done
```

#### 3.5.2 格式规范性

```bash
# 检查 Markdown 格式
for f in $(find "$BASE_DIR" -name "*.md"); do
    # 检查是否以 # 开头
    if ! head -1 "$f" | grep -q "^#"; then
        echo "MISSING TITLE: $f"
    fi
    # 检查是否有版本信息
    if ! grep -q "版本" "$f"; then
        echo "MISSING VERSION INFO: $f"
    fi
done
```

#### 3.5.3 内容完整性

| 检查项 | 标准 |
|--------|------|
| 标题完整 | 文档必须以 `# 标题` 开头 |
| 版本标注 | 必须包含版本号 |
| 更新日期 | 必须包含更新日期 |
| 目录结构 | 多级文档必须包含目录 |

### 3.6 Phase 5: 版本对比

#### 3.6.1 文档结构对比

```bash
# 对比两个版本的文档结构
echo "=== v2.6.0 ==="
find docs/releases/v2.6.0/ -name "*.md" | sort

echo "=== v2.7.0 ==="
find docs/releases/v2.7.0/ -name "*.md" | sort

# 对比差异
diff <(find docs/releases/v2.6.0/ -name "*.md" | sort) \
     <(find docs/releases/v2.7.0/ -name "*.md" | sort)
```

#### 3.6.2 新增功能文档检查

根据版本计划，检查新功能是否已文档化：

```bash
# 检查 VERSION_PLAN 中的新功能
grep -E "T-[0-9]+" docs/releases/vX.Y.Z/VERSION_PLAN.md

# 确认每个功能都有对应文档
# T-01 WAL 崩溃恢复 → WAL_DESIGN.md, DEPLOYMENT_GUIDE.md
# T-04 qmd-bridge → qmd-bridge-design.md
# T-07 GMP Top10 → gmp-top10-scenarios.md, GMP_USER_GUIDE.md
```

---

## 4. 审查执行

### 4.1 执行检查单

```markdown
## 文档审查检查单

### Phase 1: 目录结构
- [ ] 版本目录存在
- [ ] OO 目录结构完整
- [ ] 模块目录结构完整

### Phase 2: 必选文档
- [ ] 核心发布文档 (17 项) 全部存在
- [ ] 用户文档 (4 项) 全部存在
- [ ] OO 架构文档 (8 项) 全部存在
- [ ] 模块设计文档 (14 项) 全部存在

### Phase 3: 内容一致性
- [ ] 版本号全部统一
- [ ] 版本状态全部一致
- [ ] 日期已更新
- [ ] 内部引用正确

### Phase 4: 质量检查
- [ ] 链接全部有效
- [ ] 格式规范
- [ ] 内容完整

### Phase 5: 版本对比
- [ ] 与上一版本结构对齐
- [ ] 新功能已文档化
- [ ] 无悬空引用
```

### 4.2 常见问题与整改

| 问题 | 原因 | 整改命令 |
|------|------|----------|
| 版本状态显示 alpha | 文档未更新 | `sed -i '' 's/alpha\/vX.Y.Z/vX.Y.Z/g'` |
| 引用文件名错误 | 文档引用过时文件名 | 修正引用链接 |
| 链接失效 | 文件移动或删除 | 修复或创建文件 |
| 内容 TBD 占位 | 文档未填充 | 补充实际内容 |
| 状态与实际不符 | 文档未同步更新 | 更新状态标注 |

---

## 5. 门禁集成

### 5.1 门禁检查表更新

在 `RELEASE_GATE_CHECKLIST.md` 的文档层 Gate 添加用户文档检查：

```markdown
## 📚 3️⃣ 文档层 Gate

### 3.1 核心文档

| 检查项 | 要求 | 状态 | 检查人 | 日期 |
|--------|------|------|--------|------|
| README | README 更新完成 | ⏳ | | |
| CHANGELOG | CHANGELOG 更新完成 | ⏳ | | |
| Release Notes | Release Notes 完成 | ⏳ | | |
| 用户文档 | 用户文档更新完成 | ⏳ | | |

### 3.2 用户指南 (新增)

| 检查项 | 要求 | 状态 | 检查人 | 日期 |
|--------|------|------|--------|------|
| USER_MANUAL.md | 主用户手册存在 | ⏳ | | |
| GMP_USER_GUIDE.md | GMP 用户指南存在 | ⏳ | | |
| GRAPH_SEARCH_USER_GUIDE.md | 图检索用户指南存在 | ⏳ | | |
| VECTOR_SEARCH_USER_GUIDE.md | 向量检索用户指南存在 | ⏳ | | |
| 用户文档链接有效性 | 文档链接全部有效 | ⏳ | | |
```

### 5.2 门禁执行命令

```bash
# 文档完整性检查
bash scripts/gate/check_docs_links.sh

# 文档结构检查
./scripts/check_documentation.sh vX.Y.Z

# 编译检查（文档中引用的命令）
cargo build --all-features
```

---

## 6. 附录

### 6.1 检查脚本模板

#### check_user_guides.sh

```bash
#!/bin/bash
VERSION=$1
BASE_DIR="docs/releases/$VERSION"

USER_GUIDES=(
    "oo/user-guide/USER_MANUAL.md"
    "oo/user-guide/GMP_USER_GUIDE.md"
    "oo/user-guide/GRAPH_SEARCH_USER_GUIDE.md"
    "oo/user-guide/VECTOR_SEARCH_USER_GUIDE.md"
)

echo "=== Checking User Guides ==="
for guide in "${USER_GUIDES[@]}"; do
    if [ ! -f "$BASE_DIR/$guide" ]; then
        echo "ERROR: Missing user guide: $guide"
        exit 1
    else
        echo "OK: $guide exists"
    fi
done

echo "=== Checking User Guide Links ==="
for guide in "${USER_GUIDES[@]}"; do
    # 检查 USER_MANUAL.md 中是否包含其他用户指南的链接
    if [ "$guide" = "oo/user-guide/USER_MANUAL.md" ]; then
        for other in "${USER_GUIDES[@]}"; do
            if [ "$other" != "$guide" ]; then
                if ! grep -q "$(basename "$other")" "$BASE_DIR/$guide"; then
                    echo "WARNING: $guide does not link to $other"
                fi
            fi
        done
    fi
done

echo "All user guide checks passed!"
```

### 6.2 v2.7.0 审查经验总结

**审查发现问题**:
1. 部分文档仍显示 `alpha/v2.7.0` 状态
2. `UPGRADE_GUIDE.md` 引用文件名错误
3. 用户指南文档缺失 (GMP/图检索/向量检索)
4. `oo/README.md` 用户指南索引不完整

**整改措施**:
1. 批量替换版本状态标识
2. 修正文档内部引用
3. 创建缺失的用户指南
4. 更新主用户手册和 OO README

**验证结果**:
- `cargo build --all-features`: ✅ 通过
- `cargo test -p sqlrustgo-gmp --lib`: ✅ 47 tests passed
- `check_docs_links.sh`: ✅ All links valid

---

*文档审查流程规范 v1.0.0*
*基于 v2.7.0 GA 文档审查实践*
*最后更新: 2026-04-22*
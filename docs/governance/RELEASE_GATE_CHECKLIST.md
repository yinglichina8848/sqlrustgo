# 版本发布门禁检查模板

**版本**: v1.0.0
**创建日期**: 2026-04-19
**适用范围**: 版本发布前门禁检查

---

## 一、门禁检查流程

### 1.1 检查阶段

| 阶段 | 检查内容 | 责任人 |
|------|----------|--------|
| 代码检查 | 代码质量、编译状态 | 开发团队 |
| 测试检查 | 测试覆盖率、测试结果 | 测试团队 |
| 文档检查 | 文档完整性、文档质量 | 文档团队 |
| 安全检查 | 安全漏洞、合规性 | 安全团队 |
| 性能检查 | 性能指标、基准测试 | 性能团队 |

### 1.2 检查标准

| 检查项 | 标准 | 状态 |
|--------|------|------|
| 代码编译 | 无编译错误 | ✅ |
| 测试通过率 | ≥ 95% | ✅ |
| 测试覆盖率 | ≥ 80% | ✅ |
| 文档完整性 | 100% 必选文档 | ✅ |
| 安全扫描 | 无高危漏洞 | ✅ |
| 性能达标 | 达到目标指标 | ✅ |

## 二、文档检查门禁

### 2.1 文档完整性检查

**检查脚本**: `docs/governance/DOCUMENT_COMPLETENESS_CHECK.md`

**检查内容**:

#### 根目录文档 (docs/releases/vX.Y.Z/)

| 文档 | 必选 | 检查状态 |
|------|------|----------|
| README.md | ✅ | □ 完成 |
| CHANGELOG.md | ✅ | □ 完成 |
| RELEASE_NOTES.md | ✅ | □ 完成 |
| MIGRATION_GUIDE.md | ✅ | □ 完成 |
| DEPLOYMENT_GUIDE.md | ✅ | □ 完成 |
| DEVELOPMENT_GUIDE.md | ✅ | □ 完成 |
| TEST_PLAN.md | ✅ | □ 完成 |
| TEST_MANUAL.md | ✅ | □ 完成 |
| EVALUATION_REPORT.md | ✅ | □ 完成 |
| DOCUMENT_AUDIT.md | ✅ | □ 完成 |
| FEATURE_MATRIX.md | ✅ | □ 完成 |
| COVERAGE_REPORT.md | ✅ | □ 完成 |
| SECURITY_ANALYSIS.md | ✅ | □ 完成 |
| PERFORMANCE_TARGETS.md | ✅ | □ 完成 |
| QUICK_START.md | ✅ | □ 完成 |
| INSTALL.md | ✅ | □ 完成 |
| API_DOCUMENTATION.md | ✅ | □ 完成 |

#### OO 架构文档 (docs/releases/vX.Y.Z/oo/)

| 文档 | 必选 | 检查状态 |
|------|------|----------|
| oo/README.md | ✅ | □ 完成 |
| oo/architecture/ARCHITECTURE_VX.Y.md | ✅ | □ 完成 |
| oo/user-guide/USER_MANUAL.md | ✅ | □ 完成 |
| oo/reports/PERFORMANCE_ANALYSIS.md | ✅ | □ 完成 |
| oo/reports/SQL92_COMPLIANCE.md | ✅ | □ 完成 |

#### 模块设计文档 (docs/releases/vX.Y.Z/oo/modules/)

| 模块 | 必选 | 文档 | 检查状态 |
|------|------|------|----------|
| MVCC | ✅ | mvcc/MVCC_DESIGN.md | □ 完成 |
| WAL | ✅ | wal/WAL_DESIGN.md | □ 完成 |
| Executor | ✅ | executor/EXECUTOR_DESIGN.md | □ 完成 |
| Parser | ✅ | parser/PARSER_DESIGN.md | □ 完成 |
| Graph | ✅ | graph/GRAPH_DESIGN.md | □ 完成 |
| Vector | ✅ | vector/VECTOR_DESIGN.md | □ 完成 |
| Storage | ✅ | storage/STORAGE_DESIGN.md | □ 完成 |
| Optimizer | ✅ | optimizer/OPTIMIZER_DESIGN.md | □ 完成 |
| Catalog | ✅ | catalog/CATALOG_DESIGN.md | □ 完成 |
| Planner | ✅ | planner/PLANNER_DESIGN.md | □ 完成 |
| Transaction | ✅ | transaction/TRANSACTION_DESIGN.md | □ 完成 |
| Server | ✅ | server/SERVER_DESIGN.md | □ 完成 |
| Bench | ✅ | bench/BENCH_DESIGN.md | □ 完成 |
| Unified Query | ✅ | unified-query/UNIFIED_QUERY_DESIGN.md | □ 完成 |

### 2.2 文档质量检查

| 检查项 | 标准 | 检查状态 |
|--------|------|----------|
| 版本号一致 | 所有文档版本号统一 | □ 完成 |
| 日期更新 | 文档日期为当前日期 | □ 完成 |
| 链接有效 | 无失效链接 | □ 完成 |
| 格式正确 | Markdown 格式规范 | □ 完成 |
| 内容完整 | 无内容缺失 | □ 完成 |

### 2.3 版本对比检查

| 检查项 | 标准 | 检查状态 |
|--------|------|----------|
| 与上一版本对齐 | 文档结构与上一版本一致 | □ 完成 |
| 新增特性文档 | 新版本特性已文档化 | □ 完成 |
| 性能指标更新 | 性能指标已更新 | □ 完成 |
| 安全分析更新 | 安全分析已更新 | □ 完成 |

## 三、门禁检查执行

### 3.1 执行命令

```bash
# 1. 检查文档完整性
./scripts/check_documentation.sh vX.Y.Z

# 2. 检查链接有效性
./scripts/check_links.sh docs/releases/vX.Y.Z/

# 3. 检查文档格式
./scripts/check_format.sh docs/releases/vX.Y.Z/

# 4. 生成文档报告
./scripts/generate_documentation_report.sh vX.Y.Z
```

### 3.2 检查结果

| 检查项 | 结果 | 备注 |
|--------|------|------|
| 文档完整性 | □ 通过 | |
| 链接有效性 | □ 通过 | |
| 格式规范 | □ 通过 | |
| 内容质量 | □ 通过 | |

## 四、门禁决策

### 4.1 决策标准

| 状态 | 条件 | 决策 |
|------|------|------|
| 通过 | 所有检查项通过 | 允许发布 |
| 警告 | 非关键检查项未通过 | 有条件通过 |
| 失败 | 关键检查项未通过 | 不允许发布 |

### 4.2 关键检查项

| 检查项 | 级别 | 说明 |
|--------|------|------|
| 根目录必选文档 | 关键 | 必须全部存在 |
| OO 架构文档 | 关键 | 必须全部存在 |
| 模块设计文档 | 关键 | 必须全部存在 |
| 安全分析 | 关键 | 必须通过 |
| 性能测试 | 关键 | 必须达标 |

### 4.3 决策记录

| 决策 | 日期 | 责任人 | 备注 |
|------|------|--------|------|
| | | | |

## 五、附录

### 5.1 检查脚本

#### check_documentation.sh

```bash
#!/bin/bash

VERSION=$1
BASE_DIR="docs/releases/$VERSION"

# 检查根目录文档
ROOT_DOCS=(README.md CHANGELOG.md RELEASE_NOTES.md MIGRATION_GUIDE.md DEPLOYMENT_GUIDE.md DEVELOPMENT_GUIDE.md TEST_PLAN.md TEST_MANUAL.md EVALUATION_REPORT.md DOCUMENT_AUDIT.md FEATURE_MATRIX.md COVERAGE_REPORT.md SECURITY_ANALYSIS.md PERFORMANCE_TARGETS.md QUICK_START.md INSTALL.md API_DOCUMENTATION.md)

for doc in "${ROOT_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/$doc" ]; then
    echo "ERROR: Missing root document: $doc"
    exit 1
  fi
done

# 检查 OO 文档
OO_DOCS=(README.md architecture/ARCHITECTURE_V* user-guide/USER_MANUAL.md reports/PERFORMANCE_ANALYSIS.md reports/SQL92_COMPLIANCE.md)

for doc in "${OO_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/oo/$doc" ]; then
    echo "ERROR: Missing OO document: $doc"
    exit 1
  fi
done

# 检查模块文档
MODULE_DOCS=(mvcc/MVCC_DESIGN.md wal/WAL_DESIGN.md executor/EXECUTOR_DESIGN.md parser/PARSER_DESIGN.md graph/GRAPH_DESIGN.md vector/VECTOR_DESIGN.md storage/STORAGE_DESIGN.md optimizer/OPTIMIZER_DESIGN.md catalog/CATALOG_DESIGN.md planner/PLANNER_DESIGN.md transaction/TRANSACTION_DESIGN.md server/SERVER_DESIGN.md bench/BENCH_DESIGN.md unified-query/UNIFIED_QUERY_DESIGN.md)

for doc in "${MODULE_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/oo/modules/$doc" ]; then
    echo "ERROR: Missing module document: $doc"
    exit 1
  fi
done

echo "All documentation checks passed!"
```

### 5.2 参考文档

| 文档 | 说明 |
|------|------|
| DOCUMENT_COMPLETENESS_CHECK.md | 文档完整性检查指南 |
| v2.5.0 文档 | 参考标准 |
| v2.6.0 文档 | 参考标准 |

---

*门禁检查模板 v1.0.0*
*最后更新: 2026-04-19*

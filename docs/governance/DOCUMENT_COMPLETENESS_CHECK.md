# 文档完整性检查与补全提示词

**版本**: v1.0.0
**创建日期**: 2026-04-19
**适用范围**: 版本发布前文档自查

---

## 一、检查目的

确保每个版本的文档完整、一致且符合 SQLRustGo 的文档标准，为生产就绪版本提供充分的文档支持。

## 二、检查范围

### 2.1 根目录文档 (docs/releases/vX.Y.Z/)

| 文档 | 必选 | 说明 |
|------|------|------|
| README.md | ✅ | 版本入口文档 |
| CHANGELOG.md | ✅ | 变更日志 |
| RELEASE_NOTES.md | ✅ | 发布说明 |
| MIGRATION_GUIDE.md | ✅ | 升级指南 |
| DEPLOYMENT_GUIDE.md | ✅ | 部署指南 |
| DEVELOPMENT_GUIDE.md | ✅ | 开发指南 |
| TEST_PLAN.md | ✅ | 测试计划 |
| TEST_MANUAL.md | ✅ | 测试手册 |
| EVALUATION_REPORT.md | ✅ | 版本评估报告 |
| DOCUMENT_AUDIT.md | ✅ | 文档审计报告 |
| FEATURE_MATRIX.md | ✅ | 功能矩阵 |
| COVERAGE_REPORT.md | ✅ | 覆盖率报告 |
| SECURITY_ANALYSIS.md | ✅ | 安全分析 |
| PERFORMANCE_TARGETS.md | ✅ | 性能目标 |
| QUICK_START.md | ✅ | 快速开始 |
| INSTALL.md | ✅ | 安装说明 |
| API_DOCUMENTATION.md | ✅ | API 文档 |

### 2.2 OO 架构文档 (docs/releases/vX.Y.Z/oo/)

| 文档 | 必选 | 说明 |
|------|------|------|
| oo/README.md | ✅ | OO 文档索引 |
| oo/architecture/ARCHITECTURE_VX.Y.md | ✅ | 版本架构设计 |
| oo/user-guide/USER_MANUAL.md | ✅ | 用户手册 |
| oo/reports/PERFORMANCE_ANALYSIS.md | ✅ | 性能分析 |
| oo/reports/SQL92_COMPLIANCE.md | ✅ | SQL 合规报告 |

### 2.3 模块设计文档 (docs/releases/vX.Y.Z/oo/modules/)

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

## 三、检查步骤

### 步骤 1: 目录结构检查

```bash
# 检查版本目录是否存在
ls -la docs/releases/vX.Y.Z/

# 检查 OO 目录结构
ls -la docs/releases/vX.Y.Z/oo/

# 检查模块目录结构
ls -la docs/releases/vX.Y.Z/oo/modules/
```

### 步骤 2: 文档文件检查

```bash
# 检查根目录文档
find docs/releases/vX.Y.Z/ -name "*.md" | sort

# 检查 OO 文档
find docs/releases/vX.Y.Z/oo/ -name "*.md" | sort

# 检查模块文档
find docs/releases/vX.Y.Z/oo/modules/ -name "*.md" | sort
```

### 步骤 3: 文档内容检查

1. **完整性检查**: 确保所有必选文档存在
2. **一致性检查**: 确保版本号、日期等信息一致
3. **链接检查**: 确保文档内部链接有效
4. **格式检查**: 确保 Markdown 格式正确

### 步骤 4: 版本对比检查

```bash
# 与上一版本对比
ls -la docs/releases/vX.Y.Z/ | wc -l
ls -la docs/releases/v(X-1).Y.Z/ | wc -l
```

## 四、补全流程

### 4.1 缺失文档补全

1. **参考模板**: 参考 v2.5.0 和 v2.6.0 的文档结构
2. **内容填充**: 根据版本特性填充内容
3. **格式统一**: 保持与现有文档一致的格式

### 4.2 文档更新

1. **版本号更新**: 更新所有文档中的版本号
2. **日期更新**: 更新所有文档中的日期
3. **内容更新**: 根据版本特性更新内容

### 4.3 质量检查

1. **语法检查**: 确保 Markdown 语法正确
2. **链接检查**: 确保所有链接有效
3. **内容检查**: 确保内容完整、准确

## 五、门禁检查集成

在 CI/CD 门禁中添加以下检查:

```yaml
# 文档完整性检查
- name: Check Documentation Completeness
  run: |
    # 检查根目录文档
    required_docs=(README.md CHANGELOG.md RELEASE_NOTES.md MIGRATION_GUIDE.md DEPLOYMENT_GUIDE.md DEVELOPMENT_GUIDE.md TEST_PLAN.md TEST_MANUAL.md EVALUATION_REPORT.md DOCUMENT_AUDIT.md)
    for doc in "${required_docs[@]}"; do
      if [ ! -f "docs/releases/vX.Y.Z/$doc" ]; then
        echo "Missing required document: $doc"
        exit 1
      fi
    done

    # 检查 OO 文档
    if [ ! -f "docs/releases/vX.Y.Z/oo/README.md" ]; then
      echo "Missing OO README"
      exit 1
    fi

    # 检查模块文档
    module_docs=(mvcc/MVCC_DESIGN.md wal/WAL_DESIGN.md executor/EXECUTOR_DESIGN.md parser/PARSER_DESIGN.md graph/GRAPH_DESIGN.md vector/VECTOR_DESIGN.md storage/STORAGE_DESIGN.md optimizer/OPTIMIZER_DESIGN.md catalog/CATALOG_DESIGN.md planner/PLANNER_DESIGN.md transaction/TRANSACTION_DESIGN.md server/SERVER_DESIGN.md bench/BENCH_DESIGN.md unified-query/UNIFIED_QUERY_DESIGN.md)
    for doc in "${module_docs[@]}"; do
      if [ ! -f "docs/releases/vX.Y.Z/oo/modules/$doc" ]; then
        echo "Missing module document: $doc"
        exit 1
      fi
    done

    echo "All required documents are present"
```

## 六、检查清单

### 发布前自查清单

- [ ] 所有必选根目录文档存在
- [ ] 所有必选 OO 文档存在
- [ ] 所有必选模块文档存在
- [ ] 文档版本号一致
- [ ] 文档日期更新
- [ ] 文档内容完整
- [ ] 文档链接有效
- [ ] 文档格式正确
- [ ] 与上一版本文档结构对齐

### 版本特定检查

- [ ] 新版本特性文档已更新
- [ ] 性能指标文档已更新
- [ ] 安全分析文档已更新
- [ ] 测试计划文档已更新
- [ ] 升级指南已更新

## 七、参考文档

| 文档 | 说明 |
|------|------|
| v2.5.0 文档 | 参考标准 |
| v2.6.0 文档 | 参考标准 |
| DOCUMENT_AUDIT.md | 审计模板 |

---

*文档检查与补全提示词 v1.0.0*
*最后更新: 2026-04-19*

#!/bin/bash

# 文档完整性检查脚本
# 用于版本发布前的文档检查

set -e

VERSION=$1
BASE_DIR="docs/releases/$VERSION"

# 检查参数
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 v2.6.0"
  exit 1
fi

# 检查版本目录是否存在
if [ ! -d "$BASE_DIR" ]; then
  echo "ERROR: Version directory not found: $BASE_DIR"
  exit 1
fi

# 检查根目录文档
ROOT_DOCS=(README.md CHANGELOG.md RELEASE_NOTES.md MIGRATION_GUIDE.md DEPLOYMENT_GUIDE.md DEVELOPMENT_GUIDE.md TEST_PLAN.md TEST_MANUAL.md EVALUATION_REPORT.md DOCUMENT_AUDIT.md FEATURE_MATRIX.md COVERAGE_REPORT.md SECURITY_ANALYSIS.md PERFORMANCE_TARGETS.md QUICK_START.md INSTALL.md API_DOCUMENTATION.md)

missing_docs=()

for doc in "${ROOT_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/$doc" ]; then
    missing_docs+="$doc"
  fi
done

# 检查 OO 文档
OO_DOCS=(README.md user-guide/USER_MANUAL.md reports/PERFORMANCE_ANALYSIS.md reports/SQL92_COMPLIANCE.md)

for doc in "${OO_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/oo/$doc" ]; then
    missing_docs+="oo/$doc"
  fi
done

# 检查架构文档（处理版本号）
arch_doc=$(find "$BASE_DIR/oo/architecture/" -name "ARCHITECTURE_V*.md" | head -1)
if [ -z "$arch_doc" ]; then
  missing_docs+="oo/architecture/ARCHITECTURE_V*.md"
fi

# 检查模块文档
MODULE_DOCS=(mvcc/MVCC_DESIGN.md wal/WAL_DESIGN.md executor/EXECUTOR_DESIGN.md parser/PARSER_DESIGN.md graph/GRAPH_DESIGN.md vector/VECTOR_DESIGN.md storage/STORAGE_DESIGN.md optimizer/OPTIMIZER_DESIGN.md catalog/CATALOG_DESIGN.md planner/PLANNER_DESIGN.md transaction/TRANSACTION_DESIGN.md server/SERVER_DESIGN.md bench/BENCH_DESIGN.md unified-query/UNIFIED_QUERY_DESIGN.md)

for doc in "${MODULE_DOCS[@]}"; do
  if [ ! -f "$BASE_DIR/oo/modules/$doc" ]; then
    missing_docs+="oo/modules/$doc"
  fi
done

# 输出结果
if [ ${#missing_docs[@]} -eq 0 ]; then
  echo "✅ All required documentation files are present"
  echo "Version: $VERSION"
  echo "Directory: $BASE_DIR"
  echo "Documentation check passed!"
  exit 0
else
  echo "❌ Missing documentation files:"
  for doc in "${missing_docs[@]}"; do
    echo "  - $doc"
  done
  echo ""
  echo "Total missing: ${#missing_docs[@]}"
  echo "Documentation check failed!"
  exit 1
fi

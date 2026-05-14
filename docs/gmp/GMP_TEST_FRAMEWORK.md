# GMP_TEST_FRAMEWORK.md - 测试体系框架

**文件路径**: `/Users/liying/workspace/dev/yinglichina163/sqlrustgo/docs/gmp/GMP_TEST_FRAMEWORK.md`  
**版本**: v1.0.0  
**状态**: ACTIVE  
**创建日期**: 2026-05-14  
**最后更新**: 2026-05-14  

---

## 1. 概述

本框架定义了 SQLRustGo GMP 可信可证明系统的测试体系，确保所有测试活动产生可验证、可追溯的证据。

**核心目标**: 测试不仅是发现缺陷的手段，更是产生"可信证据"的过程。

---

## 2. 测试金字塔

```
                        ▲
                       /│\
                      / │ \
                     /  │  \
                    /   │   \
                   /    │    \
                  /     │     \
                 /      │      \
                /       │       \
               /────────┼────────\
              │   L1: 形式化证明    │
              │  (Formal Proof)    │
              ├────────────────────┤
              │   L2: 集成测试     │
              │ (Integration Test) │
              ├────────────────────┤
              │   L3: 组件测试     │
              │  (Unit Test)       │
              ├────────────────────┤
              │   L4: 静态分析     │
              │ (Static Analysis)  │
              └────────────────────┘

测试成本: 低 ───────────────────────────────────────▶ 高
覆盖深度: 低 ───────────────────────────────────────▶ 高
```

---

## 3. 测试级别定义

### 3.1 L1: 形式化证明 (Formal Proof)

**目标**: 证明关键算法和协议的安全性

**适用场景**:
- SQL 解析器语义正确性
- 事务隔离级别实现
- 加密/解密算法实现
- 协议状态机转换

**证据要求**:

```json
{
  "proof_type": "L1_FORMAL",
  "method": "model_checking | theorem_proving | abstract_interpretation",
  "tool": "TLA+ | Coq | K-framework | CBMC",
  "model_file": "path/to/model.pml",
  "properties_verified": [
    "safety_invariants",
    "deadlock_freedom", 
    "atomicity_preservation"
  ],
  "proof_certificate": "path/to/proof.cert",
  "coverage": "100% state space (bounded) or inductive proof"
}
```

**禁止的模式**:

| 禁止 | 原因 |
|------|------|
| 跳过边界条件证明 | 可能导致关键路径漏洞 |
| 使用非形式化"直观"论证 | 缺乏可验证性 |
| 假设环境正确 | 必须验证所有假设 |

### 3.2 L2: 集成测试 (Integration Test)

**目标**: 验证组件间交互的正确性

**适用场景**:
- 客户端与服务器交互
- SQL 引擎各组件协作
- 存储引擎与查询处理器接口
- 复制与故障转移机制

**测试结构**:

```
tests/integration/
├── sql/
│   ├── select/
│   │   ├── basic_select.rs       # 基本 SELECT 测试
│   │   ├── join_select.rs        # JOIN 操作测试
│   │   └── subquery_select.rs    # 子查询测试
│   ├── dml/
│   │   ├── insert_test.rs
│   │   ├── update_test.rs
│   │   └── delete_test.rs
│   └── transaction/
│       ├── commit_test.rs
│       ├── rollback_test.rs
│       └── isolation_level_test.rs
├── protocol/
│   ├── simple_query_test.rs
│   ├── extended_query_test.rs
│   └── prepared_statement_test.rs
└── storage/
    ├── page_read_test.rs
    ├── page_write_test.rs
    └── index_update_test.rs
```

**证据格式**:

```json
{
  "test_type": "L2_INTEGRATION",
  "test_suite": "sql/transaction/isolation_level_test",
  "execution_time_ms": 1523,
  "tests_total": 47,
  "tests_passed": 47,
  "tests_failed": 0,
  "coverage": {
    "branch": "94.2%",
    "line": "98.1%"
  },
  "evidence_files": [
    "tests/integration/sql/transaction/test_results.xml",
    "coverage_report/lcov.info"
  ],
  "deterministic": true,
  "repeatable": true
}
```

### 3.3 L3: 组件测试 (Unit Test)

**目标**: 验证每个模块的功能正确性

**适用场景**: 所有公开 API 和内部关键函数

**测试结构**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // 单一功能测试
    #[test]
    fn test_hash_join_build_phase() {
        let mut builder = HashJoinBuilder::new();
        let probe_data = vec![
            row!("A", 1),
            row!("B", 2),
        ];
        let result = builder.build(probe_data);
        assert!(result.is_ok());
    }
    
    // 边界条件测试
    #[test]
    fn test_hash_join_empty_input() {
        let empty_data: Vec<Row> = vec![];
        let result = hash_join(empty_data);
        assert!(result.is_ok());
    }
    
    // 错误处理测试
    #[test]
    fn test_hash_join_memory_exhaustion() {
        let large_data = generate_large_dataset(u64::MAX);
        let result = hash_join(large_data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HashJoinError::OutOfMemory));
    }
}
```

**覆盖率要求**:

| 代码类型 | 最低覆盖率 |
|----------|-----------|
| 新增代码 | 90% 分支覆盖 |
| 修改代码 | 85% 分支覆盖 |
| 关键路径 (T1) | 100% MC/DC |

### 3.4 L4: 静态分析 (Static Analysis)

**目标**: 在不运行代码的情况下发现潜在缺陷

**工具链**:

| 工具 | 用途 | 配置 |
|------|------|------|
| cargo clippy | Rust 代码质量 | .clippy.toml |
| cargo audit | 依赖安全 | audit.toml |
| cargo machete | 检测未使用的导入 | .machete.toml |
| gitleaks | 密钥检测 | gitleaks.toml |
| semgrep | 通用模式匹配 | .semgrep.yml |

**CI 门控配置**:

```yaml
# .gmp/ci-gate.yml
static_analysis:
  clippy:
    level: deny
    warnings_as_errors: true
    allowed_warnings: []
  
  audit:
    deny: advisories
   deny_cves: true
  
  format:
    check: cargo fmt -- --check
    enforce: true
```

---

## 4. 测试执行流程

### 4.1 流程图

```
┌─────────────────────────────────────────────────────────────────┐
│                     测试执行流程                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  代码提交 ──▶ L4 静态分析 ──▶ L3 组件测试 ──▶ L2 集成测试        │
│     │              │              │              │               │
│     │              ▼              ▼              ▼               │
│     │         格式检查        覆盖率检查     交互验证              │
│     │         警告检查        分支覆盖      性能基准              │
│     │              │              │              │               │
│     │         ─── PASS ───      │              │               │
│     │              │              │              │               │
│     │              ▼              ▼              ▼               │
│     │         ─── FAIL ─── ─── FAIL ─── ─── FAIL ───            │
│     │              │              │              │               │
│     │              ▼              ▼              ▼               │
│     │         报告问题       报告问题       报告问题              │
│     │              │              │              │               │
│     │              ▼              ▼              ▼               │
│     │         ────────────────────────────────                  │
│     │                        │                                  │
│     ▼                        ▼                                  │
│  ─── PROCEED TO L1 FORMAL PROOF (if required) ───                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 门控标准

| 测试级别 | 通过标准 | 失败处理 |
|----------|----------|----------|
| L4 静态分析 | 0 warnings/errors | 必须修复 |
| L3 组件测试 | 100% 测试通过, 覆盖率达标 | 必须修复 |
| L2 集成测试 | 100% 测试通过, 性能达标 | 必须修复 |
| L1 形式化证明 | 证明完成且验证 | 必须修复或接受风险 |

---

## 5. 证据产生与收集

### 5.1 证据自动收集配置

```yaml
# .gmp/evidence-collection.yml
evidence_collection:
  enabled: true
  output_dir: evidence/{branch}/{timestamp}/
  
  collectors:
    - type: coverage
      tool: cargo-llvm-cov
      format: lcov, html, json
      threshold:
        line: 80%
        branch: 80%
        critical: 100%
    
    - type: test_results
      tool: cargo-test
      format: json, xml
      junit_output: test-results/junit.xml
    
    - type: static_analysis
      tool: cargo-clippy
      format: json
      output: static-analysis/clippy.json
    
    - type: security_scan
      tool: cargo-audit
      format: json
      output: security/audit.json
    
    - type: complexity
      tool: cargo-msrv
      output: complexity/report.json
```

### 5.2 证据包生成

```bash
#!/bin/bash
# scripts/generate-evidence.sh

set -e

VERSION=${1:-"0.0.0-$(git rev-parse --short HEAD)"}
EVIDENCE_DIR="evidence/${VERSION}"

mkdir -p "${EVIDENCE_DIR}"

# 收集所有证据
cargo llvm-cov --lcov --output-path "${EVIDENCE_DIR}/coverage.lcov"
cargo test -- --report-time --format json > "${EVIDENCE_DIR}/test-results.json"
cargo clippy --message-format=json > "${EVIDENCE_DIR}/clippy.json"
cargo audit --json > "${EVIDENCE_DIR}/audit.json"

# 生成证据清单
cat > "${EVIDENCE_DIR}/MANIFEST.json" << EOF
{
  "version": "${VERSION}",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "commit": "$(git rev-parse HEAD)",
  "branch": "$(git rev-parse --abbrev-ref HEAD)",
  "evidence_files": [
    "coverage.lcov",
    "test-results.json",
    "clippy.json",
    "audit.json"
  ],
  "checksums": {
    "coverage.lcov": "$(sha256sum ${EVIDENCE_DIR}/coverage.lcov | cut -d' ' -f1)",
    "test-results.json": "$(sha256sum ${EVIDENCE_DIR}/test-results.json | cut -d' ' -f1)",
    "clippy.json": "$(sha256sum ${EVIDENCE_DIR}/clippy.json | cut -d' ' -f1)",
    "audit.json": "$(sha256sum ${EVIDENCE_DIR}/audit.json | cut -d' ' -f1)"
  }
}
EOF

echo "Evidence package generated: ${EVIDENCE_DIR}"
```

---

## 6. 测试检查清单

### 6.1 L3 组件测试检查清单

- [ ] 所有公开 API 都有对应测试
- [ ] 每个错误变体都被测试
- [ ] 边界条件被测试 (空输入、最大值、溢出)
- [ ] 并发安全测试覆盖关键区域
- [ ] 覆盖率报告已生成并达标
- [ ] 测试在干净环境可重复执行

### 6.2 L2 集成测试检查清单

- [ ] 组件接口兼容性测试
- [ ] 协议一致性测试
- [ ] 性能基准测试
- [ ] 故障注入测试
- [ ] 超时和取消测试
- [ ] 资源清理测试

### 6.3 L1 形式化证明检查清单

- [ ] 模型已定义并验证
- [ ] 所有不变量已声明
- [ ] 证明已完成或给出反例
- [ ] 证明已通过证明检查器验证
- [ ] 关键假设已明确记录

### 6.4 禁止的测试模式

| 禁止模式 | 原因 | 正确做法 |
|----------|------|----------|
| 测试中硬编码时间 | 环境依赖 | 使用相对时间或 mock |
| 测试间共享可变状态 | 非确定性 | 每个测试独立 setup |
| 忽略失败的测试 | 隐患积累 | 立即修复或标记 known-issue |
| 测试覆盖内部实现 | 脆弱性 | 只测试公共 API |
| 使用网络调用 | 非确定性 | 使用 mock 或 local server |

---

## 7. 性能测试要求

### 7.1 性能基准测试

**必须执行的基准测试**:

```yaml
benchmarks:
  - name: query_execution_time
    thresholds:
      p50: < 100ms
      p99: < 500ms
  
  - name: transaction_throughput
    thresholds:
      tps: > 10000
  
  - name: memory_usage
    thresholds:
      peak: < 1GB
  
  - name: concurrent_connections
    thresholds:
      max_stable: > 1000
```

### 7.2 性能回归检测

```json
{
  "regression_detection": {
    "enabled": true,
    "threshold_percent": 10,
    "comparison_base": "last_release",
    "action_on_regression": "block_merge"
  }
}
```

---

## 8. 报告格式

### 8.1 测试报告模板

```json
{
  "report": {
    "title": "SQLRustGo GMP Test Report",
    "version": "v2.1.0",
    "timestamp": "2026-05-14T12:00:00Z",
    "executive_summary": {
      "total_tests": 1234,
      "passed": 1230,
      "failed": 4,
      "coverage_branch": "87.3%",
      "coverage_line": "94.1%",
      "status": "PASS_WITH_CONDITIONS"
    },
    "level_summary": {
      "L1_FORMAL": {
        "required": true,
        "completed": true,
        "proofs_verified": 12
      },
      "L2_INTEGRATION": {
        "required": true,
        "tests": 156,
        "passed": 156,
        "failed": 0
      },
      "L3_UNIT": {
        "required": true,
        "tests": 1047,
        "passed": 1043,
        "failed": 4
      },
      "L4_STATIC": {
        "required": true,
        "warnings": 0,
        "errors": 0
      }
    },
    "defects": [
      {
        "id": "DEF-001",
        "severity": "HIGH",
        "description": "Hash join produces incorrect results on NULL keys",
        "test_case": "test_hash_join_null_handling",
        "status": "OPEN"
      }
    ]
  }
}
```

---

## 9. 参考文档

- GMP_STANDARD.md - GMP 规范总纲
- GMP_PROVABLE_SYSTEM.md - 可证明系统分析
- GMP_LIFECYCLE.md - 生命周期管理

---

## 10. 修订历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|----------|------|
| v1.0.0 | 2026-05-14 | 初始版本 | GMP Agent |

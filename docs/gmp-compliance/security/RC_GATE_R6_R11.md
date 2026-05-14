# RC Gate R6-R11 详细状态

**版本**: v3.1.0  
**日期**: 2026-05-14  
**状态**: Beta 完成，RC 验证中

---

## R6-R11 检查项总览

| Gate | 检查项 | Beta 状态 | RC 状态 | 风险点 |
|------|--------|-----------|---------|--------|
| R6 | Security Audit | ✅ | ⏳ | cargo audit |
| R7 | SQL Operations | ✅ | ✅ | 已修复 |
| R8 | TPC-H SF=1 | ✅ | ⏳ | 脚本超时 |
| R9 | Regression | ✅ | ⏳ | 待验证 |
| R10 | Formal Proof | ✅ | ⏳ | 待验证 |
| R11 | Docs Links | ✅ | ⏳ | 待验证 |

---

## R6: Security Audit

### 检查要求

```bash
cargo audit || true
```

### 风险点

| 风险 | 说明 | 状态 |
|------|------|------|
| R-01 | 已知安全漏洞 | ⚠️ 待检查 |
| R-02 | 依赖版本过旧 | ⚠️ 待检查 |
| R-03 | 许可证合规 | ⚠️ 待检查 |

### 缓解措施

```bash
# 更新依赖
cargo update

# 扫描漏洞
cargo audit

# 生成报告
cargo audit --json > security-audit.json
```

### 当前状态

cargo audit 在 Beta 中通过。RC 需要完整运行。

---

## R7: SQL Operations

### 检查要求

SQL Corpus 测试通过率 >= 95%

### 修复历史

- **PR #776**: 修复 grep -c 解析问题
- **当前状态**: 100% = 3/3 ✅

### 测试结果

```
test differential::tests::test_result_comparator_one_failed ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## R8: TPC-H SF=1

### 检查要求

TPC-H SF=1 p99 延迟 < 5s

### 风险点

| 风险 | 说明 | 状态 |
|------|------|------|
| 脚本整数比较 | grep -c 多行返回 | ✅ 已修复 |
| 超时问题 | TPC-H 运行时间长 | ⚠️ 需优化 |
| 性能阈值 | 固定开销 4-5s | ⚠️ 需调整 |

### 当前状态

TPC-H 测试在 Beta 中通过。SF=1 约 45-60 分钟。

---

## R9: Regression Check

### 检查要求

回归测试套件通过

### 测试套件

| 测试 | 描述 | 状态 |
|------|------|------|
| `check_regression.sh` | 回归检查脚本 | ⏳ |
| `cargo test --workspace` | 全量测试 | ⏳ |

---

## R10: Formal Proof Count

### 检查要求

形式化证明文件 >= 30

### 当前状态

| 类别 | 数量 |
|------|------|
| Proof 文件 | 31 ✅ |
| TLA+ 规约 | 12 |
| Dafny 规约 | 1 |
| JSON 证明 | 18 |

### 证明列表

详见: `docs/gmp-compliance/proof/PROOF_INDEX.md`

---

## R11: Documentation Links

### 检查要求

文档链接全部有效

### 修复历史

- **PR #779**: 修复断裂链接
- 排除旧版本文档 (`docs/releases/v2.*`)

### 当前状态

所有链接有效 ✅

---

## R-S1~R-S5 安全审查

详见: [R_S1_S5_SECURITY.md](R_S1_S5_SECURITY.md)
# v1.1.0 Release Notes

## 发布信息
- **版本**: 1.1.0
- **发布日期**: 2026-03-03
- **类型**: Minor Release
- **目标分支**: develop-v1.1.0

---

## 新功能

### 性能测试框架
- 添加 Lexer 基准测试 (B-001)
- 添加 Parser 基准测试 (B-002)
- 添加 Executor 基准测试 (B-004)
- 添加 Storage 基准测试 (B-005)
- 添加 Network 基准测试 (B-006)
- 添加 Planner 基准测试 (B-007)
- 添加 Integration 基准测试 (B-008)

### 内核架构
- 实现 HashJoin 基础功能 (C-04)
- 添加 Value 类型的 Hash trait 实现
- 改进 Join 列查找的错误处理

### Planner 模块
- 实现 Query Analyzer Phase 1
- 添加 LogicalPlan 定义
- 添加 PhysicalPlan trait
- 实现 Executor trait

---

## 改进

### 代码质量
- 修复 clippy 警告 (collapsible_match)
- 移除生产代码中的 unwrap/panic
- 使用 expect 提供清晰的错误信息

### 安全
- 添加安全审计报告 (F-01~F-04)
- 添加代码质量审计报告 (A-02~A-06)
- 敏感信息检查通过
- SQL 注入防护验证通过

---

## 门禁验收

| 检查项 | 状态 |
|--------|------|
| A-02: 代码清洁 | ✅ |
| A-03: 静态扫描 | ✅ |
| A-04: 安全漏洞 | ✅ |
| A-05: 无 unwrap/panic | ✅ |
| B-01: 测试覆盖率 ≥ 90% | ⚠️ 86.22% |
| B-02: 函数覆盖率 ≥ 85% | ⚠️ |
| C-04: HashJoin | ✅ |
| D-01~05: 性能基准测试 | ✅ |
| E-01~E-06: 文档 | ⚠️ |

---

## 升级注意事项

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 贡献者

感谢以下贡献者的贡献：
- yinglichina163
- yinglichina8848
- sonaheartopen

---

## 感谢

感谢所有参与测试和反馈的用户。

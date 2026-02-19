# v1.0 冻结原则

> v1.0 版本冻结规则

---

## 核心原则

v1.0 不是功能强。v1.0 是：

- ✅ 可运行
- ✅ 不崩溃
- ✅ 有测试
- ✅ 有文档
- ✅ 有版本
- ✅ 有 CI
- ✅ 有 Release Note
- ✅ 符合工程规范

**而不是**：更快、更复杂、更多 SQL、更高级优化器

---

## 冻结规则

### ❌ 禁止

| 类型 | 说明 |
|------|------|
| 新功能 | 不添加任何新功能 |
| 性能优化 | 不做性能相关改动 |
| 架构重构 | 不重构架构 |
| API 变更 | 不修改公共 API |
| 新模块 | 不添加新模块 |

### ✅ 允许

| 类型 | 说明 |
|------|------|
| panic 修复 | 修复可能导致 panic 的代码 |
| unwrap 移除 | 替换 unwrap 为正确错误处理 |
| 测试补充 | 增加测试覆盖率 |
| 文档完善 | 补充文档 |
| CI 修复 | 修复 CI 问题 |

---

## 发布条件

发布前必须满足：

| 项目 | 状态 |
|------|------|
| 无 unwrap | ✅ |
| 无 panic | ✅ |
| CI 全绿 | ✅ |
| 测试通过 | ✅ |
| 有版本 tag | ✅ |
| 有 Release Note | ✅ |
| README 清晰 | ✅ |
| 依赖锁定 | ✅ |

---

## 错误处理标准

### 统一错误类型

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Auth error: {0}")]
    AuthError(String),
}

pub type DbResult<T> = Result<T, DbError>;
```

### 禁止

```rust
❌ Result<T, String>
❌ .unwrap()
❌ .expect()
❌ panic!
```

---

## 覆盖率要求

| 模块 | 最低覆盖率 |
|------|-----------|
| parser | 80% |
| executor | 80% |
| storage | 80% |
| 整体 | 75% |

---

## 版本号策略

### Beta 阶段

```
v1.0.0-beta.1
v1.0.0-beta.2
v1.0.0-beta.3
```

### RC 阶段

```
v1.0.0-rc.1
```

RC 必须运行 ≥ 3 天，无重大问题。

### 正式发布

```
v1.0.0
```

---

## 战略定位

```
v1.0 = 工程完整性版本
v2.0 = 能力跃迁版本
```

**不要在 1.x 做架构革命。**

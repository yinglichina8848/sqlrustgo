# v2.4.0 Release Notes

## ⚠️ 版本合并声明

> **重要**: v2.3.0 开发代号与 v2.4.0 正式版本合并为 v2.4.0。
> - v2.3.0: 原开发代号
> - v2.4.0: 正式发布版本
> - 无 v2.3.0 正式发布版本

## 概述
v2.4.0 是 SQLRustGo 的图引擎与智能查询版本，引入 Graph Engine、OpenClaw API、列式存储压缩等核心功能。

## 发布日期
**2026-04-09 (GA)**

## 代号
**Graph Intelligence** - 图引擎 + 智能查询版

## 当前状态
**GA (General Availability)** - 2026-04-09 正式发布

---

## 主要功能

### Graph Engine

| 功能 | Issue | 状态 |
|------|-------|------|
| Graph Engine 核心架构 | #1077 | ✅ |
| GQL Parser 图查询解析器 | #1077 | ✅ |
| Graph Planning 图查询规划 | #1077 | ✅ |
| Graph Execution 图遍历执行 | #1077 | ✅ |
| 独立 crate 实现 | #1077 | ✅ |

### OpenClaw API

| 功能 | Issue | 状态 |
|------|-------|------|
| OpenClaw HTTP Server | #1078 | ✅ |
| `/query` SQL 查询端点 | #1078 | ✅ |
| `/nl_query` 自然语言查询 | #1078 | ✅ |
| `/schema` Schema introspection | #1078 | ✅ |
| `/stats` 执行统计 | #1078 | ✅ |
| `/memory/*` 记忆管理 | #1078 | ✅ |

### 列式存储压缩

| 功能 | Issue | 状态 |
|------|-------|------|
| LZ4 压缩支持 | #1302 | ✅ |
| Zstd 压缩支持 | #1302 | ✅ |
| ColumnDefinition 字段 | #1302 | ✅ |

### CBO 索引选择

| 功能 | Issue | 状态 |
|------|-------|------|
| Cost-based optimizer | #1303 | ✅ |
| Index selection | #1303 | ✅ |
| Query planning optimization | #1303 | ✅ |

### TPC-H SF=1 性能

| 功能 | Issue | 状态 |
|------|-------|------|
| SF=1 完整测试 | #1304 | ✅ |
| 性能基准报告 | #1304 | ✅ |
| 关键查询优化 | #1304 | ✅ |

---

## 核心技术特性

### 1. Graph Engine
- 独立 `graph-engine` crate
- GQL (Graph Query Language) 支持
- 图遍历执行器
- 属性图模型

### 2. OpenClaw API
- RESTful API 设计
- 自然语言到 SQL 转换
- Schema introspection
- 记忆管理系统

### 3. 列式存储压缩
- LZ4 高速压缩
- Zstd 高压缩率
- 可配置压缩策略

### 4. CBO 索引选择
- 基于成本的优化
- 索引选择启发式算法
- 查询计划优化

---

## ID 约束说明

- **内部 ID**: u64/u32 (高性能)
- **UUID**: 仅作为 property 字段使用
- **API**: 返回 Iterator，支持 lazy evaluation

---

## 升级说明

从 v2.3.0 升级：
1. Graph Engine 为独立 crate，无需特殊迁移
2. API 返回类型从 Vec 改为 Iterator，无需破坏性更改
3. compression 字段为可选，不影响现有功能

---

## 测试统计

| 测试类别 | 通过 | 总数 | 状态 |
|----------|------|------|------|
| 单元测试 | 35 | 35 | ✅ 100% |
| 集成测试 | 1040 | 1042 | ✅ 99.8% |
| TPC-H SF=1 | 11 | 11 | ✅ 100% |
| OpenClaw API | 11 | 11 | ✅ 100% |

---

## Issue 完成统计

| 模块 | Issue | 状态 |
|------|-------|------|
| Graph Engine | #1077 | ✅ |
| OpenClaw API | #1078 | ✅ |
| Columnar Compression | #1302 | ✅ |
| CBO Index Selection | #1303 | ✅ |
| TPC-H SF=1 | #1304 | ✅ |

**v2.4.0: 13/13 Issues 已关闭**

---

## 门禁状态

| 检查项 | 状态 |
|--------|------|
| 编译检查 | ✅ 通过 |
| 测试检查 | ✅ 通过 |
| Clippy | ✅ 通过 (warnings only) |
| 格式化 | ✅ 通过 |
| TPC-H SF=1 | ✅ 11/11 |
| OpenClaw API | ✅ 11/11 |
| 二进制文件清理 | ✅ 完成 |

---

## 重要 PR 列表

| PR | 描述 | 日期 |
|----|------|------|
| #1325 | feat(optimizer): CBO-based index selection | 2026-04-09 |
| #1323 | fix: Remove binary files from version control | 2026-04-08 |
| #1324 | fix: add missing compression field | 2026-04-08 |
| - | Graph Engine 实现 | 2026-04-05 |
| - | OpenClaw API 实现 | 2026-04-06 |

---

## 已知问题

无阻塞性问题。

---

## 贡献者

感谢所有参与 v2.4.0 开发的团队成员及 AI 助手 (OpenCode A/B, Claude A/B)。

---

## 下一步计划

- v2.5 开发启动 (GMP 内审 + OpenClaw 调度优化)

---

*发布版本: v2.4.0-GA*
*生成日期: 2026-04-09*

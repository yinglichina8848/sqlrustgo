# Executor 模块设计

**版本**: v2.7.0
**模块**: Executor (执行器)

## 概述

Executor 负责执行物理执行计划，将算子树转换为结果。

## 核心算子

- Scan
- Join
- Aggregate
- Sort
- Limit

## 相关文档

- [ARCHITECTURE_V2.7.md](../../architecture/ARCHITECTURE_V2.7.md)

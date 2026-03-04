# SQLRustGo v1.2.0 Release Notes

> **版本**: v1.2.0  
> **代号**: Vector Engine  
> **发布日期**: 待定  
> **状态**: 🔄 开发中

---

## 版本亮点

v1.2.0 是性能优化版本，核心目标是支持 100万行级数据处理。

### 🚀 核心特性

- **向量化执行**: RecordBatch + ColumnarArray 列式内存布局
- **统计信息**: TableStats + ColumnStats 支持成本估算
- **简化 CBO**: 基础成本优化器
- **网络层增强**: 异步服务器完善

---

## 新增功能

### 向量化执行 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
| RecordBatch | 列式内存布局 | 📝 计划中 |
| ColumnarArray trait | 列式数组抽象 | 📝 计划中 |
| 向量化表达式 | 批量表达式计算 | 📝 计划中 |
| 向量化 Filter | 批量过滤 | 📝 计划中 |
| 向量化 Projection | 批量投影 | 📝 计划中 |
| 向量化聚合 | 批量聚合 | 📝 计划中 |

### 统计信息 (P0)

| 功能 | 说明 | 状态 |
|------|------|------|
| TableStats | 表级统计信息 | 📝 计划中 |
| ColumnStats | 列级统计信息 | 📝 计划中 |
| 统计信息收集器 | 自动收集统计 | 📝 计划中 |
| 统计信息持久化 | 存储到磁盘 | 📝 计划中 |
| ANALYZE 命令 | 手动收集统计 | 📝 计划中 |

### 简化 CBO (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
| CostModel | 成本模型接口 | 📝 计划中 |
| 基础成本估算 | 扫描/过滤成本 | 📝 计划中 |
| Join 选择优化 | Join 顺序优化 | 📝 计划中 |
| 索引选择优化 | 索引使用决策 | 📝 计划中 |

### 网络层增强 (P1)

| 功能 | 说明 | 状态 |
|------|------|------|
| 会话管理完善 | 连接状态管理 | 📝 计划中 |
| 交互模式 (REPL) | 命令行交互 | 📝 计划中 |
| 配置文件支持 | 外部配置 | 📝 计划中 |
| 认证机制完善 | 用户认证 | 📝 计划中 |

---

## 性能改进

| 指标 | v1.1.0 | v1.2.0 目标 | 改进 |
|------|--------|-------------|------|
| 数据规模 | 10万行 | 100万行 | 10x |
| 简单查询延迟 | - | <100ms | - |
| 复杂查询延迟 | - | <1s | - |
| 内存效率 | 基准 | 优化 | - |

---

## API 变更

### 新增 API

```rust
// RecordBatch
pub struct RecordBatch {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    row_count: usize,
}

// Array trait
pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
}

// TableStats
pub struct TableStats {
    pub row_count: usize,
    pub total_bytes: usize,
    pub column_stats: HashMap<String, ColumnStats>,
}

// CostModel
pub struct CostModel {
    pub seq_scan_cost: f64,
    pub idx_scan_cost: f64,
    pub filter_cost: f64,
    pub join_cost: f64,
}
```

### 兼容性

- ✅ 向后兼容 v1.1.0 API
- ✅ 现有代码无需修改

---

## 升级指南

### 从 v1.1.0 升级

1. 更新依赖版本
2. 无需修改现有代码
3. 可选：使用新的向量化 API

详见 [UPGRADE_GUIDE.md](./UPGRADE_GUIDE.md)

---

## 贡献者

- @yinglichina8848
- @openheart-openheart
- @sonaheartopen
- @TRAE AI Assistant

---

## 已知问题

暂无

---

## 下一步计划

v1.3.0 计划：

- 完整 CBO 实现
- 分布式查询支持
- 事务隔离级别

---

## 变更历史

| 版本 | 日期 | 说明 |
|------|------|------|
| 1.0 | 2026-03-04 | 初始版本（计划中） |

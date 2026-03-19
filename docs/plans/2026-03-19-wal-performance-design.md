# WAL 性能优化设计文档

**日期**: 2026-03-19  
**版本**: 1.0  
**状态**: 已批准

## 1. 背景

当前 WAL (Write-Ahead Log) 实现存在严重性能瓶颈：
- **当前吞吐量**: ~13 MB/s
- **理论上限**: 500-1000 MB/s (NVMe SSD)
- **效率**: 仅达到 1.3%

### 瓶颈分析

| 瓶颈 | 影响 |
|------|------|
| 每次操作打开文件 | `log_insert()` 每次调用 `get_writer()` → `open()` |
| `to_bytes()` 每次分配 Vec | 每次分配堆内存 + 内存复制 |
| BufWriter 默认 8KB | 512 bytes 条目需多次系统调用 |

## 2. 架构变更

### 当前架构
```
WalManager ──get_writer()──> WalWriter (每次新建)
                                   │
                                   ▼
                           open(file) + BufWriter
                                   │
                                   ▼
                              append() 关闭
```

### 优化后架构
```
WalManager
    │
    ├── 持有单一 WalWriter 实例
    │
    ▼
WalWriter {
    file: Arc<Mutex<File>>,    // 复用文件句柄
    buffer: Vec<u8>,           // 预分配缓冲区
    lsn: u64
}
    │
    ▼
append() ──write_all──> BufWriter (不关闭)
```

## 3. 优化方案

### 3.1 复用文件句柄

**修改内容**:
- `WalWriter` 添加 `file: Arc<Mutex<File>>` 字段
- `WalManager` 持有 `Arc<Mutex<WalWriter>>`
- 移除 `get_writer()` 方法，改为 `writer(&self) -> Arc<Mutex<WalWriter>>`

**收益**: 消除每次 open/close 文件的系统调用开销

### 3.2 预分配 Buffer

**修改内容**:
- 添加 `WalEntry::serialized_size()` 方法
- 添加 `WalWriter::reserve_buffer(size: usize)`
- 使用 `write_all()` 替代多次 `push()` + `extend()`

**收益**: 减少堆分配次数，从 N 次减少到 1 次

### 3.3 批量序列化 API

**新增 API**:
```rust
impl WalManager {
    /// 批量插入 - 一次写入多条记录
    pub fn batch_insert(&self, entries: Vec<WalEntry>) -> Result<Vec<u64>>;
    
    /// 批量更新
    pub fn batch_update(&self, entries: Vec<WalEntry>) -> Result<Vec<u64>>;
}
```

**内部实现**:
```rust
pub fn batch_insert(&self, entries: Vec<WalEntry>) -> Result<Vec<u64>> {
    let mut writer = self.writer.lock().unwrap();
    let mut lsns = Vec::with_capacity(entries.len());
    
    for entry in entries {
        let lsn = writer.append(&entry)?;
        lsns.push(lsn);
    }
    Ok(lsns)
}
```

**收益**: 减少函数调用开销，提高缓存命中率

## 4. 性能目标

| 指标 | 当前 | 目标 | 提升 |
|------|------|------|------|
| 吞吐量 | 13 MB/s | 100-200 MB/s | 8-15x |
| 10000 条写入延迟 | 380ms | 25-50ms | 8-15x |
| CPU 利用率 | 高 | 中 | - |

## 5. 测试计划

### 5.1 功能测试
- 现有 29 个 WAL 测试全部通过
- 新增批量 API 测试
- 事务边界测试（begin/commit/rollback）

### 5.2 性能测试
- `test_wal_perf_throughput`: 目标 >= 100 MB/s
- `test_wal_perf_1000_insert`: 目标 < 100ms
- 恢复性能测试

### 5.3 回归测试
- 并发写入测试
- 崩溃恢复测试
- 大事务测试 (> 100MB)

## 6. 实现计划

### Phase 1: 重构架构 (1-2 天)
1. 修改 `WalWriter` 结构体
2. 修改 `WalManager` 持有模式
3. 更新 `append()` 方法

### Phase 2: 优化 Buffer (0.5 天)
1. 添加 `serialized_size()` 方法
2. 实现预分配 buffer

### Phase 3: 批量 API (0.5 天)
1. 添加 `batch_insert()` 方法
2. 添加 `batch_update()` 方法
3. 更新测试

### Phase 4: 验证 (0.5 天)
1. 运行所有测试
2. 性能基准测试
3. 文档更新

## 7. 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 文件句柄泄漏 | 高 | 使用 Arc<Mutex> 确保正确释放 |
| 并发写入冲突 | 中 | Mutex 保护写入操作 |
| 向后兼容性 | 低 | 保留原有 API，仅添加新 API |

## 8. 后续优化方向

- 异步 WAL 写入 (async/await)
- WAL 条目压缩 (lz4/zstd)
- 并发 WAL 读取
- SSD 直通写入 (O_DIRECT)

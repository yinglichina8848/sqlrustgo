# INSERT 性能优化设计文档

> **日期**: 2026-03-27  
> **版本**: v1.9.0  
> **状态**: 已批准

---

## 一、目标

优化 INSERT 性能，达成目标：
- **单条 INSERT**: 506 → 1000+ QPS
- **批量 INSERT**: 保持 20k+ rec/s

## 二、方案：混合模式

```
单条 INSERT → 直接写 (低延迟优先)
批量 INSERT (>=10条) → 缓冲后批量写 (吞吐量优先)
```

### 2.1 核心策略

| 场景 | 处理方式 | 持久化策略 |
|------|----------|------------|
| 单条 INSERT (< 10条) | 直接写入 | 立即持久化 |
| 批量 INSERT (>= 10条) | 内存缓冲 | 批量持久化 |

### 2.2 优化点

1. **减少不必要写入**：检测是否真的有数据变化
2. **批量缓冲**：累积多条后一次性写入
3. **延迟 flush**：WAL 不每次都强制刷盘

---

## 三、实现

### 3.1 FileStorage 修改

在 `FileStorage` 结构体中添加缓冲机制：

```rust
pub struct FileStorage {
    data_dir: PathBuf,
    tables: HashMap<String, TableData>,
    indexes: RwLock<HashMap<(String, String), BPlusTree>>,
    // 新增字段
    insert_buffer: HashMap<String, Vec<Record>>,  // 插入缓冲
    buffer_threshold: usize,  // 10 条触发 flush
    enable_buffer: bool,     // 是否启用缓冲
}
```

### 3.2 insert 方法优化

```rust
fn insert(&mut self, table: &str, records: Vec<Record>) -> SqlResult<()> {
    // 策略判断
    if records.len() >= 10 && self.enable_buffer {
        // 批量模式：缓冲
        self.insert_buffer
            .entry(table.to_string())
            .or_insert_with(Vec::new)
            .extend(records);
        
        // 达到阈值，批量写入
        if self.insert_buffer[table].len() >= self.buffer_threshold {
            self.flush_buffer(table)?;
        }
    } else {
        // 单条模式：直接写入
        self.insert_direct(table, records)?;
    }
    Ok(())
}
```

### 3.3 WAL 优化

```rust
// 不是每次都 flush，改为批量
pub fn append(&mut self, entry: &WalEntry) -> std::io::Result<u64> {
    // ... 写入逻辑 ...
    
    // 改为：只在大批量或超时时 flush
    if self.enable_batch_flush {
        self.writer.flush()?;  // 移除每次 flush
    }
    Ok(lsn)
}
```

---

## 四、验收标准

- [ ] 单条 INSERT QPS >= 1000 (从 506 提升)
- [ ] 批量 INSERT 保持 20k+ rec/s
- [ ] 崩溃恢复正常（WAL 仍有效）
- [ ] 现有测试全部通过

---

## 五、风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| 断电丢数据 | 缓冲数据丢失 | WAL 仍记录，可恢复 |
| 内存占用 | 大批量占用内存 | 设置上限，超限强制 flush |
| 代码冲突 | 多线程不安全 | 使用 Mutex 保护缓冲 |

---

*设计文档已批准，执行实现*
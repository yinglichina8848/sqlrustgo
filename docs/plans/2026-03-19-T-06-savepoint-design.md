# T-06 SAVEPOINT 设计

## 目标

实现事务内保存点支持，允许部分回滚。

## 方案

**B 级功能，A 级复杂度**

## 设计

### 1. 核心数据结构

```rust
enum UndoRecord {
    Insert { key: Vec<u8> },
    Delete { key: Vec<u8>, old_value: Vec<u8> },
    Update { key: Vec<u8>, old_value: Vec<u8> },
}

struct Savepoint {
    name: String,
    undo_log_index: usize,
}

struct Transaction {
    id: TxId,
    status: TransactionStatus,
    undo_log: Vec<UndoRecord>,
    savepoints: Vec<Savepoint>,
}
```

### 2. 三大操作

#### SAVEPOINT
```rust
fn savepoint(&mut self, name: String) {
    self.savepoints.push(Savepoint {
        name,
        undo_log_index: self.undo_log.len(),
    });
}
```
- 复杂度: O(1)

#### ROLLBACK TO
```rust
fn rollback_to(&mut self, name: &str) -> Result<()> {
    let sp = self.find_savepoint(name)?;
    
    while self.undo_log.len() > sp.undo_log_index {
        let record = self.undo_log.pop().unwrap();
        self.apply_undo(record);
    }
    
    // 删除该 savepoint 之后的所有 savepoints
    self.truncate_savepoints_after(name);
    Ok(())
}
```
- 复杂度: O(k)，k=undo 记录数

#### RELEASE SAVEPOINT
```rust
fn release_savepoint(&mut self, name: &str) -> Result<()> {
    self.savepoints.retain(|s| s.name != name);
    Ok(())
}
```
- 复杂度: O(n)

### 3. 关键约束

| 约束 | 处理 |
|------|------|
| 同名 savepoint | 后者覆盖前者（栈语义） |
| 查找策略 | 从 Vec 尾部往前找 |
| 嵌套关系 | Vec 栈结构 |
| ROLLBACK TO | 不删除当前 savepoint |

### 4. 与死锁系统

无直接关系 - savepoint 是本地事务逻辑，deadlock 是锁管理。

## 复杂度分析

| 操作 | 复杂度 |
|------|--------|
| SAVEPOINT | O(1) |
| RELEASE | O(n) |
| ROLLBACK TO | O(k) |

## 演进路径

- Phase 1: 基础功能（当前）
- Phase 2: undo log 压缩优化
- Phase 2.0: MVCC snapshot savepoint

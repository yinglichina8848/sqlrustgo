# 向量化执行设计

> Vectorized Execution Design

---

## 执行模型演进

### Volcano 模型（当前）

```rust
trait Executor {
    fn next(&mut self) -> Option<Tuple>;
}
```

特点：
- Tuple-at-a-time
- 虚函数频繁调用
- 分支预测差
- Cache miss 多

### 向量化模型（v2.0）

```rust
trait Operator {
    fn next_batch(&mut self) -> DbResult<Option<Batch>>;
}
```

特点：
- Batch-at-a-time
- Cache 友好
- SIMD 可扩展
- 减少函数调用

---

## 数据结构

### Batch

```rust
pub struct Batch {
    pub columns: Vec<Vec<Value>>,
    pub row_count: usize,
}

impl Batch {
    pub fn select(&self, indices: &[usize]) -> Batch {
        let mut columns = Vec::new();
        for col in &self.columns {
            let selected: Vec<Value> = indices.iter()
                .map(|&i| col[i].clone())
                .collect();
            columns.push(selected);
        }
        Batch {
            columns,
            row_count: indices.len(),
        }
    }
}
```

### ColumnVector

```rust
pub struct IntVector {
    pub data: Vec<i32>,
}

pub struct StringVector {
    pub data: Vec<String>,
}

pub enum ColumnVector {
    Int(IntVector),
    String(StringVector),
    // ...
}
```

---

## 执行算子

### Filter

```rust
impl Operator for FilterExec {
    fn next_batch(&mut self) -> DbResult<Option<Batch>> {
        let input_batch = self.input.next_batch()?;

        if let Some(batch) = input_batch {
            let mut selected = vec![];

            for i in 0..batch.row_count {
                if evaluate(&self.predicate, &batch, i) {
                    selected.push(i);
                }
            }

            Ok(Some(batch.select(&selected)))
        } else {
            Ok(None)
        }
    }
}
```

### Projection

```rust
impl Operator for ProjectionExec {
    fn next_batch(&mut self) -> DbResult<Option<Batch>> {
        let input_batch = self.input.next_batch()?;

        if let Some(batch) = input_batch {
            let columns: Vec<Vec<Value>> = self.columns.iter()
                .map(|&idx| batch.columns[idx].clone())
                .collect();

            Ok(Some(Batch {
                columns,
                row_count: batch.row_count,
            }))
        } else {
            Ok(None)
        }
    }
}
```

---

## SIMD 优化

### 标量版本

```rust
for i in 0..n {
    if col[i] > 10 {
        result.push(col[i]);
    }
}
```

### SIMD 版本

```rust
use std::simd::{Simd, SimdPartialOrd};

const LANES: usize = 8;

impl IntVector {
    pub fn filter_gt(&self, value: i32) -> Vec<i32> {
        let mut result = Vec::new();

        let chunks = self.data.chunks_exact(LANES);
        let remainder = chunks.remainder();

        for chunk in chunks {
            let simd = Simd::<i32, LANES>::from_slice(chunk);
            let mask = simd.simd_gt(Simd::splat(value));

            for i in 0..LANES {
                if mask.test(i) {
                    result.push(chunk[i]);
                }
            }
        }

        for &r in remainder {
            if r > value {
                result.push(r);
            }
        }

        result
    }
}
```

---

## 批大小选择

| 批大小 | 优点 | 缺点 |
|--------|------|------|
| 64 | 低延迟 | 函数调用多 |
| 256 | 平衡 | - |
| 1024 | Cache 友好 | 延迟高 |
| 4096 | SIMD 高效 | 内存占用大 |

**推荐**：1024 - 4096

---

## 优势

1. **更少函数调用**：每批一次
2. **Cache 友好**：连续内存访问
3. **SIMD 可扩展**：向量化计算
4. **并行友好**：批级别并行

---

## 迁移策略

### 阶段 1：兼容层

```rust
trait Executor {
    fn next(&mut self) -> Option<Tuple> {
        // 从 batch 中取一个
    }
}
```

### 阶段 2：混合执行

```rust
enum ExecMode {
    Row,
    Batch,
}
```

### 阶段 3：全面向量化

所有算子实现 `next_batch`

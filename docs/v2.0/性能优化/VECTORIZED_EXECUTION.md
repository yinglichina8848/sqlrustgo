# 向量化执行模型设计

> 版本：v1.0
> 日期：2026-02-18
> 目标：从行模式升级到向量化模式

---

## 一、当前执行器（行模式）

### 1.1 行模式代码

```rust
❌ for row in rows {
    evaluate(row)
}
```

**问题**：
- 每行单独处理
- 函数调用开销大
- 无法利用 CPU 缓存
- 无法使用 SIMD

### 1.2 性能瓶颈

| 问题 | 说明 |
|:-----|:-----|
| CPU 缓存 | 每行数据不连续 |
| 函数调用 | 每行一次调用 |
| 分支预测 | 条件判断频繁 |
| SIMD | 无法使用 |

---

## 二、向量化模式

### 2.1 批处理结构

```rust
pub struct RecordBatch {
    columns: Vec<ArrayRef>,
    row_count: usize,
}

pub type ArrayRef = Arc<dyn Array>;

pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
    fn slice(&self, offset: usize, length: usize) -> ArrayRef;
}
```

### 2.2 向量化执行

```rust
处理 1024 行一批：

RecordBatch {
    columns: Vec<Array>,
    row_count: 1024,
}
```

---

## 三、表达式执行改造

### 3.1 行模式

```rust
❌ fn evaluate(row: &Row) -> Value {
    match self {
        Expr::Column(col) => row.get(col),
        Expr::Literal(val) => val.clone(),
        Expr::BinaryExpr { left, op, right } => {
            let l = left.evaluate(row)?;
            let r = right.evaluate(row)?;
            op.apply(l, r)
        }
    }
}
```

### 3.2 向量化模式

```rust
✅ fn evaluate(batch: &RecordBatch) -> Result<ArrayRef> {
    match self {
        Expr::Column(col) => {
            Ok(batch.column_by_name(col)?.clone())
        }
        Expr::Literal(val) => {
            Ok(create_constant_array(val, batch.row_count()))
        }
        Expr::BinaryExpr { left, op, right } => {
            let l = left.evaluate(batch)?;
            let r = right.evaluate(batch)?;
            op.apply_batch(&l, &r)
        }
    }
}
```

### 3.3 向量化操作

```rust
pub trait BinaryOp {
    fn apply_batch(&self, left: &ArrayRef, right: &ArrayRef) -> Result<ArrayRef>;
}

pub struct AddOp;

impl BinaryOp for AddOp {
    fn apply_batch(&self, left: &ArrayRef, right: &ArrayRef) -> Result<ArrayRef> {
        match (left.data_type(), right.data_type()) {
            (DataType::Int64, DataType::Int64) => {
                let left = left.as_any().downcast_ref::<Int64Array>().unwrap();
                let right = right.as_any().downcast_ref::<Int64Array>().unwrap();
                
                let result: Int64Array = left.iter()
                    .zip(right.iter())
                    .map(|(l, r)| match (l, r) {
                        (Some(l), Some(r)) => Some(l + r),
                        _ => None,
                    })
                    .collect();
                
                Ok(Arc::new(result))
            }
            _ => Err(Error::TypeMismatch),
        }
    }
}
```

---

## 四、向量化优势

| 类型 | 行模式 | 向量模式 |
|:-----|:-------|:---------|
| **CPU** | 差（频繁函数调用） | 好（批量处理） |
| **Cache** | 差（数据不连续） | 好（列式存储） |
| **SIMD** | 不可用 | 可用 |
| **批处理** | 不可 | 可 |
| **内存** | 高（每行分配） | 低（批量分配） |

---

## 五、SIMD 优化

### 5.1 手动 SIMD

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn add_simd(left: &[i64], right: &[i64]) -> Vec<i64> {
    let len = left.len();
    let mut result = vec![0i64; len];
    
    let chunks = len / 4;
    
    for i in 0..chunks {
        unsafe {
            let l = _mm256_loadu_si256(left.as_ptr().add(i * 4) as *const __m256i);
            let r = _mm256_loadu_si256(right.as_ptr().add(i * 4) as *const __m256i);
            let sum = _mm256_add_epi64(l, r);
            _mm256_storeu_si256(result.as_mut_ptr().add(i * 4) as *mut __m256i, sum);
        }
    }
    
    for i in (chunks * 4)..len {
        result[i] = left[i] + right[i];
    }
    
    result
}
```

### 5.2 自动向量化

```rust
pub fn add_auto(left: &[i64], right: &[i64]) -> Vec<i64> {
    left.iter()
        .zip(right.iter())
        .map(|(l, r)| l + r)
        .collect()
}
```

编译器会自动向量化这段代码。

---

## 六、推荐批大小

| 批大小 | 说明 |
|:-------|:-----|
| 512 | 小批量，低延迟 |
| 1024 | 推荐，平衡 |
| 2048 | 大批量，高吞吐 |
| 4096 | 超大批量，可能影响延迟 |

**推荐**：512-2048 rows

---

## 七、向量化执行器实现

```rust
pub struct VectorizedExecutor {
    batch_size: usize,
}

impl VectorizedExecutor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
    
    pub fn execute(&self, plan: &PhysicalPlan) -> Result<Vec<RecordBatch>> {
        let mut results = Vec::new();
        let stream = plan.execute(0)?;
        
        while let Some(batch) = stream.next() {
            let batch = batch?;
            results.push(batch);
        }
        
        Ok(results)
    }
    
    pub fn execute_with_batch_size(
        &self,
        source: &dyn TableSource,
        projection: &[usize],
        filter: Option<&Expr>,
    ) -> Result<RecordBatchStream> {
        let stream = source.scan(projection, filter, self.batch_size)?;
        Ok(stream)
    }
}
```

---

## 八、性能对比

### 8.1 测试场景

```
表大小：100万行
操作：过滤 + 投影
```

### 8.2 结果预估

| 模式 | 时间 | 提升 |
|:-----|:-----|:-----|
| 行模式 | ~1000ms | 基准 |
| 向量化 | ~100ms | 10x |
| 向量化 + SIMD | ~50ms | 20x |

---

## 九、迁移路径

### 9.1 第一阶段

- [ ] 引入 RecordBatch
- [ ] 实现 Array trait
- [ ] 改造表达式执行

### 9.2 第二阶段

- [ ] 向量化算子
- [ ] 向量化 Join
- [ ] 向量化 Aggregate

### 9.3 第三阶段

- [ ] SIMD 优化
- [ ] 列式存储
- [ ] 内存池

---

*本文档由 TRAE (GLM-5.0) 创建*

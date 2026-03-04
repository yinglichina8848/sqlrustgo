# RC-002 覆盖率测试报告

## 测试结果

**覆盖率**: 85.68% (1442/1683 行)

**目标**: >= 90%

**状态**: ❌ 未达标

## 详细覆盖率

| 模块 | 覆盖行 | 总行数 | 覆盖率 |
|------|--------|--------|--------|
| auth/mod.rs | 86 | 87 | 98.85% |
| storage/buffer_pool.rs | 28 | 28 | 100% |
| storage/page.rs | 6 | 6 | 100% |
| types/error.rs | 6 | 6 | 100% |
| types/mod.rs | 12 | 12 | 100% |
| transaction/wal.rs | 37 | 39 | 94.87% |
| transaction/manager.rs | 54 | 56 | 96.43% |
| storage/bplus_tree/tree.rs | 112 | 121 | 92.56% |
| executor/mod.rs | 315 | 353 | 89.24% |
| lexer/lexer.rs | 144 | 174 | 82.76% |
| parser/mod.rs | 219 | 274 | 79.93% |
| types/value.rs | 22 | 33 | 66.67% |
| main.rs | 0 | 65 | 0% (忽略) |

## 需要改进的模块

1. **types/value.rs** - 需增加 11 行测试
2. **parser/mod.rs** - 需增加 55 行测试
3. **lexer/lexer.rs** - 需增加 30 行测试

## 验收
- [x] 覆盖率测试完成
- [ ] 覆盖率 >= 90% (当前 85.68%)

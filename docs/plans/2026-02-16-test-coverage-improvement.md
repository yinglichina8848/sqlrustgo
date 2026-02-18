# 测试覆盖率提升设计文档

**日期**: 2026-02-16
**目标**: 提升测试覆盖率至 90% 行覆盖率, 95% 函数覆盖率
**排除**: main.rs 入口文件

## 1. 背景

当前测试覆盖率（排除 main.rs）:
- 行覆盖率: 76.64%
- 函数覆盖率: 80.49%

差距分析:
- network/mod.rs: 54.61% 行, 61.22% 函数
- executor/mod.rs: 77.31% 行, 79.17% 函数
- storage/bplus_tree/tree.rs: 79.43% 行, 82.76% 函数

## 2. 方法

采用混合方式:
- **P0**: network 模块单独处理（复杂度高，需要隔离测试）
- **P1**: executor + bplus_tree 并行处理

## 3. 架构设计

### 3.1 Network 模块测试 (P0)

目标: 54.61% → 85%+ 行覆盖

关键测试项:
| 函数 | 测试场景 |
|------|----------|
| HandshakeV10::to_bytes() | PLUGIN_AUTH capability, auth_plugin_data > 8 bytes |
| OkPacket::to_bytes() | Empty message path |
| RowData::to_bytes() | Value::Boolean, Value::Blob 序列化 |
| NetworkHandler::handle() | 完整集成测试 |
| NetworkHandler::read_packet() | 不完整 header, IO error |
| NetworkHandler::execute_query() | SELECT VERSION(), SELECT 1 |

### 3.2 Executor 模块测试 (P1)

目标: 77.31% → 85%+ 行覆盖

关键测试项:
| 函数 | 测试场景 |
|------|----------|
| create_index() | 索引创建 |
| has_index() | 索引存在检查 |
| execute_select_with_index() | 索引搜索路径 |
| execute_update | 无 WHERE 子句 |
| execute_delete | 无 WHERE 子句 |
| evaluate_where | 所有比较运算符 (!=, >, <, >=, <=) |

### 3.3 B+ Tree 模块测试 (P1)

目标: 79.43% → 85%+ 行覆盖

关键测试项:
| 函数 | 测试场景 |
|------|----------|
| split_leaf_root() | 触发叶子节点分裂 (>4 条目) |
| insert_into_internal() | 内部节点插入 |
| search_node() | 内部节点搜索 |
| range_query_node() | 内部节点范围查询 |
| collect_keys() | 内部节点键收集 |

## 4. 验收标准

- [ ] network/mod.rs 达到 85%+ 行覆盖
- [ ] executor/mod.rs 达到 85%+ 行覆盖
- [ ] bplus_tree/tree.rs 达到 85%+ 行覆盖
- [ ] 总计 (排除 main.rs) 达到 90%+ 行, 95%+ 函数
- [ ] 所有测试通过 (`cargo test`)

## 5. 时间规划

预计分 3 个阶段:
1. Phase 1: Network 模块 (单独处理)
2. Phase 2: Executor 模块 (并行)
3. Phase 3: B+ Tree 模块 (并行)

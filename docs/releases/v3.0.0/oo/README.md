# v3.0.0 OO Design Documents

> **面向对象分析设计文档** - 从语句执行链路的角度进行深入分析
>
> 解决当前文档的问题：横向模块文档多，纵向执行链路文档少；缺少算法时序图、状态图、活动图

## 完整文档结构

```
oo/
├── README.md                           # 本文档
├── cbo/                               # CBO 查询优化器
│   ├── CBO_DESIGN.md                 # CBO 架构与策略选取
│   ├── CBO_COST_MODEL.md             # 代价模型详解
│   └── CBO_JOIN_ORDERING.md          # Join Order 算法
├── bptree/                            # B+Tree 索引
│   └── BPTREE_DESIGN.md             # B+Tree 完整链路分析
├── execution/                         # 查询执行
│   ├── EXECUTION_PIPELINE.md       # 完整执行流水线
│   └── SELECT_EXECUTION.md          # SELECT 执行链路
├── dml/                               # DML 语句
│   ├── INSERT_EXECUTION.md          # INSERT 执行链路
│   ├── UPDATE_EXECUTION.md          # UPDATE 执行链路
│   └── DELETE_EXECUTION.md          # DELETE 执行链路
├── ddl/                               # DDL 语句
│   └── DDL_EXECUTION.md            # CREATE/DROP/ALTER 执行链路
├── dcl/                               # DCL 语句
│   └── DCL_EXECUTION.md            # GRANT/REVOKE 执行链路
├── join/                               # JOIN 算法
│   └── JOIN_ALGORITHMS.md           # Hash/NestLoop/MergeJoin
├── transaction/                       # 事务处理
│   ├── TX_MANAGEMENT.md            # 事务管理
│   ├── MVCC_IMPLEMENTATION.md      # MVCC 实现
│   └── WAL_PROTOCOL.md              # WAL 协议
├── recovery/                         # 崩溃恢复
│   └── CRASH_RECOVERY.md           # 崩溃恢复链路分析
├── distributed/                      # 分布式
│   ├── REPLICATION.md              # 主从复制
│   ├── SEMISYNC.md                 # 半同步复制
│   ├── XA_TRANSACTION.md           # XA 事务
│   └── MTS.md                      # 多线程复制
└── coverage/                        # 覆盖率提升
    ├── COVERAGE_GAPS.md            # 覆盖率差距分析
    └── COVERAGE_IMPROVEMENT_PLAN.md # 覆盖率提升计划
```

## 功能专题链路总览

| 功能 | 关键字 | 核心文件 | 覆盖率 |
|------|--------|----------|--------|
| SELECT | scan/filter/aggregate | executor/src/*.rs | ~70% |
| INSERT | insert/write/wal | executor/src/*.rs | ~65% |
| UPDATE | update/delete+insert | executor/src/*.rs | ~60% |
| DELETE | delete/merge | executor/src/*.rs | ~60% |
| DDL | create_table/drop_table | executor/src/ddl_executor.rs | ~50% |
| JOIN | hash_join/nested_loop | executor/src/*.rs | ~65% |
| WAL | wal/write/commit | transaction/src/wal.rs | ~75% |
| MVCC | snapshot/version_chain | transaction/src/mvcc.rs | ~70% |
| Recovery | recovery/replay | transaction/src/recovery.rs | ~40% |
| Replication | replication/semisync | storage/src/replication.rs | ~50% |
| XA | xa/prepare/commit | distributed/src/xa_coordinator.rs | ~45% |
| MTS | mts/worker | distributed/src/mts.rs | ~50% |
| DCL | grant/revoke | catalog/src/auth.rs | ~30% |

## 已完成文档

| 文档 | 状态 | 说明 |
|------|--------|------|
| `dml/DML_EXECUTION.md` | ✅ | INSERT/UPDATE/DELETE/SELECT 执行链路 |
| `ddl/DDL_EXECUTION.md` | ✅ | CREATE/DROP/ALTER TABLE 执行链路 |
| `dcl/DCL_EXECUTION.md` | ✅ | GRANT/REVOKE/用户管理/角色 |
| `join/JOIN_ALGORITHMS.md` | ✅ | Hash/NestLoop/MergeJoin 算法 |
| `wal/WAL_PROTOCOL.md` | ✅ | WAL 协议/崩溃恢复/检查点 |
| `recovery/CRASH_RECOVERY.md` | ✅ | 崩溃恢复链路/2PC/XA |
| `distributed/DISTRIBUTED_SYNC.md` | ✅ | 复制/半同步/XA/MTS |

## 文档更新记录

| 日期 | 更新内容 | 状态 |
|------|----------|------|
| 2026-05-11 | 初始化 OO 设计文档结构 | ✅ |
| 2026-05-11 | 补充 DML/DDL 执行链路 | ✅ |
| 2026-05-11 | 补充 DCL/JOIN/WAL/Recovery/Distributed | ✅ |

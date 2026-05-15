## Context

v3.2.0 Alpha 阶段即将完成，需要进入 RC/GA 阶段。Alpha 阶段完成了 GMP 基础框架（数字签名审计链、电子签名、Immutable Record、Correction Chain、Provenance Tracking、Trusted Timestamp、HSM/KMS）。

RC Gate 要求：
- R1-R16: 16 项核心检查（Build、Test、Clippy、Format、Coverage、Security、SQL Compat、MERGE、Event Scheduler、GMP Workflow、GMP Mobile、GMP SOP/Training、GMP Device、TPC-H SF=10、Sysbench、Stability 72h、OO Documentation）
- R-S1~S16: 16 项稳定性测试

GA Gate 要求：
- G1-G12: 12 项核心检查
- G-QA1~QA10: 10 项 QA 增强测试
- G-S1~S20: 20 项稳定性测试

## Goals / Non-Goals

**Goals:**
- 确保 RC/GA 门禁顺利通过
- 明确剩余工作的优先级和顺序
- 建立可执行的开发和测试计划

**Non-Goals:**
- 不重新实现 Alpha 已完成的功能
- 不改变已确定的技术架构

## Decisions

1. **RC 优先 SQL Compat 增强**
   - MERGE 语句实现和 Event Scheduler 是新的 SQL 功能，需要重点开发

2. **GA 优先 QA 增强测试**
   - 10 项 QA 增强测试是 GA 特有，需要设计和实现具体的验证脚本

3. **稳定性测试采用渐进式方法**
   - 先通过 RC 门禁的 72h 稳定性测试
   - 再扩展到 GA 门禁的 20 项稳定性测试

## Risks / Trade-offs

- [Risk] TPC-H SF=10 可能 OOM → [Mitigation] 内存优化先于 TPC-H 测试
- [Risk] QPS 优化目标 30K 可能不达标 → [Mitigation] PERF-1 优先处理
- [Risk] 形式化验证 30 proofs 工作量大 → [Mitigation] 与形式化验证团队并行工作

## Open Questions

- PERF-01 (MySQL 协议 flush 优化) 的具体实现方案待定
- Formal proofs 的具体覆盖范围和工具选型待定

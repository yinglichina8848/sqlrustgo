# v2.9.0 分阶段测试任务清单

## Develop: 回归+集成门禁
- D1: scripts/test/run_integration.sh (28文件)
- D2: regression_baseline.json (3630 tests)
- D3: SQL缺失命令补齐 (REPLACE, ALTER TABLE, TRUNCATE等)
- D4: Sysbench协议修复 (COM_STMT_PREPARE)
- D5: TPC-H协议修复 (LEFT JOIN, EXISTS等)
- D6: CI test-regression job

## Alpha: SQL Corpus + Sysbench
- A1: SQL Corpus 103文件 ≥85%
- A2: Sysbench OLTP 4套件基线
- A3: coverage ≥50%

## Beta: TPC-H + 安全
- B1: TPC-H 22查询 ≥18/22
- B2: Sysbench ≥5K QPS
- B3: Security 81 tests 100%

## RC: 全量验证
- R1: TPC-H SF0.1 全量
- R2: Sysbench ≥10K QPS
- R3: P011-P012 verified

## GA: 最终审计
- G1: 错误修正回溯
- G2: 混沌工程
- G3: coverage ≥85%

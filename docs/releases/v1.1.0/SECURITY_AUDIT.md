# 依赖安全审计报告 (F-01)

**审计日期**: 2026-03-03
**审计工具**: cargo-audit (本地缓存), GitHub Security Advisories API
**审计结果**: ✅ 通过

## 依赖版本总览

| 依赖 | 当前版本 | 状态 |
|------|----------|------|
| tokio | 1.49.0 | ✅ 安全 |
| bytes | 1.11.1 | ✅ 安全 |
| regex | 1.12.3 | ✅ 安全 |
| serde_json | 1.0.149 | ✅ 安全 |
| thiserror | 1.0.69 | ✅ 安全 |
| anyhow | 1.0.101 | ✅ 安全 |
| env_logger | 0.10.2 | ✅ 安全 |
| ctrlc | 3.5.2 | ✅ 安全 |
| lru-cache | 0.1.2 | ✅ 安全 |
| hex | 0.4.3 | ✅ 安全 |
| log | 0.4.29 | ✅ 安全 |
| serde | 1.0.228 | ✅ 安全 |

## 历史漏洞（已修复）

| 漏洞 ID | 包 | 严重程度 | 描述 | 修复版本 |
|---------|-----|----------|------|----------|
| RUSTSEC-2022-0013 | regex | High | ReDoS 正则表达式 DoS 攻击 | >= 1.5.5 |
| RUSTSEC-2025-0023 | tokio | Unsound | Broadcast channel 内存安全问题 | >= 1.44.2 |
| RUSTSEC-2023-0001 | tokio | Medium | Windows named pipe 配置丢失 | >= 1.23.1 |
| RUSTSEC-2023-0005 | tokio | Unsound | ReadHalf::unsplit 内存安全问题 | >= 1.24.2 |
| RUSTSEC-2021-0072 | tokio | Medium | LocalSet 任务中止竞态条件 | >= 1.8.1 |
| RUSTSEC-2021-0124 | tokio | Medium | oneshot channel 数据竞态 | >= 1.13.1 |
| RUSTSEC-2026-0007 | bytes | High | BytesMut::reserve 整数溢出 | >= 1.11.1 |

## 结论

✅ **所有依赖无已知安全漏洞**

当前项目使用的依赖版本均已包含所有安全修复，可以安全用于生产环境。

## 建议

1. 建议定期更新依赖以获取最新安全修复
2. 建议将 `criterion` 从 0.5.1 升级到 0.8.2（dev-dependency）
3. 建议将 `tempfile` 从 3.25.0 升级到 3.26.0（dev-dependency）

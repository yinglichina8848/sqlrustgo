## Why

企业级GMP部署需要硬件级密钥保护。HSM/KMS集成将签名操作卸载到硬件安全模块，提供企业级密钥安全和合规性保障。

## What Changes

- 新增 HSM/KMS 集成模块
- 实现 TPM 2.0 支持（可选，需要硬件）
- 实现 PKCS#11 HSM 支持
- 实现软件TPM模拟器作为开发和测试备选
- 实现密钥轮换机制
- 与现有签名模块集成，提供硬件加速签名

## Capabilities

### New Capabilities

- `tpm-provider`: TPM 2.0 密钥管理和签名提供者
- `pkcs11-provider`: PKCS#11 HSM 接口实现
- `kms-provider`: 云KMS接口（AWS/Azure/GCP）
- `software-tpm`: 软件TPM模拟器，用于开发测试环境
- `key-rotation`: 密钥轮换管理机制
- `hsm-signing`: 硬件签名卸载接口

## Impact

- 新增 `crates/gmp/src/hsm/` 子目录
- 新增 `crates/gmp/src/hsm/tpm.rs`
- 新增 `crates/gmp/src/hsm/pkcs11.rs`
- 新增 `crates/gmp/src/hsm/kms.rs`
- 新增 `crates/gmp/src/hsm/software_tpm.rs`
- 新增 `crates/gmp/src/hsm/mod.rs`
- 与 `signature` 模块集成

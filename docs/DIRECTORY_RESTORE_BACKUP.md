# 目录整理回滚备份记录

> **创建时间**: 2026-05-11
> **最后更新**: 2026-05-11
> **用途**: 记录目录整理前的状态，用于回滚

---

## 执行状态

| 阶段 | 描述 | 状态 |
|------|------|------|
| 1 | 模块设计文档迁移到 docs/architecture/modules/ | ✅ 已完成 |
| 2 | 工具脚本迁移到 scripts/ | ✅ 已完成 |
| 3 | Contract 文件迁移到 docs/formal/contracts/ | ✅ 已完成 |
| 4 | releases/ 迁移到 release-artifacts/ | ✅ 已完成 |
| 5 | openspec/ 迁移到 tools/openspec/ | ✅ 已完成 |

---

## 已迁移的内容

### 1. 模块设计文档目录（7个）

| 原路径 | 目标路径 |
|--------|----------|
| `mvcc/MVCC_DESIGN.md` | `docs/architecture/modules/MVCC_DESIGN.md` |
| `executor/EXECUTOR_DESIGN.md` | `docs/architecture/modules/EXECUTOR_DESIGN.md` |
| `parser/PARSER_DESIGN.md` | `docs/architecture/modules/PARSER_DESIGN.md` |
| `optimizer/OPTIMIZER_DESIGN.md` | `docs/architecture/modules/OPTIMIZER_DESIGN.md` |
| `storage/STORAGE_DESIGN.md` | `docs/architecture/modules/STORAGE_DESIGN.md` |
| `transaction/TRANSACTION_DESIGN.md` | `docs/architecture/modules/TRANSACTION_DESIGN.md` |
| `wal/WAL_DESIGN.md` | `docs/architecture/modules/WAL_DESIGN.md` |

### 2. 工具脚本目录

| 原路径 | 目标路径 |
|--------|----------|
| `verify/verification_engine.py` | `scripts/verify/verification_engine.py` |
| `audit/self_audit.py` | `scripts/audit/self_audit.py` |
| `extensions/openclaw/` | `scripts/extensions/openclaw/` |

### 3. Contract 目录

| 原路径 | 目标路径 |
|--------|----------|
| `contract/v2.8.0.json` | `docs/formal/contracts/v2.8.0.json` |
| `contract/v2.9.0.json` | `docs/formal/contracts/v2.9.0.json` |

### 4. releases/ 目录迁移

| 原路径 | 目标路径 |
|--------|----------|
| `releases/v1.2.0/` | `release-artifacts/v1.2.0/` |
| `releases/v1.3.0/` | `release-artifacts/v1.3.0/` |
| `releases/v1.4.0/` | `release-artifacts/v1.4.0/` |
| `releases/v1.5.0/` | `release-artifacts/v1.5.0/` |
| `releases/v1.2.0-darwin-arm64.tar.gz` | `release-artifacts/v1.2.0-darwin-arm64.tar.gz` |

### 5. openspec/ 目录迁移

| 原路径 | 目标路径 |
|--------|----------|
| `openspec/config.yaml` | `tools/openspec/config.yaml` |
| `openspec/specs/` | `tools/openspec/specs/` |

---

## 回滚命令

如果需要回滚，执行以下命令：

```bash
# 1. 模块设计文档回滚
mkdir -p mvcc executor parser optimizer storage transaction wal
mv docs/architecture/modules/MVCC_DESIGN.md mvcc/
mv docs/architecture/modules/EXECUTOR_DESIGN.md executor/
mv docs/architecture/modules/PARSER_DESIGN.md parser/
mv docs/architecture/modules/OPTIMIZER_DESIGN.md optimizer/
mv docs/architecture/modules/STORAGE_DESIGN.md storage/
mv docs/architecture/modules/TRANSACTION_DESIGN.md transaction/
mv docs/architecture/modules/WAL_DESIGN.md wal/

# 2. 工具脚本回滚
mkdir -p verify audit extensions
mv scripts/verify/verification_engine.py verify/
mv scripts/audit/self_audit.py audit/
mv scripts/extensions/openclaw extensions/

# 3. contract 回滚
mkdir -p contract
mv docs/formal/contracts/v2.8.0.json contract/
mv docs/formal/contracts/v2.9.0.json contract/
rmdir docs/formal/contracts 2>/dev/null || true

# 4. releases/ 回滚
mkdir -p releases
mv release-artifacts/v1.2.0 releases/
mv release-artifacts/v1.3.0 releases/
mv release-artifacts/v1.4.0 releases/
mv release-artifacts/v1.5.0 releases/
mv release-artifacts/v1.2.0-darwin-arm64.tar.gz releases/

# 5. openspec/ 回滚
mkdir -p openspec
mv tools/openspec/config.yaml openspec/
mv tools/openspec/specs openspec/
rmdir tools/openspec 2>/dev/null || true
```

---

## 当前目录快照（整理后）

```
docs/architecture/modules/:
  - EXECUTOR_DESIGN.md
  - MVCC_DESIGN.md
  - OPTIMIZER_DESIGN.md
  - PARSER_DESIGN.md
  - STORAGE_DESIGN.md
  - TRANSACTION_DESIGN.md
  - WAL_DESIGN.md

scripts/verify/:
  - verification_engine.py

scripts/audit/:
  - self_audit.py

scripts/extensions/:
  - openclaw/

docs/formal/contracts/:
  - v2.8.0.json
  - v2.9.0.json

release-artifacts/:
  - v1.2.0/
  - v1.3.0/
  - v1.4.0/
  - v1.5.0/
  - v1.2.0-darwin-arm64.tar.gz

tools/openspec/:
  - config.yaml
  - specs/
```

---

## 注意事项

1. **v2.5.0 不在迁移范围内** - 该目录是符号链接，包含完整的发布文档
2. **二进制文件保留在 releases/** - sqlrustgo 和 sqlrustgo-bench 是构建产物
3. **contract/changes/ 和 contract/specs/** - 原始目录不存在，未迁移

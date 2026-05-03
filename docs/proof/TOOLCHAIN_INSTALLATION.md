# SQLRustGo Formal Verification Toolchain

> **Version**: 1.9
> **Updated**: 2026-05-03
> **Registry**: `docs/proof/REGISTRY_INDEX.json`

---

## 架构概览

```
docs/formal/          <- 可执行的 TLA+ / Dafny 规格（实际运行 TLC/Dafny 验证器）
docs/proof/           <- Markdown 格式 proof 文档 + Formulog .formulog 文件
  *.formulog          <- Formulog 证明（java -jar formulog.jar 运行）
  PROOF-*.dfy         <- Markdown（不是可执行的 .dfy）
  PROOF-*.tla         <- Markdown（不是可执行的 .tla）
```

---

## 工具链组件

| 工具 | 版本 | 用途 | 可执行文件位置 |
|------|------|------|--------------|
| **TLA+ TLC** | 2026.04.29 | 并发/事务模型检测 | `/tmp/tla2tools.jar` |
| **Dafny** | 2.3.0.10506 | 数据结构/类型安全证明 | `/usr/lib/dafny/Dafny.exe` |
| **Formulog** | 0.8.0 | SQL 语义逻辑证明 | `/tmp/formulog-0.8.0.jar` |
| **Z3** | 4.8.12 | SMT 求解器（Formulog 后端） | `/usr/bin/z3` |
| **Docker** | - | 隔离容器（Formulog JVM 污染问题） | `tla-java17:latest` |

---

## 安装步骤

### 1. TLA+ TLC

```bash
# 方法 1: 从 Docker 镜像提取（推荐）
docker run --rm tla-java17 cat /opt/tla2tools.jar > /tmp/tla2tools.jar

# 方法 2: 直接下载
wget https://github.com/tlaplus/tlaplus/releases/download/v1.7.0/tla2tools.jar -O /tmp/tla2tools.jar

# 验证
java -cp /tmp/tla2tools.jar tlc2.TLC -version
# 输出: TLC2 Version 2026.04.29.170100

# 别名（可选）
echo 'alias tlc2="java -cp /tmp/tla2tools.jar tlc2.TLC"' >> ~/.bashrc
source ~/.bashrc
```

### 2. Dafny

```bash
# 安装包（已有 /usr/bin/dafny wrapper）
# 验证
/usr/bin/cli /usr/lib/dafny/Dafny.exe /help | head -5
# 输出: Dafny 2.3.0.10506

# 注意：旧版 Dafny 2.3.0 CLI 语法与 v4 不同
# 正确语法: /usr/bin/cli /usr/lib/dafny/Dafny.exe /dafnyVerify:1 /compile:0 <file.dfy>
# 错误语法: dafny verify <file.dfy>  (v4 语法，不兼容)
```

### 3. Formulog

```bash
# 下载
curl -L -o /tmp/formulog-0.8.0.jar \
  https://github.com/HarvardPL/formulog/releases/download/v0.8.0/formulog-0.8.0.jar

# 验证
java -jar /tmp/formulog-0.8.0.jar --version 2>&1 | head -3

# Docker 隔离运行器（解决 JVM 状态污染）
# 使用 scripts/formalog/run_formalog_isolated.sh
chmod +x scripts/formalog/run_formulog_isolated.sh
./scripts/formalog/run_formalog_isolated.sh docs/proof/PROOF-017-update-semantics.formulog
```

### 4. Z3

```bash
# Ubuntu/Debian
apt-get install z3

# 或源码编译
wget https://github.com/Z3Prover/z3/releases/download/z3-4.8.12/z3-4.8.12-x64-glibc-2.31.zip
unzip z3-4.8.12-x64-glibc-2.31.zip
ln -s $(pwd)/z3-4.8.12-x64-glibc-2.31/bin/z3 /usr/local/bin/z3

# 验证
z3 --version
# 输出: Z3 version 4.8.12 - 64 bit
```

---

## 验证命令（单机运行）

### TLA+ TLC

```bash
# 单个规格
java -cp /tmp/tla2tools.jar tlc2.TLC \
  -metadir /tmp/tlc_meta_<spec_name> \
  -workers 16 \
  docs/formal/<spec>.tla

# 示例：MVCC SSI
java -cp /tmp/tla2tools.jar tlc2.TLC \
  -metadir /tmp/tlc_meta_mvcc_ssi \
  -workers 16 \
  docs/formal/PROOF_016_mvcc_ssi.tla

# 示例：DDL 原子性
java -cp /tmp/tla2tools.jar tlc2.TLC \
  -metadir /tmp/tlc_meta_ddl \
  -workers 16 \
  docs/formal/PROOF_015_ddl_atomicity.tla

# 示例：WAL 恢复
java -cp /tmp/tla2tools.jar tlc2.TLC \
  -metadir /tmp/tlc_meta_wal \
  -workers 16 \
  docs/formal/WAL_Recovery.tla

# 示例：LEFT/RIGHT JOIN
java -cp /tmp/tla2tools.jar tlc2.TLC \
  -metadir /tmp/tlc_meta_join \
  -workers 16 \
  docs/formal/PROOF_019_left_right_join.tla
```

**预期输出**: `Model checking completed. No error has been found.`

### Dafny

```bash
# 单个规格（使用旧版 CLI 语法）
/usr/bin/cli /usr/lib/dafny/Dafny.exe \
  /dafnyVerify:1 /compile:0 \
  docs/formal/btree_invariants.dfy

# 预期输出: "Dafny program verifier finished with 1 verified, 0 errors"
```

### Formulog

```bash
# 单个证明（带 Docker 隔离，推荐）
./scripts/formalog/run_formulog_isolated.sh docs/proof/PROOF-017-update-semantics.formulog

# 或直接运行（注意 JVM 状态污染）
java -jar /tmp/formulog-0.8.0.jar docs/proof/PROOF-020-null-three-valued-logic.formulog

# 批量运行
for f in docs/proof/PROOF-01{7,20,21,22}-*.formulog; do
  echo "=== $f ==="
  java -jar /tmp/formulog-0.8.0.jar "$f" 2>&1 | grep -E "Finished|type|Error"
done
```

**预期输出**: `Finished type checking` + `Finished evaluating`

---

## CI/CD 集成

### Gitea Actions (`.gitea/workflows/ci.yml`)

```yaml
# 关键片段：formal-verification job
jobs:
  formal-verification:
    runs-on: [hp-z6g4]
    needs: [lint-build]
    steps:
      - name: Checkout
        run: |
          REPO="openclaw/sqlrustgo"
          git clone --depth=1 "http://192.168.0.252:3000/$REPO" repo
          cd repo
          git fetch --depth=1 origin "$GITEA_SHA"
          git checkout "$GITEA_SHA"

      - name: Verify Dafny Proofs
        run: |
          cd repo
          for spec in docs/formal/*.dfy; do
            [ -f "$spec" ] || continue
            /usr/bin/cli /usr/lib/dafny/Dafny.exe /dafnyVerify:1 /compile:0 "$spec" 2>&1 || exit 1
          done

      - name: Verify TLA+ Specs
        run: |
          cd repo
          for spec in docs/formal/*.tla; do
            base=$(basename "$spec" .tla)
            echo "$base" | grep -q "TTrace" && continue
            mkdir -p /tmp/tlc_meta_"$base"
            timeout 300 java -cp /tmp/tla2tools.jar tlc2.TLC \
              -metadir /tmp/tlc_meta_"$base" \
              -workers 16 \
              "$spec" 2>&1 || exit 1
          done

      - name: Verify Formulog Proofs
        run: |
          cd repo
          for spec in docs/proof/*.formulog; do
            [ -f "$spec" ] || continue
            java -jar /tmp/formulog-0.8.0.jar "$spec" 2>&1 || exit 1
          done
```

### Branch Protection

在 Gitea Settings → Branch Protection 启用：
- ✅ Require status checks: `formal-verification`
- ✅ Block on failure: **ENFORCED**

---

## Proof 位置速查表

| Proof ID | 可执行文件 | 类型 | 验证命令 |
|---------|-----------|------|---------|
| PROOF-015 DDL | `docs/formal/PROOF_015_ddl_atomicity.tla` | TLA+ | `tlc2.TLC ...` |
| PROOF-016 MVCC | `docs/formal/PROOF_016_mvcc_ssi.tla` | TLA+ | `tlc2.TLC ...` |
| PROOF-017 UPDATE | `docs/proof/PROOF-017-update-semantics.formulog` | Formulog | `java -jar formulog.jar ...` |
| PROOF-019 JOIN | `docs/formal/PROOF_019_left_right_join.tla` | TLA+ | `tlc2.TLC ...` |
| PROOF-020 NULL 3VL | `docs/proof/PROOF-020-null-three-valued-logic.formulog` | Formulog | `java -jar formulog.jar ...` |
| PROOF-021 HAVING | `docs/proof/PROOF-021-having-semantics.formulog` | Formulog | `java -jar formulog.jar ...` |
| PROOF-022 CTE | `docs/proof/PROOF-022-cte-nonrecursive.formulog` | Formulog | `java -jar formulog.jar ...` |
| WAL 恢复 | `docs/formal/WAL_Recovery.tla` | TLA+ | `tlc2.TLC ...` |
| BTree 不变量 | `docs/formal/btree_invariants.dfy` | Dafny | `Dafny.exe /dafnyVerify:1 ...` |

---

## 常见问题

### Q: `Error: Missing input TLA+ module`
A: 没有指定 `.tla` 文件，或文件是 markdown 格式。检查 `docs/formal/*.tla` 而非 `docs/proof/`

### Q: `Error: 'verify': Filename extension '' is not supported`
A: 使用旧版 Dafny 2.3.0 CLI：`/usr/bin/cli /usr/lib/dafny/Dafny.exe /dafnyVerify:1 /compile:0 <file>`，不是 `dafny verify`

### Q: Formulog JVM 状态污染（第二次运行失败）
A: 使用 Docker 隔离运行器：`scripts/formalog/run_formulog_isolated.sh`

### Q: TLA+ TLC 模型检测超时
A: MVCC SSI 规格状态空间巨大，加 `-workers 16` 和 `timeout 300`；TTrace 文件不是规格文件，跳过

### Q: `Prover error: unknown parameter 'model_compress'`
A: Dafny 2.3.0 旧版 Z3 参数不兼容，忽略该警告，验证仍通过（`1 verified, 0 errors`）

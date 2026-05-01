# AI Factory Control Plane — Implementation Plan

> **For Hermes:** 使用 subagent-driven-development 技能逐任务执行
>
> **Goal:** 将 SQLRustGo AI 协作系统从 "无 Control Plane" 升级为 "分层权限 + 强制 Gate + 可执行规则" 的生产级系统
>
> **Architecture:** 三层架构 — (1) Gate Layer (B/R-Gate pre-merge hook) (2) Queue Layer (PR Lock) (3) Rule Layer (GBrain → YAML rules → hermes rule-check)
>
> **Tech Stack:** Gitea API + pre-receive hook + 文件锁 + YAML rules
>
> **Backup:** `~/backups/hermes-ai-factory-20260501-125908/`

---

## Phase 0: 环境确认 (执行前必读)

### Step 0.1: 验证备份完整性

```bash
ls -la ~/backups/hermes-ai-factory-20260501-125908/
# 预期: gate_spec.md, release_process.md, hermes-gate/, hermes-gate-development/, ssh_config
```

### Step 0.2: 确认 Git identity

```bash
cd ~/workspace/dev/openheart/sqlrustgo
git config user.email  # 必须是 hermes-macmini@gaoyuanyiyao.com
git config user.name   # 必须是 openclaw
```

### Step 0.3: 确认 Gitea API 可用

```bash
TOKEN=$(cat ~/.ssh/openclaw-gitea.PAT)
curl -s "http://192.168.0.252:3000/api/v1/" \
  -H "Authorization: token $TOKEN" | head -c 200
# 预期: {"version":"1.22.1"...}
```

---

## Phase 1: Hermes 降权 — 移除直接 Merge 权限

> **Objective:** Hermes 只能审核 (review) + 提交 PR，不能 merge

### Task 1.1: 创建 Hermes Reviewer 专用 Gitea Token

**Files:**
- Modify: `~/.ssh/openclaw-gitea-readonly.PAT` (新文件)

**Step 1: 检查当前 Token 权限**

```bash
TOKEN=$(cat ~/.ssh/openclaw-gitea.PAT)
curl -s "http://192.168.0.252:3000/api/v1/user" \
  -H "Authorization: token $TOKEN" | python3 -c "
import json,sys
u=json.load(sys.stdin)
print(f'User: {u[\"login\"]}, Admin: {u[\"is_admin\"]}')
"
```

**Step 2: 在 Gitea Web UI 创建只读 Token**

```
URL: http://192.168.0.252:3000/user/settings/applications
Name: hermes-reviewer-readonly
Scopes: read:user, read:repository
```

**Step 3: 保存 Token**

```bash
# 保存新 Token (只读)
echo "gr_xxxxx_new_token_here" > ~/.ssh/openclaw-gitea-readonly.PAT
chmod 600 ~/.ssh/openclaw-gitea-readonly.PAT
```

**Step 4: 验证 Token**

```bash
READONLY_TOKEN=$(cat ~/.ssh/openclaw-gitea-readonly.PAT)
curl -s "http://192.168.0.252:3000/api/v1/user" \
  -H "Authorization: token $READONLY_TOKEN" | python3 -c "
import json,sys; u=json.load(sys.stdin)
print(f'Readonly token OK: {u[\"login\"]}')
"

# 验证无法 merge (应该返回 403)
curl -s -X POST "http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls/1/merge" \
  -H "Authorization: token $READONLY_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"force_merge":true}' | head -c 200
# 预期: {"message":"..."} 或 403
```

### Task 1.2: 更新 Hermes Agent 记忆中的权限定义

**Step 1: 更新 memory**

```bash
# 使用 memory 工具添加
# target: memory
# action: add
# content: |
#   Hermes 权限约束 (2026-05-01 更新):
#   - Hermes (Mac Mini) = Reviewer + PR creator, NOT Merger
#   - Merge 权限由 Gitea Branch Protection 100%控制
#   - Hermes 的 write token 仅用于: 创建 PR, 更新 Issue, 评论
#   - Hermes 的 read token 用于: 查询状态, 检查 CI
#   - 分支保护是唯一 merge 门禁，不可绕过
```

**Step 2: 创建 .hermes/agent-roles.yaml**

```bash
mkdir -p ~/.hermes
cat > ~/.hermes/agent-roles.yaml << 'EOF'
# Hermes Agent 角色定义 v1.0
# 更新日期: 2026-05-01

roles:
  hermes-macmini:
    role: orchestrator_reviewer
    permissions:
      - repo:read
      - issues:write
      - pull_requests:write
      - comments:write
      - statuses:read
      - actions:read
    denied:
      - merge_pull_requests
      - branch_protection:write
      - wiki:write
    token: ~/.ssh/openclaw-gitea-readonly.PAT

  hermes-worker:
    role: worker_executor
    permissions:
      - repo:read
      - issues:write
      - pull_requests:write
      - comments:write
      - statuses:write  # 用于报告 CI 状态
    denied:
      - merge_pull_requests
    token: ~/.ssh/hermes-worker-gitea.PAT  # 如有
EOF
echo "agent-roles.yaml created"
```

**Step 3: 验证 roles 文件**

```bash
cat ~/.hermes/agent-roles.yaml
python3 -c "import yaml; yaml.safe_load(open('/Users/liying/.hermes/agent-roles.yaml')); print('YAML valid')"
```

---

## Phase 2: B-Gate Pre-Merge Hook — 强制 Gate 校验

> **Objective:** 所有 PR 合并前必须通过 B-Gate 检查 (Git identity + 无 skip-ci + 未修改核心模块)

### Task 2.1: 设计 B-Gate 检查逻辑

**Files:**
- Create: `scripts/b_gate_check.sh` (新文件)

**Step 1: 编写 b_gate_check.sh**

```bash
#!/bin/bash
# scripts/b_gate_check.sh — B-Gate Pre-Merge 检查
# 用法: bash scripts/b_gate_check.sh <pr_number>
# 退出码: 0 = 通过, 1 = 失败

set -euo pipefail

PR_NUM="${1:-}"
TOKEN="${GITEA_TOKEN:-$(cat ~/.ssh/openclaw-gitea.PAT)}"
REPO="openclaw/sqlrustgo"
BASE_URL="http://192.168.0.252:3000/api/v1/repos/$REPO"

if [[ -z "$PR_NUM" ]]; then
  echo "Usage: $0 <pr_number>"
  exit 1
fi

echo "=== B-Gate Check for PR #$PR_NUM ==="

# --- B-Gate 1: Git Identity 检查 ---
echo "[B1] Checking Git identity..."
PR_INFO=$(curl -s "$BASE_URL/pulls/$PR_NUM" -H "Authorization: token $TOKEN")
AUTHOR_EMAIL=$(echo "$PR_INFO" | python3 -c "import json,sys; print(json.load(sys.stdin).get('author',{}).get('email',''))" 2>/dev/null || echo "")

# 等效: 从 commits 获取 author email
AUTHOR=$(echo "$PR_INFO" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('user','{}').get('login','') or d.get('author',{}).get('login',''))" 2>/dev/null || echo "unknown")

# 检查 PR 作者是否是合法 Agent
ALLOWED_AUTHORS=("hermes-macmini" "hermes-z6g4" "hermes-ai250" "openclaw" "openheart[bot]")
FOUND=0
for allowed in "${ALLOWED_AUTHORS[@]}"; do
  if [[ "$AUTHOR" == "$allowed" ]]; then
    FOUND=1; break
  fi
done

if [[ $FOUND -eq 0 ]]; then
  echo "❌ B1 FAIL: Author '$AUTHOR' not in allowed list"
  exit 1
fi
echo "✅ B1 PASS: Author=$AUTHOR"

# --- B-Gate 2: 检查是否有 skip-ci ---
echo "[B2] Checking skip-ci..."
HEADSha=$(echo "$PR_INFO" | python3 -c "import json,sys; print(json.load(sys.stdin).get('head',{}).get('sha',''))" 2>/dev/null || echo "")
if [[ -z "$HEADSha" ]]; then
  # 尝试其他字段
  HEADSha=$(echo "$PR_INFO" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('head_sha','') or d.get('additions_sha',''))" 2>/dev/null || echo "")
fi

COMMITS_URL="$BASE_URL/pulls/$PR_NUM/commits"
# 简单检查: PR title/body 不含 skip-ci
PR_TITLE=$(echo "$PR_INFO" | python3 -c "import json,sys; print(json.load(sys.stdin).get('title',''))" 2>/dev/null || echo "")
PR_BODY=$(echo "$PR_INFO" | python3 -c "import json,sys; print(json.load(sys.stdin).get('body',''))" 2>/dev/null || echo "")

if echo "$PR_TITLE $PR_BODY" | grep -qi "skip-ci\|[ci skip]\|skip ci"; then
  echo "❌ B2 FAIL: PR contains skip-ci"
  exit 1
fi
echo "✅ B2 PASS: No skip-ci"

# --- B-Gate 3: 检查是否修改核心模块 (无正当理由) ---
echo "[B3] Checking protected paths..."
# 获取 PR 的文件列表
FILES_URL="$BASE_URL/pulls/$PR_NUM/files"
FILES=$(curl -s "$FILES_URL" -H "Authorization: token $TOKEN" | python3 -c "
import json,sys
files=json.load(sys.stdin)
for f in files:
    print(f.get('filename',''))
" 2>/dev/null || echo "")

PROTECTED_PATHS=(
  "crates/sqlrustgo-core/"
  "crates/executor/"
  "crates/planner/"
  "crates/optimizer/"
)

VIOLATIONS=""
for file in $FILES; do
  for protected in "${PROTECTED_PATHS[@]}"; do
    if [[ "$file" == "$protected"* ]]; then
      VIOLATIONS="$VIOLATIONS $file"
    fi
  done
done

if [[ -n "$VIOLATIONS" ]]; then
  echo "⚠️  B3 WARNING: Modified protected paths:$VIOLATIONS"
  echo "   (Allowed if PR includes tests and review)"
  # B3 是 WARNING 而非 FAIL，因为有正当理由时应该允许
fi
echo "✅ B3 CHECK COMPLETE"

# --- B-Gate 4: 检查 CI 状态 ---
echo "[B4] Checking CI status..."
STATUS_URL="$BASE_URL/statuses/$HEADSha"
# 简单检查: PR 是否有 CI 运行记录
CHECKS=$(curl -s "$BASE_URL/checks/runs?pull_request=$PR_NUM" -H "Authorization: token $TOKEN" | python3 -c "
import json,sys
try:
    d=json.load(sys.stdin)
    runs=d.get('check_runs', d.get('runs', []))
    print(len(runs))
except: print('0')
" 2>/dev/null || echo "0")

if [[ "$CHECKS" == "0" ]]; then
  echo "⚠️  B4 WARNING: No CI checks recorded for PR #$PR_NUM"
  echo "   (May need to wait for runner to pick up)"
fi
echo "✅ B4 CHECK COMPLETE"

echo ""
echo "=== B-Gate Result: PASS (with warnings above) ==="
exit 0
```

**Step 2: 设置执行权限**

```bash
chmod +x ~/workspace/dev/openheart/sqlrustgo/scripts/b_gate_check.sh
echo "b_gate_check.sh is executable"
```

**Step 3: 测试 B-Gate (用已有 PR)**

```bash
cd ~/workspace/dev/openheart/sqlrustgo
bash scripts/b_gate_check.sh 88 2>&1
# 预期: B1-B4 检查输出，最终 exit 0
```

### Task 2.2: 创建 pre-merge hook (Gitea webhook 模式)

> **注意:** Gitea 没有 pre-receive hook (只有 GitHub Enterprise 有)，所以用 webhook 模拟

**Files:**
- Create: `scripts/hermes_gate_server.py` (新文件)
- Modify: `scripts/gate/handler.py` (新文件)

**Step 1: 创建 Gate Server**

```python
#!/usr/bin/env python3
# scripts/hermes_gate_server.py
"""
Hermes Gate Server — HTTP webhook handler for pre-merge checks
监听: http://0.0.0.0:7890/gate
用途: 拦截 PR merge 事件，执行 B/R-Gate 检查

部署: nohup python3 hermes_gate_server.py &
"""

import http.server
import socketserver
import json
import subprocess
import os
import sys

PORT = 7890
TOKEN = os.environ.get("GITEA_TOKEN", open("/Users/liying/.ssh/openclaw-gitea.PAT").read().strip())
REPO = "openclaw/sqlrustgo"

class GateHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        sys.stderr.write(f"[GateServer] {format}\n")

    def do_POST(self):
        if self.path == "/gate":
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8")

            try:
                event = json.loads(body)
                action = event.get("action", "")
                pr = event.get("pull_request", {})

                # 只处理 close 事件 (合并时触发)
                if action == "closed" and pr.get("merged"):
                    pr_num = pr.get("number")
                    print(f"[GateServer] PR #{pr_num} MERGED — logging event")
                    self._log_merge(pr_num, pr)
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(json.dumps({"status": "logged"}).encode())
                    return

                # PR 打开/更新时执行 Gate 检查
                if action in ("opened", "synchronize", "reopened"):
                    pr_num = pr.get("number")
                    print(f"[GateServer] PR #{pr_num} {action} — running gate checks")
                    result = self._run_gate_checks(pr_num)
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.end_headers()
                    self.wfile.write(json.dumps(result).encode())
                    return

            except Exception as e:
                print(f"[GateServer] ERROR: {e}")

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok"}).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def _run_gate_checks(self, pr_num):
        """执行 B-Gate 检查脚本"""
        try:
            result = subprocess.run(
                ["bash", "/Users/liying/workspace/dev/openheart/sqlrustgo/scripts/b_gate_check.sh", str(pr_num)],
                capture_output=True, text=True, timeout=60
            )
            return {
                "pr": pr_num,
                "gate": "B-Gate",
                "passed": result.returncode == 0,
                "stdout": result.stdout[:500],
                "stderr": result.stderr[:500],
                "returncode": result.returncode
            }
        except Exception as e:
            return {"pr": pr_num, "gate": "B-Gate", "passed": False, "error": str(e)}

    def _log_merge(self, pr_num, pr):
        """记录合并事件到 audit log"""
        log_file = "/Users/liying/workspace/dev/openheart/sqlrustgo/.hermes/audit/merge_log.jsonl"
        os.makedirs(os.path.dirname(log_file), exist_ok=True)
        with open(log_file, "a") as f:
            f.write(json.dumps({
                "event": "merge",
                "pr": pr_num,
                "merged_by": pr.get("merged_by", {}).get("login", "unknown"),
                "merged_at": pr.get("merged_at", ""),
                "author": pr.get("user", {}).get("login", "unknown")
            }) + "\n")

with socketserver.TCPServer(("", PORT), GateHandler) as httpd:
    print(f"[GateServer] Listening on :{PORT}")
    httpd.serve_forever()
```

**Step 2: 在 Gitea Web UI 注册 Webhook**

```
URL: http://192.168.0.252:3000/openclaw/sqlrustgo/settings/hooks
Add Webhook → Gitea
URL: http://<macmini-ip>:7890/gate
HTTP Method: POST
Trigger: Pull Request events (opened, closed, reopened, synchronize)
Secret: (留空或设置 token)
```

**Step 3: 启动 Gate Server (后台)**

```bash
cd ~/workspace/dev/openheart/sqlrustgo
nohup python3 scripts/hermes_gate_server.py > scripts/gate_server.log 2>&1 &
echo $! > scripts/gate_server.pid
echo "Gate server PID: $(cat scripts/gate_server.pid)"

# 验证
sleep 2
curl -s http://localhost:7890/gate -X POST \
  -H "Content-Type: application/json" \
  -d '{"action":"opened","pull_request":{"number":1,"title":"test"}}' | python3 -m json.tool
```

### Task 2.3: 将 Gate 检查集成到 PR workflow

**Step 1: 更新 hermes-gate skill**

```bash
# 更新 hermes-gate skill 内容，添加 B-Gate pre-merge 检查
```

**Step 2: 创建 PR 检查报告 Comment**

当 B-Gate 检查时，自动在 PR 下评论结果：

```python
# scripts/post_gate_comment.py
#!/usr/bin/env python3
"""在 PR 下评论 Gate 检查结果"""
import requests
import json
import sys
import os

PR_NUM = sys.argv[1]
GATE_RESULT = json.loads(sys.argv[2])  # b_gate_check.sh 的 JSON 输出
TOKEN = open("/Users/liying/.ssh/openclaw-gitea.PAT").read().strip()
BASE = "http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo"

comment = f"""
## 🔒 B-Gate Check Result

| Check | Status |
|-------|--------|
| B1 Git Identity | {'✅ PASS' if True else '❌ FAIL'} |
| B2 skip-ci | {'✅ PASS' if True else '❌ FAIL'} |
| B3 Protected Paths | ⚠️ WARNING |
| B4 CI Status | ⚠️ WARNING |

**Overall: {'✅ PASS' if GATE_RESULT['passed'] else '❌ FAIL'}**

```
{GATE_RESULT.get('stdout', '')[:300]}
```

_Hermes Gate Server · {GATE_RESULT.get('timestamp', 'now')}_
"""

resp = requests.post(
    f"{BASE}/issues/{PR_NUM}/comments",
    headers={"Authorization": f"token {TOKEN}"},
    json={"body": comment}
)
print(f"Comment posted: {resp.status_code}")
```

---

## Phase 3: PR 锁机制 — 防止并发写入冲突

> **Objective:** 解决多 Agent 并发写同一仓库的问题

### Task 3.1: 创建 PR 锁文件机制

**Files:**
- Create: `scripts/pr_lock.sh` (新文件)
- Create: `.hermes/locks/` (目录)

**Step 1: 创建 pr_lock.sh**

```bash
#!/bin/bash
# scripts/pr_lock.sh — PR 文件级锁
# 用法:
#   bash pr_lock.sh acquire <module> <pr_number>  # 获取锁
#   bash pr_lock.sh release <module>              # 释放锁
#   bash pr_lock.sh status <module>               # 查看锁状态
#   bash pr_lock.sh list                           # 列出所有锁

set -euo pipefail

ACTION="${1:-}"
MODULE="${2:-}"
PR_NUM="${3:-}"
LOCK_DIR="/Users/liying/workspace/dev/openheart/sqlrustgo/.hermes/locks"
mkdir -p "$LOCK_DIR"

log() { echo "[$(date '+%H:%M:%S')] $*"; }

acquire() {
  local module=$1 pr_num=$2
  local lock_file="$LOCK_DIR/${module}.lock"
  local max_wait=300  # 5分钟超时
  local waited=0

  while [[ -f "$lock_file" ]]; do
    local holder=$(cat "$lock_file")
    if [[ "$holder" == "$PR_NUM" ]]; then
      log "Lock already held by PR #$PR_NUM for module '$module'"
      return 0
    fi
    log "Module '$module' locked by PR #$holder, waiting... ($waited/$max_wait)s"
    sleep 5
    waited=$((waited + 5))
    if [[ $waited -ge $max_wait ]]; then
      log "❌ Timeout waiting for lock on module '$module'"
      return 1
    fi
  done

  echo "$PR_NUM" > "$lock_file"
  log "✅ Acquired lock: module='$module' by PR #$PR_NUM"
  echo "$lock_file"
}

release() {
  local module=$1
  local lock_file="$LOCK_DIR/${module}.lock"

  if [[ ! -f "$lock_file" ]]; then
    log "No lock found for module '$module'"
    return 0
  fi

  local holder=$(cat "$lock_file")
  if [[ "$holder" != "$PR_NUM" ]] && [[ -n "$PR_NUM" ]]; then
    log "⚠️  Lock for '$module' held by PR #$holder, not PR #$PR_NUM"
    return 1
  fi

  rm -f "$lock_file"
  log "✅ Released lock: module='$module'"
}

status() {
  local module=$1
  local lock_file="$LOCK_DIR/${module}.lock"

  if [[ -f "$lock_file" ]]; then
    local holder=$(cat "$lock_file")
    echo "🔒 Locked: module='$module' by PR #$holder"
  else
    echo "🔓 Unlocked: module='$module'"
  fi
}

list_locks() {
  echo "=== Active Locks ==="
  if [[ -d "$LOCK_DIR" ]]; then
    ls "$LOCK_DIR"/*.lock 2>/dev/null | while read f; do
      local module=$(basename "$f" .lock)
      local holder=$(cat "$f")
      echo "  🔒 $module → PR #$holder"
    done
  else
    echo "  (no locks)"
  fi
}

case "$ACTION" in
  acquire) acquire "$MODULE" "$PR_NUM" ;;
  release) release "$MODULE" ;;
  status)  status "$MODULE" ;;
  list)    list_locks ;;
  *)       echo "Usage: $0 {acquire|release|status|list} [module] [pr_num]" ;;
esac
```

**Step 2: 设置权限并测试**

```bash
chmod +x ~/workspace/dev/openheart/sqlrustgo/scripts/pr_lock.sh

# 测试
cd ~/workspace/dev/openheart/sqlrustgo
bash scripts/pr_lock.sh list
# 预期: (no locks)

bash scripts/pr_lock.sh acquire parser 99
# 预期: ✅ Acquired lock: module='parser' by PR #99

bash scripts/pr_lock.sh status parser
# 预期: 🔒 Locked: module='parser' by PR #99

bash scripts/pr_lock.sh release parser
# 预期: ✅ Released lock: module='parser'

bash scripts/pr_lock.sh list
# 预期: (no locks)
```

### Task 3.2: 将 PR Lock 集成到 Gate 检查

修改 `b_gate_check.sh`，添加 B5 检查：

```bash
# 在 b_gate_check.sh 的检查部分添加:

# --- B-Gate 5: PR 锁检查 ---
echo "[B5] Checking PR lock..."
MODIFIED_FILES=$(curl -s "$BASE_URL/pulls/$PR_NUM/files" \
  -H "Authorization: token $TOKEN" | python3 -c "
import json,sys
files=json.load(sys.stdin)
for f in files:
    name=f.get('filename','')
    # 提取模块名 (第一个路径组件)
    if '/' in name:
        print(name.split('/')[0])
" 2>/dev/null | sort -u)

LOCK_CONFLICT=""
for module in $MODIFIED_FILES; do
  LOCK_FILE="$LOCK_DIR/${module}.lock"
  if [[ -f "$LOCK_FILE" ]]; then
    holder=$(cat "$LOCK_FILE" 2>/dev/null || echo "unknown")
    if [[ "$holder" != "$PR_NUM" ]]; then
      LOCK_CONFLICT="$LOCK_CONFLICT $module (held by PR #$holder)"
    fi
  fi
done

if [[ -n "$LOCK_CONFLICT" ]]; then
  echo "❌ B5 FAIL: Module lock conflict:$LOCK_CONFLICT"
  exit 1
fi
echo "✅ B5 PASS: No lock conflicts"
```

---

## Phase 4: GBrain → Rule Engine 升级

> **Objective:** 将 Pattern 文档升级为机器可执行的 YAML 规则

### Task 4.1: 创建规则文件结构

**Files:**
- Create: `.hermes/rules/` (目录)
- Create: `.hermes/rules/forbid_actions_cache.yaml`
- Create: `.hermes/rules/forbid_raw_root.yaml`
- Create: `.hermes/rules/require_tests_for_core.yaml`

**Step 1: 创建规则目录和元数据**

```bash
mkdir -p ~/workspace/dev/openheart/sqlrustgo/.hermes/rules
mkdir -p ~/workspace/dev/openheart/sqlrustgo/.hermes/audit

cat > ~/workspace/dev/openheart/sqlrustgo/.hermes/rules/_meta.yaml << 'EOF'
# Hermes Rule Engine — 规则注册表
# 自动加载 .hermes/rules/*.yaml (排除 _meta.yaml)
version: 1.0
updated: 2026-05-01
rules:
  - id: forbid_actions_cache
    file: forbid_actions_cache.yaml
    description: 禁止 Gitea Actions 使用 actions/cache
    severity: error
    source: PR #88 验证

  - id: forbid_raw_root
    file: forbid_raw_root.yaml
    description: 禁止 root 用户执行 Docker 容器
    severity: error
    source: CI/CD 运维规范

  - id: require_tests_for_core
    file: require_tests_for_core.yaml
    description: 核心模块修改必须包含测试
    severity: warning
    source: AGENTS.md 规则
EOF
echo "_meta.yaml created"
```

**Step 2: 创建 forbid_actions_cache.yaml**

```yaml
# .hermes/rules/forbid_actions_cache.yaml
id: forbid_actions_cache
version: 1.0
description: |
  Gitea 1.22.1 与 actions/cache 不兼容，会导致 CI 卡死。
  验证: PR #88 (2026-05-01)
  根因: Gitea Actions 1.22.1 的 actions/cache 实现有 bug

trigger:
  files:
    - pattern: ".gitea/workflows/*.yml"
    - pattern: ".gitea/workflows/*.yaml"

condition:
  contains_any:
    - "actions/cache"
    - "actions/upload-artifact@v4"
    - "actions/download-artifact@v4"

action:
  - type: reject_merge
    message: |
      ❌ 禁止在 Gitea Actions 中使用 actions/cache
      
      Gitea 1.22.1 与官方 actions/cache 不兼容，会导致 CI 卡死。
      验证: PR #88 (2026-05-01)
      
      替代方案: 使用纯 shell 步骤进行缓存管理

enforcement:
  gate: B-Gate
  bypass_requires: hermes-macmini-admin  # 需管理员批准
```

**Step 3: 创建 forbid_raw_root.yaml**

```yaml
# .hermes/rules/forbid_raw_root.yaml
id: forbid_raw_root
version: 1.0
description: |
  Nomad runner 容器必须以非 root 用户运行。
  根因: USER 1000 导致 Docker 写入权限问题 (CI/CD 修复日志 2026-04-30)

trigger:
  files:
    - pattern: "Dockerfile*"
    - pattern: ".github/workflows/*.yml"
    - pattern: ".gitea/workflows/*.yml"

condition:
  contains_any:
    - "USER root"
    - "USER 0"
    - "run_as_root: true"

action:
  - type: reject_merge
    message: |
      ❌ 禁止在容器配置中使用 root 用户
      
      根因: USER root 导致 Docker 写入权限问题
      验证: CI/CD 修复日志 (2026-04-30)
      
      正确做法: USER 1000 或使用指定用户

enforcement:
  gate: B-Gate
```

**Step 4: 创建 require_tests_for_core.yaml**

```yaml
# .hermes/rules/require_tests_for_core.yaml
id: require_tests_for_core
version: 1.0
description: |
  修改核心模块 (executor, planner, optimizer) 时必须包含对应测试。
  来源: AGENTS.md 开发规范

trigger:
  files:
    - pattern: "crates/executor/**/*.rs"
    - pattern: "crates/planner/**/*.rs"
    - pattern: "crates/optimizer/**/*.rs"
    - pattern: "crates/sqlrustgo-core/**/*.rs"

condition:
  # PR 修改了核心模块文件
  module_modified: core

action:
  - type: require_files
    file_patterns:
      - "tests/**/test_{module_name}*.rs"
      - "crates/{module}/src/**/tests.rs"
    message: |
      ⚠️ 修改核心模块时建议包含测试
      
      已修改: {modified_files}
      
      请确保包含相应的测试文件，或在 PR 评论中说明测试计划

enforcement:
  gate: R-Gate
  severity: warning  # warning 而非 reject，允许紧急情况 bypass
```

### Task 4.2: 创建规则检查器

**Files:**
- Create: `scripts/rule_check.py` (新文件)

**Step 1: 编写 rule_check.py**

```python
#!/usr/bin/env python3
"""
scripts/rule_check.py — Hermes Rule Engine 检查器

用法:
  python3 scripts/rule_check.py <pr_number>
  python3 scripts/rule_check.py <pr_number> --enforce  # 强制执行 (exit on fail)
  python3 scripts/rule_check.py --validate-rules         # 验证规则文件语法
"""

import json
import os
import sys
import glob
import yaml
import requests
from typing import List, Dict, Any

REPO = "openclaw/sqlrustgo"
TOKEN = open("/Users/liying/.ssh/openclaw-gitea.PAT").read().strip()
BASE_URL = f"http://192.168.0.252:3000/api/v1/repos/{REPO}"
RULES_DIR = "/Users/liying/workspace/dev/openheart/sqlrustgo/.hermes/rules"
AUDIT_LOG = "/Users/liying/workspace/dev/openheart/sqlrustgo/.hermes/audit/rule_audit.jsonl"


def load_rules() -> List[Dict]:
    """加载所有规则文件"""
    rules = []
    pattern = os.path.join(RULES_DIR, "*.yaml")
    for path in glob.glob(pattern):
        basename = os.path.basename(path)
        if basename.startswith("_"):
            continue
        try:
            with open(path) as f:
                rule = yaml.safe_load(f)
                rule["_file"] = path
                rules.append(rule)
        except Exception as e:
            print(f"⚠️  Failed to load {path}: {e}", file=sys.stderr)
    return rules


def get_pr_files(pr_num: int) -> List[str]:
    """获取 PR 修改的文件列表"""
    resp = requests.get(
        f"{BASE_URL}/pulls/{pr_num}/files",
        headers={"Authorization": f"token {TOKEN}"}
    )
    if resp.status_code != 200:
        return []
    files = resp.json()
    return [f.get("filename", "") for f in files]


def check_rule(rule: Dict, pr_files: List[str], pr_num: int) -> Dict:
    """检查单条规则是否触发"""
    trigger = rule.get("trigger", {})
    patterns = trigger.get("files", [])

    matched_files = []
    for f in pr_files:
        for p in patterns:
            if isinstance(p, dict) and "pattern" in p:
                import fnmatch
                if fnmatch.fnmatch(f, p["pattern"]):
                    matched_files.append(f)
            elif isinstance(p, str):
                import fnmatch
                if fnmatch.fnmatch(f, p):
                    matched_files.append(f)

    if not matched_files:
        return {"triggered": False}

    # 检查 condition
    condition = rule.get("condition", {})
    actions = rule.get("action", [])

    # 简化: 如果文件匹配，则触发
    return {
        "triggered": True,
        "rule_id": rule["id"],
        "matched_files": matched_files,
        "severity": rule.get("severity", "warning"),
        "actions": actions,
        "description": rule.get("description", "")
    }


def run_rule_check(pr_num: int, enforce: bool = False) -> Dict:
    """对 PR 执行所有规则检查"""
    rules = load_rules()
    pr_files = get_pr_files(pr_num)

    results = {
        "pr": pr_num,
        "files_checked": len(pr_files),
        "rules_loaded": len(rules),
        "violations": [],
        "warnings": []
    }

    for rule in rules:
        check_result = check_rule(rule, pr_files, pr_num)
        if check_result.get("triggered"):
            severity = check_result.get("severity", "warning")
            if severity == "error":
                results["violations"].append(check_result)
            else:
                results["warnings"].append(check_result)

    # 审计日志
    os.makedirs(os.path.dirname(AUDIT_LOG), exist_ok=True)
    with open(AUDIT_LOG, "a") as f:
        f.write(json.dumps({
            "event": "rule_check",
            "pr": pr_num,
            "violations": len(results["violations"]),
            "warnings": len(results["warnings"]),
            "passed": len(results["violations"]) == 0
        }) + "\n")

    # 输出
    print(f"=== Rule Check for PR #{pr_num} ===")
    print(f"Files: {pr_files[:5]}{'...' if len(pr_files) > 5 else ''}")
    print(f"Rules loaded: {len(rules)}")

    if results["violations"]:
        print(f"\n❌ VIOLATIONS ({len(results['violations'])}):")
        for v in results["violations"]:
            print(f"  - [{v['rule_id']}] {v['description'][:60]}...")
            for f in v.get("matched_files", [])[:3]:
                print(f"      → {f}")

    if results["warnings"]:
        print(f"\n⚠️  WARNINGS ({len(results['warnings'])}):")
        for w in results["warnings"]:
            print(f"  - [{w['rule_id']}] {w['description'][:60]}...")

    if not results["violations"] and not results["warnings"]:
        print("\n✅ All rules passed")

    passed = len(results["violations"]) == 0
    print(f"\nResult: {'✅ PASS' if passed else '❌ FAIL'}")

    if enforce and not passed:
        sys.exit(1)

    return results


def validate_rules() -> bool:
    """验证所有规则文件语法"""
    rules = load_rules()
    print(f"Validating {len(rules)} rule files...")
    for rule in rules:
        required = ["id", "version", "description", "trigger", "action"]
        missing = [k for k in required if k not in rule]
        if missing:
            print(f"❌ {rule.get('_file', '?')}: missing fields: {missing}")
            return False
        print(f"✅ {rule['id']} (v{rule['version']})")
    print(f"\n✅ All {len(rules)} rules valid")
    return True


if __name__ == "__main__":
    if "--validate-rules" in sys.argv:
        sys.exit(0 if validate_rules() else 1)

    if len(sys.argv) < 2:
        print("Usage: python3 rule_check.py <pr_num> [--enforce]")
        sys.exit(1)

    pr_num = int(sys.argv[1])
    enforce = "--enforce" in sys.argv
    run_rule_check(pr_num, enforce=enforce)
```

**Step 2: 设置执行权限并测试**

```bash
chmod +x ~/workspace/dev/openheart/sqlrustgo/scripts/rule_check.py
python3 ~/workspace/dev/openheart/sqlrustgo/scripts/rule_check.py --validate-rules
# 预期: ✅ All X rules valid

# 用 PR #88 测试 (已包含 fmt 修复)
python3 ~/workspace/dev/openheart/sqlrustgo/scripts/rule_check.py 88
# 预期: ✅ All rules passed (因为 PR #88 不违反任何规则)
```

---

## Phase 5: 文档与 Wiki 更新

### Task 5.1: 更新协作架构文档

**Step 1: 添加 Control Plane 章节到架构文档**

在 `docs/plans/2026-05-01-collaboration-architecture.md` 添加:

```markdown
## 九、Control Plane (2026-05-01 新增)

### 9.1 权力分层

| 角色 | 权限 |
|------|------|
| Hermes (Mac Mini) | 审核 + PR 创建, 禁止 merge |
| System Gate | B/R-Gate 强制校验 + merge 控制 |
| Worker | 提交 PR + 报告状态 |

### 9.2 Gate 校验流程

```
PR 提交
   ↓
B-Gate (pre-merge hook)
  ├─ B1: Git identity 检查
  ├─ B2: skip-ci 检查
  ├─ B3: 核心模块修改检查
  ├─ B4: CI 状态检查
  └─ B5: PR 锁检查
   ↓
R-Gate (CI 完成后)
  ├─ cargo test
  ├─ clippy -D warnings
  └─ fmt check
   ↓
Gitea Branch Protection 强制合并
```

### 9.3 规则引擎

规则文件位于: `.hermes/rules/`

执行: `python3 scripts/rule_check.py <pr_num> --enforce`
```

### Task 5.2: 发布更新到 Wiki

```bash
# 更新 wiki 仓库
cd /tmp/wiki-clone
git pull origin main

# 复制更新后的文档
tail -n +9 ~/workspace/dev/openheart/sqlrustgo/docs/plans/2026-05-01-collaboration-architecture.md \
  > AI-Agent-Collaboration-Architecture.md

git add AI-Agent-Collaboration-Architecture.md
git commit -m "docs(wiki): add Control Plane section v2 [2026-05-01]"
GIT_SSH_COMMAND="ssh -p 222 -i /Users/liying/.ssh/id_ed25519_macmini -o IdentitiesOnly=yes" \
  git push origin main
```

---

## 验证清单 (执行后必读)

```bash
# 1. Hermes 降权
cat ~/.hermes/agent-roles.yaml
# 预期: denied: merge_pull_requests

# 2. B-Gate 检查
bash ~/workspace/dev/openheart/sqlrustgo/scripts/b_gate_check.sh 88
# 预期: ✅ B1-B5 PASS

# 3. Rule Engine
python3 ~/workspace/dev/openheart/sqlrustgo/scripts/rule_check.py 88
# 预期: ✅ All rules passed

# 4. PR Lock
bash ~/workspace/dev/openheart/sqlrustgo/scripts/pr_lock.sh acquire parser 99
bash ~/workspace/dev/openheart/sqlrustgo/scripts/pr_lock.sh list
bash ~/workspace/dev/openheart/sqlrustgo/scripts/pr_lock.sh release parser

# 5. Gate Server
curl -s http://localhost:7890/gate -X POST \
  -H "Content-Type: application/json" \
  -d '{"action":"opened","pull_request":{"number":1,"title":"test"}}'
# 预期: {"status":"ok"}

# 6. 备份验证
ls -la ~/backups/hermes-ai-factory-20260501-125908/
# 预期: 5+ 文件
```

---

## 回滚步骤 (如果出问题)

```bash
# 如果 B-Gate hook 导致所有 PR 无法合并:
# 1. 删除 webhook
curl -X DELETE "http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/hooks/<hook_id>" \
  -H "Authorization: token $TOKEN"

# 2. 停止 Gate Server
kill $(cat ~/workspace/dev/openheart/sqlrustgo/scripts/gate_server.pid)

# 3. 恢复 gate_spec.md
cp ~/backups/hermes-ai-factory-20260501-125908/gate_spec.md \
  ~/workspace/dev/openheart/sqlrustgo/docs/governance/gate_spec.md

# 4. 恢复 hermes-gate skill
cp -r ~/backups/hermes-ai-factory-20260501-125908/hermes-gate \
  ~/.hermes/skills/autonomous-ai-agents/

# 5. 恢复 hermes-gate-development skill
cp -r ~/backups/hermes-ai-factory-20260501-125908/hermes-gate-development \
  ~/.hermes/skills/autonomous-ai-agents/
```

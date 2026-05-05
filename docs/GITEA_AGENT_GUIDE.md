# Gitea 操作指南（AI Agent 版）

> 给 Hermes、OpenCode 等 AI Agent 的推送/PR/Issue 操作规范。
> 最后更新: 2026-05-03

---

## ⚠️ 铁律：先读这个，再做任何操作

### 1. Gitea 只有 HTTP，没有 HTTPS

Gitea 的 Web/Git 服务全部通过纯 HTTP 提供，服务器没有 TLS/SSL 证书。

```
❌ 错误：git push https://openclaw:details8848@192.168.0.252:3000/...
    → SSL routines:ST_CONNECT:tlsv1 alert protocol version

✅ 正确：git push http://openclaw:details8848@192.168.0.252:3000/...
```

**所有 Git 操作必须使用 `http://`，绝不能写 `https://`。**

### 2. 先在 develop 分支完成功能，再创建 PR

```
所有功能在 develop/v2.9.0 分支完成
alpha/beta 阶段只做集成+修复，不做新功能
```

### 3. 动大模块前先备份

容器/DB/Gitea/runner，任何可能有破坏性的操作之前先备份。

---

## 一、添加 remote

```bash
# Gitea（内网开发主仓）— 推荐
git remote add gitea http://openclaw:details8848@192.168.0.252:3000/openclaw/sqlrustgo.git

# GitCode（云端备份）
git remote add gitcode git@gitcode.com:BreavHeart/sqlrustgo.git

# GitHub（外部备份）
git remote add github git@github.com:minzuuniversity/sqlrustgo.git

# Gitee（国内备份）
git remote add gitee git@gitee.com:yinglichina/sqlrustgo.git
```

---

## 二、推送代码

### 标准流程：创建功能分支 → 提交 → 推送

```bash
# 1. 从最新 develop 切出功能分支
git checkout develop/v2.9.0
git pull gitea develop/v2.9.0
git checkout -b feature/my-feature

# 2. 开发... 提交...
git add .
git commit -m "feat: 描述修改内容"

# 3. 推送到 Gitea
git push gitea feature/my-feature

# 4. 同步到 GitCode（云端备份）
git push gitcode feature/my-feature
```

### 推送铁律

- **不直接推送到 `develop/*`** — 这些分支有分支保护（需 PR + status check + 1 approval）
- **不要用 `--force` 推送** — 除非确实知道在做什么
- **功能分支命名：`feature/<英文描述>`** — 如 `feature/cte-fix-156`
- **测试分支命名：`test/<描述>`** — 用完即删

### 常见失败

```
# 报错：unable to migrate objects to permanent storage
# Gitea 1.22.1 存储迁移 bug，用 bundle 兜底

# 报错：Decoding Failed (pre-receive hook)
# 重建容器时漏设环境变量，需重设 GITEA__repository__ROOT
```

---

## 三、创建 PR

### 方法 A：推送完成后终端会显示链接

推送成功后 terminal 输出末尾会显示：
```
remote: To create a merge request for feature/my-branch, visit:
remote:   http://192.168.0.252:3000/openclaw/sqlrustgo/compare/develop/v2.9.0...feature/my-branch
```

### 方法 B：API 创建

```bash
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "feat: 实现 XXX 功能",
    "head": "feature/my-branch",
    "base": "develop/v2.9.0",
    "body": "## 变更内容\n\n- 新增 XXX\n- 修复 YYY"
  }'
```

### PR 创建注意事项

| 条件 | 说明 |
|------|------|
| head ≠ base | 两个分支不能相同，否则返回 `422 There are no changes` |
| 先推送后 PR | head 分支必须先 `git push gitea feature/X`，API 中才能找到它 |
| title 清晰 | 格式：`<type>: <描述>`，如 `feat:`, `fix:`, `docs:`, `test:` |
| base 存在 | base 分支必须在仓库中存在（通常是 `develop/v2.9.0`） |

---

## 四、创建 Issue

### API 创建

```bash
TOKEN=04bcda86dd601364a53eec33dc37aa6efa98a5b7

curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Issue 标题（简洁明确）",
    "body": "## 描述\n\n详细说明...\n\n## 复现步骤\n\n1. ...\n2. ...\n\n## 期望行为\n\n...",
    "labels": [1, 2]
  }'
```

### ⚠️ labels 参数用 ID，不是名称

```bash
# 先查 label ID
curl -s http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/labels \
  -H "Authorization: token $TOKEN" | python3 -c "
import json,sys
for label in json.load(sys.stdin):
    print(f'  ID {label[\"id\"]}: {label[\"name\"]}')
"

# 然后创建时传 ID 数组
"labels": [1, 2, 3]
```

### 添加评论

```bash
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues/11/comments \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"body": "评论内容"}'
```

---

## 五、更新/关闭 PR/Issue

```bash
TOKEN=04bcda86dd601364a53eec33dc37aa6efa98a5b7

# 更新 Issue/PR 内容
curl -X PATCH http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues/{NUM} \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"body": "新内容"}'

# 关闭 Issue/PR
curl -X PATCH http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues/{NUM} \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"state": "closed"}'

# 合并 PR
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls/{NUM}/merge \
  -H "Authorization: token $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"Do": "merge", "MergeMessageField": "merge: PR #{NUM}"}'
```

---

## 六、分支保护规则

`develop/*` 和 `release/*` 分支有保护：

```yaml
# develop/v2.9.0 保护规则
enable_push: false                    # 禁止直接推送
required_approvals: 1                 # 需要 1 人审批
status_check_contexts: ["Hermes Pipeline"]  # 需要流水线通过
```

所以必须：
1. 从 `develop/v2.9.0` 切出 `feature/X`
2. 在 `feature/X` 上开发
3. 推送 `feature/X` 到 Gitea
4. API 创建 PR
5. 等审批和流水线通过后合入

---

## 七、三平台同步

```bash
# 推送 feature 分支
git push gitea feature/my-branch   # 开发主仓
git push gitcode feature/my-branch  # 云端备份

# 推送 develop
git push gitea develop/v2.9.0       # 仅当确实需要同步时
git push gitcode develop/v2.9.0     # 云端日常同步
git push github develop/v2.9.0      # 外部备份
git push gitee develop/v2.9.0       # 国内备份
```

---

## 八、健康检查

```bash
# Gitea 是否运行
curl -s http://192.168.0.252:3000/api/healthz
# → 返回 {"pretty_name":"Gitea:Git with a cup of tea","status":"running","api_version":"1.22.1"}

# 仓库是否可访问
curl -s http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('description',''))"

# 检查分支
curl -s http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/branches \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  | python3 -c "import json,sys; [print(b['name']) for b in json.load(sys.stdin)]"
```

---

## 九、快速参考（一行命令速查）

```bash
# 推送
git push gitea feature/my-branch

# 创建 PR
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/pulls \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  -H "Content-Type: application/json" \
  -d '{"title":"PR标题","head":"feature/X","base":"develop/v2.9.0","body":"描述"}'

# 创建 Issue
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  -H "Content-Type: application/json" \
  -d '{"title":"标题","body":"描述"}'

# 添加评论
curl -X POST http://192.168.0.252:3000/api/v1/repos/openclaw/sqlrustgo/issues/11/comments \
  -H "Authorization: token 04bcda86dd601364a53eec33dc37aa6efa98a5b7" \
  -H "Content-Type: application/json" \
  -d '{"body":"评论"}'
```

---

## 附录：凭证汇总

| 用途 | 值 |
|------|-----|
| Gitea URL | `http://192.168.0.252:3000` |
| Git remote | `http://openclaw:details8848@192.168.0.252:3000/openclaw/sqlrustgo.git` |
| API Token | `04bcda86dd601364a53eec33dc37aa6efa98a5b7` |
| Gitea SSH | `ssh://git@192.168.0.252:222/openclaw/sqlrustgo.git`（⚠️ key 未通） |
| GitCode | `git@gitcode.com:BreavHeart/sqlrustgo.git` |
| GitHub | `git@github.com:minzuuniversity/sqlrustgo.git` |
| Gitee | `git@gitee.com:yinglichina/sqlrustgo.git` |

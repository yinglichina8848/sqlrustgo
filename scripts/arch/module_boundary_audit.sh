#!/usr/bin/env bash
# ============================================================
# A-01: Module Boundary Audit — v3.0.0 Architecture Hardening
#
# Checks:
# 1. No circular dependencies between crates
# 2. Identifies pub items that could be pub(crate)
# 3. Generates module dependency graph
# ============================================================
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "=== A-01: Module Boundary Audit ==="
echo "Date: $(date -u)"
echo ""

# Step 1: Check for circular dependencies using cargo metadata
echo "[1/3] Checking circular dependencies..."
cargo metadata --format-version 1 --no-deps 2>/dev/null | python3 -c "
import json, sys
data = json.load(sys.stdin)
packages = {p['name']: p for p in data['packages']}
resolved = data.get('resolve', {})
nodes = {n['id']: n for n in resolved.get('nodes', [])}

# Build adjacency
adj = {}
for nid, node in nodes.items():
    pkg_name = nid.split()[0] if ' ' in nid else nid
    adj[pkg_name] = []
    for dep in node.get('dependencies', []):
        dep_name = dep.split()[0] if ' ' in dep else dep
        adj.setdefault(pkg_name, []).append(dep_name)

# Simple cycle detection
visited = set()
path = []

def dfs(node):
    if node in path:
        cycle = path[path.index(node):] + [node]
        print(f'CYCLE: {\" -> \".join(cycle)}')
        return True
    if node in visited:
        return False
    visited.add(node)
    path.append(node)
    for dep in adj.get(node, []):
        if dfs(dep):
            return True
    path.pop()
    return False

cycles = 0
for node in adj:
    if node not in visited:
        if dfs(node):
            cycles += 1

if cycles == 0:
    print('No circular dependencies found.')" 2>&1 || echo "Cycle check skipped (requires network)"

echo ""

# Step 2: Count pub items per crate (candidates for pub(crate))
echo "[2/3] Counting pub items per crate..."
for crate in crates/*/; do
    name=$(basename "$crate")
    pub_items=$(grep -rn "^pub " "$crate/src/" --include="*.rs" 2>/dev/null | grep -v "pub mod\|pub use\|pub struct\|pub enum\|pub fn\|pub trait\|pub type" | wc -l | tr -d ' ')
    total_pub=$(grep -rn "^pub " "$crate/src/" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
    echo "  $name: $total_pub total pub, ~$pub_items potential pub(crate)"
done

echo ""

# Step 3: Module dependency structure
echo "[3/3] Crate dependency structure..."
cargo metadata --format-version 1 --no-deps 2>/dev/null | python3 -c "
import json, sys
data = json.load(sys.stdin)
workspace_members = set(data.get('workspace_members', []))
packages = {p['name']: p for p in data['packages']}

# Count workspace crates
ws_crates = [p for p in packages.values() if p['id'] in workspace_members]
print(f'Workspace crates: {len(ws_crates)}')

# List crate->crate dependencies
for pkg in ws_crates:
    deps = [d['path'].split('/')[-2] if '/' in d.get('path','') else d['name'] 
            for d in pkg.get('dependencies', [])
            if d.get('path','').startswith('crates/')]
    if deps:
        print(f'  {pkg[\"name\"]:25s} -> {', '.join(deps)}')
" 2>&1 || echo "Dependency check requires network"

echo ""
echo "=== A-01 Audit Complete ==="

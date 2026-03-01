#!/bin/bash

set -e

echo "=== 综合权限验证测试脚本 ==="
echo ""
echo "🎯 测试目标: 全面验证所有权限规则和保护措施"
echo ""
echo "📅 测试时间: $(date)"
echo ""

# 颜色定义
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
BLUE="\033[0;34m"
NC="\033[0m" # No Color

# 测试结果记录
PASSED=0
FAILED=0
WARNINGS=0

# 日志文件
LOG_DIR=".test-logs"
LOG_FILE="$LOG_DIR/permission-test-$(date +%Y%m%d-%H%M%S).log"

# 创建日志目录
mkdir -p "$LOG_DIR"
touch "$LOG_FILE"
echo "$(date) - 开始综合权限验证测试" > "$LOG_FILE"
echo "=========================================" >> "$LOG_FILE"

# 函数: 记录日志
log() {
    local message="$1"
    echo "$message" >> "$LOG_FILE"
}

# 函数: 执行测试并检查结果
run_test() {
    local test_category="$1"
    local test_name="$2"
    local test_command="$3"
    local expected_failure="$4"
    local expected_message="$5"
    local skip="$6"
    
    if [ "$skip" = "true" ]; then
        echo -e "${YELLOW}⚠ $test_name... 跳过${NC}"
        log "⚠ $test_name: 跳过"
        echo "" >> "$LOG_FILE"
        return
    fi
    
    echo -n "🔍 $test_name... "
    log "🔍 $test_name"
    
    # 执行测试命令，捕获输出
    local output
    output=$($test_command 2>&1)
    local exit_code=$?
    
    # 检查结果
    if [ "$expected_failure" = "true" ]; then
        if [ $exit_code -ne 0 ]; then
            # 检查是否包含预期错误信息
            if [[ "$output" == *"$expected_message"* ]]; then
                echo -e "${GREEN}通过${NC}"
                echo "✓ $test_name: 通过" >> "$LOG_FILE"
                log "✓ $test_name: 通过"
                ((PASSED++))
            else
                echo -e "${YELLOW}警告${NC}"
                echo "⚠ $test_name: 失败，但错误信息不符合预期"
                echo "   实际输出: $output"
                log "⚠ $test_name: 失败 - 错误信息不符合预期"
                log "   实际输出: $output"
                ((WARNINGS++))
            fi
        else
            echo -e "${RED}失败${NC}"
            echo "✗ $test_name: 应该失败但成功了"
            log "✗ $test_name: 失败 - 应该失败但成功了"
            ((FAILED++))
        fi
    else
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}通过${NC}"
            echo "✓ $test_name: 通过" >> "$LOG_FILE"
            log "✓ $test_name: 通过"
            ((PASSED++))
        else
            echo -e "${RED}失败${NC}"
            echo "✗ $test_name: 失败"
            echo "   错误: $output"
            log "✗ $test_name: 失败"
            log "   错误: $output"
            ((FAILED++))
        fi
    fi
    
    echo "" >> "$LOG_FILE"
}

# 函数: 测试分组
start_test_group() {
    local group_name="$1"
    echo ""
    echo -e "${BLUE}=== $group_name ===${NC}"
    echo ""
    log ""
    log "=== $group_name ==="
    log ""
}

# 准备测试环境
echo "📋 准备测试环境..."
log "📋 准备测试环境"

# 保存当前分支
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "main")
log "当前分支: $CURRENT_BRANCH"

# 切换到 main 分支
git stash -u > /dev/null 2>&1 || true
git checkout main > /dev/null 2>&1 || true
git pull origin main > /dev/null 2>&1 || true

# 清理测试文件和标签
rm -f ".test-*" > /dev/null 2>&1 || true
git tag -d "v1.0.0-test" > /dev/null 2>&1 || true
git push origin --delete "v1.0.0-test" > /dev/null 2>&1 || true

echo ""
log "测试环境准备完成"

# 测试 1: 分支保护规则
start_test_group "测试 1: 分支保护规则"

# 测试 1.1: 直接 push 到 main
run_test "分支保护" "直接 push 到 main" "git commit --allow-empty -m 'test direct push' && git push origin main" "true" "protected branch"

# 测试 1.2: force push 到 main
run_test "分支保护" "force push 到 main" "git commit --allow-empty -m 'test force push' && git push --force origin main" "true" "force push"

# 测试 1.3: 删除 main 分支（需要在 UI 测试）
echo "⚠ 分支删除保护需要在 GitHub UI 上手动测试"
echo "   请尝试删除 main 分支，应该被拒绝"
log "⚠ 分支删除保护需要在 GitHub UI 上手动测试"

# 测试 2: Tag 保护规则
start_test_group "测试 2: Tag 保护规则"

# 创建测试 tag
git tag v1.0.0-test > /dev/null 2>&1 || true
git push origin v1.0.0-test > /dev/null 2>&1 || true

# 测试 2.1: 删除 tag
run_test "Tag 保护" "删除 tag" "git tag -d v1.0.0-test && git push origin :refs/tags/v1.0.0-test" "true" "delete tag"

# 清理
git tag -d v1.0.0-test > /dev/null 2>&1 || true

# 测试 3: PR 相关规则
start_test_group "测试 3: PR 相关规则"

# 测试 3.1: 创建测试分支
git checkout -b test-pr-branch > /dev/null 2>&1 || true

# 测试 3.2: 提交更改
echo "test" > .test-pr-file
git add .test-pr-file
git commit -m "test: add test file" > /dev/null 2>&1 || true

# 测试 3.3: push 测试分支（应该允许）
run_test "PR 规则" "push 测试分支" "git push origin test-pr-branch" "false" ""

# 测试 4: 签名提交规则
start_test_group "测试 4: 签名提交规则"

# 测试 4.1: 创建未签名提交
echo "test" > .test-unsigned
git add .test-unsigned
git commit -m "test: unsigned commit" --no-gpg-sign > /dev/null 2>&1 || true

# 测试 4.2: push 未签名提交（如果启用了签名要求，应该失败）
# 注意：如果未启用签名要求，此测试会通过
run_test "签名规则" "push 未签名提交" "git push origin test-pr-branch" "false" ""

# 测试 5: 其他保护规则
start_test_group "测试 5: 其他保护规则"

# 测试 5.1: 尝试修改受保护文件
echo "modified" > README.md
git add README.md
git commit -m "test: modify README" > /dev/null 2>&1 || true

# 测试 5.2: 尝试 push 修改（应该被拒绝）
run_test "保护规则" "push 修改到 main" "git push origin test-pr-branch:main" "true" "protected branch"

# 清理测试环境
echo ""
echo "🧹 清理测试环境..."
log "🧹 清理测试环境"

# 切换回原分支
git checkout "$CURRENT_BRANCH" > /dev/null 2>&1 || true

# 清理测试分支和文件
git branch -D test-pr-branch > /dev/null 2>&1 || true
git push origin --delete test-pr-branch > /dev/null 2>&1 || true
rm -f ".test-*" > /dev/null 2>&1 || true

# 恢复 stash
git stash pop > /dev/null 2>&1 || true

# 生成测试报告
echo ""
echo "=== 测试报告 ==="
echo "通过: $PASSED"
echo "失败: $FAILED"
echo "警告: $WARNINGS"
echo ""

log ""
log "=== 测试报告 ==="
log "通过: $PASSED"
log "失败: $FAILED"
log "警告: $WARNINGS"
log ""

if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo -e "${GREEN}🎉 所有测试通过！权限规则配置正确。${NC}"
        log "🎉 所有测试通过！权限规则配置正确。"
    else
        echo -e "${YELLOW}⚠️  测试基本通过，但有警告需要检查。${NC}"
        log "⚠️  测试基本通过，但有警告需要检查。"
    fi
else
    echo -e "${RED}❌ 有测试失败，需要检查权限配置。${NC}"
    log "❌ 有测试失败，需要检查权限配置。"
fi

echo ""
echo "=== 验证完成 ==="
echo ""
echo "📋 手动测试项:"
echo "1. Owner账号绕过测试（使用Owner账号重复上述测试）"
echo "2. GitHub UI直接编辑测试（尝试直接编辑main文件）"
echo "3. PR审批绕过测试（不等待审批尝试merge）"
echo "4. 分支删除测试（在UI上尝试删除main分支）"
echo ""
echo "🚨 重要提示:"
echo "- 确保 'Include administrators' 已开启"
echo "- 确保所有分支保护规则已正确配置"
echo "- 定期运行此脚本进行验证"
echo ""
echo "📊 测试详情请查看日志文件: $LOG_FILE"

log ""
log "=== 验证完成 ==="
log "测试详情请查看日志文件: $LOG_FILE"

#!/bin/bash

set -e

echo "=== 企业级权限验证脚本 ==="
echo ""
echo "🎯 验证目标: 确保所有权限规则正确生效"
echo ""

# 颜色定义
GREEN="\033[0;32m"
RED="\033[0;31m"
YELLOW="\033[1;33m"
NC="\033[0m" # No Color

# 测试结果记录
PASSED=0
FAILED=0

# 日志文件
LOG_FILE=".permission-validation.log"
touch "$LOG_FILE"
echo "$(date) - 开始权限验证测试" > "$LOG_FILE"

# 函数: 执行测试并检查结果
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_failure="$3"
    local expected_message="$4"
    
    echo -n "🔍 $test_name... "
    
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
                ((PASSED++))
            else
                echo -e "${YELLOW}警告${NC}"
                echo "⚠ $test_name: 失败，但错误信息不符合预期"
                echo "   实际输出: $output"
                echo "⚠ $test_name: 失败 - 错误信息不符合预期" >> "$LOG_FILE"
                ((FAILED++))
            fi
        else
            echo -e "${RED}失败${NC}"
            echo "✗ $test_name: 应该失败但成功了"
            echo "✗ $test_name: 失败 - 应该失败但成功了" >> "$LOG_FILE"
            ((FAILED++))
        fi
    else
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}通过${NC}"
            echo "✓ $test_name: 通过" >> "$LOG_FILE"
            ((PASSED++))
        else
            echo -e "${RED}失败${NC}"
            echo "✗ $test_name: 失败"
            echo "   错误: $output"
            echo "✗ $test_name: 失败 - $output" >> "$LOG_FILE"
            ((FAILED++))
        fi
    fi
    
    echo "" >> "$LOG_FILE"
}

# 准备测试环境
echo "📋 准备测试环境..."
git stash -u > /dev/null 2>&1 || true
git checkout main > /dev/null 2>&1 || true
git pull origin main > /dev/null 2>&1 || true
echo ""

# 测试 1: 绕过 main 分支保护
echo "=== 测试 1: 分支保护 ==="
run_test "绕过 main 分支保护" "git commit --allow-empty -m 'test bypass' && git push origin main" "true" "protected branch"

# 测试 2: force push 保护
run_test "force push 保护" "git commit --allow-empty -m 'test force push' && git push --force origin main" "true" "force push"

# 测试 3: 删除 tag 保护
echo "=== 测试 2: Tag 保护 ==="
git tag v1.0.0-test > /dev/null 2>&1 || true
git push origin v1.0.0-test > /dev/null 2>&1 || true
git tag -d v1.0.0-test > /dev/null 2>&1 || true
run_test "删除 tag 保护" "git push origin :refs/tags/v1.0.0-test" "true" "delete tag"

# 测试 4: 分支删除保护
echo "=== 测试 3: 其他保护 ==="
# 注意: 分支删除需要在GitHub UI上测试，这里只做提示
echo "⚠ 分支删除保护需要在GitHub UI上手动测试"
echo "   请尝试删除main分支，应该被拒绝"

# 清理测试环境
echo ""
echo "🧹 清理测试环境..."
git stash pop > /dev/null 2>&1 || true
git checkout - > /dev/null 2>&1 || true

# 生成测试报告
echo ""
echo "=== 测试报告 ==="
echo "通过: $PASSED"
echo "失败: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 所有测试通过！权限规则生效。${NC}"
    echo "$(date) - 所有测试通过" >> "$LOG_FILE"
else
    echo -e "${RED}❌ 有测试失败，需要检查权限配置。${NC}"
    echo "$(date) - 有测试失败: $FAILED" >> "$LOG_FILE"
    echo "详细日志请查看: $LOG_FILE"
fi

echo ""
echo "=== 验证完成 ==="
echo ""
echo "📋 手动测试项:"
echo "1. Owner账号绕过测试（使用Owner账号重复上述测试）"
echo "2. GitHub UI直接编辑测试（尝试直接编辑main文件）"
echo "3. PR审批绕过测试（不等待审批尝试merge）"
echo "4. 签名提交验证（创建未签名commit并push）"
echo ""
echo "🚨 重要提示:"
echo "- 确保 'Include administrators' 已开启"
echo "- 确保所有分支保护规则已正确配置"
echo "- 定期运行此脚本进行验证"

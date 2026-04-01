#!/bin/bash

# 统计Rust项目的实现代码和测试代码（简化版）

echo "=== SQLRustGo 代码统计分析 ==="
echo "日期: $(date)"
echo

# 统计核心实现代码（排除测试代码）
echo "=== 核心实现代码统计 ==="
core_impl=$(find . -name "*.rs" -path "*/src/*" -not -path "*/target/*" | xargs grep -v "#[cfg(test)]" | wc -l)
echo "核心实现代码: $core_impl 行"
echo

# 统计文件内测试代码
echo "=== 文件内测试代码统计 ==="
inline_tests=$(find . -name "*.rs" -path "*/src/*" -not -path "*/target/*" | xargs grep -A 1000 "#[cfg(test)]" | wc -l)
echo "文件内测试代码: $inline_tests 行"
echo

# 统计单独的测试文件
echo "=== 单独测试文件统计 ==="
separate_tests=$(find . -name "*.rs" -path "*/tests/*" -not -path "*/target/*" | xargs wc -l | tail -1 | awk '{print $1}')
if [ -z "$separate_tests" ]; then
    separate_tests=0
fi
echo "单独测试文件: $separate_tests 行"
echo

# 统计crates中的测试文件
echo "=== crates测试文件统计 ==="
crate_tests=$(find . -name "*test*.rs" -path "*/crates/*" -not -path "*/src/*" -not -path "*/target/*" | xargs wc -l | tail -1 | awk '{print $1}')
if [ -z "$crate_tests" ]; then
    crate_tests=0
fi
echo "crates测试文件: $crate_tests 行"
echo

# 计算总测试代码
total_tests=$((inline_tests + separate_tests + crate_tests))

# 计算测试覆盖率
if [ $core_impl -gt 0 ]; then
    coverage=$(echo "scale=2; $total_tests * 100 / $core_impl" | bc)
else
    coverage=0
fi

# 输出总统计
echo "=== 总统计 ==="
echo "核心实现代码: $core_impl 行"
echo "文件内测试代码: $inline_tests 行"
echo "单独测试文件: $separate_tests 行"
echo "crates测试文件: $crate_tests 行"
echo "总测试代码: $total_tests 行"
echo "测试覆盖率: ${coverage}%"
echo "总代码量: $((core_impl + total_tests)) 行"
echo

# 分析主要模块
echo "=== 主要模块分析 ==="
for module in crates/*; do
    if [ -d "$module/src" ]; then
        module_impl=$(find "$module/src" -name "*.rs" | xargs grep -v "#[cfg(test)]" | wc -l)
        module_test=$(find "$module/src" -name "*.rs" | xargs grep -A 1000 "#[cfg(test)]" | wc -l)
        
        # 查找该模块的单独测试文件
        module_sep_test=$(find "$module" -name "*test*.rs" -not -path "*/src/*" -not -path "*/target/*" | xargs wc -l | tail -1 | awk '{print $1}')
        if [ -z "$module_sep_test" ]; then
            module_sep_test=0
        fi
        
        module_total_test=$((module_test + module_sep_test))
        
        if [ $module_impl -gt 0 ]; then
            module_coverage=$(echo "scale=2; $module_total_test * 100 / $module_impl" | bc)
        else
            module_coverage=0
        fi
        
        echo "$module: 实现 $module_impl, 测试 $module_total_test, 覆盖率 ${module_coverage}%"
    fi
done

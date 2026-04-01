#!/bin/bash

# 统计Rust项目的实现代码和测试代码

echo "=== SQLRustGo 代码统计分析 ==="
echo "日期: $(date)"
echo

# 定义函数：统计文件中的实现代码和测试代码行数
count_code() {
    local file="$1"
    local impl_lines=0
    local test_lines=0
    local in_test=false
    
    while IFS= read -r line; do
        if [[ "$line" == *"#[cfg(test)]"* ]]; then
            in_test=true
            continue
        fi
        
        if $in_test; then
            test_lines=$((test_lines + 1))
        else
            impl_lines=$((impl_lines + 1))
        fi
    done < "$file"
    
    echo "$impl_lines $test_lines"
}

# 初始化统计变量
total_impl=0
total_test=0
total_separate_test=0

# 遍历所有src目录下的.rs文件
echo "=== 核心源码分析 ==="
find . -name "*.rs" -path "*/src/*" -not -path "*/target/*" | while read -r file; do
    read impl_lines test_lines <<< "$(count_code "$file")"
    total_impl=$((total_impl + impl_lines))
    total_test=$((total_test + test_lines))
    
    # 只显示有测试代码的文件
    if [ $test_lines -gt 0 ]; then
        echo "$file: 实现 $impl_lines, 测试 $test_lines"
    fi
done

# 统计单独的测试文件
echo "\n=== 单独测试文件分析 ==="
find . -name "*.rs" -path "*/tests/*" -not -path "*/target/*" | while read -r file; do
    test_lines=$(wc -l < "$file")
    total_separate_test=$((total_separate_test + test_lines))
    echo "$file: 测试 $test_lines"
done

# 统计crates中的test文件
echo "\n=== crates测试文件分析 ==="
find . -name "*test*.rs" -path "*/crates/*" -not -path "*/target/*" | while read -r file; do
    if [[ "$file" != *"src/*" ]]; then
        test_lines=$(wc -l < "$file")
        total_separate_test=$((total_separate_test + test_lines))
        echo "$file: 测试 $test_lines"
    fi
done

# 计算总测试代码
total_all_test=$((total_test + total_separate_test))

# 计算测试覆盖率
if [ $total_impl -gt 0 ]; then
    coverage=$(echo "scale=2; $total_all_test * 100 / $total_impl" | bc)
else
    coverage=0
fi

# 输出总统计
echo "\n=== 总统计 ==="
echo "核心实现代码: $total_impl 行"
echo "文件内测试代码: $total_test 行"
echo "单独测试文件: $total_separate_test 行"
echo "总测试代码: $total_all_test 行"
echo "测试覆盖率: ${coverage}%"
echo "总代码量: $((total_impl + total_all_test)) 行"
echo

# 分析模块分布
echo "=== 模块代码分布 ==="
for crate in crates/*; do
    if [ -d "$crate/src" ]; then
        crate_impl=0
        crate_test=0
        
        find "$crate/src" -name "*.rs" | while read -r file; do
            read impl_lines test_lines <<< "$(count_code "$file")"
            crate_impl=$((crate_impl + impl_lines))
            crate_test=$((crate_test + test_lines))
        done
        
        # 查找该crate的单独测试文件
        crate_separate_test=0
        find "$crate" -name "*test*.rs" -not -path "*/src/*" -not -path "*/target/*" | while read -r file; do
            test_lines=$(wc -l < "$file")
            crate_separate_test=$((crate_separate_test + test_lines))
        done
        
        crate_total_test=$((crate_test + crate_separate_test))
        
        if [ $crate_impl -gt 0 ]; then
            crate_coverage=$(echo "scale=2; $crate_total_test * 100 / $crate_impl" | bc)
        else
            crate_coverage=0
        fi
        
        echo "$crate: 实现 $crate_impl, 测试 $crate_total_test, 覆盖率 ${crate_coverage}%"
    fi
done

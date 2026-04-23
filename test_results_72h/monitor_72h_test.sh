#!/bin/bash
# 72小时测试监控脚本
# 用法: ./monitor_72h_test.sh

RESULTS_DIR="/home/ai/sqlrustgo/test_results_72h"

echo "=========================================="
echo "  72小时长稳测试监控"
echo "=========================================="
echo ""

# 检查测试进程
echo "【进程状态】"
if pgrep -f "long_run_stability_72h_test" > /dev/null; then
    ps -ef | grep long_run_stability_72h_test | grep -v grep | head -2
    echo "状态: 运行中 ✅"
else
    echo "状态: 未运行 ❌"
fi

echo ""

# 检查运行时间
echo "【运行时间】"
if pgrep -f "long_run_stability_72h_test" > /dev/null; then
    PID=$(pgrep -f "long_run_stability_72h_test" | head -1)
    ps -p $PID -o etime= 2>/dev/null || echo "无法获取运行时间"
else
    echo "进程不存在"
fi

echo ""

# 检查进度日志
echo "【最新进度】"
if [ -f "$RESULTS_DIR/72h_test_progress.log" ]; then
    tail -5 "$RESULTS_DIR/72h_test_progress.log"
else
    echo "进度日志不存在"
fi

echo ""

# 检查输出日志大小
echo "【输出日志】"
if [ -f "$RESULTS_DIR/72h_test_output.log" ]; then
    echo "文件大小: $(wc -c < "$RESULTS_DIR/72h_test_output.log") bytes"
    echo "行数: $(wc -l < "$RESULTS_DIR/72h_test_output.log") lines"
    echo "最后更新: $(stat -c %y "$RESULTS_DIR/72h_test_output.log" 2>/dev/null | cut -d' ' -f1,2 | cut -d'.' -f1)"
else
    echo "输出日志不存在"
fi

echo ""

# 预计完成时间
echo "【预计完成时间】"
START_TIME="2026-04-22 17:36:00"
END_TIME_72H="2026-05-01 17:36:00"
echo "开始时间: $START_TIME"
echo "72小时后: $END_TIME_72H"

echo ""
echo "=========================================="

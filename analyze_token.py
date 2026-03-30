#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

# 读取CSV文件
df = pd.read_csv('export_bill_1774840354.csv')

# 转换消费数为数值类型
df['输入消费数'] = pd.to_numeric(df['输入消费数'], errors='coerce').fillna(0).astype(int)
df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce').fillna(0).astype(int)
df['总消费数'] = pd.to_numeric(df['总消费数'], errors='coerce').fillna(0).astype(int)

# 解析消费时间，提取日期部分（格式：2026-03-29 23:00-24:00）
df['日期'] = df['消费时间'].str.split(' ').str[0]

# 转换为日期类型
df['日期'] = pd.to_datetime(df['日期']).dt.date

# 按日期汇总
daily_stats = df.groupby('日期').agg({
    '输入消费数': 'sum',
    '输出消费数': 'sum',
    '总消费数': 'sum'
}).reset_index()

# 调用次数（每条记录算一次）
call_counts = df.groupby('日期').size().reset_index(name='调用次数')
daily_stats = daily_stats.merge(call_counts, on='日期')

# 按日期排序
daily_stats = daily_stats.sort_values('日期')

print("=" * 80)
print("📊 Token 用量数据分析报告")
print("=" * 80)

total_days = len(daily_stats)
total_calls = int(daily_stats['调用次数'].sum())
total_input = int(daily_stats['输入消费数'].sum())
total_output = int(daily_stats['输出消费数'].sum())
total_consumption = int(daily_stats['总消费数'].sum())

print(f"\n📅 数据时间范围: {daily_stats['日期'].min()} 至 {daily_stats['日期'].max()}")
print(f"📅 数据总天数: {total_days} 天")
print(f"📋 总调用次数: {total_calls:,} 次")
print(f"📋 总输入消费: {total_input:,} Tokens")
print(f"📋 总输出消费: {total_output:,} Tokens")
print(f"📋 总消费: {total_consumption:,} Tokens")

print("\n" + "=" * 80)
print("📈 每日详细数据")
print("=" * 80)
print(f"{'日期':<12} {'调用次数':>10} {'输入消费':>18} {'输出消费':>15} {'总消费':>18}")
print("-" * 80)

for _, row in daily_stats.iterrows():
    print(f"{str(row['日期']):<12} {row['调用次数']:>10,} {row['输入消费数']:>18,} {row['输出消费数']:>15,} {row['总消费数']:>18,}")

print("\n" + "-" * 80)
total_row = daily_stats.sum(numeric_only=True)
total_calls = int(total_row['调用次数'])
total_input = int(total_row['输入消费数'])
total_output = int(total_row['输出消费数'])
total_consumption = int(total_row['总消费数'])
print(f"{'合计':<12} {total_calls:>10,} {total_input:>18,} {total_output:>15,} {total_consumption:>18,}")
# 计算统计指标
total_days = len(daily_stats)
total_calls = daily_stats['调用次数'].sum()
total_input = daily_stats['输入消费数'].sum()
total_output = daily_stats['输出消费数'].sum()
total_consumption = daily_stats['总消费数'].sum()

# 计算周数和月数
weeks = total_days / 7
months = total_days / 30

print("\n" + "=" * 80)
print("📊 统计指标")
print("=" * 80)
print(f"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                        Token 用量统计指标                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  指标           │    日均           │    周均           │    月均           │    总体           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  调用次数       │  {total_calls/total_days:>12,.0f}  │  {total_calls/weeks:>12,.0f}  │  {total_calls/months:>12,.0f}  │  {total_calls:>12,.0f}  ║
║  输入消费       │  {total_input/total_days:>12,.0f}  │  {total_input/weeks:>12,.0f}  │  {total_input/months:>12,.0f}  │  {total_input:>12,.0f}  ║
║  输出消费       │  {total_output/total_days:>12,.0f}  │  {total_output/weeks:>12,.0f}  │  {total_output/months:>12,.0f}  │  {total_output:>12,.0f}  ║
║  总消费         │  {total_consumption/total_days:>12,.0f}  │  {total_consumption/weeks:>12,.0f}  │  {total_consumption/months:>12,.0f}  │  {total_consumption:>12,.0f}  ║
╚══════════════════════════════════════════════════════════════════════════════╝
""")

# 绘制4个子图（使用英文标签避免字体问题）
fig, axes = plt.subplots(2, 2, figsize=(14, 10))
fig.suptitle('Token Usage Trend Analysis (2026-03-25 to 2026-03-30)', fontsize=14, fontweight='bold')

dates = pd.to_datetime(daily_stats['日期'])

# 子图1：调用次数趋势（蓝色）
axes[0, 0].plot(dates, daily_stats['调用次数'], 'b-o', linewidth=2, markersize=6)
axes[0, 0].set_title('API Call Trend', fontsize=12, color='blue')
axes[0, 0].set_xlabel('Date')
axes[0, 0].set_ylabel('Calls')
axes[0, 0].grid(True, alpha=0.3)
axes[0, 0].tick_params(axis='x', rotation=45)

# 子图2：输入消费趋势（绿色）
axes[0, 1].plot(dates, daily_stats['输入消费数'], 'g-o', linewidth=2, markersize=6)
axes[0, 1].set_title('Input Consumption Trend', fontsize=12, color='green')
axes[0, 1].set_xlabel('Date')
axes[0, 1].set_ylabel('Input Tokens')
axes[0, 1].grid(True, alpha=0.3)
axes[0, 1].tick_params(axis='x', rotation=45)

# 子图3：输出消费趋势（橙色）
axes[1, 0].plot(dates, daily_stats['输出消费数'], 'orange', marker='o', linewidth=2, markersize=6)
axes[1, 0].set_title('Output Consumption Trend', fontsize=12, color='orange')
axes[1, 0].set_xlabel('Date')
axes[1, 0].set_ylabel('Output Tokens')
axes[1, 0].grid(True, alpha=0.3)
axes[1, 0].tick_params(axis='x', rotation=45)

# 子图4：总消费趋势（红色）
axes[1, 1].plot(dates, daily_stats['总消费数'], 'r-o', linewidth=2, markersize=6)
axes[1, 1].set_title('Total Consumption Trend', fontsize=12, color='red')
axes[1, 1].set_xlabel('Date')
axes[1, 1].set_ylabel('Total Tokens')
axes[1, 1].grid(True, alpha=0.3)
axes[1, 1].tick_params(axis='x', rotation=45)

plt.tight_layout()
plt.savefig('token_usage_analysis.png', dpi=150, bbox_inches='tight')
print("\n📊 图表已保存至: token_usage_analysis.png")

# 输出每日数据表格
print("\n" + "=" * 80)
print("📋 每日数据明细")
print("=" * 80)
print(daily_stats.to_string(index=False))

# 筛选3/25之后的数据
print("\n" + "=" * 80)
print("📅 3/25至今每日数据")
print("=" * 80)

from datetime import date
cutoff_date = date(2026, 3, 25)
recent_stats = daily_stats[daily_stats['日期'] >= cutoff_date].copy()

print(f"{'日期':<12} {'调用次数':>10} {'输入消费':>18} {'输出消费':>15} {'总消费':>18}")
print("-" * 80)

recent_total_calls = 0
recent_total_input = 0
recent_total_output = 0
recent_total_consumption = 0

for _, row in recent_stats.iterrows():
    print(f"{str(row['日期']):<12} {row['调用次数']:>10,} {row['输入消费数']:>18,} {row['输出消费数']:>15,} {row['总消费数']:>18,}")
    recent_total_calls += row['调用次数']
    recent_total_input += row['输入消费数']
    recent_total_output += row['输出消费数']
    recent_total_consumption += row['总消费数']

print("-" * 80)
print(f"{'合计':<12} {recent_total_calls:>10,} {recent_total_input:>18,} {recent_total_output:>15,} {recent_total_consumption:>18,}")

recent_days = len(recent_stats)
print(f"\n3/25至今: {recent_days}天")
print(f"  - 日均调用: {recent_total_calls/recent_days:.0f}次")
print(f"  - 日均输入: {recent_total_input/recent_days:,.0f} Tokens")
print(f"  - 日均输出: {recent_total_output/recent_days:,.0f} Tokens")
print(f"  - 日均总消费: {recent_total_consumption/recent_days:,.0f} Tokens")

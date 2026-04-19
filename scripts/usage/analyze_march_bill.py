#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

# 读取CSV文件
df = pd.read_csv('/Users/liying/Downloads/export_bill_march.csv')

# 解析日期
df['日期'] = df['消费时间'].str.split(' ').str[0]
df['日期'] = pd.to_datetime(df['日期']).dt.date

# 转换消费数为数值类型
df['输入消费数'] = pd.to_numeric(df['输入消费数'], errors='coerce').fillna(0).astype(int)
df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce').fillna(0).astype(int)
df['总消费数'] = pd.to_numeric(df['总消费数'], errors='coerce').fillna(0).astype(int)

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

# 计算统计指标
total_days = len(daily_stats)
total_calls = daily_stats['调用次数'].sum()
total_output = daily_stats['输出消费数'].sum()
total_consumption = daily_stats['总消费数'].sum()

# 计算周数和月数
weeks = total_days / 7
months = total_days / 30

# 计算日均、周均、月均
daily_avg_calls = total_calls / total_days
daily_avg_output = total_output / total_days
daily_avg_consumption = total_consumption / total_days

weekly_avg_calls = total_calls / weeks
weekly_avg_output = total_output / weeks
weekly_avg_consumption = total_consumption / weeks

monthly_avg_calls = total_calls / months
monthly_avg_output = total_output / months
monthly_avg_consumption = total_consumption / months

print("=" * 80)
print("📊 3月 Token 用量分析报告")
print("=" * 80)
print(f"\n📅 数据时间范围: {daily_stats['日期'].min()} 至 {daily_stats['日期'].max()}")
print(f"📅 数据总天数: {total_days} 天")

print("\n" + "=" * 80)
print("📊 统计指标")
print("=" * 80)
print(f"""
╔══════════════════════════════════════════════════════════╗
║                       统计指标                          ║
╠══════════════╦══════════╦══════════╦══════════╦══════════╣
║  指标        ║  日均    ║  周均    ║  月均    ║  总体    ║
╠══════════════╬══════════╬══════════╬══════════╬══════════╣
║  调用次数    ║  {daily_avg_calls:>8.0f}  ║  {weekly_avg_calls:>8.0f}  ║  {monthly_avg_calls:>8.0f}  ║  {total_calls:>8,}  ║
║  输出消费    ║  {daily_avg_output:>8,}  ║  {weekly_avg_output:>8,}  ║  {monthly_avg_output:>8,}  ║  {total_output:>8,}  ║
║  总消费数    ║  {daily_avg_consumption:>8,}  ║  {weekly_avg_consumption:>8,}  ║  {monthly_avg_consumption:>8,}  ║  {total_consumption:>8,}  ║
╚══════════════╩══════════╩══════════╩══════════╩══════════╝
""")

# 筛选3月25-31日数据
print("\n" + "=" * 80)
print("📅 3月25-31日详细数据")
print("=" * 80)

from datetime import date
start_date = date(2026, 3, 25)
end_date = date(2026, 3, 31)

march_data = daily_stats[(daily_stats['日期'] >= start_date) & (daily_stats['日期'] <= end_date)].copy()

print(f"{'日期':<12} {'调用次数':>10} {'输出消费':>15} {'总消费数':>20}")
print("-" * 80)

for _, row in march_data.iterrows():
    print(f"{str(row['日期']):<12} {row['调用次数']:>10,} {row['输出消费数']:>15,} {row['总消费数']:>20,}")

print("-" * 80)
march_total_calls = march_data['调用次数'].sum()
march_total_output = march_data['输出消费数'].sum()
march_total_consumption = march_data['总消费数'].sum()
print(f"{'合计':<12} {march_total_calls:>10,} {march_total_output:>15,} {march_total_consumption:>20,}")

# 绘制3月25-31日总消费数折线图
print("\n" + "=" * 80)
print("📊 3月25-31日总消费数趋势")
print("=" * 80)

plt.figure(figsize=(12, 6))
dates = pd.to_datetime(march_data['日期'])
plt.plot(dates, march_data['总消费数'], 'r-o', linewidth=2, markersize=6)
plt.title('3月25-31日总消费数趋势', fontsize=14, fontweight='bold')
plt.xlabel('日期')
plt.ylabel('总消费数 (Tokens)')
plt.grid(True, alpha=0.3)
plt.xticks(rotation=45)
plt.tight_layout()
plt.savefig('march_consumption_trend.png', dpi=150, bbox_inches='tight')
print("\n📊 图表已保存至: march_consumption_trend.png")

# 输出完整的每日数据
print("\n" + "=" * 80)
print("📋 每日详细数据")
print("=" * 80)
print(daily_stats.to_string(index=False))

#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime
import numpy as np
import os

plt.rcParams['font.sans-serif'] = ['Arial Unicode MS', 'SimHei', 'PingFang SC']
plt.rcParams['axes.unicode_minus'] = False

CSV_FILE = '/tmp/bill.csv'
OUTPUT_DIR = os.path.dirname(os.path.abspath(__file__))

df = pd.read_csv(CSV_FILE, encoding='utf-8')
print(f"Loaded {len(df)} records")

df['date'] = df['消费时间'].apply(lambda x: x.split(' ')[0])
df['date'] = pd.to_datetime(df['date'])
df['输入消费数'] = pd.to_numeric(df['输入消费数'], errors='coerce').fillna(0)
df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce').fillna(0)
df['总消费数'] = pd.to_numeric(df['总消费数'], errors='coerce').fillna(0)

daily = df.groupby('date').agg({
    '输入消费数': 'sum',
    '输出消费数': 'sum',
    '总消费数': 'sum',
    '消费时间': 'count'
}).rename(columns={'消费时间': '调用次数'}).reset_index()
daily = daily.sort_values('date')

# Convert to yi (hundred millions)
daily['总消费_亿'] = daily['总消费数'] / 1e8
daily['输入_亿'] = daily['输入消费数'] / 1e8
daily['输出_万'] = daily['输出消费数'] / 1e4

# ============================================================
# Chart 1: Overall Token Usage Trend (minimax_usage_trend.png)
# ============================================================
fig, axes = plt.subplots(3, 1, figsize=(14, 14))

# 1a: Daily total tokens (yi)
ax1 = axes[0]
ax1.fill_between(daily['date'], daily['总消费_亿'], alpha=0.3, color='#3498db')
ax1.plot(daily['date'], daily['总消费_亿'], color='#2980b9', linewidth=2, marker='o', markersize=3)
ax1.set_title('每日总 Token 用量（亿）', fontsize=16, fontweight='bold')
ax1.set_ylabel('总消费数（亿 Token）', fontsize=12)
ax1.grid(True, linestyle='--', alpha=0.5)
ax1.xaxis.set_major_formatter(mdates.DateFormatter('%m/%d'))
ax1.xaxis.set_major_locator(mdates.WeekdayLocator(byweekday=0))
# Add monthly average line
monthly_avg = daily.set_index('date').resample('ME')['总消费_亿'].mean()
for m_date, m_val in monthly_avg.items():
    m_start = m_date.replace(day=1)
    ax1.axhline(y=m_val, xmin=0, xmax=1, color='red', linestyle=':', alpha=0.5, linewidth=1)

# 1b: Daily calls
ax2 = axes[1]
ax2.bar(daily['date'], daily['调用次数'], color='#27ae60', alpha=0.7, width=0.8)
ax2.set_title('每日调用次数', fontsize=16, fontweight='bold')
ax2.set_ylabel('调用次数', fontsize=12)
ax2.grid(True, linestyle='--', alpha=0.5)
ax2.xaxis.set_major_formatter(mdates.DateFormatter('%m/%d'))
ax2.xaxis.set_major_locator(mdates.WeekdayLocator(byweekday=0))

# 1c: Input vs Output
ax3 = axes[2]
ax3.plot(daily['date'], daily['输入_亿'], color='#e74c3c', linewidth=2, label='输入消费（亿）', marker='o', markersize=3)
ax3_twin = ax3.twinx()
ax3_twin.plot(daily['date'], daily['输出_万'], color='#f39c12', linewidth=2, label='输出消费（万）', marker='s', markersize=3)
ax3.set_title('每日输入/输出消费趋势', fontsize=16, fontweight='bold')
ax3.set_ylabel('输入消费（亿 Token）', fontsize=12, color='#e74c3c')
ax3_twin.set_ylabel('输出消费（万 Token）', fontsize=12, color='#f39c12')
ax3.grid(True, linestyle='--', alpha=0.5)
ax3.xaxis.set_major_formatter(mdates.DateFormatter('%m/%d'))
ax3.xaxis.set_major_locator(mdates.WeekdayLocator(byweekday=0))
lines1, labels1 = ax3.get_legend_handles_labels()
lines2, labels2 = ax3_twin.get_legend_handles_labels()
ax3.legend(lines1 + lines2, labels1 + labels2, loc='upper left')

fig.autofmt_xdate()
plt.tight_layout()
trend_path = os.path.join(OUTPUT_DIR, 'minimax_usage_trend.png')
plt.savefig(trend_path, dpi=150, bbox_inches='tight')
print(f"Saved: {trend_path}")
plt.close()

# ============================================================
# Chart 2: Last 2 Weeks Detail (minimax_2weeks_detail.png)
# ============================================================
recent_14 = daily.tail(14).copy()

fig, axes = plt.subplots(2, 1, figsize=(14, 10))

# 2a: Stacked bar chart - input vs output
ax1 = axes[0]
bar_width = 0.6
dates_str = recent_14['date'].dt.strftime('%m/%d')
bars1 = ax1.bar(dates_str, recent_14['输入_亿'], bar_width, label='输入消费（亿）', color='#3498db', alpha=0.8)
bars2 = ax1.bar(dates_str, recent_14['输出_万'] / 100, bar_width, bottom=recent_14['输入_亿'], label='输出消费（百万）', color='#e67e22', alpha=0.8)
ax1.set_title('最近2周每日用量明细', fontsize=16, fontweight='bold')
ax1.set_ylabel('Token 用量', fontsize=12)
ax1.legend(loc='upper left')
ax1.grid(True, linestyle='--', alpha=0.3, axis='y')
# Add value labels on top
for i, (idx, row) in enumerate(recent_14.iterrows()):
    ax1.annotate(f"{row['总消费_亿']:.1f}亿", (i, row['总消费_亿']),
                textcoords="offset points", xytext=(0, 5), ha='center', fontsize=9)

# 2b: Calls + 7-day moving average
ax2 = axes[1]
ax2.bar(dates_str, recent_14['调用次数'], color='#27ae60', alpha=0.7, width=0.6, label='调用次数')
# 7-day MA
if len(daily) >= 7:
    daily_ma = daily.copy()
    daily_ma['MA7'] = daily_ma['调用次数'].rolling(7).mean()
    ma_recent = daily_ma.tail(14)
    ax2.plot(dates_str, ma_recent['MA7'].values, color='#e74c3c', linewidth=2, marker='D', markersize=5, label='7日移动平均')
ax2.set_title('最近2周调用次数', fontsize=16, fontweight='bold')
ax2.set_ylabel('调用次数', fontsize=12)
ax2.legend(loc='upper left')
ax2.grid(True, linestyle='--', alpha=0.3, axis='y')

fig.autofmt_xdate()
plt.tight_layout()
detail_path = os.path.join(OUTPUT_DIR, 'minimax_2weeks_detail.png')
plt.savefig(detail_path, dpi=150, bbox_inches='tight')
print(f"Saved: {detail_path}")
plt.close()

# ============================================================
# Chart 3: Monthly Comparison (NEW)
# ============================================================
monthly = daily.copy()
monthly['month'] = monthly['date'].dt.to_period('M')
monthly_group = monthly.groupby('month').agg({
    '总消费_亿': 'sum',
    '调用次数': 'sum',
    '输出_万': 'sum'
}).reset_index()
monthly_group['month_str'] = monthly_group['month'].astype(str)

fig, axes = plt.subplots(1, 2, figsize=(14, 6))

# 3a: Monthly total tokens
ax1 = axes[0]
colors = ['#3498db', '#2ecc71', '#e74c3c', '#f39c12']
bars = ax1.bar(monthly_group['month_str'], monthly_group['总消费_亿'], color=colors[:len(monthly_group)], alpha=0.8)
ax1.set_title('月度总 Token 用量（亿）', fontsize=16, fontweight='bold')
ax1.set_ylabel('总消费数（亿 Token）', fontsize=12)
ax1.grid(True, linestyle='--', alpha=0.3, axis='y')
for bar, val in zip(bars, monthly_group['总消费_亿']):
    ax1.text(bar.get_x() + bar.get_width()/2., bar.get_height() + 0.5,
             f'{val:.1f}亿', ha='center', va='bottom', fontsize=12, fontweight='bold')

# 3b: Monthly calls
ax2 = axes[1]
bars2 = ax2.bar(monthly_group['month_str'], monthly_group['调用次数'], color=colors[:len(monthly_group)], alpha=0.8)
ax2.set_title('月度调用次数', fontsize=16, fontweight='bold')
ax2.set_ylabel('调用次数', fontsize=12)
ax2.grid(True, linestyle='--', alpha=0.3, axis='y')
for bar, val in zip(bars2, monthly_group['调用次数']):
    ax2.text(bar.get_x() + bar.get_width()/2., bar.get_height() + 10,
             f'{val:,}', ha='center', va='bottom', fontsize=12, fontweight='bold')

plt.tight_layout()
monthly_path = os.path.join(OUTPUT_DIR, 'minimax_monthly_comparison.png')
plt.savefig(monthly_path, dpi=150, bbox_inches='tight')
print(f"Saved: {monthly_path}")
plt.close()

print("\nAll charts generated successfully!")

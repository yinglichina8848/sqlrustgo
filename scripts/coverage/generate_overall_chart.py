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

# 绘制4个子图
fig, axes = plt.subplots(2, 2, figsize=(14, 10))
fig.suptitle('Token 用量趋势分析 (2026-02-02 至 2026-03-31)', fontsize=14, fontweight='bold')

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

plt.tight_layout(rect=[0, 0, 1, 0.95])  # 调整布局，留出标题空间
plt.savefig('overall_token_usage_trend.png', dpi=150, bbox_inches='tight')
print("📊 总体折线图已保存至: overall_token_usage_trend.png")

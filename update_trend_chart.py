import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime, timedelta

CSV_FILE = 'artifacts/usage/export_2026_04_24.csv'

df = pd.read_csv(CSV_FILE)

df['输入消费数'] = pd.to_numeric(df['输入消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)
df['输出消费数'] = pd.to_numeric(df['输出消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)
df['总消费数'] = pd.to_numeric(df['总消费数'].replace('<nil>', '0'), errors='coerce').fillna(0).astype(int)

def parse_date_time(time_str):
    date_part = time_str.split(' ')[0]
    return datetime.strptime(date_part, '%Y-%m-%d')

df['日期'] = df['消费时间'].apply(parse_date_time)

today = datetime(2026, 4, 24)
two_weeks_ago = today - timedelta(days=14)

daily_summary = df.groupby('日期').agg({
    '输入消费数': 'sum',
    '总消费数': 'sum',
    '输出消费数': 'sum',
    '消费时间': 'count'
}).rename(columns={'消费时间': '调用次数'}).sort_index()

plt.rcParams['font.sans-serif'] = ['Arial Unicode MS', 'SimHei', 'PingFang SC']
plt.rcParams['axes.unicode_minus'] = False

fig, axes = plt.subplots(4, 1, figsize=(14, 16))

axes[0].plot(daily_summary.index, daily_summary['调用次数'], marker='o', linestyle='-', color='#2196F3', linewidth=1.5, markersize=3)
axes[0].axvspan(two_weeks_ago, today, alpha=0.15, color='orange', label='最近2周')
axes[0].set_title('每日调用次数', fontsize=13, fontweight='bold')
axes[0].set_ylabel('调用次数')
axes[0].grid(True, alpha=0.3)
axes[0].legend()
axes[0].xaxis.set_major_formatter(mdates.DateFormatter('%m-%d'))

axes[1].plot(daily_summary.index, daily_summary['输入消费数']/1e8, marker='o', linestyle='-', color='#FF9800', linewidth=1.5, markersize=3)
axes[1].axvspan(two_weeks_ago, today, alpha=0.15, color='orange', label='最近2周')
axes[1].set_title('每日输入消费（亿 Token）', fontsize=13, fontweight='bold')
axes[1].set_ylabel('输入消费（亿）')
axes[1].grid(True, alpha=0.3)
axes[1].legend()
axes[1].xaxis.set_major_formatter(mdates.DateFormatter('%m-%d'))

axes[2].plot(daily_summary.index, daily_summary['输出消费数']/1e4, marker='o', linestyle='-', color='#4CAF50', linewidth=1.5, markersize=3)
axes[2].axvspan(two_weeks_ago, today, alpha=0.15, color='orange', label='最近2周')
axes[2].set_title('每日输出消费（万 Token）', fontsize=13, fontweight='bold')
axes[2].set_ylabel('输出消费（万）')
axes[2].grid(True, alpha=0.3)
axes[2].legend()
axes[2].xaxis.set_major_formatter(mdates.DateFormatter('%m-%d'))

axes[3].plot(daily_summary.index, daily_summary['总消费数']/1e8, marker='o', linestyle='-', color='#F44336', linewidth=1.5, markersize=3)
axes[3].axvspan(two_weeks_ago, today, alpha=0.15, color='orange', label='最近2周')
axes[3].set_title('每日总消费数（亿 Token）', fontsize=13, fontweight='bold')
axes[3].set_ylabel('总消费数（亿）')
axes[3].set_xlabel('日期')
axes[3].grid(True, alpha=0.3)
axes[3].legend()
axes[3].xaxis.set_major_formatter(mdates.DateFormatter('%m-%d'))

fig.autofmt_xdate()
plt.tight_layout()
plt.savefig('scripts/usage/minimax_usage_trend.png', dpi=150, bbox_inches='tight')
print("趋势图已更新: scripts/usage/minimax_usage_trend.png")

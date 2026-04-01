import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

# 读取CSV文件
df = pd.read_csv('export_bill_1774668153.csv')

# 转换数据类型
df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce')
df['输出消费数'] = df['输出消费数'].fillna(0)
df['总消费数'] = pd.to_numeric(df['总消费数'], errors='coerce')
df['总消费数'] = df['总消费数'].fillna(0)

# 解析日期
def parse_date_time(time_str):
    date_part = time_str.split(' ')[0]
    return datetime.strptime(date_part, '%Y-%m-%d')

df['日期'] = df['消费时间'].apply(parse_date_time)

# 按天汇总
daily_summary = df.groupby('日期').agg({
    '总消费数': 'sum',
    '输出消费数': 'sum',
    '消费时间': 'count'  # 调用次数
}).rename(columns={'消费时间': '调用次数'})

# 生成折线图
plt.figure(figsize=(14, 8))

# 子图1：每日调用次数
plt.subplot(3, 1, 1)
plt.plot(daily_summary.index, daily_summary['调用次数'], marker='o', linestyle='-', color='b')
plt.title('每日调用次数')
plt.xlabel('日期')
plt.ylabel('调用次数')
plt.grid(True)

# 子图2：每日总消费数
plt.subplot(3, 1, 2)
plt.plot(daily_summary.index, daily_summary['总消费数'], marker='o', linestyle='-', color='r')
plt.title('每日总消费数')
plt.xlabel('日期')
plt.ylabel('总消费数')
plt.grid(True)

# 子图3：每日输出消费数
plt.subplot(3, 1, 3)
plt.plot(daily_summary.index, daily_summary['输出消费数'], marker='o', linestyle='-', color='g')
plt.title('每日输出消费数')
plt.xlabel('日期')
plt.ylabel('输出消费数')
plt.grid(True)

plt.tight_layout()
plt.savefig('minimax_usage_trend.png')
print("折线图已更新为 minimax_usage_trend.png，包含输出消费数")

import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime
import numpy as np

# 读取CSV文件
df = pd.read_csv('export_bill_1774668153.csv')

# 解析日期
def parse_date_time(time_str):
    # 提取日期部分，格式为 '2026-03-27 23:00-24:00'
    date_part = time_str.split(' ')[0]
    return datetime.strptime(date_part, '%Y-%m-%d')

df['日期'] = df['消费时间'].apply(parse_date_time)
df['日期_str'] = df['日期'].dt.strftime('%Y-%m-%d')

# 按天汇总
daily_summary = df.groupby('日期').agg({
    '总消费数': 'sum',
    '消费时间': 'count'  # 调用次数
}).rename(columns={'消费时间': '调用次数'})

# 过滤2月1日到3月底的数据
start_date = datetime(2026, 2, 1)
end_date = datetime(2026, 3, 31)
filtered_df = df[(df['日期'] >= start_date) & (df['日期'] <= end_date)]

# 按周汇总
weekly_summary = filtered_df.groupby(pd.Grouper(key='日期', freq='W-MON')).agg({
    '总消费数': 'sum',
    '消费时间': 'count'
}).rename(columns={'消费时间': '调用次数'})

# 按月汇总
monthly_summary = filtered_df.groupby(pd.Grouper(key='日期', freq='ME')).agg({
    '总消费数': 'sum',
    '消费时间': 'count'
}).rename(columns={'消费时间': '调用次数'})

# 打印结果
print("=== 每日汇总 ===")
print(daily_summary)
print("\n=== 2月1日到3月底每周汇总 ===")
print(weekly_summary)
print("\n=== 2月1日到3月底每月汇总 ===")
print(monthly_summary)

# 生成折线图
plt.figure(figsize=(12, 6))

# 折线图：每日调用次数
plt.subplot(2, 1, 1)
plt.plot(daily_summary.index, daily_summary['调用次数'], marker='o', linestyle='-', color='b')
plt.title('每日调用次数')
plt.xlabel('日期')
plt.ylabel('调用次数')
plt.grid(True)

# 折线图：每日总消费数
plt.subplot(2, 1, 2)
plt.plot(daily_summary.index, daily_summary['总消费数'], marker='o', linestyle='-', color='r')
plt.title('每日总消费数')
plt.xlabel('日期')
plt.ylabel('总消费数')
plt.grid(True)

plt.tight_layout()
plt.savefig('minimax_usage_trend.png')
print("\n折线图已保存为 minimax_usage_trend.png")

# 生成汇总数据的Excel文件
excel_writer = pd.ExcelWriter('minimax_usage_summary.xlsx')
daily_summary.to_excel(excel_writer, sheet_name='每日汇总')
weekly_summary.to_excel(excel_writer, sheet_name='每周汇总')
monthly_summary.to_excel(excel_writer, sheet_name='每月汇总')
excel_writer.close()
print("汇总数据已保存为 minimax_usage_summary.xlsx")

import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

# 读取CSV文件
df = pd.read_csv('export_bill_1774668153.csv')

# 转换输出消费数为数值类型
df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce')
df['输出消费数'] = df['输出消费数'].fillna(0)

# 解析日期
def parse_date_time(time_str):
    date_part = time_str.split(' ')[0]
    return datetime.strptime(date_part, '%Y-%m-%d')

df['日期'] = df['消费时间'].apply(parse_date_time)

# 按天汇总输出消费数
daily_output = df.groupby('日期').agg({
    '输出消费数': 'sum'
})

# 过滤2月1日到3月底的数据
start_date = datetime(2026, 2, 1)
end_date = datetime(2026, 3, 31)
filtered_df = df[(df['日期'] >= start_date) & (df['日期'] <= end_date)]

# 按天汇总过滤后的数据
daily_output_filtered = filtered_df.groupby('日期').agg({
    '输出消费数': 'sum'
})

# 按周汇总
weekly_output = filtered_df.groupby(pd.Grouper(key='日期', freq='W-MON')).agg({
    '输出消费数': 'sum'
})

# 按月汇总
monthly_output = filtered_df.groupby(pd.Grouper(key='日期', freq='ME')).agg({
    '输出消费数': 'sum'
})

# 计算平均值和总体
daily_avg = daily_output_filtered['输出消费数'].mean()
weekly_avg = weekly_output['输出消费数'].mean()
monthly_avg = monthly_output['输出消费数'].mean()
total = filtered_df['输出消费数'].sum()

# 打印结果
print("=== 输出消费数统计 ===")
print(f"日均输出消费数: {daily_avg:,.0f} Token")
print(f"周均输出消费数: {weekly_avg:,.0f} Token")
print(f"月均输出消费数: {monthly_avg:,.0f} Token")
print(f"总体输出消费数: {total:,.0f} Token")

# 生成折线图
plt.figure(figsize=(12, 6))
plt.plot(daily_output.index, daily_output['输出消费数'], marker='o', linestyle='-', color='green')
plt.title('每日输出消费数')
plt.xlabel('日期')
plt.ylabel('输出消费数')
plt.grid(True)
plt.tight_layout()
plt.savefig('output_usage_trend.png')
print("\n折线图已保存为 output_usage_trend.png")

# 生成汇总数据的Excel文件
excel_writer = pd.ExcelWriter('output_usage_summary.xlsx')
daily_output.to_excel(excel_writer, sheet_name='每日汇总')
weekly_output.to_excel(excel_writer, sheet_name='每周汇总')
monthly_output.to_excel(excel_writer, sheet_name='每月汇总')
excel_writer.close()
print("汇总数据已保存为 output_usage_summary.xlsx")

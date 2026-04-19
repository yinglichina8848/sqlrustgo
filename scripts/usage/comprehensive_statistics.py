import pandas as pd
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

# 过滤2月1日到3月底的数据
start_date = datetime(2026, 2, 1)
end_date = datetime(2026, 3, 31)
filtered_df = df[(df['日期'] >= start_date) & (df['日期'] <= end_date)]

# 按天汇总
daily_summary = filtered_df.groupby('日期').agg({
    '消费时间': 'count',  # 调用次数
    '输出消费数': 'sum',
    '总消费数': 'sum'
}).rename(columns={'消费时间': '调用次数'})

# 按周汇总
weekly_summary = filtered_df.groupby(pd.Grouper(key='日期', freq='W-MON')).agg({
    '消费时间': 'count',
    '输出消费数': 'sum',
    '总消费数': 'sum'
}).rename(columns={'消费时间': '调用次数'})

# 按月汇总
monthly_summary = filtered_df.groupby(pd.Grouper(key='日期', freq='ME')).agg({
    '消费时间': 'count',
    '输出消费数': 'sum',
    '总消费数': 'sum'
}).rename(columns={'消费时间': '调用次数'})

# 计算平均值和总体
daily_avg = daily_summary.mean()
weekly_avg = weekly_summary.mean()
monthly_avg = monthly_summary.mean()
total = filtered_df.agg({
    '消费时间': 'count',
    '输出消费数': 'sum',
    '总消费数': 'sum'
}).rename({'消费时间': '调用次数'})

# 格式化输出
print("### 平均值统计")
print("| 统计项 | 日均 | 周均 | 月均 | 总体 |")
print("|-------|------|------|------|------|")

# 调用次数
print(f"| 调用次数 | {daily_avg['调用次数']:.0f} | {weekly_avg['调用次数']:.0f} | {monthly_avg['调用次数']:.0f} | {total['调用次数']:.0f} |")

# 输出消费数
print(f"| 输出消费数 | {daily_avg['输出消费数']:,.0f} Token | {weekly_avg['输出消费数']:,.0f} Token | {monthly_avg['输出消费数']:,.0f} Token | {total['输出消费数']:,.0f} Token |")

# 总消费数
print(f"| 总消费数 | {daily_avg['总消费数']:,.0f} Token | {weekly_avg['总消费数']:,.0f} Token | {monthly_avg['总消费数']:,.0f} Token | {total['总消费数']:,.0f} Token |")

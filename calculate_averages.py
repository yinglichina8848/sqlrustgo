import pandas as pd
from datetime import datetime

# 读取CSV文件
df = pd.read_csv('export_bill_1774668153.csv')

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
    '总消费数': 'sum'
})

# 按周汇总
weekly_summary = filtered_df.groupby(pd.Grouper(key='日期', freq='W-MON')).agg({
    '总消费数': 'sum'
})

# 计算平均值
daily_avg = daily_summary['总消费数'].mean()
weekly_avg = weekly_summary['总消费数'].mean()

print(f"平均每天消费数: {daily_avg:,.0f} Token")
print(f"平均每周消费数: {weekly_avg:,.0f} Token")

# 检查是否在1亿左右
if 90000000 <= daily_avg <= 110000000:
    print("平均每天消费数在1亿左右")
else:
    print("平均每天消费数不在1亿左右")

if 630000000 <= weekly_avg <= 770000000:
    print("平均每周消费数在7亿左右")
else:
    print("平均每周消费数不在7亿左右")

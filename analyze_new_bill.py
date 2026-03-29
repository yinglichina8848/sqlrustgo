#!/usr/bin/env python3
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
from datetime import datetime
import os

# 设置中文字体
plt.rcParams['font.sans-serif'] = ['SimHei']  # 用来正常显示中文标签
plt.rcParams['axes.unicode_minus'] = False  # 用来正常显示负号

# CSV文件路径
csv_file = "export_bill_1774760227.csv"

# 读取CSV文件
def read_csv():
    try:
        df = pd.read_csv(csv_file, encoding='utf-8')
        print(f"成功读取文件，共 {len(df)} 条记录")
        return df
    except Exception as e:
        print(f"读取文件时出错: {e}")
        return None

# 数据预处理
def preprocess_data(df):
    # 提取日期
    df['日期'] = df['消费时间'].apply(lambda x: x.split(' ')[0])
    
    # 转换数据类型
    df['输入消费数'] = pd.to_numeric(df['输入消费数'], errors='coerce')
    df['输出消费数'] = pd.to_numeric(df['输出消费数'], errors='coerce')
    df['总消费数'] = pd.to_numeric(df['总消费数'], errors='coerce')
    
    # 按日期分组
    daily_data = df.groupby('日期').agg({
        '输入消费数': 'sum',
        '输出消费数': 'sum',
        '总消费数': 'sum'
    }).reset_index()
    
    # 转换日期格式
    daily_data['日期'] = pd.to_datetime(daily_data['日期'])
    daily_data = daily_data.sort_values('日期')
    
    return daily_data

# 计算统计数据
def calculate_statistics(daily_data):
    # 日均值
    daily_avg = daily_data.mean()
    
    # 周均值
    weekly_data = daily_data.resample('W-MON', on='日期').sum()
    weekly_avg = weekly_data.mean()
    
    # 月均值
    monthly_data = daily_data.resample('ME', on='日期').sum()
    monthly_avg = monthly_data.mean()
    
    # 总体
    total = daily_data[['输入消费数', '输出消费数', '总消费数']].sum()
    
    return daily_avg, weekly_avg, monthly_avg, total

# 生成表格
def generate_table(daily_avg, weekly_avg, monthly_avg, total):
    print("### 统计表格")
    print("| 指标 | 日均 | 周均 | 月均 | 总体 |")
    print("|------|------|------|------|------|")
    print(f"| 调用次数 | {int(daily_avg['输入消费数']):,} | {int(weekly_avg['输入消费数']):,} | {int(monthly_avg['输入消费数']):,} | {int(total['输入消费数']):,} |")
    print(f"| 输出消费 | {int(daily_avg['输出消费数']):,} | {int(weekly_avg['输出消费数']):,} | {int(monthly_avg['输出消费数']):,} | {int(total['输出消费数']):,} |")
    print(f"| 总消费数 | {int(daily_avg['总消费数']):,} | {int(weekly_avg['总消费数']):,} | {int(monthly_avg['总消费数']):,} | {int(total['总消费数']):,} |")

# 绘制折线图
def plot_trend(daily_data):
    # 创建3个独立的图表
    fig, axes = plt.subplots(3, 1, figsize=(12, 12))
    
    # 绘制调用次数
    axes[0].plot(daily_data['日期'], daily_data['输入消费数'], label='调用次数', marker='o', linewidth=2, color='blue')
    axes[0].set_title('调用次数趋势', fontsize=14)
    axes[0].set_xlabel('日期', fontsize=12)
    axes[0].set_ylabel('调用次数', fontsize=12)
    axes[0].legend(fontsize=10)
    axes[0].grid(True, linestyle='--', alpha=0.7)
    axes[0].xaxis.set_major_formatter(mdates.DateFormatter('%Y-%m-%d'))
    
    # 绘制输出消费
    axes[1].plot(daily_data['日期'], daily_data['输出消费数'], label='输出消费', marker='s', linewidth=2, color='green')
    axes[1].set_title('输出消费趋势', fontsize=14)
    axes[1].set_xlabel('日期', fontsize=12)
    axes[1].set_ylabel('输出消费', fontsize=12)
    axes[1].legend(fontsize=10)
    axes[1].grid(True, linestyle='--', alpha=0.7)
    axes[1].xaxis.set_major_formatter(mdates.DateFormatter('%Y-%m-%d'))
    
    # 绘制总消费数
    axes[2].plot(daily_data['日期'], daily_data['总消费数'], label='总消费数', marker='^', linewidth=2, color='red')
    axes[2].set_title('总消费数趋势', fontsize=14)
    axes[2].set_xlabel('日期', fontsize=12)
    axes[2].set_ylabel('总消费数', fontsize=12)
    axes[2].legend(fontsize=10)
    axes[2].grid(True, linestyle='--', alpha=0.7)
    axes[2].xaxis.set_major_formatter(mdates.DateFormatter('%Y-%m-%d'))
    
    # 调整布局
    fig.autofmt_xdate()
    plt.tight_layout()
    
    # 保存图表
    output_file = 'minimax_new_usage_trend.png'
    plt.savefig(output_file, dpi=150)
    print(f"图表已保存到 {output_file}")

# 主函数
def main():
    print(f"开始分析 {csv_file}...")
    df = read_csv()
    if df is None:
        return
    
    daily_data = preprocess_data(df)
    if daily_data.empty:
        print("处理后的数据为空")
        return
    
    daily_avg, weekly_avg, monthly_avg, total = calculate_statistics(daily_data)
    generate_table(daily_avg, weekly_avg, monthly_avg, total)
    plot_trend(daily_data)
    
    print("分析完成！")

if __name__ == "__main__":
    main()

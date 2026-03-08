#!/usr/bin/env python3
import re
import os

def process_markdown_to_marp(input_file, output_file, image_dir):
    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')
    result = []
    result.append('---')
    result.append('marp: true')
    result.append('theme: gaia')
    result.append('size: 16:9')
    result.append('paginate: true')
    result.append('---')
    result.append('')

    i = 0
    diagram_idx = 0

    while i < len(lines):
        line = lines[i]

        if line.strip() == '':
            result.append(line)
            i += 1
            continue

        if line.startswith('# '):
            result.append(line)
            result.append('')
            i += 1
            continue

        if line.startswith('## '):
            result.append(line)
            result.append('')
            i += 1
            continue

        if 'PlantUML' in line and i + 1 < len(lines) and lines[i+1].strip() == '':
            result.append(line)
            i += 1
            continue

        if line.strip().startswith('@startuml'):
            diagram_idx += 1
            img_path = f"images/diagram_{diagram_idx}.png"
            if os.path.exists(os.path.join(os.path.dirname(output_file), img_path)):
                result.append(f"![](../教学计划/PPT/{img_path})")
                result.append('')

            while i < len(lines) and '@enduml' not in lines[i]:
                i += 1
            i += 1
            continue

        if line.strip().startswith('六、') or \
           line.strip().startswith('七、') or \
           line.strip().startswith('八、') or \
           line.strip().startswith('九、') or \
           line.strip().startswith('十、') or \
           line.strip().startswith('十一、') or \
           line.strip().startswith('十二、') or \
           line.strip().startswith('十三、') or \
           line.strip().startswith('十四、') or \
           line.strip().startswith('十五、') or \
           line.strip().startswith('十六、') or \
           line.strip().startswith('十七、'):
            result.append('---')
            result.append('')
            result.append('## ' + line.strip()[2:])
            result.append('')
            i += 1
            continue

        result.append(line)
        i += 1

    with open(output_file, 'w', encoding='utf-8') as f:
        f.write('\n'.join(result))

    print(f"Generated Marp file: {output_file}")

def main():
    input_file = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/第2讲-结构化分析、设计与实现.md"
    output_dir = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT"
    output_file = os.path.join(output_dir, "第2讲-结构化分析、设计与实现-marp.md")
    image_dir = os.path.join(output_dir, "images")

    process_markdown_to_marp(input_file, output_file, image_dir)

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
import re
import urllib.request
import urllib.parse
import base64
import zlib
import os

def encode_plantuml(plantuml_code):
    compressed = zlib.compress(plantuml_code.encode('utf-8'), 9)
    encoded = base64.urlsafe_b64encode(compressed).decode('utf-8')
    encoded = encoded.rstrip('=')
    return encoded

def generate_plantuml_image(plantuml_code, output_path):
    plantuml_server = "https://www.plantuml.com/plantuml/png/"
    encoded = encode_plantuml(plantuml_code)
    url = plantuml_server + encoded

    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req, timeout=30) as response:
            image_data = response.read()
            with open(output_path, 'wb') as f:
                f.write(image_data)
        print(f"Generated: {output_path}")
        return True
    except Exception as e:
        print(f"Error generating {output_path}: {e}")
        return False

def extract_plantuml_blocks_with_context(md_content):
    """Extract PlantUML blocks with their surrounding context (title)"""
    pattern = r'(#+\s*[^\n]+\n)?```plantuml\n(.*?)```'
    matches = re.findall(pattern, md_content, re.DOTALL)
    return matches

def main():
    input_file = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/第2讲-结构化方法的实例和练习.md"
    output_dir = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/images"

    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()

    matches = extract_plantuml_blocks_with_context(content)
    print(f"Found {len(matches)} PlantUML blocks")

    for i, (title, block) in enumerate(matches):
        output_file = os.path.join(output_dir, f"practice_diagram_{i+1}.png")
        full_code = f"@startuml\n{block.strip()}\n@enduml"
        generate_plantuml_image(full_code, output_file)

if __name__ == "__main__":
    main()

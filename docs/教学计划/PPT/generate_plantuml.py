#!/usr/bin/env python3
import re
import urllib.request
import urllib.parse
import base64
import zlib
import os

def encode_plantuml(plantuml_code):
    """Encode PlantUML code for URL - uses zlib compression + base64"""
    # Add PlantUML header if not present
    if not plantuml_code.strip().startswith('@startuml'):
        plantuml_code = '@startuml\n' + plantuml_code + '\n@enduml'
    
    # Compress with zlib
    compressed = zlib.compress(plantuml_code.encode('utf-8'), 9)
    # Base64 encode
    encoded = base64.urlsafe_b64encode(compressed).decode('utf-8')
    # Remove padding
    encoded = encoded.rstrip('=')
    return encoded

def generate_plantuml_image(plantuml_code, output_path):
    """Generate PNG image from PlantUML code using online server"""
    plantuml_server = "https://www.plantuml.com/plantuml/png/"

    encoded = encode_plantuml(plantuml_code)
    url = plantuml_server + encoded
    print(f"URL: {url[:80]}...")

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

def extract_plantuml_blocks(md_content):
    """Extract PlantUML code blocks from markdown"""
    # Match @startuml ... @enduml pairs
    pattern = r'@startuml\s*\n(.*?)\n@enduml'
    matches = re.findall(pattern, md_content, re.DOTALL)
    return matches

def main():
    input_file = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/第2讲-结构化分析、设计与实现.md"
    output_dir = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/images"

    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()

    plantuml_blocks = extract_plantuml_blocks(content)
    print(f"Found {len(plantuml_blocks)} PlantUML blocks")

    for i, block in enumerate(plantuml_blocks):
        output_file = os.path.join(output_dir, f"diagram_{i+1}.png")
        # Add proper PlantUML wrapper
        full_code = f"@startuml\n{block}\n@enduml"
        generate_plantuml_image(full_code, output_file)

if __name__ == "__main__":
    main()

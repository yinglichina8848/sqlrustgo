#!/usr/bin/env python3
import re
import urllib.request
import urllib.parse
import base64
import zlib
import os

def encode_plantuml(plantuml_code):
    """Encode PlantUML code for URL - uses zlib compression + base64"""
    compressed = zlib.compress(plantuml_code.encode('utf-8'), 9)
    encoded = base64.urlsafe_b64encode(compressed).decode('utf-8')
    encoded = encoded.rstrip('=')
    return encoded

def generate_plantuml_image(plantuml_code, output_path):
    """Generate PNG image from PlantUML code using online server"""
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

def main():
    output_dir = "/Users/liying/workspace/yinglichina/sqlrustgo/docs/教学计划/PPT/images"

    # DFD Diagram for Library System - Context Diagram
    dfd_context = """@startuml
left to right direction

actor 读者
actor 图书管理员

rectangle 图书馆管理系统 {
}

database 图书馆数据库

读者 --> 图书馆管理系统 : 借书/还书
图书管理员 --> 图书馆管理系统 : 图书管理
图书馆管理系统 --> 图书馆数据库 : 存储数据
@enduml"""

    # DFD Diagram for Library System - Level 1
    dfd_level1 = """@startuml

rectangle "图书馆管理系统" {

  rectangle "借书处理"
  rectangle "还书处理"
  rectangle "图书查询"
  rectangle "读者管理"

}

database 读者文件
database 图书文件
database 借阅记录

借书处理 --> 读者文件
借书处理 --> 图书文件
借书处理 --> 借阅记录

还书处理 --> 读者文件
还书处理 --> 图书文件
还书处理 --> 借阅记录

图书查询 --> 图书文件
图书查询 --> 借阅记录

读者管理 --> 读者文件

@enduml"""

    # ER Diagram for Library System
    er_diagram = """@startuml

entity "读者" as reader {
  * 读者编号 : varchar
  --
  姓名 : varchar
  联系电话 : varchar
  注册日期 : date
}

entity "图书" as book {
  * ISBN : varchar
  --
  书名 : varchar
  作者 : varchar
  出版社编号 : varchar
}

entity "出版社" as publisher {
  * 出版社编号 : varchar
  --
  出版社名称 : varchar
  地址 : varchar
}

entity "借阅记录" as borrow {
  * 流水号 : int
  --
  读者编号 : varchar
  图书ISBN : varchar
  借阅日期 : date
  应还日期 : date
  实际归还日期 : date
}

reader ||--o{ borrow
book ||--o{ borrow
publisher ||--o{ book

@enduml"""

    # Module Structure Diagram
    module_diagram = """@startuml

package "图书馆管理系统" {

  component "读者管理模块"
  component "图书管理模块"
  component "借阅管理模块"
  component "查询统计模块"
  component "数据库访问层"

}

"读者管理模块" --> "数据库访问层"
"图书管理模块" --> "数据库访问层"
"借阅管理模块" --> "数据库访问层"
"查询统计模块" --> "数据库访问层"

@enduml"""

    # Generate images
    generate_plantuml_image(dfd_context, os.path.join(output_dir, "diagram_1.png"))
    generate_plantuml_image(dfd_level1, os.path.join(output_dir, "diagram_2.png"))
    generate_plantuml_image(er_diagram, os.path.join(output_dir, "diagram_3.png"))
    generate_plantuml_image(module_diagram, os.path.join(output_dir, "diagram_4.png"))

if __name__ == "__main__":
    main()

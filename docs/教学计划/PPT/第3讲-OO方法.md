## **marp: true theme: gaia paginate: true backgroundColor: #fff color: #333**

\<!-- \_class: lead -->

# **第三讲：面向对象分析、设计与实现**

## **从结构化到面向对象的方法演进**

### **以高校图书管理系统为例**

**80分钟 | 两节课**

***

\<!-- \_class: lead -->

# **第一部分：OO的历史与演进（What & Why）**

***

## **1. 软件危机与结构化方法（1960s-1980s）**

### **历史背景**

- **1960-1970年代**：软件规模爆炸，项目频繁延期
- **典型问题**：需求变更导致大规模返工、代码难以维护、团队沟通困难

### **结构化方法**

- **核心思想**：自顶向下、逐步求精、功能分解
- **三大工具**：数据流图（DFD）、实体-关系图（ERD）、流程图

### **核心缺陷：数据与操作分离**

**plain**

复制

```
[学生数据] ←→ [借书处理程序] ←→ [图书数据]
     ↑              ↓              ↑
   数据库        业务逻辑        数据库
```

**痛点**：修改借书期限，需改所有相关程序

***

## **2. 面向对象方法的诞生（1980s-1990s）**

### **图形界面革命的推动**

- Macintosh、Windows普及，界面元素天然具有**对象特征**
- 窗口、按钮都有**状态**和**行为**

### **OO发展里程碑**

**表格**

**年代**

**事件**

1967

Simula首次引入"类"和"对象"

1970s

Smalltalk第一个纯OO语言

1980s

C++将OO带入主流工程

1990s

UML统一建模语言诞生

### **核心洞察**

> **软件系统应由自治对象组成，对象通过消息协作完成任务**

***

## **3. 方法演进对比与OO核心思想**

**表格**

**维度**

**结构化方法**

**面向对象方法**

**基本单位**

功能模块

对象（数据+操作封装）

**系统视角**

功能分解

责任分配与协作

**应对变化**

修改连锁反应

局部修改，不影响其他对象

**复用方式**

代码复制

继承、组合、多态

### **OO核心思想：责任驱动**

> **对象 = 责任（Responsibility）的封装体**

- **封装**：隐藏内部细节，只暴露必要接口
- **继承**：复用和扩展责任
- **多态**：替换责任执行者

***

## **4. Why OO？应对软件复杂性**

**表格**

**复杂性来源**

**OO解决方案**

**图书管理系统示例**

**认知负担**

封装隐藏细节

学生只需知道"借书"接口

**需求变化**

继承和多态支持扩展

新增"访客"类型，继承基础类即可

**团队协作**

对象职责清晰

前后端通过"借书接口"契约协作

**系统演化**

对象自治，修改局部化

修改罚款规则，只影响"借阅记录"对象

### **修改成本对比：增加"电子书借阅"**

- **结构化**：修改学生、图书、借书、还书、查询...（整个系统）
- **面向对象**：新增EBook类，修改Book类（两个类）

***

## **5. OOA/OOD/OOP的关系与本讲目标**

**plain**

复制

```
现实世界                    软件系统
─────────                   ─────────
学生 ──借书──→ 图书          Student对象 ──消息──→ Book对象
   │                            │
   └── 产生 ──→ 借阅记录        └── 创建 ──→ BorrowRecord对象
```

**表格**

**阶段**

**目标**

**产出**

**OOA**

理解问题，建立概念模型

用例图、概念类图

**OOD**

设计解决方案

设计类图、顺序图、详细方法

**OOP**

编码实现

可执行代码

### **学习目标**

1. **理解** OO思想的核心（责任、封装、协作）
2. **掌握** OOA过程（从需求发现对象）
3. **掌握** OOD过程（设计类、应用SOLID原则）
4. **能够** 完成图书管理系统的OOA/OOD

***

\<!-- \_class: lead -->

# **第二部分：面向对象分析（OOA）**

***

## **6. OOA的核心任务与需求获取**

### **什么是OOA？**

> **面向对象分析（OOA）= 理解问题域，建立概念模型的过程**

### **核心任务**

1. **理解需求**：与领域专家沟通，收集功能需求
2. **识别对象**：从需求描述中发现业务实体
3. **确定属性**：对象的特征信息
4. **识别关系**：对象之间的关联、继承、依赖
5. **产出模型**：用例图、概念类图、活动图

### **图书管理系统需求（简化版）**

> 1. 学生和教师可以借阅图书，每本书最多借30天
> 2. 逾期每天罚款0.5元，最高不超过书价
> 3. 管理员可以添加、删除、修改图书信息
> 4. 用户可以查询图书信息和借阅历史
> 5. 系统需要记录所有借阅历史

***

## **7. 名词分析法：发现对象**

### **方法步骤**

1. **划线**：从需求描述中划出所有名词
2. **筛选**：排除系统、界面、外部系统，保留业务实体
3. **分类**：区分对象、属性、值对象
4. **验证**：检查对象是否有明确的责任

### **图书管理系统名词分析**

> "**学生**和**教师**可以**借阅图书**。每本**图书**有**ISBN**、**书名**、**作者**、**出版社**。**借阅**时需登记**借阅人**、**借阅日期**、**应还日期**。**图书管理员**负责管理**图书**和**借阅记录**。**系统**需要记录**借阅历史**，并支持**逾期罚款**计算。"

### **候选对象筛选**

**表格**

**候选名词**

**判断**

**理由**

学生

✅ 保留

业务实体，有明确责任

教师

✅ 保留

业务实体，与学生类似但权限不同

图书

✅ 保留

核心业务实体

ISBN

❌ 排除

图书的属性

借阅

✅ 保留

业务实体，需要记录状态

借阅记录

✅ 保留

核心业务实体

逾期罚款

⚠️ 值对象

无独立身份，依附于借阅记录

***

## **8. 对象分类与泛化**

### **识别泛化关系（继承）**

**plain**

复制

```
        用户（User）
       /              学生（Student）  教师（Teacher）
    - 学号          - 教师工号
    - 年级          - 院系
    - 最大借书数=5   - 最大借书数=10
```

### **图书管理系统核心对象（第一版）**

**表格**

**对象**

**类型**

**核心责任**

User（用户）

抽象类

登录、查看个人信息

Student（学生）

具体类

借书、还书、查询（限制5本）

Teacher（教师）

具体类

借书、还书、查询（限制10本）

Book（图书）

具体类

记录状态（可借/已借出）、验证ISBN

BorrowRecord（借阅记录）

具体类

记录借阅信息、计算逾期罚款

Librarian（图书管理员）

具体类

管理图书信息

***

## **9. 识别对象关系与属性**

### **关系类型说明**

**表格**

**关系**

**含义**

**图书管理系统示例**

**关联**

对象之间的结构连接

用户创建借阅记录

**聚合**

整体-部分，可独立存在

图书管理系统包含用户（可分离）

**组合**

整体-部分，生命周期依赖

借阅记录包含逾期罚款（不可分离）

**依赖**

使用关系

借阅记录依赖图书获取信息

**继承**

is-a关系

学生是用户，教师是用户

### **图书管理系统对象属性**

**表格**

**对象**

**属性**

**说明**

User

userId, name, password, email

基础信息

Student

studentNo, grade, major

特有信息

Teacher

teacherNo, department, title

特有信息

Book

isbn, title, author, publisher, publishDate, status

status表示可借/已借出/下架

BorrowRecord

recordId, borrowDate, dueDate, returnDate, status

status表示借阅中/已归还/逾期

***

## **10. 课堂练习：完善对象模型**

### **任务**

基于以下补充需求，完善图书管理系统的OOA模型：

> **补充需求**：
>
> 1. 图书可以预约，当图书归还时通知预约者
> 2. 图书馆有多个分馆，图书属于特定分馆
> 3. 学生可以续借一次，教师可以续借两次

### **思考要点**

1. 需要增加哪些新对象？
2. 预约与借阅记录是什么关系？
3. 分馆与图书是什么关系（聚合还是关联）？
4. 续借功能如何体现（修改记录还是创建新记录）？

**时间：10分钟 | 小组讨论后分享**

***

## **11. 练习点评与OOA阶段产出**

### **典型问题与解决方案**

**表格**

**问题**

**优化建议**

将"预约"作为借阅记录的属性

预约是独立业务概念，应作为独立对象（Reservation）

分馆作为图书的属性

分馆有独立管理责任，应作为对象（LibraryBranch）

忽略续借历史记录

续借应创建新的借阅记录或记录续借历史，便于追溯

所有用户都继承User

管理员也是用户，应统一继承体系

### **优化后的对象列表**

- User（用户）← Student, Teacher, Librarian
- Book（图书）
- LibraryBranch（分馆）
- BorrowRecord（借阅记录）
- Reservation（预约记录）

### **OOA阶段产出：概念类图**

- **类名**：业务概念名称（中文或英文）
- **属性**：只写名称，不写类型和可见性（分析阶段不关心）
- **方法**：不写或只写关键业务方法
- **关系**：关联、继承、聚合、组合

***

\<!-- \_class: lead -->

# **课间休息（5分钟）**

***

\<!-- \_class: lead -->

# **第三部分：从OOA到OOD**

***

## **12. OOA vs OOD：关键区别**

**表格**

**维度**

**OOA（分析）**

**OOD（设计）**

**关注点**

问题域（What）

解决方案（How）

**抽象层次**

概念层（业务概念）

逻辑层（软件组件）

**技术细节**

不涉及

涉及（数据库、接口、框架）

**属性定义**

名称即可

名称+类型+可见性+约束

**方法定义**

业务责任

具体方法签名+算法

**关系实现**

概念关联

具体实现（引用、集合、外键）

### **映射关系示例**

**plain**

复制

```
OOA: 用户 ──借阅──► 图书  （概念关联）
        ↓
OOD: User类持有BorrowRecord列表引用
     Book类持有BorrowRecord列表引用
     BorrowRecord持有User和Book的引用
```

***

## **13. OOD的核心任务**

### **设计阶段任务清单**

1. **细化类设计**
   - 确定属性和方法的完整签名
   - 确定可见性（public/private/protected）
   - 确定数据类型和约束
2. **设计关系实现**
   - 关联→引用或集合
   - 继承→ extends/implements
   - 依赖→参数传递或局部变量
3. **应用设计原则**
   - SOLID原则指导类设计
   - 设计模式解决常见问题
4. **设计交互**
   - 用顺序图描述对象协作
   - 用状态图描述对象生命周期

***

## **14. 从分析到设计：User类转换示例**

**OOA阶段：User类**

**plain**

复制

```
User
- userId
- name
- email
- 借书()
- 还书()
```

**OOD阶段：User类**

**java**

复制

```
public abstract class User {
    private String userId;          // 私有，通过getter访问
    private String name;
    private String email;
    private List<BorrowRecord> borrowRecords;  // 关联实现

    public boolean canBorrow() {    // 业务规则封装
        return borrowRecords.stream()
               .filter(r -> r.getStatus() == BorrowStatus.BORROWED)
               .count() < getMaxBooks();
    }

    public abstract int getMaxBooks();  // 子类实现不同限额

    public BorrowRecord borrowBook(Book book) throws BorrowException {
        if (!canBorrow()) throw new BorrowException("超出借书限额");
        if (!book.isAvailable()) throw new BorrowException("图书不可借");

        BorrowRecord record = new BorrowRecord(this, book);
        borrowRecords.add(record);
        book.setStatus(BookStatus.BORROWED);
        return record;
    }
}
```

***

## **15. 关联与继承的实现设计**

### **1对多关联的实现**

**场景**：一个用户有多个借阅记录

**java**

复制

```
public class User {
    private List<BorrowRecord> borrowRecords = new ArrayList<>();

    public void addBorrowRecord(BorrowRecord record) {
        borrowRecords.add(record);
        record.setUser(this);  // 维护双向关联
    }
}

public class BorrowRecord {
    private User user;  // 外键引用
    private Book book;

    public void setUser(User user) {
        this.user = user;
    }
}
```

### **User继承体系设计**

**java**

复制

```
public abstract class User {
    protected String userId;    // protected：子类可访问
    protected String name;

    public abstract int getMaxBooks();  // 抽象方法
    public abstract int getBorrowDays(); // 借书期限
}

public class Student extends User {
    private String studentNo;
    private String grade;

    @Override
    public int getMaxBooks() { return 5; }

    @Override
    public int getBorrowDays() { return 30; }
}

public class Teacher extends User {
    private String teacherNo;
    private String department;

    @Override
    public int getMaxBooks() { return 10; }

    @Override
    public int getBorrowDays() { return 60; }
}
```

***

## **16. 方法设计的细节**

### **方法设计检查清单**

**表格**

**检查项**

**图书管理系统示例**

**方法名**

动词+名词，如`borrowBook`而非`borrow`

**参数**

最小化，优先使用对象而非多个参数

**返回值**

明确成功/失败，使用异常或结果对象

**可见性**

对外接口public，内部辅助private

**单一职责**

一个方法只做一件事

### **反例与正例**

**java**

复制

```
// ❌ 反例：方法职责过多，参数混乱
public void handleBook(String userId, String bookId, 
                      boolean isBorrow, Date date) { ... }

// ✅ 正例：职责单一，参数清晰
public BorrowRecord borrowBook(Book book) throws BorrowException { ... }
public void returnBook(BorrowRecord record) throws ReturnException { ... }
```

***

## **17. 设计类图（详细版）**

### **图书管理系统设计类图（简化）**

**plain**

复制

```
┌─────────────────────────────────────────────────────────┐
│                      User (abstract)                     │
│  - userId: String                                        │
│  - name: String                                          │
│  - email: String                                         │
│  - borrowRecords: List<BorrowRecord>                     │
│  + canBorrow(): boolean                                  │
│  + borrowBook(Book): BorrowRecord                        │
│  + getMaxBooks(): int {abstract}                         │
└─────────────────────────────────────────────────────────┘
     △
    /|\
┌───┐ ┌───┐
│Student│ │Teacher│
│-studentNo│ │-teacherNo│
│-grade: String│ │-department: String│
│+getMaxBooks(): 5│ │+getMaxBooks(): 10│
└───┘ └───┘

┌─────────────┐         ┌──────────────────┐
│     Book    │         │  BorrowRecord    │
│ - isbn: String│◄──────│  - recordId: String│
│ - title: String│  1 * │  - borrowDate: Date│
│ - author: String│      │  - dueDate: Date   │
│ - status: Enum │      │  - returnDate: Date│
│ + isAvailable(): boolean│ - status: Enum    │
│ + borrow(): void        │  + isOverdue(): boolean│
└─────────────┘         │  + calculateFine(): double│
                        └──────────────────┘
```

***

\<!-- \_class: lead -->

# **第四部分：SOLID设计原则**

***

## **18. 什么是好的OO设计？**

### **设计质量评估标准**

**表格**

**标准**

**含义**

**反面（坏味道）**

**可维护性**

易于修改bug

修改一处，到处报错

**可扩展性**

易于添加功能

新增功能需要修改大量现有代码

**可复用性**

组件可在其他系统使用

代码高度耦合，无法单独提取

**可读性**

易于理解设计意图

命名混乱，结构复杂

### **SOLID原则**

> **SOLID = 5个原则的首字母缩写，指导高内聚、低耦合的设计**

- **S**ingle Responsibility Principle（单一职责）
- **O**pen/Closed Principle（开闭原则）
- **L**iskov Substitution Principle（里氏替换）
- **I**nterface Segregation Principle（接口隔离）
- **D**ependency Inversion Principle（依赖倒置）

***

## **19. S - 单一职责原则（SRP）**

### **定义**

> **一个类应该只有一个引起它变化的原因**

### **图书管理系统反例**

**java**

复制

```
// ❌ 违反SRP：User类同时处理业务逻辑和数据库操作
public class User {
    private String userId;
    private String name;

    public void borrowBook(Book book) { ... }  // 业务逻辑
    public void saveToDatabase() { ... }        // 持久化逻辑
    public void sendEmailNotification() { ... } // 通知逻辑
}
```

### **正例：职责分离**

**java**

复制

```
// ✅ 业务逻辑
public class User {
    public BorrowRecord borrowBook(Book book) { ... }
}

// ✅ 持久化职责
public class UserRepository {
    public void save(User user) { ... }
    public User findById(String id) { ... }
}

// ✅ 通知职责
public class NotificationService {
    public void sendEmail(User user, String message) { ... }
}
```

***

## **20. O - 开闭原则（OCP）**

### **定义**

> **对扩展开放，对修改关闭**

- **扩展开放**：新增功能时，可以添加新代码
- **修改关闭**：不修改已有正常工作的代码

### **图书管理系统场景：逾期罚款计算**

**需求变化**：平时罚款0.5元/天，寒暑假期间0.3元/天

**java**

复制

```
// ❌ 违反OCP：每次变化都要修改现有代码
public class BorrowRecord {
    public double calculateFine() {
        if (isHolidayPeriod()) {  // 新增判断，修改了原有方法
            return overdueDays * 0.3;
        }
        return overdueDays * 0.5;
    }
}

// ✅ 遵循OCP：通过扩展实现变化
public interface FineCalculator {
    double calculate(int overdueDays);
}

public class DefaultFineCalculator implements FineCalculator {
    public double calculate(int days) { return days * 0.5; }
}

public class HolidayFineCalculator implements FineCalculator {
    public double calculate(int days) { return days * 0.3; }
}

// 使用
public class BorrowRecord {
    private FineCalculator calculator;  // 依赖抽象，可替换实现
}
```

***

## **21. L - 里氏替换原则（LSP）**

### **定义**

> **子类必须能够替换父类，且程序行为不变**

### **图书管理系统示例**

**java**

复制

```
public abstract class User {
    public abstract int getMaxBooks();  // 子类决定借书限额
    public abstract boolean canBorrow(); // 子类决定借书资格
}

public class Student extends User {
    public int getMaxBooks() { return 5; }
    public boolean canBorrow() { 
        return 当前借书数 < 5 && 无逾期记录; 
    }
}

public class Teacher extends User {
    public int getMaxBooks() { return 10; }
    public boolean canBorrow() { 
        return 当前借书数 < 10;  // 教师不受逾期限制
    }
}

// 使用：任何User子类都可以替换
public void processBorrow(User user, Book book) {
    if (user.canBorrow()) {  // 无需知道具体类型
        user.borrowBook(book);
    }
}
```

***

## **22. I - 接口隔离原则（ISP）**

### **定义**

> **客户端不应该依赖它不需要的接口**

### **图书管理系统反例**

**java**

复制

```
// ❌ 胖接口：强迫实现不需要的方法
public interface LibraryService {
    void addBook(Book book);           // 管理员需要
    void deleteBook(String isbn);      // 管理员需要
    BorrowRecord borrowBook(String userId, String isbn);  // 用户需要
    void returnBook(String recordId);  // 用户需要
    double calculateFine(String recordId);  // 系统需要
}

public class StudentService implements LibraryService {
    // 学生被迫实现addBook、deleteBook，抛出异常
    public void addBook(Book book) { 
        throw new UnsupportedOperationException(); 
    }
}
```

### **正例：接口拆分**

**java**

复制

```
// ✅ 小接口，职责单一
public interface BookManagementService {
    void addBook(Book book);
    void deleteBook(String isbn);
}

public interface BorrowService {
    BorrowRecord borrowBook(String userId, String isbn);
    void returnBook(String recordId);
}

public interface FineCalculationService {
    double calculateFine(String recordId);
}

// 学生只需实现BorrowService
public class StudentBorrowService implements BorrowService { ... }
```

***

## **23. D - 依赖倒置原则（DIP）**

### **定义**

> **高层模块不应该依赖低层模块，两者都应该依赖抽象**

### **图书管理系统示例**

**java**

复制

```
// ❌ 违反DIP：BorrowService直接依赖具体实现
public class BorrowService {
    private MySQLUserRepository userRepo;  // 依赖具体数据库
    private MySQLBookRepository bookRepo;  // 依赖具体数据库

    public BorrowService() {
        this.userRepo = new MySQLUserRepository();  // 硬编码
        this.bookRepo = new MySQLBookRepository();
    }
}

// ✅ 遵循DIP：依赖抽象接口
public class BorrowService {
    private UserRepository userRepo;  // 依赖接口
    private BookRepository bookRepo;  // 依赖接口

    public BorrowService(UserRepository userRepo, 
                        BookRepository bookRepo) {
        this.userRepo = userRepo;  // 通过构造器注入
        this.book_repo = book_repo;
    }
}

// 可以注入MySQL实现、MongoDB实现、内存实现（测试用）
```

***

## **24. SOLID原则应用总结**

### **图书管理系统设计检查**

**表格**

**原则**

**应用点**

**验证问题**

**SRP**

User管业务，Repository管持久化，Service管流程

User类是否只处理用户相关逻辑？

**OCP**

FineCalculator接口支持不同罚款策略

新增节假日策略是否无需修改BorrowRecord？

**LSP**

Student/Teacher替换User

替换后canBorrow()行为是否符合预期？

**ISP**

拆分ManagementService/BorrowService

Student是否被迫实现管理方法？

**DIP**

Service依赖Repository接口

更换数据库是否需要修改Service？

### **设计评审清单**

- \[ ] 每个类只有一个明确的职责
- \[ ] 新增功能主要通过扩展实现
- \[ ] 子类可以完全替换父类
- \[ ] 接口小而专，没有强迫实现的方法
- \[ ] 具体类依赖抽象，而非具体实现

***

## **25. 课堂练习：应用SOLID优化设计**

### **任务描述**

以下代码存在哪些SOLID违反？如何优化？

**java**

复制

```
public class LibrarySystem {
    private List<Book> books = new ArrayList<>();
    private List<User> users = new ArrayList<>();

    public void addBook(String isbn, String title, String author) {
        books.add(new Book(isbn, title, author));
    }

    public void borrowBook(String userId, String isbn) {
        User user = findUser(userId);
        Book book = findBook(isbn);

        if (user.getBorrowedCount() < 5 && book.isAvailable()) {
            BorrowRecord record = new BorrowRecord(user, book);
            user.addRecord(record);
            book.setAvailable(false);

            // 发送邮件通知
            EmailSender sender = new EmailSender();
            sender.send(user.getEmail(), "借书成功");
        }
    }

    public double calculateFine(String recordId) {
        BorrowRecord record = findRecord(recordId);
        long days = ChronoUnit.DAYS.between(record.getDueDate(), LocalDate.now());
        return days * 0.5;
    }
}
```

**时间：10分钟 | 识别问题并提出优化方案**

***

## **26. 练习点评与OOD阶段产出**

### **识别的SOLID违反**

**表格**

**问题**

**违反原则**

**解决方案**

LibrarySystem处理所有逻辑

SRP

拆分为BookService、BorrowService、NotificationService

硬编码EmailSender

DIP

依赖Notification接口，注入实现

罚款计算固定0.5元

OCP

提取FineCalculator接口

借书限额固定为5

OCP

移到User类，子类重写

直接操作Book状态

封装

Book自己处理借出逻辑

### **优化后的结构**

**plain**

复制

```
LibrarySystem
├── BookService (addBook, queryBook)
├── BorrowService (依赖UserRepository, BookRepository)
│   └── 依赖Notification接口（而非EmailSender）
├── NotificationService (实现Notification接口)
└── FineCalculator (接口，支持多种计算策略)
```

### **OOD阶段产出物**

- **设计类图**：完整类结构、属性、方法、关系
- **顺序图**：关键用例的对象交互时序
- **状态图**：对象生命周期（如图书状态）
- **数据库设计**：表结构、外键、索引
- **接口设计**：API签名、参数、返回值
- **设计说明**：关键决策、模式应用、约束

***

## **27. 本讲总结与课后作业**

### **核心知识点回顾**

**plain**

复制

```
OOA（分析）                OOD（设计）               OOP（实现）
───────                   ─────────                 ─────────
理解问题域                 设计解决方案               编码实现
发现对象                   细化类设计                 编写代码
识别关系                   应用设计原则               单元测试
建立概念模型               设计交互与状态             集成测试
```

### **关键技能**

1. **名词分析法**：从需求文本中提取对象
2. **责任分配**：将行为分配到合适对象
3. **SOLID原则**：指导高质量设计
4. **设计文档**：类图、顺序图、状态图

### **课后作业**

基于本讲内容，完成图书管理系统的详细OOD设计：

1. **完善类图**：包含所有属性（含类型、可见性）和方法（含参数、返回值）
2. **绘制顺序图**：选择"借书"或"还书"用例，绘制对象交互顺序图
3. **应用SOLID**：说明设计中如何应用至少3个SOLID原则
4. **设计说明**：200字左右，说明关键设计决策

### **下节预告**

**第四讲：UML规范与建模实践**

- UML图的历史与分类
- 用例图、类图、顺序图的规范画法
- 状态图与活动图的应用场景
- 图书管理系统的完整UML建模

***

\<!-- \_class: lead -->

# **谢谢！**

## **Q\&A**

**联系方式：<xxx@xxx.edu.cn>** kRepository bookRepo;  // 依赖接口

**plain**

复制

```
public BorrowService(UserRepository userRepo, 
                    BookRepository bookRepo) {
    this.userRepo = userRepo;  // 通过构造器注入
    this.book_repo = book_repo;
}
```

}

// 可以注入MySQL实现、MongoDB实现、内存实现（测试用）

**plain**

复制

````

---

## 31. SOLID原则应用总结

### 图书管理系统设计检查

| 原则 | 应用点 | 验证问题 |
|------|--------|---------|
| **SRP** | User管业务，Repository管持久化，Service管流程 | User类是否只处理用户相关逻辑？ |
| **OCP** | FineCalculator接口支持不同罚款策略 | 新增节假日策略是否无需修改BorrowRecord？ |
| **LSP** | Student/Teacher替换User | 替换后canBorrow()行为是否符合预期？ |
| **ISP** | 拆分ManagementService/BorrowService | Student是否被迫实现管理方法？ |
| **DIP** | Service依赖Repository接口 | 更换数据库是否需要修改Service？ |

### 设计评审清单
- [ ] 每个类只有一个明确的职责
- [ ] 新增功能主要通过扩展实现
- [ ] 子类可以完全替换父类
- [ ] 接口小而专，没有强迫实现的方法
- [ ] 具体类依赖抽象，而非具体实现

---

## 32. 课堂练习：应用SOLID优化设计

### 任务描述

以下代码存在哪些SOLID违反？如何优化？

```java
public class LibrarySystem {
    private List<Book> books = new ArrayList<>();
    private List<User> users = new ArrayList<>();

    public void addBook(String isbn, String title, String author) {
        books.add(new Book(isbn, title, author));
    }

    public void borrowBook(String userId, String isbn) {
        User user = findUser(userId);
        Book book = findBook(isbn);

        if (user.getBorrowedCount() < 5 && book.isAvailable()) {
            BorrowRecord record = new BorrowRecord(user, book);
            user.addRecord(record);
            book.setAvailable(false);

            // 发送邮件通知
            EmailSender sender = new EmailSender();
            sender.send(user.getEmail(), "借书成功");
        }
    }

    public double calculateFine(String recordId) {
        BorrowRecord record = findRecord(recordId);
        long days = ChronoUnit.DAYS.between(record.getDueDate(), LocalDate.now());
        return days * 0.5;
    }
}
````

**时间：10分钟 | 识别问题并提出优化方案**

***

## **33. 练习点评**

### **识别的SOLID违反**

**表格**

**问题**

**违反原则**

**解决方案**

LibrarySystem处理所有逻辑

SRP

拆分为BookService、BorrowService、NotificationService

硬编码EmailSender

DIP

依赖Notification接口，注入实现

罚款计算固定0.5元

OCP

提取FineCalculator接口

借书限额固定为5

OCP

移到User类，子类重写

直接操作Book状态

封装

Book自己处理借出逻辑

### **优化后的结构**

**plain**

复制

```
LibrarySystem
├── BookService (addBook, queryBook)
├── BorrowService (依赖UserRepository, BookRepository)
│   └── 依赖Notification接口（而非EmailSender）
├── NotificationService (实现Notification接口)
└── FineCalculator (接口，支持多种计算策略)
```

***

## **34. OOD阶段产出与设计评审**

### **完整设计文档清单**

**表格**

**文档**

**内容**

**工具**

**设计类图**

完整类结构、属性、方法、关系

UML工具、Mermaid

**顺序图**

关键用例的对象交互时序

UML工具

**状态图**

对象生命周期（如图书状态）

UML工具

**数据库设计**

表结构、外键、索引

ERD工具

**接口设计**

API签名、参数、返回值

Markdown、Swagger

**设计说明**

关键决策、模式应用、约束

Word/Markdown

### **设计评审检查表**

**表格**

**维度**

**检查项**

**图书管理系统示例**

**完整性**

是否覆盖所有需求用例？

借书、还书、预约、管理都涉及？

**正确性**

类关系是否符合业务规则？

借阅记录是否正确关联用户和图书？

**一致性**

类图与顺序图是否一致？

类图中的方法是否在顺序图中使用？

**可实现性**

设计能否在技术上实现？

多对多关系是否有解决方案？

**设计质量**

SOLID原则遵循程度？

是否存在明显违反？

***

## **35. 本讲总结**

### **核心知识点回顾**

**plain**

复制

```
OOA（分析）                OOD（设计）               OOP（实现）
───────                   ─────────                 ─────────
理解问题域                 设计解决方案               编码实现
发现对象                   细化类设计                 编写代码
识别关系                   应用设计原则               单元测试
建立概念模型               设计交互与状态             集成测试
```

### **关键技能**

1. **名词分析法**：从需求文本中提取对象
2. **责任分配**：将行为分配到合适对象
3. **SOLID原则**：指导高质量设计
4. **设计文档**：类图、顺序图、状态图

***

## **36. 课后作业与下节预告**

### **作业内容**

基于本讲内容，完成图书管理系统的详细OOD设计：

1. **完善类图**：包含所有属性（含类型、可见性）和方法（含参数、返回值）
2. **绘制顺序图**：选择"借书"或"还书"用例，绘制对象交互顺序图
3. **应用SOLID**：说明设计中如何应用至少3个SOLID原则
4. **设计说明**：200字左右，说明关键设计决策

### **提交要求**

- 格式：Markdown或图片（手绘拍照也可）
- 截止时间：下次课前
- 评价标准：完整性、正确性、SOLID应用

### **下节预告**

**第四讲：UML规范与建模实践**

- UML图的历史与分类
- 用例图、类图、顺序图的规范画法
- 状态图与活动图的应用场景
- 图书管理系统的完整UML建模

***

\<!-- \_class: lead -->

# **谢谢！**

## **Q\&A**

**联系方式：<xxx@xxx.edu.cn>**

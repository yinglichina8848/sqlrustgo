# FTS 全文搜索 (v3.1.0)

> **基于 GitNexus 分析** | 67,755 符号, 102,165 关系
> InvertedIndex / MultiLanguageTokenizer / 模糊搜索

## 1. FTS 架构

### 1.1 核心数据结构

```rust
pub struct InvertedIndex {
    index: HashMap<String, HashSet<u64>>,
    tokenizer: MultiLanguageTokenizer,
    doc_count: u64,
}

pub struct MultiLanguageTokenizer {
    simple: SimpleTokenizer,
    chinese: ChineseTokenizer,
}
```

### 1.2 关键文件

| 文件 | 行数 | 作用 |
|------|------|------|
| [fts/inverted_index.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/fts/inverted_index.rs) | 164 | 内存倒排索引 |
| [rag/inverted_index.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/rag/src/inverted_index.rs) | 284 | RAG 倒排索引 |
| [fts/tokenizer.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/fts/tokenizer.rs) | - | 多语言分词器 |
| [fts/mod.rs](file:///Users/liying/workspace/dev/yinglichina163/sqlrustgo/crates/storage/src/fts/mod.rs) | 76 | FTS 模块入口 |

## 2. 索引构建链路

### 2.1 索引构建时序图

```
CREATE FULLTEXT INDEX idx_content ON articles(content)
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. DDL 执行: CREATE INDEX                        │
│    ├── 扫描表所有行                               │
│    └── 对每行调用 add_document()                  │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. InvertedIndex.add_document(doc_id, text)       │
│    ├── tokenizer.tokenize(text)                   │
│    │   ├── SimpleTokenizer:                       │
│    │   │   ├── lowercase                          │
│    │   │   ├── remove stop words                  │
│    │   │   └── filter < 2 chars                   │
│    │   └── ChineseTokenizer:                      │
│    │       ├── unigram (单字)                     │
│    │       └── bigram (双字)                      │
│    └── for each token:                            │
│        └── index[token].insert(doc_id)            │
└──────────────────────────────────────────────────┘
```

## 3. 搜索执行链路

### 3.1 精确搜索时序图

```
SELECT * FROM articles WHERE MATCH(content) AGAINST('数据库系统')
    │
    ▼
┌──────────────────────────────────────────────────┐
│ 1. InvertedIndex.search("数据库系统")             │
│    ├── tokenizer.tokenize("数据库系统")           │
│    │   → ["数据", "据库", "库系", "系统",         │
│    │      "数据库", "库系统"]                     │
│    └── AND 搜索: 所有 token 的 doc_id 交集       │
│        ├── index["数据"] = {1, 3, 5}             │
│        ├── index["据库"] = {1, 5}                │
│        ├── index["库系"] = {1}                    │
│        ├── index["系统"] = {1, 2, 5}             │
│        ├── index["数据库"] = {1, 5}              │
│        ├── index["库系统"] = {1}                  │
│        └── 交集 = {1}                            │
└──────────────────┬───────────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────────┐
│ 2. Fetch rows by matched doc_ids                  │
│    └── storage.get_row(table, doc_id=1)           │
└──────────────────────────────────────────────────┘
```

### 3.2 模糊搜索活动图

```
    ┌──────────────────────────────┐
    │ fuzzy_search("databse", 2)   │
    │ (编辑距离 ≤ 2)              │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │ 遍历词汇表所有 token         │
    │ ⚠️ O(T), T=词汇表大小       │
    └──────────────┬───────────────┘
                   │
                   ▼
    ┌──────────────────────────────┐
    │ 对每个 token 计算            │
    │ Levenshtein 距离             │
    │ O(a*b), a=查询长, b=token长 │
    └──────────────┬───────────────┘
                   │
            ┌──────┴──────┐
            │ distance ≤  │
            │ max_dist?   │
            └──┬──────┬───┘
            YES│      │NO
               ▼      │
    ┌──────────────┐  │
    │ 合并 doc_ids │  │
    │ + 距离分数   │  │
    └──────┬───────┘  │
           │          │
           ▼          ▼
    ┌──────────────────────────┐
    │ 按距离排序返回结果       │
    └──────────────────────────┘
```

## 4. 算法复杂度与性能分析

### 4.1 操作复杂度

| 操作 | 复杂度 | 内存 | 说明 |
|------|--------|------|------|
| 精确搜索 | O(k * m) | O(n) | k=token数, m=每个token的doc数 |
| 模糊搜索 | O(T * k * a * b) | O(n) | ⚠️ T=词汇表大小 |
| Levenshtein | O(a * b) | O(a * b) | 动态规划 |
| 添加文档 | O(t) | O(t) | t=文档token数 |
| 删除文档 | O(t) | O(t) | 逐token移除 |
| 中文分词 | O(n) | O(n) | n-gram |

### 4.2 ⚠️ 已知问题

| 问题 | 严重性 | 影响 | 修复建议 |
|------|--------|------|---------|
| **模糊搜索 O(T)** | 🔴 严重 | 词汇量大时极慢 | BK-Tree 或 Levenshtein Automaton |
| **中文 n-gram 分词** | 🟡 中等 | 精度低，噪声多 | 集成 jieba 字典分词 |
| **无 TF-IDF 排序** | 🟡 中等 | 结果无相关性排序 | 实现 BM25 排序 |
| **无布尔模式** | 🟡 中等 | 不支持 AND/OR/NOT | 实现查询解析器 |
| **索引无持久化** | 🟡 中等 | 重启后需重建 | WAL 集成 |

### 4.3 性能优化建议

```
优化1: BK-Tree 加速模糊搜索
  当前: 遍历整个词汇表 O(T)
  建议: 构建 BK-Tree, 只搜索距离 ≤ max_dist 的子树
  预期: O(log T * k) 替代 O(T * k)

优化2: Levenshtein Automaton
  当前: 对每个 token 做 DP 计算 O(a*b)
  建议: 构建 Levenshtein Automaton, 一次遍历 Trie
  预期: O(a * |Trie|) 替代 O(T * a * b)

优化3: BM25 排序
  当前: 搜索结果无排序
  建议: 计算 BM25 分数 = IDF * (tf * (k1+1)) / (tf + k1 * (1-b+b*dl/avgdl))
  预期: 按相关性排序结果

优化4: 中文字典分词
  当前: unigram + bigram (噪声多)
  建议: 集成 jieba 分词 (基于前缀词典)
  预期: 分词精度大幅提升
```

## 5. 与其他模块的依赖

```
InvertedIndex
  ├── 依赖: MultiLanguageTokenizer (分词)
  ├── 被依赖: FTS 执行链路 (MATCH...AGAINST)
  ├── 被依赖: RAG pipeline (检索增强)
  └── 被依赖: BTreeIndex (FullTextIndex 类型)
```

## 变更记录

| 日期 | 版本 | 说明 |
|------|------|------|
| 2026-05-15 | v2.0 | 基于 GitNexus 重新分析，补充模糊搜索 O(T) 问题、BM25 建议 |

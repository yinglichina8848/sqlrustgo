# Cypher Phase-1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 Cypher 子集执行引擎，支持 MATCH/WHERE/RETURN 单跳关系查询

**Architecture:** 直接复用 GraphStore API + Traversal BFS/DFS，绕过 Query Planner

```
Cypher Query → Lexer → Parser → AST → Pattern Matcher → Traversal API (BFS/DFS) → Filter → Projection → Result
```

**Tech Stack:** Rust, 复用现有 GraphStore API, BFS/DFS

---

## Task 1: 创建 Cypher Lexer

**Files:**
- Create: `crates/graph/src/cypher/lexer.rs`
- Modify: `crates/graph/src/cypher/mod.rs`
- Test: `crates/graph/tests/cypher_lexer_test.rs`

**Step 1: Write failing test**

```rust
// crates/graph/tests/cypher_lexer_test.rs
#[cfg(test)]
mod cypher_lexer_test {
    use super::*;
    
    #[test]
    fn test_tokenize_match() {
        let mut lexer = CypherLexer::new("MATCH (n) RETURN n");
        assert_eq!(lexer.next_token(), CypherToken::Match);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Return);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
    }
    
    #[test]
    fn test_tokenize_relationship() {
        let mut lexer = CypherLexer::new("MATCH (n)-[:REL]->(m)");
        assert_eq!(lexer.next_token(), CypherToken::Match);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("n".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
        assert_eq!(lexer.next_token(), CypherToken::Dash);
        assert_eq!(lexer.next_token(), CypherToken::LBracket);
        assert_eq!(lexer.next_token(), CypherToken::Colon);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("REL".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RBracket);
        assert_eq!(lexer.next_token(), CypherToken::Arrow);
        assert_eq!(lexer.next_token(), CypherToken::LParen);
        assert_eq!(lexer.next_token(), CypherToken::Identifier("m".to_string()));
        assert_eq!(lexer.next_token(), CypherToken::RParen);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph cypher_lexer_test -- --nocapture`
Expected: FAIL - module not found

**Step 3: Create lexer module structure**

```rust
// crates/graph/src/cypher/lexer.rs
use crate::error::GraphError;

#[derive(Debug, Clone, PartialEq)]
pub enum CypherToken {
    Match,
    Return,
    Where,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Dash,
    Arrow,
    Greater,
    Less,
    Equals,
    Comma,
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    And,
    Or,
    Not,
    Eof,
}

pub struct CypherLexer {
    input: Vec<char>,
    position: usize,
}

impl CypherLexer {
    pub fn new(input: &str) -> Self {
        CypherLexer {
            input: input.chars().collect(),
            position: 0,
        }
    }
    
    pub fn next_token(&mut self) -> CypherToken {
        self.skip_whitespace();
        
        if self.position >= self.input.len() {
            return CypherToken::Eof;
        }
        
        match self.peek() {
            '(' => { self.advance(); CypherToken::LParen }
            ')' => { self.advance(); CypherToken::RParen }
            '[' => { self.advance(); CypherToken::LBracket }
            ']' => { self.advance(); CypherToken::RBracket }
            ':' => { self.advance(); CypherToken::Colon }
            '-' => { 
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    CypherToken::Arrow
                } else {
                    CypherToken::Dash
                }
            }
            '>' => { self.advance(); CypherToken::Greater }
            '=' => { self.advance(); CypherToken::Equals }
            ',' => { self.advance(); CypherToken::Comma }
            '"' | '\'' => self.read_string(),
            c if c.is_alphabetic() => self.read_identifier(),
            c if c.is_ascii_digit() => self.read_number(),
            _ => CypherToken::Eof,
        }
    }
    
    fn peek(&self) -> char {
        self.input.get(self.position).copied().unwrap_or('\0')
    }
    
    fn advance(&mut self) {
        self.position += 1;
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.peek().is_whitespace() {
            self.advance();
        }
    }
    
    fn read_identifier(&mut self) -> CypherToken {
        let mut result = String::new();
        while self.position < self.input.len() && self.peek().is_alphanumeric() {
            result.push(self.peek());
            self.advance();
        }
        match result.to_uppercase().as_str() {
            "MATCH" => CypherToken::Match,
            "RETURN" => CypherToken::Return,
            "WHERE" => CypherToken::Where,
            "AND" => CypherToken::And,
            "OR" => CypherToken::Or,
            "NOT" => CypherToken::Not,
            _ => CypherToken::Identifier(result),
        }
    }
    
    fn read_string(&mut self) -> CypherToken {
        let quote = self.peek();
        self.advance();
        let mut result = String::new();
        while self.position < self.input.len() && self.peek() != quote {
            result.push(self.peek());
            self.advance();
        }
        self.advance(); // consume closing quote
        CypherToken::StringLiteral(result)
    }
    
    fn read_number(&mut self) -> CypherToken {
        let mut result = String::new();
        while self.position < self.input.len() && self.peek().is_ascii_digit() {
            result.push(self.peek());
            self.advance();
        }
        CypherToken::Integer(result.parse().unwrap_or(0))
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-graph cypher_lexer_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/graph/src/cypher/lexer.rs crates/graph/src/cypher/mod.rs crates/graph/tests/cypher_lexer_test.rs
git commit -m "feat(graph): add Cypher lexer with basic tokenization"
```

---

## Task 2: 创建 Cypher Parser

**Files:**
- Create: `crates/graph/src/cypher/parser.rs`
- Modify: `crates/graph/src/cypher/mod.rs`
- Test: `crates/graph/tests/cypher_parser_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_parse_simple_match() {
    let tokens = vec![
        CypherToken::Match,
        CypherToken::LParen,
        CypherToken::Identifier("n".to_string()),
        CypherToken::RParen,
        CypherToken::Return,
        CypherToken::Identifier("n".to_string()),
    ];
    let parser = CypherParser::new(tokens);
    let query = parser.parse_query().unwrap();
    
    assert!(matches!(query.pattern, CypherPattern::Node(_)));
    assert_eq!(query.return_items.len(), 1);
}

#[test]
fn test_parse_match_with_label() {
    let tokens = vec![
        CypherToken::Match,
        CypherToken::LParen,
        CypherToken::Identifier("n".to_string()),
        CypherToken::Colon,
        CypherToken::Identifier("User".to_string()),
        CypherToken::RParen,
        CypherToken::Return,
        CypherToken::Identifier("n".to_string()),
    ];
    let parser = CypherParser::new(tokens);
    let query = parser.parse_query().unwrap();
    
    match &query.pattern {
        CypherPattern::Node(node) => {
            assert_eq!(node.label.as_deref(), Some("User"));
        }
        _ => panic!("Expected Node pattern"),
    }
}

#[test]
fn test_parse_match_with_relationship() {
    let tokens = vec![
        CypherToken::Match,
        CypherToken::LParen,
        CypherToken::Identifier("n".to_string()),
        CypherToken::RParen,
        CypherToken::Dash,
        CypherToken::LBracket,
        CypherToken::Colon,
        CypherToken::Identifier("KNOWS".to_string()),
        CypherToken::RBracket,
        CypherToken::Arrow,
        CypherToken::LParen,
        CypherToken::Identifier("m".to_string()),
        CypherToken::RParen,
        CypherToken::Return,
        CypherToken::Identifier("n".to_string()),
        CypherToken::Comma,
        CypherToken::Identifier("m".to_string()),
    ];
    let parser = CypherParser::new(tokens);
    let query = parser.parse_query().unwrap();
    
    match &query.pattern {
        CypherPattern::Relationship { from, to, rel_label, .. } => {
            assert_eq!(from.label.as_deref(), None);
            assert_eq!(to.label.as_deref(), None);
            assert_eq!(rel_label.as_deref(), Some("KNOWS"));
        }
        _ => panic!("Expected Relationship pattern"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph cypher_parser_test -- --nocapture`
Expected: FAIL - module not found

**Step 3: Create AST types and parser**

```rust
// crates/graph/src/cypher/parser.rs
use super::lexer::{CypherLexer, CypherToken};

#[derive(Debug, Clone, PartialEq)]
pub struct CypherQuery {
    pub pattern: CypherPattern,
    pub where_clause: Option<CypherPredicate>,
    pub return_items: Vec<ReturnItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CypherPattern {
    Node(NodePattern),
    Relationship {
        from: Box<NodePattern>,
        to: Box<NodePattern>,
        rel_label: Option<String>,
        rel_vars: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub variable: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnItem {
    pub variable: String,
    pub property: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CypherPredicate {
    PropertyComparison {
        variable: String,
        property: String,
        operator: ComparisonOp,
        value: Literal,
    },
    And(Box<CypherPredicate>, Box<CypherPredicate>),
    Or(Box<CypherPredicate>, Box<CypherPredicate>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOp {
    Equals,
    NotEquals,
    Greater,
    Less,
    GreaterEq,
    LessEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Integer(i64),
    Boolean(bool),
}

pub struct CypherParser {
    tokens: Vec<CypherToken>,
    position: usize,
}

impl CypherParser {
    pub fn new(tokens: Vec<CypherToken>) -> Self {
        CypherParser { tokens, position: 0 }
    }
    
    pub fn parse_query(&mut self) -> Result<CypherQuery, GraphError> {
        let pattern = self.parse_pattern()?;
        let where_clause = self.parse_where_clause().ok();
        self.expect_token(&CypherToken::Return)?;
        let return_items = self.parse_return_items()?;
        
        Ok(CypherQuery {
            pattern,
            where_clause,
            return_items,
        })
    }
    
    fn parse_pattern(&mut self) -> Result<CypherPattern, GraphError> {
        self.expect_token(&CypherToken::Match)?;
        self.parse_pattern_body()
    }
    
    fn parse_pattern_body(&mut self) -> Result<CypherPattern, GraphError> {
        let from = self.parse_node_pattern()?;
        
        // Check for relationship
        if self.current_token() == &CypherToken::Dash {
            self.advance();
            let rel_label = self.parse_relationship_pattern()?;
            let to = self.parse_node_pattern()?;
            
            return Ok(CypherPattern::Relationship {
                from: Box::new(from),
                to: Box::new(to),
                rel_label: Some(rel_label),
                rel_vars: None,
            });
        }
        
        Ok(CypherPattern::Node(from))
    }
    
    fn parse_node_pattern(&mut self) -> Result<NodePattern, GraphError> {
        self.expect_token(&CypherToken::LParen)?;
        
        let variable = match self.current_token() {
            CypherToken::Identifier(_) => {
                let var = self.advance_and_get_identifier();
                Some(var)
            }
            _ => None,
        };
        
        let label = if self.current_token() == &CypherToken::Colon {
            self.advance();
            Some(self.advance_and_get_identifier())
        } else {
            None
        };
        
        self.expect_token(&CypherToken::RParen)?;
        
        Ok(NodePattern { variable, label })
    }
    
    fn parse_relationship_pattern(&mut self) -> Result<String, GraphError> {
        self.expect_token(&CypherToken::LBracket)?;
        self.expect_token(&CypherToken::Colon)?;
        let rel_label = self.advance_and_get_identifier();
        self.expect_token(&CypherToken::RBracket)?;
        self.expect_token(&CypherToken::Arrow)?;
        Ok(rel_label)
    }
    
    fn parse_where_clause(&mut self) -> Result<CypherPredicate, GraphError> {
        if self.current_token() != &CypherToken::Where {
            return Err(GraphError::ParseError("Expected WHERE".to_string()));
        }
        self.advance();
        self.parse_predicate()
    }
    
    fn parse_predicate(&mut self) -> Result<CypherPredicate, GraphError> {
        let left = self.parse_comparison()?;
        
        match self.current_token() {
            CypherToken::And => {
                self.advance();
                let right = self.parse_predicate()?;
                Ok(CypherPredicate::And(Box::new(left), Box::new(right)))
            }
            CypherToken::Or => {
                self.advance();
                let right = self.parse_predicate()?;
                Ok(CypherPredicate::Or(Box::new(left), Box::new(right)))
            }
            _ => Ok(left),
        }
    }
    
    fn parse_comparison(&mut self) -> Result<CypherPredicate, GraphError> {
        let variable = self.advance_and_get_identifier();
        self.expect_token(&CypherToken::Dot)?;
        let property = self.advance_and_get_identifier();
        
        let operator = match self.current_token() {
            CypherToken::Equals => { self.advance(); ComparisonOp::Equals }
            CypherToken::Greater => { self.advance(); ComparisonOp::Greater }
            CypherToken::Less => { self.advance(); ComparisonOp::Less }
            _ => return Err(GraphError::ParseError("Expected operator".to_string())),
        };
        
        let value = self.parse_literal()?;
        
        Ok(CypherPredicate::PropertyComparison {
            variable,
            property,
            operator,
            value,
        })
    }
    
    fn parse_literal(&mut self) -> Result<Literal, GraphError> {
        match self.current_token().clone() {
            CypherToken::StringLiteral(s) => { self.advance(); Ok(Literal::String(s)) }
            CypherToken::Integer(i) => { self.advance(); Ok(Literal::Integer(i)) }
            CypherToken::Identifier(s) if s.eq_ignore_ascii_case("true") => { self.advance(); Ok(Literal::Boolean(true)) }
            CypherToken::Identifier(s) if s.eq_ignore_ascii_case("false") => { self.advance(); Ok(Literal::Boolean(false)) }
            _ => Err(GraphError::ParseError("Expected literal".to_string())),
        }
    }
    
    fn parse_return_items(&mut self) -> Result<Vec<ReturnItem>, GraphError> {
        let mut items = Vec::new();
        
        loop {
            let variable = self.advance_and_get_identifier();
            let property = if self.current_token() == &CypherToken::Dot {
                self.advance();
                Some(self.advance_and_get_identifier())
            } else {
                None
            };
            
            items.push(ReturnItem { variable, property });
            
            if self.current_token() == &CypherToken::Comma {
                self.advance();
            } else {
                break;
            }
        }
        
        Ok(items)
    }
    
    fn current_token(&self) -> &CypherToken {
        self.tokens.get(self.position).unwrap_or(&CypherToken::Eof)
    }
    
    fn advance(&mut self) {
        self.position += 1;
    }
    
    fn advance_and_get_identifier(&mut self) -> String {
        if let CypherToken::Identifier(s) = self.tokens.get(self.position).cloned().unwrap_or(CypherToken::Identifier(String::new())) {
            self.advance();
            s
        } else {
            String::new()
        }
    }
    
    fn expect_token(&mut self, expected: &CypherToken) -> Result<(), GraphError> {
        if self.current_token() == expected {
            self.advance();
            Ok(())
        } else {
            Err(GraphError::ParseError(format!("Expected {:?}", expected)))
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-graph cypher_parser_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/graph/src/cypher/parser.rs crates/graph/src/cypher/mod.rs crates/graph/tests/cypher_parser_test.rs
git commit -m "feat(graph): add Cypher parser with pattern matching"
```

---

## Task 3: 实现 Pattern Matcher + Traversal Executor

**Files:**
- Create: `crates/graph/src/cypher/executor.rs`
- Modify: `crates/graph/src/cypher/mod.rs`
- Test: `crates/graph/tests/cypher_executor_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_execute_simple_match() {
    let mut store = InMemoryGraphStore::new();
    let alice_id = store.create_node("User", prop!{"name": "Alice", "age": 30});
    let bob_id = store.create_node("User", prop!{"name": "Bob", "age": 25});
    store.create_edge(alice_id, bob_id, "KNOWS", PropertyMap::new()).unwrap();
    
    let result = execute_cypher("MATCH (n) RETURN n", &store).unwrap();
    assert_eq!(result.rows.len(), 2);
}

#[test]
fn test_execute_match_with_label() {
    let mut store = InMemoryGraphStore::new();
    store.create_node("User", prop!{"name": "Alice"});
    store.create_node("Product", prop!{"name": "Widget"});
    
    let result = execute_cypher("MATCH (n:User) RETURN n", &store).unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_execute_match_with_relationship() {
    let mut store = InMemoryGraphStore::new();
    let alice_id = store.create_node("User", prop!{"name": "Alice"});
    let bob_id = store.create_node("User", prop!{"name": "Bob"});
    store.create_edge(alice_id, bob_id, "KNOWS", PropertyMap::new()).unwrap();
    
    let result = execute_cypher("MATCH (n)-[:KNOWS]->(m) RETURN n, m", &store).unwrap();
    assert_eq!(result.rows.len(), 1);
}

#[test]
fn test_execute_match_with_where() {
    let mut store = InMemoryGraphStore::new();
    store.create_node("User", prop!{"name": "Alice", "age": 30});
    store.create_node("User", prop!{"name": "Bob", "age": 25});
    
    let result = execute_cypher("MATCH (n) WHERE n.age > 28 RETURN n.name", &store).unwrap();
    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][0], Value::String("Alice".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph cypher_executor_test -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Create executor**

```rust
// crates/graph/src/cypher/executor.rs
use crate::model::*;
use crate::store::InMemoryGraphStore;
use crate::GraphStore;
use super::parser::{CypherQuery, CypherPattern, CypherPredicate, ReturnItem, ComparisonOp, Literal};
use crate::error::GraphError;
use std::collections::HashMap;

pub struct CypherResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

pub fn execute_cypher(query: &str, store: &InMemoryGraphStore) -> Result<CypherResult, GraphError> {
    let lexer = super::lexer::CypherLexer::new(query);
    let tokens: Vec<_> = std::iter::from_fn(|| {
        let mut lexer = super::lexer::CypherLexer::new(query);
        None
    }).collect();
    
    // Actually use the lexer properly
    let mut lexer = super::lexer::CypherLexer::new(query);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == super::lexer::CypherToken::Eof {
            break;
        }
        tokens.push(token);
    }
    
    let mut parser = super::parser::CypherParser::new(tokens);
    let cypher_query = parser.parse_query()?;
    
    execute_query(cypher_query, store)
}

fn execute_query(query: CypherQuery, store: &InMemoryGraphStore) -> Result<CypherResult, GraphError> {
    match query.pattern {
        CypherPattern::Node(node_pattern) => {
            execute_node_pattern(node_pattern, query.where_clause, query.return_items, store)
        }
        CypherPattern::Relationship { from, to, rel_label, .. } => {
            execute_relationship_pattern(*from, *to, rel_label, query.where_clause, query.return_items, store)
        }
    }
}

fn execute_node_pattern(
    node: super::parser::NodePattern,
    where_clause: Option<CypherPredicate>,
    return_items: Vec<ReturnItem>,
    store: &InMemoryGraphStore,
) -> Result<CypherResult, GraphError> {
    let label = node.label.as_deref();
    
    // Get candidate nodes
    let candidate_ids: Vec<NodeId> = if let Some(label) = label {
        store.nodes_by_label(label)
    } else {
        (0..store.node_count() as NodeId).collect()
    };
    
    // Filter by WHERE clause
    let matched_ids: Vec<NodeId> = candidate_ids.into_iter().filter(|&id| {
        if let Some(pred) = &where_clause {
            evaluate_predicate(pred, id, store)
        } else {
            true
        }
    }).collect();
    
    // Project RETURN items
    let columns: Vec<String> = return_items.iter().map(|r| {
        if let Some(ref prop) = r.property {
            format!("{}.{}", r.variable, prop)
        } else {
            r.variable.clone()
        }
    }).collect();
    
    let rows: Vec<Vec<Value>> = matched_ids.iter().map(|&id| {
        return_items.iter().map(|item| {
            if let Some(ref prop) = item.property {
                store.get_node(id)
                    .and_then(|n| n.properties.get(prop).cloned())
                    .unwrap_or(Value::Null)
            } else {
                Value::String(format!("Node({})", id))
            }
        }).collect()
    }).collect();
    
    Ok(CypherResult { columns, rows })
}

fn execute_relationship_pattern(
    from: super::parser::NodePattern,
    to: super::parser::NodePattern,
    rel_label: Option<String>,
    where_clause: Option<CypherPredicate>,
    return_items: Vec<ReturnItem>,
    store: &InMemoryGraphStore,
) -> Result<CypherResult, GraphError> {
    // Get source nodes
    let source_ids: Vec<NodeId> = if let Some(ref label) = from.label {
        store.nodes_by_label(label)
    } else {
        (0..store.node_count() as NodeId).collect()
    };
    
    let mut results = Vec::new();
    
    for source_id in source_ids {
        // Get outgoing neighbors with relationship
        let neighbors = if let Some(ref rel) = rel_label {
            store.neighbors_by_edge_label(source_id, rel)
        } else {
            store.outgoing_neighbors(source_id)
        };
        
        for &target_id in &neighbors {
            // Filter by WHERE if present
            if let Some(ref pred) = &where_clause {
                if !evaluate_predicate(pred, source_id, store) {
                    continue;
                }
            }
            
            // Build result row
            let row: Vec<Value> = return_items.iter().map(|item| {
                match item.variable.as_str() {
                    "n" => node_to_value(source_id, &item.property, store),
                    "m" => node_to_value(target_id, &item.property, store),
                    _ => Value::Null,
                }
            }).collect();
            
            results.push(row);
        }
    }
    
    let columns: Vec<String> = return_items.iter().map(|r| {
        if let Some(ref prop) = r.property {
            format!("{}.{}", r.variable, prop)
        } else {
            r.variable.clone()
        }
    }).collect();
    
    Ok(CypherResult { columns, rows: results })
}

fn node_to_value(node_id: NodeId, property: &Option<String>, store: &InMemoryGraphStore) -> Value {
    if let Some(prop) = property {
        store.get_node(node_id)
            .and_then(|n| n.properties.get(prop).cloned())
            .unwrap_or(Value::Null)
    } else {
        Value::String(format!("Node({})", node_id))
    }
}

fn evaluate_predicate(predicate: &CypherPredicate, node_id: NodeId, store: &InMemoryGraphStore) -> bool {
    match predicate {
        CypherPredicate::PropertyComparison { variable, property, operator, value } => {
            let node = match store.get_node(node_id) {
                Some(n) => n,
                None => return false,
            };
            
            let prop_value = match node.properties.get(property) {
                Some(v) => v.clone(),
                None => return false,
            };
            
            let literal_value = match value {
                Literal::Integer(i) => Value::Integer(*i),
                Literal::String(s) => Value::String(s.clone()),
                Literal::Boolean(b) => Value::Boolean(*b),
            };
            
            match operator {
                ComparisonOp::Equals => prop_value == literal_value,
                ComparisonOp::NotEquals => prop_value != literal_value,
                ComparisonOp::Greater => prop_value > literal_value,
                ComparisonOp::Less => prop_value < literal_value,
                ComparisonOp::GreaterEq => prop_value >= literal_value,
                ComparisonOp::LessEq => prop_value <= literal_value,
            }
        }
        CypherPredicate::And(left, right) => {
            evaluate_predicate(left, node_id, store) && evaluate_predicate(right, node_id, store)
        }
        CypherPredicate::Or(left, right) => {
            evaluate_predicate(left, node_id, store) || evaluate_predicate(right, node_id, store)
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-graph cypher_executor_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/graph/src/cypher/executor.rs crates/graph/tests/cypher_executor_test.rs
git commit -m "feat(graph): add Cypher executor with pattern matching"
```

---

## Task 4: 集成到 Graph 模块

**Files:**
- Modify: `crates/graph/src/lib.rs`
- Modify: `crates/graph/src/cypher/mod.rs`
- Test: `crates/graph/tests/cypher_integration_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_cypher_integration() {
    let mut store = InMemoryGraphStore::new();
    
    // Create GMP traceability chain
    let batch_id = store.create_node("Batch", prop!{"id": "B202401"});
    let device_id = store.create_node("Device", prop!{"id": "D001"});
    let product_id = store.create_node("Product", prop!{"id": "P100"});
    
    store.create_edge(batch_id, device_id, "USED_IN", PropertyMap::new()).unwrap();
    store.create_edge(device_id, product_id, "PRODUCES", PropertyMap::new()).unwrap();
    
    // Test MATCH
    let result = cypher::execute("MATCH (b:Batch) RETURN b.id").unwrap();
    assert_eq!(result.rows.len(), 1);
    
    // Test relationship
    let result = cypher::execute("MATCH (b:Batch)-[:USED_IN]->(d:Device) RETURN b.id, d.id").unwrap();
    assert_eq!(result.rows.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph cypher_integration_test -- --nocapture`
Expected: FAIL - module not found

**Step 3: Update module structure**

```rust
// crates/graph/src/cypher/mod.rs
pub mod lexer;
pub mod parser;
pub mod executor;

pub use lexer::{CypherLexer, CypherToken};
pub use parser::{CypherQuery, CypherPattern, NodePattern, ReturnItem};
pub use executor::{CypherResult, execute_cypher};

// Convenience function
pub fn execute(query: &str) -> Result<CypherResult, GraphError> {
    execute_cypher(query, &super::store::InMemoryGraphStore::new())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p sqlrustgo-graph cypher_integration_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/graph/src/cypher/mod.rs crates/graph/src/lib.rs
git commit -m "feat(graph): integrate Cypher into graph module"
```

---

## Task 5: GMP 场景测试

**Files:**
- Create: `crates/graph/tests/cypher_gmp_test.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_gmp_traceability_query() {
    let mut store = InMemoryGraphStore::new();
    
    // Setup GMP chain
    let batch = store.create_node("Batch", prop!{"id": "B202401", "material": "Steel"});
    let device = store.create_node("Device", prop!{"id": "D001", "type": "CNC"});
    let calibration = store.create_node("Calibration", prop!{"id": "C001", "result": "PASS"});
    let product = store.create_node("Product", prop!{"id": "P100", "quality": "A"});
    
    store.create_edge(batch, device, "USED_IN", PropertyMap::new()).unwrap();
    store.create_edge(device, calibration, "CALIBRATED_BY", PropertyMap::new()).unwrap();
    store.create_edge(device, product, "PRODUCES", PropertyMap::new()).unwrap();
    
    // Query: find product made by device used for batch
    let result = cypher::execute(
        "MATCH (b:Batch)-[:USED_IN]->(d:Device)-[:PRODUCES]->(p:Product) WHERE b.id = 'B202401' RETURN p.id"
    ).unwrap();
    
    assert_eq!(result.rows.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p sqlrustgo-graph cypher_gmp_test -- --nocapture`
Expected: FAIL

**Step 3: Verify tests pass after implementation**

Run: `cargo test -p sqlrustgo-graph cypher_gmp_test -- --nocapture`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/graph/tests/cypher_gmp_test.rs
git commit -m "test(graph): add GMP traceability Cypher queries"
```

---

## Verification

### Build Check
```bash
cargo build -p sqlrustgo-graph
```

### Test Check
```bash
cargo test -p sqlrustgo-graph --lib
cargo test -p sqlrustgo-graph --test graph_tests
cargo test -p sqlrustgo-graph --test cypher_lexer_test
cargo test -p sqlrustgo-graph --test cypher_parser_test
cargo test -p sqlrustgo-graph --test cypher_executor_test
cargo test -p sqlrustgo-graph --test cypher_integration_test
cargo test -p sqlrustgo-graph --test cypher_gmp_test
```

### Expected Results
- All 55+ graph tests pass
- All 11 integration tests pass
- New Cypher tests: 15+ pass

---

## Files Summary

### New Files
- `crates/graph/src/cypher/mod.rs`
- `crates/graph/src/cypher/lexer.rs`
- `crates/graph/src/cypher/parser.rs`
- `crates/graph/src/cypher/executor.rs`
- `crates/graph/tests/cypher_lexer_test.rs`
- `crates/graph/tests/cypher_parser_test.rs`
- `crates/graph/tests/cypher_executor_test.rs`
- `crates/graph/tests/cypher_integration_test.rs`
- `crates/graph/tests/cypher_gmp_test.rs`

### Modified Files
- `crates/graph/src/lib.rs`
- `crates/graph/Cargo.toml` (add dependencies if needed)
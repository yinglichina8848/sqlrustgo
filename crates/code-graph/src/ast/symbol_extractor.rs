//! Symbol extractor using `syn`
//!
//! Parses Rust source files and extracts structural symbols

use std::cell::Cell;
use std::fs;
use std::path::Path;
use syn::{parse_file, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait};

/// Maximum function complexity we track
const MAX_COMPLEXITY: f32 = 100.0;

/// Extracted symbol from Rust source
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: String, // fn, struct, trait, enum, impl, mod, test
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: Option<String>,
    pub module_path: Vec<String>,
    pub is_public: bool,
    pub is_test: bool,
    pub complexity: Option<f32>,
}

/// Extract all symbols from a Rust source file
pub fn extract_symbols_from_file(
    file_path: &Path,
    module_path: &[String],
) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let file = parse_file(&content)?;
    let file_path_str = file_path.to_string_lossy().to_string();
    let line_offsets = compute_line_offsets(&content);
    let mut symbols = Vec::new();
    let offset = Cell::new(0usize);

    for item in &file.items {
        let _item_start = offset.get();
        extract_item(
            item,
            &file_path_str,
            module_path,
            &line_offsets,
            &mut symbols,
            &offset,
        );
    }

    Ok(symbols)
}

/// Compute byte offset for each line start (0-indexed lines)
fn compute_line_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (i, c) in content.char_indices() {
        if c == '\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Convert a byte offset to a 1-indexed line number
#[allow(dead_code)]
fn offset_to_line(offset: usize, offsets: &[usize]) -> usize {
    offsets.iter().take_while(|&&o| o <= offset).count()
}

/// Extract all symbols from a parsed Rust file AST
pub fn extract_symbols_from_ast(
    content: &str,
    file_path: &str,
    module_path: &[String],
) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let file = parse_file(content)?;
    let line_offsets = compute_line_offsets(content);
    let mut symbols = Vec::new();
    let offset = Cell::new(0usize);

    for item in &file.items {
        extract_item(
            item,
            file_path,
            module_path,
            &line_offsets,
            &mut symbols,
            &offset,
        );
    }

    Ok(symbols)
}

fn extract_item(
    item: &Item,
    file_path: &str,
    module_path: &[String],
    line_offsets: &[usize],
    symbols: &mut Vec<Symbol>,
    _offset: &Cell<usize>,
) {
    match item {
        Item::Fn(fn_item) => {
            symbols.push(extract_fn(fn_item, file_path, module_path, line_offsets));
        }
        Item::Struct(struct_item) => {
            symbols.push(extract_struct(
                struct_item,
                file_path,
                module_path,
                line_offsets,
            ));
        }
        Item::Trait(trait_item) => {
            symbols.push(extract_trait(
                trait_item,
                file_path,
                module_path,
                line_offsets,
            ));
        }
        Item::Enum(enum_item) => {
            symbols.push(extract_enum(
                enum_item,
                file_path,
                module_path,
                line_offsets,
            ));
        }
        Item::Impl(impl_item) => {
            symbols.push(extract_impl(
                impl_item,
                file_path,
                module_path,
                line_offsets,
            ));
        }
        Item::Mod(mod_item) => {
            let mut new_mod_path = module_path.to_vec();
            let ident = &mod_item.ident;
            new_mod_path.push(ident.to_string());

            let line_start = 1; // syn doesn't give us byte offsets without extra deps
            let line_end = line_start + 1;
            let is_public = matches!(mod_item.vis, syn::Visibility::Public(_));

            symbols.push(Symbol {
                name: ident.to_string(),
                kind: "mod".to_string(),
                file_path: file_path.to_string(),
                line_start,
                line_end,
                signature: None,
                module_path: module_path.to_vec(),
                is_public,
                is_test: false,
                complexity: None,
            });

            // Inline module content
            if let Some((_, items)) = &mod_item.content {
                for item in items {
                    extract_item(
                        item,
                        file_path,
                        &new_mod_path,
                        line_offsets,
                        symbols,
                        _offset,
                    );
                }
            }
        }
        _ => {}
    }
}

fn extract_fn(
    item: &ItemFn,
    file_path: &str,
    module_path: &[String],
    _line_offsets: &[usize],
) -> Symbol {
    let ident = &item.sig.ident;
    let line_start = 1;
    let line_end = line_start + 1;
    let is_test = item.attrs.iter().any(|a| a.path().is_ident("test"))
        || ident.to_string().starts_with("test_");
    let is_public = matches!(item.vis, syn::Visibility::Public(_));
    let raw_sig = format!("{}", quote::quote! { #item.sig });
    let signature = Some(raw_sig.chars().take(250).collect::<String>());
    let complexity = estimate_complexity(&item.block);

    Symbol {
        name: ident.to_string(),
        kind: "fn".to_string(),
        file_path: file_path.to_string(),
        line_start,
        line_end,
        signature,
        module_path: module_path.to_vec(),
        is_public,
        is_test,
        complexity: Some(complexity),
    }
}

fn extract_struct(
    item: &ItemStruct,
    file_path: &str,
    module_path: &[String],
    _line_offsets: &[usize],
) -> Symbol {
    let ident = &item.ident;
    let line_start = 1;
    let line_end = line_start + 1;
    let is_public = matches!(item.vis, syn::Visibility::Public(_));

    Symbol {
        name: ident.to_string(),
        kind: "struct".to_string(),
        file_path: file_path.to_string(),
        line_start,
        line_end,
        signature: None,
        module_path: module_path.to_vec(),
        is_public,
        is_test: false,
        complexity: None,
    }
}

fn extract_trait(
    item: &ItemTrait,
    file_path: &str,
    module_path: &[String],
    _line_offsets: &[usize],
) -> Symbol {
    let ident = &item.ident;
    let line_start = 1;
    let line_end = line_start + 1;
    let is_public = matches!(item.vis, syn::Visibility::Public(_));

    Symbol {
        name: ident.to_string(),
        kind: "trait".to_string(),
        file_path: file_path.to_string(),
        line_start,
        line_end,
        signature: None,
        module_path: module_path.to_vec(),
        is_public,
        is_test: false,
        complexity: None,
    }
}

fn extract_enum(
    item: &ItemEnum,
    file_path: &str,
    module_path: &[String],
    _line_offsets: &[usize],
) -> Symbol {
    let ident = &item.ident;
    let line_start = 1;
    let line_end = line_start + 1;
    let is_public = matches!(item.vis, syn::Visibility::Public(_));

    Symbol {
        name: ident.to_string(),
        kind: "enum".to_string(),
        file_path: file_path.to_string(),
        line_start,
        line_end,
        signature: None,
        module_path: module_path.to_vec(),
        is_public,
        is_test: false,
        complexity: None,
    }
}

fn extract_impl(
    item: &ItemImpl,
    file_path: &str,
    module_path: &[String],
    _line_offsets: &[usize],
) -> Symbol {
    let line_start = 1;
    let line_end = line_start + 1;

    let name = if let Some((_, path, _)) = &item.trait_ {
        format!(
            "impl_{}",
            path.segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("_")
        )
    } else {
        format!("impl_{}", "self")
    };

    Symbol {
        name,
        kind: "impl".to_string(),
        file_path: file_path.to_string(),
        line_start,
        line_end,
        signature: None,
        module_path: module_path.to_vec(),
        is_public: true,
        is_test: false,
        complexity: None,
    }
}

fn estimate_complexity(block: &syn::Block) -> f32 {
    let block_str = format!("{block:#?}");
    let mut complexity = 1.0f32;
    complexity += (block_str.matches("If").count() + block_str.matches("Match").count()) as f32;
    complexity += block_str.matches("For").count() as f32;
    complexity += block_str.matches("While").count() as f32;
    complexity.min(MAX_COMPLEXITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_rust_code(code: &str) -> Vec<Symbol> {
        extract_symbols_from_ast(code, "test.rs", &[]).unwrap()
    }

    #[test]
    fn test_extract_fn() {
        let code = r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
fn internal(x: i32) -> i32 { x * 2 }
"#;
        let symbols = parse_rust_code(code);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "add");
        assert!(symbols[0].is_public);
    }

    #[test]
    fn test_extract_struct() {
        let code = r#"pub struct User { name: String }"#;
        let symbols = parse_rust_code(code);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].kind, "struct");
        assert!(symbols[0].is_public);
    }

    #[test]
    fn test_extract_test() {
        let code = r#"#[test] fn test_basic() { assert_eq!(1, 1); }"#;
        let symbols = parse_rust_code(code);
        let test_sym = symbols.iter().find(|s| s.is_test).unwrap();
        assert_eq!(test_sym.name, "test_basic");
    }

    #[test]
    fn test_complexity() {
        let code = r#"
fn complex(a: i32) -> i32 {
    if a > 0 { match a { 1 => 1, 2 => 2, _ => 3 } } else { 0 }
}
"#;
        let symbols = parse_rust_code(code);
        let c = symbols[0].complexity.unwrap();
        assert!(c >= 3.0, "Expected >= 3, got {}", c);
    }
}

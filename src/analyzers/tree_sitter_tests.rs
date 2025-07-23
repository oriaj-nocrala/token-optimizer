//! Tree-sitter integration tests
//! Tests for verifying tree-sitter parsers are working correctly with modern API

#[cfg(test)]
mod tests {
    use tree_sitter::{Parser, Language};
    use anyhow::Result;
    use std::collections::HashMap;

    #[test]
    fn test_tree_sitter_rust_parser_creation() {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_rust::LANGUAGE.into();
        let result = parser.set_language(&language);
        assert!(result.is_ok(), "Failed to set Rust language for parser");
    }

    #[test]
    fn test_tree_sitter_typescript_parser_creation() {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let result = parser.set_language(&language);
        assert!(result.is_ok(), "Failed to set TypeScript language for parser");
    }

    #[test]
    fn test_tree_sitter_javascript_parser_creation() {
        let mut parser = Parser::new();
        let language: Language = tree_sitter_javascript::LANGUAGE.into();
        let result = parser.set_language(&language);
        assert!(result.is_ok(), "Failed to set JavaScript language for parser");
    }

    #[test]
    fn test_parse_simple_rust_code() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let source_code = r#"
fn main() {
    println!("Hello, world!");
}

pub struct Person {
    name: String,
    age: u32,
}

impl Person {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}
        "#;

        let tree = parser.parse(source_code, None);
        assert!(tree.is_some(), "Failed to parse Rust code");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        // Should have parsed successfully
        assert!(!root_node.has_error(), "Parse tree contains errors");
        assert_eq!(root_node.kind(), "source_file");
        
        // Should contain main function and struct
        let source_bytes = source_code.as_bytes();
        let mut found_main = false;
        let mut found_struct = false;
        let mut found_impl = false;
        
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            match child.kind() {
                "function_item" => {
                    // Check if it's the main function
                    if let Ok(text) = child.utf8_text(source_bytes) {
                        if text.contains("fn main") {
                            found_main = true;
                        }
                    }
                }
                "struct_item" => {
                    found_struct = true;
                }
                "impl_item" => {
                    found_impl = true;
                }
                _ => {}
            }
        }
        
        assert!(found_main, "Should find main function");
        assert!(found_struct, "Should find struct");
        assert!(found_impl, "Should find impl block");
        
        Ok(())
    }

    #[test]
    fn test_parse_typescript_code() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;

        let source_code = r#"
interface User {
    name: string;
    age: number;
}

class UserService {
    private users: User[] = [];
    
    addUser(user: User): void {
        this.users.push(user);
    }
    
    getUsers(): User[] {
        return this.users;
    }
}

export { User, UserService };
        "#;

        let tree = parser.parse(source_code, None);
        assert!(tree.is_some(), "Failed to parse TypeScript code");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        assert!(!root_node.has_error(), "Parse tree contains errors");
        assert_eq!(root_node.kind(), "program");
        
        // Should contain interface and class
        let mut found_interface = false;
        let mut found_class = false;
        
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            match child.kind() {
                "interface_declaration" => {
                    found_interface = true;
                }
                "class_declaration" => {
                    found_class = true;
                }
                _ => {}
            }
        }
        
        assert!(found_interface, "Should find interface declaration");
        assert!(found_class, "Should find class declaration");
        
        Ok(())
    }

    #[test]
    fn test_parse_javascript_code() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;

        let source_code = r#"
function greet(name) {
    return `Hello, ${name}!`;
}

class Calculator {
    constructor() {
        this.result = 0;
    }
    
    add(num) {
        this.result += num;
        return this;
    }
    
    multiply(num) {
        this.result *= num;
        return this;
    }
    
    getResult() {
        return this.result;
    }
}

const calc = new Calculator();
const result = calc.add(5).multiply(2).getResult();

export { greet, Calculator };
        "#;

        let tree = parser.parse(source_code, None);
        assert!(tree.is_some(), "Failed to parse JavaScript code");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        assert!(!root_node.has_error(), "Parse tree contains errors");
        assert_eq!(root_node.kind(), "program");
        
        // Should contain function and class
        let mut found_function = false;
        let mut found_class = false;
        let mut found_variable = false;
        
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            match child.kind() {
                "function_declaration" => {
                    found_function = true;
                }
                "class_declaration" => {
                    found_class = true;
                }
                "lexical_declaration" => {
                    found_variable = true;
                }
                _ => {}
            }
        }
        
        assert!(found_function, "Should find function declaration");
        assert!(found_class, "Should find class declaration");
        assert!(found_variable, "Should find variable declarations");
        
        Ok(())
    }

    #[test]
    fn test_parse_complex_rust_patterns() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let source_code = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending { reason: String },
    Error(String),
}

pub trait Configurable {
    fn configure(&mut self, config: &Config) -> Result<(), String>;
}

impl Configurable for MyService {
    fn configure(&mut self, config: &Config) -> Result<(), String> {
        self.port = config.port;
        Ok(())
    }
}

pub struct MyService {
    config: Config,
    port: u16,
}

impl MyService {
    pub fn new() -> Self {
        Self {
            config: Config {
                database_url: "localhost".to_string(),
                port: 3000,
                debug: false,
            },
            port: 3000,
        }
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format!($($arg)*));
    };
}

const DEFAULT_PORT: u16 = 8080;
static mut GLOBAL_COUNTER: i32 = 0;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_creation() {
        let service = MyService::new();
        assert_eq!(service.port, 3000);
    }
}
        "#;

        let tree = parser.parse(source_code, None);
        assert!(tree.is_some(), "Failed to parse complex Rust code");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        assert!(!root_node.has_error(), "Parse tree contains errors");
        
        // Count different node types
        let mut counts = HashMap::new();
        let mut cursor = root_node.walk();
        
        fn count_nodes(node: tree_sitter::Node, counts: &mut HashMap<String, i32>) {
            let kind = node.kind().to_string();
            *counts.entry(kind).or_insert(0) += 1;
            
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                count_nodes(child, counts);
            }
        }
        
        count_nodes(root_node, &mut counts);
        
        // Verify we found the expected constructs
        assert!(counts.get("use_declaration").unwrap_or(&0) > &0, "Should find use declarations");
        assert!(counts.get("struct_item").unwrap_or(&0) > &0, "Should find struct items");
        assert!(counts.get("enum_item").unwrap_or(&0) > &0, "Should find enum items");
        assert!(counts.get("trait_item").unwrap_or(&0) > &0, "Should find trait items");
        assert!(counts.get("impl_item").unwrap_or(&0) > &0, "Should find impl items");
        assert!(counts.get("macro_definition").unwrap_or(&0) > &0, "Should find macro definitions");
        assert!(counts.get("const_item").unwrap_or(&0) > &0, "Should find const items");
        assert!(counts.get("static_item").unwrap_or(&0) > &0, "Should find static items");
        assert!(counts.get("type_item").unwrap_or(&0) > &0, "Should find type items");
        
        Ok(())
    }

    #[test]
    fn test_parse_error_handling() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        // Intentionally malformed Rust code
        let bad_source_code = r#"
fn main( {
    let x = ;
    if true {
        println!("test"
    }
}
        "#;

        let tree = parser.parse(bad_source_code, None);
        assert!(tree.is_some(), "Parser should still return a tree for malformed code");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        // Should detect errors in the parse tree
        assert!(root_node.has_error(), "Should detect parse errors in malformed code");
        
        Ok(())
    }

    #[test]
    fn test_parse_empty_file() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let empty_source = "";
        let tree = parser.parse(empty_source, None);
        assert!(tree.is_some(), "Should parse empty file");

        let tree = tree.unwrap();
        let root_node = tree.root_node();
        
        assert!(!root_node.has_error(), "Empty file should not have parse errors");
        assert_eq!(root_node.kind(), "source_file");
        assert_eq!(root_node.child_count(), 0, "Empty file should have no children");
        
        Ok(())
    }

    #[test]
    fn test_node_text_extraction() -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;

        let source_code = r#"
pub fn hello_world() -> &'static str {
    "Hello, World!"
}
        "#;

        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();
        let source_bytes = source_code.as_bytes();
        
        // Find the function node
        let mut cursor = root_node.walk();
        for child in root_node.children(&mut cursor) {
            if child.kind() == "function_item" {
                // Test text extraction
                let function_text = child.utf8_text(source_bytes)?;
                assert!(function_text.contains("hello_world"), "Should extract function name");
                assert!(function_text.contains("Hello, World!"), "Should extract function body");
                
                // Test position information
                let start_pos = child.start_position();
                let end_pos = child.end_position();
                assert!(start_pos.row < end_pos.row, "End row should be after start row");
                
                break;
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_parser_reusability() -> Result<()> {
        let mut parser = Parser::new();
        
        // Test switching between languages
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
        let rust_tree = parser.parse("fn main() {}", None);
        assert!(rust_tree.is_some());
        
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;
        let js_tree = parser.parse("function main() {}", None);
        assert!(js_tree.is_some());
        
        parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;
        let ts_tree = parser.parse("function main(): void {}", None);
        assert!(ts_tree.is_some());
        
        Ok(())
    }
}
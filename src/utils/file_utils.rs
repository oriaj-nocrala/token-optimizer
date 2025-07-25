use anyhow::Result;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::types::FileType;

pub fn read_file_content(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(_) => {
            // If UTF-8 reading fails, try to read as bytes and convert with lossy conversion
            let bytes = fs::read(path)?;
            let content = String::from_utf8_lossy(&bytes).to_string();
            Ok(content)
        }
    }
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)?.len())
}

pub fn count_lines(content: &str) -> usize {
    content.lines().count()
}

pub fn detect_file_type(path: &Path) -> FileType {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    // Check for test files first (both TypeScript and Rust patterns)
    if path_str.contains(".spec.") || path_str.contains(".test.") || 
       (path_str.contains("test") && path_str.ends_with(".rs")) {
        return FileType::Test;
    }
    
    // Special Rust files
    match file_name {
        "Cargo.toml" | "Cargo.lock" => return FileType::Config,
        "lib.rs" | "main.rs" => return FileType::Other, // Will be refined by content analysis
        "mod.rs" => return FileType::Other,
        _ => {}
    }
    
    match path.extension().and_then(|s| s.to_str()) {
        Some("rs") => {
            // Rust file type detection based on path patterns (support both / and \ separators)
            if path_str.contains("/bin/") || path_str.contains("\\bin\\") {
                FileType::Other // Binary
            } else if path_str.contains("/examples/") || path_str.contains("\\examples\\") {
                FileType::Other // Example
            } else if path_str.contains("/tests/") || path_str.contains("\\tests\\") || 
                      path_str.starts_with("tests/") || path_str.starts_with("tests\\") {
                FileType::Test
            } else if path_str.contains("/benches/") || path_str.contains("\\benches\\") ||
                      path_str.starts_with("benches/") || path_str.starts_with("benches\\") {
                FileType::Test // Benchmark
            } else {
                // Default Rust file - will be refined by content analysis
                FileType::Other
            }
        }
        Some("ts") => {
            if path_str.contains("component") {
                FileType::Component
            } else if path_str.contains("service") {
                FileType::Service
            } else {
                // For other .ts files, we'll determine type from content analysis later
                FileType::Other
            }
        }
        Some("scss") | Some("css") => FileType::Style,
        Some("json") => FileType::Config,
        Some("toml") => FileType::Config,
        Some("md") => FileType::Other, // Documentation
        _ => FileType::Other,
    }
}

// New function to detect file type from content for better accuracy
pub fn detect_file_type_from_content(path: &Path, content: &str) -> FileType {
    // First check file name patterns
    let basic_type = detect_file_type(path);
    
    // For Rust files, we can refine the type based on content
    if path.extension().and_then(|s| s.to_str()) == Some("rs") {
        return refine_rust_file_type(path, content, basic_type);
    }
    
    // For Cargo.toml files
    if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        return FileType::Cargo;
    }
    
    // For TypeScript/JavaScript files, analyze content for Angular patterns
    if matches!(path.extension().and_then(|s| s.to_str()), Some("ts") | Some("js")) {
        return refine_typescript_file_type(path, content, basic_type);
    }
    
    basic_type
}

fn refine_typescript_file_type(path: &Path, content: &str, basic_type: FileType) -> FileType {
    // Check for Angular patterns in content
    if content.contains("@Component") {
        return FileType::Component;
    }
    
    if content.contains("@Injectable") {
        return FileType::Service;
    }
    
    if content.contains("@Pipe") || content.contains("implements PipeTransform") {
        return FileType::Pipe;
    }
    
    if content.contains("@NgModule") {
        return FileType::Module;
    }
    
    // If no specific Angular patterns found, return basic type
    basic_type
}

fn refine_rust_file_type(path: &Path, content: &str, _basic_type: FileType) -> FileType {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    // Special file names have priority
    match file_name {
        "lib.rs" => return FileType::RustLibrary,
        "main.rs" => return FileType::RustBinary,
        "mod.rs" => return FileType::RustModule,
        _ => {}
    }
    
    // Check content patterns for test detection
    if content.contains("#[cfg(test)]") || 
       content.contains("#[test]") ||
       content.contains("mod tests") {
        return FileType::RustTest;
    }
    
    // Check for benchmark patterns
    if content.contains("#[bench]") ||
       content.contains("test::Bencher") ||
       path.to_string_lossy().contains("benches") {
        return FileType::RustBench;
    }
    
    // Check for binary patterns
    if content.contains("fn main()") {
        return FileType::RustBinary;
    }
    
    // Default to module for other .rs files
    FileType::RustModule
}

/// Calculate complexity based on various metrics
pub fn calculate_complexity(content: &str, line_count: usize) -> crate::types::Complexity {
    // Count language-agnostic complexity patterns
    let mut total_complexity = 0;
    
    // Rust-specific patterns
    total_complexity += content.matches("fn ").count();
    total_complexity += content.matches("struct ").count();
    total_complexity += content.matches("enum ").count();
    total_complexity += content.matches("trait ").count();
    total_complexity += content.matches("impl ").count();
    
    // TypeScript/JavaScript patterns
    total_complexity += content.matches("function ").count();
    total_complexity += content.matches("class ").count();
    total_complexity += content.matches("interface ").count();
    total_complexity += content.matches("async ").count();
    
    // Control flow complexity (cyclomatic complexity approximation)
    let cyclomatic = calculate_cyclomatic_complexity(content);
    let complexity_score = total_complexity as f64 + cyclomatic / 5.0;
    
    if complexity_score > 15.0 || line_count > 500 {
        crate::types::Complexity::High
    } else if complexity_score >= 3.0 || line_count > 200 {
        crate::types::Complexity::Medium
    } else {
        crate::types::Complexity::Low
    }
}

fn calculate_cyclomatic_complexity(content: &str) -> f64 {
    let mut complexity = 1.0;
    
    // Count decision points
    complexity += content.matches("if ").count() as f64;
    complexity += content.matches("else ").count() as f64;
    complexity += content.matches("for ").count() as f64;
    complexity += content.matches("while ").count() as f64;
    complexity += content.matches("switch ").count() as f64;
    complexity += content.matches("case ").count() as f64;
    complexity += content.matches("catch ").count() as f64;
    complexity += content.matches(" && ").count() as f64;
    complexity += content.matches(" || ").count() as f64;
    
    complexity
}

pub fn walk_project_files(root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && !is_ignored_file(path) {
            // Support for hybrid projects with multiple languages
            if let Some(extension) = path.extension() {
                if matches!(extension.to_str(), 
                    Some("ts") | Some("js") | Some("scss") | Some("css") | 
                    Some("json") | Some("rs") | Some("toml")) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    Ok(files)
}

pub fn is_ignored_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    path_str.contains("node_modules") ||
    path_str.contains(".git") ||
    path_str.contains("dist") ||
    path_str.contains("build") ||
    path_str.contains("target") ||
    path_str.ends_with(".min.js") ||
    path_str.ends_with(".min.css")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use crate::types::Complexity;

    #[test]
    fn test_count_lines() {
        assert_eq!(count_lines(""), 0);
        assert_eq!(count_lines("single line"), 1);
        assert_eq!(count_lines("line 1\nline 2"), 2);
        assert_eq!(count_lines("line 1\nline 2\nline 3"), 3);
        assert_eq!(count_lines("line 1\n\nline 3"), 3);
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type(Path::new("app.component.ts")), FileType::Component);
        assert_eq!(detect_file_type(Path::new("user.service.ts")), FileType::Service);
        assert_eq!(detect_file_type(Path::new("utils.ts")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("styles.scss")), FileType::Style);
        assert_eq!(detect_file_type(Path::new("styles.css")), FileType::Style);
        assert_eq!(detect_file_type(Path::new("config.json")), FileType::Config);
        assert_eq!(detect_file_type(Path::new("app.spec.ts")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("app.test.ts")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("README.md")), FileType::Other);
    }
    
    #[test]
    fn test_detect_file_type_from_content() {
        let component_content = r#"
        import { Component } from '@angular/core';
        
        @Component({
          selector: 'app-test',
          template: '<div>Test</div>'
        })
        export class TestComponent { }
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("test.ts"), component_content), FileType::Component);
        
        let service_content = r#"
        import { Injectable } from '@angular/core';
        
        @Injectable({
          providedIn: 'root'
        })
        export class TestService { }
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("test.ts"), service_content), FileType::Service);
        
        let pipe_content = r#"
        import { Pipe, PipeTransform } from '@angular/core';
        
        @Pipe({
          name: 'testPipe'
        })
        export class TestPipe implements PipeTransform {
          transform(value: any): any {
            return value;
          }
        }
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("test.ts"), pipe_content), FileType::Pipe);
        
        let pipe_transform_content = r#"
        import { PipeTransform } from '@angular/core';
        
        export class CustomPipe implements PipeTransform {
          transform(value: string, ...args: any[]): string {
            return value.toUpperCase();
          }
        }
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("test.ts"), pipe_transform_content), FileType::Pipe);
        
        let module_content = r#"
        import { NgModule } from '@angular/core';
        import { CommonModule } from '@angular/common';
        
        @NgModule({
          imports: [CommonModule],
          declarations: [],
          exports: []
        })
        export class TestModule { }
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("test.ts"), module_content), FileType::Module);
    }

    #[test]
    fn test_calculate_complexity() {
        let simple_content = "function hello() { return 'world'; }";
        assert_eq!(calculate_complexity(simple_content, 1), Complexity::Low);

        let medium_content = "function test() { if (true) { for (let i = 0; i < 10; i++) { console.log(i); } } }";
        assert_eq!(calculate_complexity(medium_content, 1), Complexity::Low);

        let complex_content = "function complex() { if (a) { if (b) { if (c) { while (d) { for (e) { if (f) { switch (g) { case 1: break; case 2: break; } } } } } } } }";
        assert_eq!(calculate_complexity(complex_content, 1), Complexity::Medium);

        let very_complex_content = "x".repeat(2000);
        assert_eq!(calculate_complexity(&very_complex_content, 2000), Complexity::High);
    }

    #[test]
    fn test_calculate_cyclomatic_complexity() {
        assert_eq!(calculate_cyclomatic_complexity("function hello() { return 'world'; }"), 1.0);
        assert_eq!(calculate_cyclomatic_complexity("if (true) { }"), 2.0);
        assert_eq!(calculate_cyclomatic_complexity("if (true) { } else { }"), 3.0);
        assert_eq!(calculate_cyclomatic_complexity("for (let i = 0; i < 10; i++) { }"), 2.0);
        assert_eq!(calculate_cyclomatic_complexity("while (true) { }"), 2.0);
        assert_eq!(calculate_cyclomatic_complexity("switch (x) { case 1: break; case 2: break; }"), 4.0); // switch + 2 cases
        assert_eq!(calculate_cyclomatic_complexity("try { } catch (e) { }"), 2.0);
        assert_eq!(calculate_cyclomatic_complexity("if (a && b) { }"), 3.0);
        assert_eq!(calculate_cyclomatic_complexity("if (a || b) { }"), 3.0);
    }

    #[test]
    fn test_is_ignored_file() {
        assert!(is_ignored_file(Path::new("node_modules/package/index.js")));
        assert!(is_ignored_file(Path::new(".git/config")));
        assert!(is_ignored_file(Path::new("dist/main.js")));
        assert!(is_ignored_file(Path::new("build/output.js")));
        assert!(is_ignored_file(Path::new("target/debug/main")));
        assert!(is_ignored_file(Path::new("script.min.js")));
        assert!(is_ignored_file(Path::new("styles.min.css")));
        
        assert!(!is_ignored_file(Path::new("src/main.ts")));
        assert!(!is_ignored_file(Path::new("src/components/app.component.ts")));
        assert!(!is_ignored_file(Path::new("styles.scss")));
        assert!(!is_ignored_file(Path::new("package.json")));
    }

    #[test]
    fn test_read_file_content() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = "Test file content\nSecond line";
        write!(temp_file, "{}", test_content)?;
        
        let content = read_file_content(temp_file.path())?;
        assert_eq!(content, test_content);
        Ok(())
    }

    #[test]
    fn test_get_file_size() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = "Test content";
        write!(temp_file, "{}", test_content)?;
        
        let size = get_file_size(temp_file.path())?;
        assert_eq!(size, test_content.len() as u64);
        Ok(())
    }

    #[test]
    fn test_walk_project_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        // Create test files
        fs::write(temp_path.join("test.ts"), "// TypeScript file")?;
        fs::write(temp_path.join("test.js"), "// JavaScript file")?;
        fs::write(temp_path.join("styles.scss"), "// SCSS file")?;
        fs::write(temp_path.join("styles.css"), "// CSS file")?;
        fs::write(temp_path.join("config.json"), "// JSON file")?;
        fs::write(temp_path.join("README.md"), "// Markdown file")?;
        
        // Create subdirectory
        fs::create_dir(temp_path.join("src"))?;
        fs::write(temp_path.join("src/main.ts"), "// Main TypeScript file")?;
        
        let files = walk_project_files(temp_path)?;
        
        // Should include supported file types
        assert!(files.iter().any(|f| f.ends_with("test.ts")));
        assert!(files.iter().any(|f| f.ends_with("test.js")));
        assert!(files.iter().any(|f| f.ends_with("styles.scss")));
        assert!(files.iter().any(|f| f.ends_with("styles.css")));
        assert!(files.iter().any(|f| f.ends_with("config.json")));
        assert!(files.iter().any(|f| f.ends_with("src/main.ts")));
        
        // Should exclude unsupported file types
        assert!(!files.iter().any(|f| f.ends_with("README.md")));
        
        Ok(())
    }

    #[test]
    fn test_complexity_edge_cases() {
        let empty_content = "";
        assert_eq!(calculate_complexity(empty_content, 0), Complexity::Low);
        
        let single_line = "const x = 1;";
        assert_eq!(calculate_complexity(single_line, 1), Complexity::Low);
        
        let medium_lines = "x".repeat(500);
        assert_eq!(calculate_complexity(&medium_lines, 500), Complexity::Medium); // 1 + 5.0 = 6.0 which is Medium
        
        let many_lines = "x".repeat(1500);
        assert_eq!(calculate_complexity(&many_lines, 1500), Complexity::High); // 1 + 15.0 = 16.0 which is High
    }

    #[test]
    fn test_file_type_edge_cases() {
        assert_eq!(detect_file_type(Path::new("file.unknown")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("file")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("my-component.ts")), FileType::Component);
        assert_eq!(detect_file_type(Path::new("my-service.ts")), FileType::Service);
        assert_eq!(detect_file_type(Path::new("my_component.ts")), FileType::Component);
        assert_eq!(detect_file_type(Path::new("my_service.ts")), FileType::Service);
    }

    #[test]
    fn test_complexity_with_nested_structures() {
        let nested_content = r#"
        function complexFunction() {
            if (condition1) {
                for (let i = 0; i < 10; i++) {
                    if (condition2) {
                        while (condition3) {
                            switch (value) {
                                case 1:
                                    if (condition4 && condition5) {
                                        break;
                                    }
                                case 2:
                                    if (condition6 || condition7) {
                                        break;
                                    }
                                default:
                                    break;
                            }
                        }
                    }
                }
            } else {
                try {
                    // some code
                } catch (error) {
                    // error handling
                }
            }
        }
        "#;
        
        // This has: 3 if's + 1 for + 1 while + 1 switch + 2 cases + 1 else + 1 catch + 2 &&/|| = 12 + 1 (base) = 13
        // Plus 30/100 = 0.3, so total = 13.3, which is Medium
        let complexity = calculate_complexity(nested_content, 30);
        assert_eq!(complexity, Complexity::Medium);
    }

    #[test]
    fn test_read_file_content_with_utf8_issues() -> Result<()> {
        // Test with valid UTF-8 content
        let mut temp_file = NamedTempFile::new()?;
        let utf8_content = "Test file with válid UTF-8 characters: ñáéíóú";
        write!(temp_file, "{}", utf8_content)?;
        
        let content = read_file_content(temp_file.path())?;
        assert_eq!(content, utf8_content);
        
        // Test with ISO-8859-1 (Latin-1) encoded content that would cause UTF-8 errors
        let mut temp_file2 = NamedTempFile::new()?;
        // This creates bytes that are invalid UTF-8 but valid ISO-8859-1
        let iso_bytes = vec![
            // "Test content with special chars: " in ASCII
            0x54, 0x65, 0x73, 0x74, 0x20, 0x63, 0x6f, 0x6e, 0x74, 0x65, 0x6e, 0x74, 0x20, 0x77, 0x69, 0x74, 0x68, 0x20, 0x73, 0x70, 0x65, 0x63, 0x69, 0x61, 0x6c, 0x20, 0x63, 0x68, 0x61, 0x72, 0x73, 0x3a, 0x20,
            // ISO-8859-1 characters that are invalid UTF-8
            0xf1, // ñ in ISO-8859-1
            0xe1, // á in ISO-8859-1  
            0xe9, // é in ISO-8859-1
            0xed, // í in ISO-8859-1
            0xf3, // ó in ISO-8859-1
            0xfa, // ú in ISO-8859-1
        ];
        
        use std::io::Write;
        temp_file2.write_all(&iso_bytes)?;
        
        // This should not panic and should return some valid string (using lossy conversion)
        let content = read_file_content(temp_file2.path())?;
        assert!(!content.is_empty());
        assert!(content.contains("Test content with special chars"));
        // The exact characters may be replaced with replacement characters (�) but function should not fail
        
        Ok(())
    }

    #[test]
    fn test_read_file_content_with_binary_data() -> Result<()> {
        // Test with completely invalid UTF-8 (binary data)
        let mut temp_file = NamedTempFile::new()?;
        let binary_data = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD];
        
        use std::io::Write;
        temp_file.write_all(&binary_data)?;
        
        // This should not panic and should return some string (with replacement characters)
        let content = read_file_content(temp_file.path())?;
        assert!(!content.is_empty()); // Should contain replacement characters
        
        Ok(())
    }
    
    // Tests specific to Rust file type detection
    #[test]
    fn test_rust_file_type_detection() {
        // Test special Rust files
        assert_eq!(detect_file_type(Path::new("lib.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("main.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("mod.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("Cargo.toml")), FileType::Config);
        assert_eq!(detect_file_type(Path::new("Cargo.lock")), FileType::Config);
        
        // Test path-based detection
        assert_eq!(detect_file_type(Path::new("src/bin/tool.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("examples/demo.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("tests/integration.rs")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("benches/benchmark.rs")), FileType::Test);
        
        // Test pattern-based detection
        assert_eq!(detect_file_type(Path::new("lib.test.rs")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("module.spec.rs")), FileType::Test);
    }
    
    #[test]
    fn test_refine_rust_file_type() {
        // Test lib.rs detection
        let lib_content = r#"
//! Library documentation
pub mod utils;
pub use utils::*;
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("lib.rs"), lib_content), FileType::RustLibrary);
        
        // Test main.rs detection
        let main_content = r#"
fn main() {
    println!("Hello, world!");
}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("main.rs"), main_content), FileType::RustBinary);
        
        // Test module detection
        let mod_content = r#"
pub struct MyStruct;
pub fn my_function() {}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("mod.rs"), mod_content), FileType::RustModule);
        
        // Test test file detection by content
        let test_content = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("utils.rs"), test_content), FileType::RustTest);
        
        // Test benchmark detection
        let bench_content = r#"
use test::Bencher;

#[bench]
fn bench_function(b: &mut Bencher) {
    b.iter(|| {
        // benchmark code
    });
}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("bench.rs"), bench_content), FileType::RustBench);
        
        // Test binary detection by main function
        let binary_content = r#"
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Args: {:?}", args);
}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("tool.rs"), binary_content), FileType::RustBinary);
        
        // Test Cargo.toml detection
        let cargo_content = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("Cargo.toml"), cargo_content), FileType::Cargo);
        
        // Test default module case
        let module_content = r#"
pub struct Config {
    pub debug: bool,
}

impl Config {
    pub fn new() -> Self {
        Self { debug: false }
    }
}
        "#;
        assert_eq!(detect_file_type_from_content(Path::new("config.rs"), module_content), FileType::RustModule);
    }
    
    #[test]
    fn test_rust_complexity_calculation() {
        // Test Rust-specific complexity patterns
        let simple_rust = r#"
pub fn hello() -> &'static str {
    "Hello, world!"
}
        "#;
        assert_eq!(calculate_complexity(simple_rust, 4), Complexity::Low);
        
        let medium_rust = r#"
pub struct User {
    name: String,
    age: u32,
}

impl User {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
    
    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

pub enum Status {
    Active,
    Inactive,
}

pub trait Displayable {
    fn display(&self) -> String;
}

impl Displayable for User {
    fn display(&self) -> String {
        format!("{} ({})", self.name, self.age)
    }
}
        "#;
        let complexity = calculate_complexity(medium_rust, medium_rust.lines().count());
        assert!(matches!(complexity, Complexity::Medium | Complexity::High));
        
        // Test high complexity with many functions, structs, enums, traits, and impls
        let complex_rust = format!("{}\n{}\n{}\n{}\n{}", 
            "fn func() {}\n".repeat(15),  // 15 functions
            "struct S {}\n".repeat(8),     // 8 structs
            "enum E {}\n".repeat(5),       // 5 enums
            "trait T {}\n".repeat(3),      // 3 traits
            "impl S {}\n".repeat(4)        // 4 impl blocks
        );
        assert_eq!(calculate_complexity(&complex_rust, 600), Complexity::High);
    }
    
    #[test]
    fn test_rust_file_patterns() {
        // Test various Rust file naming patterns
        assert_eq!(detect_file_type(Path::new("src/lib.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("src/main.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("src/utils/mod.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("src/models/user.rs")), FileType::Other);
        
        // Windows paths
        assert_eq!(detect_file_type(Path::new("src\\bin\\tool.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("examples\\demo.rs")), FileType::Other);
        assert_eq!(detect_file_type(Path::new("tests\\integration.rs")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("benches\\benchmark.rs")), FileType::Test);
        
        // Test files with test patterns
        assert_eq!(detect_file_type(Path::new("src/lib.test.rs")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("tests/unit_test.rs")), FileType::Test);
        assert_eq!(detect_file_type(Path::new("test_utils.rs")), FileType::Test);
    }
    
    #[test]
    fn test_is_ignored_file_rust() {
        // Test Rust-specific ignored files/directories
        assert!(is_ignored_file(Path::new("target/debug/main")));
        assert!(is_ignored_file(Path::new("target/release/app.exe")));
        assert!(is_ignored_file(Path::new("target/doc/index.html")));
        
        // Test that source files are not ignored
        assert!(!is_ignored_file(Path::new("src/main.rs")));
        assert!(!is_ignored_file(Path::new("src/lib.rs")));
        assert!(!is_ignored_file(Path::new("Cargo.toml")));
        assert!(!is_ignored_file(Path::new("Cargo.lock")));
    }
    
    #[test]
    fn test_walk_project_files_rust() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        // Create Rust project structure
        fs::create_dir_all(temp_path.join("src"))?;
        fs::create_dir_all(temp_path.join("tests"))?;
        fs::create_dir_all(temp_path.join("examples"))?;
        fs::create_dir_all(temp_path.join("benches"))?;
        
        // Create Rust files
        fs::write(temp_path.join("Cargo.toml"), "# Cargo.toml")?;
        fs::write(temp_path.join("src/lib.rs"), "// lib.rs")?;
        fs::write(temp_path.join("src/main.rs"), "// main.rs")?;
        fs::write(temp_path.join("src/utils.rs"), "// utils.rs")?;
        fs::write(temp_path.join("tests/integration.rs"), "// integration test")?;
        fs::write(temp_path.join("examples/demo.rs"), "// demo example")?;
        fs::write(temp_path.join("benches/benchmark.rs"), "// benchmark")?;
        
        // Create ignored files
        fs::create_dir_all(temp_path.join("target/debug"))?;
        fs::write(temp_path.join("target/debug/app"), "binary")?;
        
        // Update walk_project_files to include Rust files
        let files = walk_project_files_extended(temp_path)?;
        
        // Should include Rust files
        assert!(files.iter().any(|f| f.ends_with("Cargo.toml")));
        assert!(files.iter().any(|f| f.ends_with("src/lib.rs")));
        assert!(files.iter().any(|f| f.ends_with("src/main.rs")));
        assert!(files.iter().any(|f| f.ends_with("src/utils.rs")));
        
        // Should exclude target directory
        assert!(!files.iter().any(|f| f.contains("target")));
        
        Ok(())
    }
    
    // Helper function for testing Rust file walking
    fn walk_project_files_extended(root: &Path) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && !is_ignored_file(path) {
                if let Some(extension) = path.extension() {
                    if matches!(extension.to_str(), 
                        Some("ts") | Some("js") | Some("scss") | Some("css") | 
                        Some("json") | Some("rs") | Some("toml")) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }
}
use anyhow::Result;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use crate::types::{FileType, Complexity};

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
    
    // Check for test files first
    if path_str.contains(".spec.") || path_str.contains(".test.") {
        return FileType::Test;
    }
    
    match path.extension().and_then(|s| s.to_str()) {
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
        _ => FileType::Other,
    }
}

// New function to detect file type from content for better accuracy
pub fn detect_file_type_from_content(path: &Path, content: &str) -> FileType {
    // First check file name patterns
    let path_based_type = detect_file_type(path);
    
    // If we already have a specific type from path, return it
    if !matches!(path_based_type, FileType::Other) {
        return path_based_type;
    }
    
    // For TypeScript files that weren't classified by path, check content
    if path.extension().and_then(|s| s.to_str()) == Some("ts") {
        // Check for Angular component patterns
        if content.contains("@Component") {
            return FileType::Component;
        }
        
        // Check for Angular service patterns
        if content.contains("@Injectable") {
            return FileType::Service;
        }
        
        // Check for Angular pipe patterns
        if content.contains("@Pipe") {
            return FileType::Pipe;
        }
        
        // Check for Angular module patterns
        if content.contains("@NgModule") {
            return FileType::Module;
        }
        
        // Check for class patterns that might be components
        if content.contains("export class") && content.contains("Component") {
            return FileType::Component;
        }
        
        // Check for class patterns that might be services
        if content.contains("export class") && content.contains("Service") {
            return FileType::Service;
        }
        
        // Check for class patterns that might be pipes
        if content.contains("export class") && content.contains("Pipe") {
            return FileType::Pipe;
        }
        
        // Check for class patterns that might be modules
        if content.contains("export class") && content.contains("Module") {
            return FileType::Module;
        }
        
        // Check for transform method (common in pipes)
        if content.contains("transform(") && content.contains("PipeTransform") {
            return FileType::Pipe;
        }
    }
    
    path_based_type
}

pub fn calculate_complexity(content: &str, line_count: usize) -> Complexity {
    let cyclomatic_complexity = calculate_cyclomatic_complexity(content);
    let size_factor = line_count as f64 / 100.0;
    
    let total_complexity = cyclomatic_complexity + size_factor;
    
    match total_complexity {
        x if x < 5.0 => Complexity::Low,
        x if x < 15.0 => Complexity::Medium,
        _ => Complexity::High,
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
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if matches!(extension.to_str(), Some("ts") | Some("js") | Some("scss") | Some("css") | Some("json")) {
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
}
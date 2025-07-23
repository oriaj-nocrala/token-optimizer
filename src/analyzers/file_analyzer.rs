use anyhow::Result;
use std::path::Path;
use chrono::Utc;
use crate::types::{FileMetadata, FileType, DetailedAnalysis, LocationInfo};
use crate::utils::file_utils::*;
use crate::analyzers::ts_ast_analyzer::TypeScriptASTAnalyzer;
use crate::analyzers::rust_analyzer::RustAnalyzer;

pub struct FileAnalyzer;

impl FileAnalyzer {
    pub fn new() -> Self {
        FileAnalyzer
    }

    pub fn analyze_file(&self, path: &Path) -> Result<FileMetadata> {
        let content = read_file_content(path)?;
        let size = get_file_size(path)?;
        let line_count = count_lines(&content);
        let file_type = detect_file_type_from_content(path, &content);
        let complexity = calculate_complexity(&content, line_count);
        
        let detailed_analysis = self.generate_detailed_analysis(&content, &file_type)?;
        
        let metadata = FileMetadata {
            path: path.to_string_lossy().to_string(),
            size,
            line_count,
            last_modified: Utc::now(),
            file_type: file_type.clone(),
            summary: self.generate_summary(&content, &file_type),
            relevant_sections: self.extract_relevant_sections(&content, &file_type),
            exports: self.extract_exports(&content, &file_type),
            imports: self.extract_imports(&content, &file_type),
            complexity,
            detailed_analysis,
        };

        Ok(metadata)
    }

    fn generate_detailed_analysis(&self, content: &str, file_type: &FileType) -> Result<Option<DetailedAnalysis>> {
        match file_type {
            FileType::Component | FileType::Service | FileType::Pipe | FileType::Other if self.is_typescript_file(content) => {
                self.analyze_typescript_content(content)
            }
            FileType::RustLibrary | FileType::RustBinary | FileType::RustModule | 
            FileType::RustTest | FileType::RustBench | FileType::RustExample => {
                self.analyze_rust_content(content, Path::new("dummy"))
            }
            _ => Ok(None)
        }
    }

    fn is_typescript_file(&self, content: &str) -> bool {
        // Simple heuristic to detect TypeScript content
        content.contains("interface ") || 
        content.contains("type ") || 
        content.contains(": string") ||
        content.contains(": number") ||
        content.contains("@Component") ||
        content.contains("@Injectable") ||
        content.contains("@Pipe") ||
        content.contains("export class") ||
        content.contains("export interface")
    }

    fn analyze_typescript_content(&self, content: &str) -> Result<Option<DetailedAnalysis>> {
        let mut ts_analyzer = TypeScriptASTAnalyzer::new()?;
        let tree = ts_analyzer.parse_file(content)?;
        
        let functions = ts_analyzer.extract_functions(&tree, content);
        let classes = ts_analyzer.extract_classes(&tree, content);
        let component_info = ts_analyzer.extract_component_info(&tree, content);
        let service_info = ts_analyzer.extract_service_info(&tree, content);
        let pipe_info = ts_analyzer.extract_pipe_info(&tree, content);
        
        // Extract additional elements
        let elements = ts_analyzer.extract_elements(&tree, content);
        let mut interfaces = Vec::new();
        let mut enums = Vec::new();
        let mut types = Vec::new();
        let mut variables = Vec::new();
        
        for element in elements {
            match element.kind.as_str() {
                "Interface" => {
                    interfaces.push(crate::types::InterfaceInfo {
                        name: element.name,
                        properties: Vec::new(), // TODO: Extract from element
                        methods: Vec::new(),     // TODO: Extract from element
                        extends: Vec::new(),     // TODO: Extract from element
                        location: self.parse_location(&element.location),
                    });
                }
                "Enum" => {
                    enums.push(crate::types::EnumInfo {
                        name: element.name,
                        variants: Vec::new(), // TODO: Extract from element
                        location: self.parse_location(&element.location),
                    });
                }
                "Type" => {
                    types.push(crate::types::TypeAliasInfo {
                        name: element.name.clone(),
                        type_definition: element.signature.split(" = ").nth(1).unwrap_or("unknown").to_string(),
                        generics: Vec::new(), // TODO: Extract generics
                        location: self.parse_location(&element.location),
                    });
                }
                "Variable" => {
                    variables.push(crate::types::VariableInfo {
                        name: element.name.clone(),
                        var_type: element.signature.split(": ").nth(1).unwrap_or("unknown").to_string(),
                        is_const: element.signature.contains("const"),
                        is_exported: false, // TODO: Detect exports
                        location: self.parse_location(&element.location),
                        initial_value: None, // TODO: Extract initial value
                    });
                }
                _ => {}
            }
        }
        
        Ok(Some(DetailedAnalysis {
            functions,
            classes,
            interfaces,
            enums,
            types,
            variables,
            component_info,
            service_info,
            pipe_info,
            module_info: None,
        }))
    }

    fn parse_location(&self, location_str: &str) -> LocationInfo {
        let parts: Vec<&str> = location_str.split(':').collect();
        let line = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(1);
        let column = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
        LocationInfo { line, column }
    }

    fn generate_summary(&self, content: &str, file_type: &FileType) -> String {
        match file_type {
            FileType::Component => {
                let lines = content.lines().count();
                format!("Angular component with {} lines", lines)
            }
            FileType::Service => {
                let lines = content.lines().count();
                format!("Angular service with {} lines", lines)
            }
            FileType::Style => {
                let selectors = content.matches('{').count();
                format!("Style sheet with {} selectors", selectors)
            }
            FileType::Config => {
                format!("Configuration file")
            }
            FileType::Pipe => {
                let transform_methods = content.matches("transform(").count();
                let pipe_name = self.extract_pipe_name(content);
                if let Some(name) = pipe_name {
                    format!("Angular pipe '{}' with {} transform method(s)", name, transform_methods)
                } else {
                    format!("Angular pipe with {} transform method(s)", transform_methods)
                }
            }
            FileType::Module => {
                let line_count = content.lines().count();
                format!("Angular module with {} lines", line_count)
            }
            FileType::Test => {
                let test_cases = content.matches("it(").count() + content.matches("test(").count();
                format!("Test file with {} test cases", test_cases)
            }
            FileType::RustLibrary => {
                let lines = content.lines().count();
                format!("Rust library with {} lines", lines)
            }
            FileType::RustBinary => {
                let lines = content.lines().count();
                format!("Rust binary with {} lines", lines)
            }
            FileType::RustModule => {
                let lines = content.lines().count();
                format!("Rust module with {} lines", lines)
            }
            FileType::RustTest => {
                let test_cases = content.matches("#[test]").count();
                format!("Rust test file with {} test cases", test_cases)
            }
            FileType::RustBench => {
                let bench_cases = content.matches("#[bench]").count();
                format!("Rust benchmark file with {} benchmark cases", bench_cases)
            }
            FileType::RustExample => {
                let lines = content.lines().count();
                format!("Rust example with {} lines", lines)
            }
            FileType::Cargo => {
                format!("Cargo configuration file")
            }
            FileType::Other => {
                let lines = content.lines().count();
                format!("File with {} lines", lines)
            }
        }
    }

    fn extract_relevant_sections(&self, content: &str, file_type: &FileType) -> Vec<String> {
        match file_type {
            FileType::Component => {
                let mut sections = Vec::new();
                if content.contains("@Component") {
                    sections.push("Component decorator".to_string());
                }
                if content.contains("ngOnInit") {
                    sections.push("OnInit lifecycle".to_string());
                }
                if content.contains("ngOnDestroy") {
                    sections.push("OnDestroy lifecycle".to_string());
                }
                sections
            }
            FileType::Service => {
                let mut sections = Vec::new();
                if content.contains("@Injectable") {
                    sections.push("Injectable service".to_string());
                }
                if content.contains("HttpClient") {
                    sections.push("HTTP client usage".to_string());
                }
                sections
            }
            FileType::Style => {
                let mut sections = Vec::new();
                if content.contains("@media") {
                    sections.push("Media queries".to_string());
                }
                if content.contains("@mixin") {
                    sections.push("SCSS mixins".to_string());
                }
                sections
            }
            FileType::Pipe => {
                let mut sections = Vec::new();
                if content.contains("@Pipe") {
                    sections.push("Pipe decorator".to_string());
                }
                if content.contains("PipeTransform") {
                    sections.push("PipeTransform interface".to_string());
                }
                if content.contains("pure: false") {
                    sections.push("Impure pipe".to_string());
                } else if content.contains("pure: true") {
                    sections.push("Pure pipe".to_string());
                }
                if content.contains("standalone: true") {
                    sections.push("Standalone pipe".to_string());
                }
                sections
            }
            _ => vec![],
        }
    }

    fn extract_exports(&self, content: &str, _file_type: &FileType) -> Vec<String> {
        let mut exports = Vec::new();
        
        for line in content.lines() {
            let mut line = line.trim();
            
            // Remove comments from the line
            if let Some(comment_pos) = line.find("//") {
                line = &line[..comment_pos].trim();
            }
            if let Some(comment_pos) = line.find("/*") {
                line = &line[..comment_pos].trim();
            }
            
            if line.starts_with("export class ") {
                if let Some(class_name) = line.split_whitespace().nth(2) {
                    let clean_name = class_name.split('{').next().unwrap_or(class_name);
                    exports.push(clean_name.to_string());
                }
            } else if line.starts_with("export function ") {
                if let Some(func_name) = line.split_whitespace().nth(2) {
                    let func_name = func_name.split('(').next().unwrap_or(func_name);
                    exports.push(func_name.to_string());
                }
            } else if line.starts_with("export const ") {
                if let Some(const_name) = line.split_whitespace().nth(2) {
                    let const_name = const_name.split('=').next().unwrap_or(const_name);
                    exports.push(const_name.to_string());
                }
            }
        }
        
        exports
    }

    fn extract_imports(&self, content: &str, _file_type: &FileType) -> Vec<String> {
        let mut imports = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("import ") && line.contains(" from ") {
                if let Some(from_part) = line.split(" from ").nth(1) {
                    // Remove quotes, semicolon, and comments
                    let mut import_path = from_part.trim();
                    
                    // Remove comments (// or /* style)
                    if let Some(comment_pos) = import_path.find("//") {
                        import_path = &import_path[..comment_pos].trim();
                    }
                    if let Some(comment_pos) = import_path.find("/*") {
                        import_path = &import_path[..comment_pos].trim();
                    }
                    
                    // Remove trailing semicolon
                    if import_path.ends_with(';') {
                        import_path = &import_path[..import_path.len() - 1];
                    }
                    
                    // Remove quotes
                    if (import_path.starts_with('"') && import_path.ends_with('"')) ||
                       (import_path.starts_with('\'') && import_path.ends_with('\'')) {
                        import_path = &import_path[1..import_path.len() - 1];
                    }
                    
                    if !import_path.is_empty() {
                        imports.push(import_path.to_string());
                    }
                }
            }
        }
        
        imports
    }
    
    fn extract_pipe_name(&self, content: &str) -> Option<String> {
        // Look for @Pipe decorator with name property
        for line in content.lines() {
            let line = line.trim();
            if line.contains("@Pipe") {
                // Look for name property in the next few lines
                let pipe_start = content.find("@Pipe").unwrap_or(0);
                let pipe_end = content[pipe_start..].find("})").map(|i| pipe_start + i + 2).unwrap_or(content.len());
                let pipe_section = &content[pipe_start..pipe_end];
                
                if let Some(name_start) = pipe_section.find("name:") {
                    let name_part = &pipe_section[name_start + 5..];
                    if let Some(quote_start) = name_part.find('\'').or_else(|| name_part.find('"')) {
                        let quote_char = name_part.chars().nth(quote_start).unwrap();
                        let name_content = &name_part[quote_start + 1..];
                        if let Some(quote_end) = name_content.find(quote_char) {
                            return Some(name_content[..quote_end].to_string());
                        }
                    }
                }
                break;
            }
        }
        None
    }
    
    /// Analyze Rust content using the RustAnalyzer
    fn analyze_rust_content(&self, content: &str, path: &Path) -> Result<Option<DetailedAnalysis>> {
        // Handle Cargo.toml files separately
        if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            return self.analyze_cargo_toml_content(content);
        }
        
        let mut rust_analyzer = RustAnalyzer::new()?;
        let metadata = rust_analyzer.analyze_file(path, content)?;
        Ok(metadata.detailed_analysis)
    }
    
    /// Analyze Cargo.toml content specifically
    fn analyze_cargo_toml_content(&self, content: &str) -> Result<Option<DetailedAnalysis>> {
        use crate::analyzers::rust_analyzer::CargoAnalyzer;
        
        match CargoAnalyzer::analyze_cargo_toml(content) {
            Ok(cargo_info) => {
                // Create a DetailedAnalysis with Cargo-specific information
                let mut analysis = DetailedAnalysis {
                    functions: Vec::new(),
                    classes: Vec::new(),
                    interfaces: Vec::new(),
                    enums: Vec::new(),
                    types: Vec::new(),
                    variables: Vec::new(),
                    component_info: None,
                    service_info: None,
                    pipe_info: None,
                    module_info: None,
                };
                
                // Convert cargo dependencies to "functions" for display purposes
                // This is a temporary solution to show cargo info in the existing structure
                for dep in &cargo_info.dependencies {
                    let modifiers = match &dep.source {
                        crate::types::CargoDependencySource::Git { .. } => vec!["git".to_string()],
                        crate::types::CargoDependencySource::Path { .. } => vec!["path".to_string()],
                        crate::types::CargoDependencySource::CratesIo => Vec::new(),
                    };
                    
                    analysis.functions.push(crate::types::FunctionInfo {
                        name: format!("dep:{}", dep.name),
                        parameters: Vec::new(),
                        return_type: dep.version.clone().unwrap_or_else(|| "latest".to_string()),
                        is_async: dep.optional,
                        modifiers,
                        location: crate::types::LocationInfo { line: 1, column: 1 },
                        description: Some(format!("Dependency: {}", dep.name)),
                    });
                }
                
                // Add dev dependencies
                for dep in &cargo_info.dev_dependencies {
                    analysis.functions.push(crate::types::FunctionInfo {
                        name: format!("dev-dep:{}", dep.name),
                        parameters: Vec::new(),
                        return_type: dep.version.clone().unwrap_or_else(|| "latest".to_string()),
                        is_async: false,
                        modifiers: vec!["dev".to_string()],
                        location: crate::types::LocationInfo { line: 1, column: 1 },
                        description: Some(format!("Dev dependency: {}", dep.name)),
                    });
                }
                
                // Add build dependencies
                for dep in &cargo_info.build_dependencies {
                    analysis.functions.push(crate::types::FunctionInfo {
                        name: format!("build-dep:{}", dep.name),
                        parameters: Vec::new(),
                        return_type: dep.version.clone().unwrap_or_else(|| "latest".to_string()),
                        is_async: false,
                        modifiers: vec!["build".to_string()],
                        location: crate::types::LocationInfo { line: 1, column: 1 },
                        description: Some(format!("Build dependency: {}", dep.name)),
                    });
                }
                
                Ok(Some(analysis))
            }
            Err(_) => Ok(None), // Return None if cargo analysis fails
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_analyze_typescript_component() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let component_content = r#"
        import { Component, OnInit } from '@angular/core';

        @Component({
            selector: 'app-test',
            templateUrl: './test.component.html'
        })
        export class TestComponent implements OnInit {
            ngOnInit() {
                console.log('Component initialized');
            }
        }
        "#;
        
        write!(temp_file, "{}", component_content)?;
        
        let path = temp_file.path().with_extension("component.ts");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Component);
        assert!(metadata.summary.contains("Angular component"));
        assert!(metadata.relevant_sections.contains(&"Component decorator".to_string()));
        assert!(metadata.relevant_sections.contains(&"OnInit lifecycle".to_string()));
        assert!(metadata.exports.contains(&"TestComponent".to_string()));
        assert!(metadata.imports.contains(&"@angular/core".to_string()));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_analyze_typescript_service() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let service_content = r#"
        import { Injectable } from '@angular/core';
        import { HttpClient } from '@angular/common/http';

        @Injectable({
            providedIn: 'root'
        })
        export class DataService {
            constructor(private http: HttpClient) {}
            
            getData() {
                return this.http.get('/api/data');
            }
        }
        "#;
        
        write!(temp_file, "{}", service_content)?;
        
        let path = temp_file.path().with_extension("service.ts");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Service);
        assert!(metadata.summary.contains("Angular service"));
        assert!(metadata.relevant_sections.contains(&"Injectable service".to_string()));
        assert!(metadata.relevant_sections.contains(&"HTTP client usage".to_string()));
        assert!(metadata.exports.contains(&"DataService".to_string()));
        assert!(metadata.imports.contains(&"@angular/core".to_string()));
        assert!(metadata.imports.contains(&"@angular/common/http".to_string()));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_analyze_scss_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let scss_content = r#"
        @mixin button-style {
            padding: 10px;
            border: none;
        }

        .button {
            @include button-style;
            background-color: blue;
        }

        @media (max-width: 768px) {
            .button {
                width: 100%;
            }
        }
        "#;
        
        write!(temp_file, "{}", scss_content)?;
        
        let path = temp_file.path().with_extension("scss");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Style);
        assert!(metadata.summary.contains("Style sheet"));
        assert!(metadata.relevant_sections.contains(&"Media queries".to_string()));
        assert!(metadata.relevant_sections.contains(&"SCSS mixins".to_string()));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_analyze_test_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = r#"
        import { TestBed } from '@angular/core/testing';
        import { AppComponent } from './app.component';

        describe('AppComponent', () => {
            beforeEach(() => {
                TestBed.configureTestingModule({
                    declarations: [AppComponent]
                });
            });

            it('should create the app', () => {
                const fixture = TestBed.createComponent(AppComponent);
                const app = fixture.componentInstance;
                expect(app).toBeTruthy();
            });

            it('should have title', () => {
                const fixture = TestBed.createComponent(AppComponent);
                const app = fixture.componentInstance;
                expect(app.title).toEqual('test-app');
            });
        });
        "#;
        
        write!(temp_file, "{}", test_content)?;
        
        let path = temp_file.path().with_extension("spec.ts");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Test);
        assert!(metadata.summary.contains("Test file"));
        assert!(metadata.summary.contains("2 test cases"));
        assert!(metadata.imports.contains(&"@angular/core/testing".to_string()));
        assert!(metadata.imports.contains(&"./app.component".to_string()));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_analyze_json_config() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let json_content = r#"
        {
            "name": "test-project",
            "version": "1.0.0",
            "scripts": {
                "start": "ng serve",
                "build": "ng build"
            }
        }
        "#;
        
        write!(temp_file, "{}", json_content)?;
        
        let path = temp_file.path().with_extension("json");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Config);
        assert!(metadata.summary.contains("Configuration file"));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_extract_exports() {
        let analyzer = FileAnalyzer::new();
        let content = r#"
        export class MyClass {
            constructor() {}
        }
        
        export function myFunction() {
            return true;
        }
        
        export const myConstant = 'value';
        "#;
        
        let exports = analyzer.extract_exports(content, &FileType::Other);
        
        assert!(exports.contains(&"MyClass".to_string()));
        assert!(exports.contains(&"myFunction".to_string()));
        assert!(exports.contains(&"myConstant".to_string()));
    }

    #[test]
    fn test_extract_imports() {
        let analyzer = FileAnalyzer::new();
        let content = r#"import { Component } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { MyService } from './my-service';"#;
        
        let imports = analyzer.extract_imports(content, &FileType::Other);
        
        assert!(imports.contains(&"@angular/core".to_string()));
        assert!(imports.contains(&"@angular/common/http".to_string()));
        assert!(imports.contains(&"./my-service".to_string()));
    }

    #[test]
    fn test_extract_imports_with_comments() {
        let analyzer = FileAnalyzer::new();
        let content = r#"import { Component } from '@angular/core'; // Angular component decorator
import { HttpClient } from '@angular/common/http'; /* HTTP client for API calls */
import { MyService } from './my-service'; // Custom service
import { Utils } from './utils' // No semicolon comment"#;
        
        let imports = analyzer.extract_imports(content, &FileType::Other);
        
        // Should only contain the clean import paths without comments
        assert!(imports.contains(&"@angular/core".to_string()));
        assert!(imports.contains(&"@angular/common/http".to_string()));
        assert!(imports.contains(&"./my-service".to_string()));
        assert!(imports.contains(&"./utils".to_string()));
        
        // Should not contain any comment text
        for import in &imports {
            assert!(!import.contains("//"));
            assert!(!import.contains("/*"));
            assert!(!import.contains("Angular"));
            assert!(!import.contains("HTTP"));
            assert!(!import.contains("Custom"));
            assert!(!import.contains("comment"));
        }
    }

    #[test]
    fn test_extract_exports_with_comments() {
        let analyzer = FileAnalyzer::new();
        let content = r#"export class MyClass { // Main class
            constructor() {}
        }
        
        export function myFunction() { // Utility function
            return true;
        }
        
        export const myConstant = 'value'; /* Global constant */"#;
        
        let exports = analyzer.extract_exports(content, &FileType::Other);
        
        // Should only contain clean export names
        assert!(exports.contains(&"MyClass".to_string()));
        assert!(exports.contains(&"myFunction".to_string()));
        assert!(exports.contains(&"myConstant".to_string()));
        
        // Should not contain any comment text
        for export in &exports {
            assert!(!export.contains("//"));
            assert!(!export.contains("/*"));
            assert!(!export.contains("Main"));
            assert!(!export.contains("Utility"));
            assert!(!export.contains("Global"));
            assert!(!export.contains("comment"));
        }
    }

    #[test]
    fn test_generate_summary() {
        let analyzer = FileAnalyzer::new();
        
        let component_content = "export class TestComponent {}";
        let summary = analyzer.generate_summary(component_content, &FileType::Component);
        assert!(summary.contains("Angular component"));
        
        let service_content = "export class TestService {}";
        let summary = analyzer.generate_summary(service_content, &FileType::Service);
        assert!(summary.contains("Angular service"));
        
        let style_content = ".class1 {} .class2 {}";
        let summary = analyzer.generate_summary(style_content, &FileType::Style);
        assert!(summary.contains("Style sheet"));
        
        let test_content = "it('should work', () => {}); it('should pass', () => {});";
        let summary = analyzer.generate_summary(test_content, &FileType::Test);
        assert!(summary.contains("2 test cases"));
    }

    #[test]
    fn test_extract_relevant_sections() {
        let analyzer = FileAnalyzer::new();
        
        let component_content = r#"
        @Component({
            selector: 'app-test'
        })
        export class TestComponent implements OnInit, OnDestroy {
            ngOnInit() {}
            ngOnDestroy() {}
        }
        "#;
        
        let sections = analyzer.extract_relevant_sections(component_content, &FileType::Component);
        assert!(sections.contains(&"Component decorator".to_string()));
        assert!(sections.contains(&"OnInit lifecycle".to_string()));
        assert!(sections.contains(&"OnDestroy lifecycle".to_string()));
        
        let service_content = r#"
        @Injectable({
            providedIn: 'root'
        })
        export class TestService {
            constructor(private http: HttpClient) {}
        }
        "#;
        
        let sections = analyzer.extract_relevant_sections(service_content, &FileType::Service);
        assert!(sections.contains(&"Injectable service".to_string()));
        assert!(sections.contains(&"HTTP client usage".to_string()));
        
        let pipe_content = r#"
        @Pipe({
            name: 'testPipe',
            pure: false,
            standalone: true
        })
        export class TestPipe implements PipeTransform {
            transform(value: any): any {
                return value;
            }
        }
        "#;
        
        let sections = analyzer.extract_relevant_sections(pipe_content, &FileType::Pipe);
        assert!(sections.contains(&"Pipe decorator".to_string()));
        assert!(sections.contains(&"PipeTransform interface".to_string()));
        assert!(sections.contains(&"Impure pipe".to_string()));
        assert!(sections.contains(&"Standalone pipe".to_string()));
    }

    #[test]
    fn test_analyze_angular_pipe() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let pipe_content = r#"
        import { Pipe, PipeTransform } from '@angular/core';

        @Pipe({
            name: 'uppercase',
            pure: true,
            standalone: false
        })
        export class UppercasePipe implements PipeTransform {
            transform(value: string, ...args: any[]): string {
                return value ? value.toUpperCase() : '';
            }
        }
        "#;
        
        write!(temp_file, "{}", pipe_content)?;
        
        let path = temp_file.path().with_extension("pipe.ts");
        fs::copy(temp_file.path(), &path)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(&path)?;
        
        assert_eq!(metadata.file_type, FileType::Pipe);
        assert!(metadata.summary.contains("Angular pipe"));
        assert!(metadata.summary.contains("uppercase"));
        assert!(metadata.relevant_sections.contains(&"Pipe decorator".to_string()));
        assert!(metadata.relevant_sections.contains(&"PipeTransform interface".to_string()));
        assert!(metadata.relevant_sections.contains(&"Pure pipe".to_string()));
        assert!(metadata.exports.contains(&"UppercasePipe".to_string()));
        assert!(metadata.imports.contains(&"@angular/core".to_string()));
        
        fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn test_extract_pipe_name() {
        let analyzer = FileAnalyzer::new();
        
        let pipe_content_with_name = r#"
        @Pipe({
            name: 'customPipe'
        })
        export class CustomPipe {
        }
        "#;
        
        let pipe_name = analyzer.extract_pipe_name(pipe_content_with_name);
        assert_eq!(pipe_name, Some("customPipe".to_string()));
        
        let pipe_content_without_name = r#"
        export class SomePipe {
        }
        "#;
        
        let no_name = analyzer.extract_pipe_name(pipe_content_without_name);
        assert_eq!(no_name, None);
        
        let pipe_content_with_double_quotes = r#"
        @Pipe({
            name: "testPipe"
        })
        export class TestPipe {
        }
        "#;
        
        let pipe_name_double = analyzer.extract_pipe_name(pipe_content_with_double_quotes);
        assert_eq!(pipe_name_double, Some("testPipe".to_string()));
    }

    #[test]
    fn test_complexity_calculation() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let complex_content = r#"
        function complexFunction() {
            if (condition1) {
                for (let i = 0; i < 10; i++) {
                    if (condition2) {
                        while (condition3) {
                            switch (value) {
                                case 1:
                                    break;
                                case 2:
                                    break;
                                default:
                                    break;
                            }
                        }
                    }
                }
            }
        }
        "#;
        
        write!(temp_file, "{}", complex_content)?;
        
        let analyzer = FileAnalyzer::new();
        let metadata = analyzer.analyze_file(temp_file.path())?;
        
        // The complexity should be Medium or High given the nested structures
        assert!(matches!(metadata.complexity, crate::types::Complexity::Medium | crate::types::Complexity::High));
        assert!(metadata.line_count > 0);
        assert!(metadata.size > 0);
        
        Ok(())
    }
}
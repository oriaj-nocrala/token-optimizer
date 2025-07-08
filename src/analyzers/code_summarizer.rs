use anyhow::Result;
use std::path::Path;
use crate::types::{CodeSummary, FunctionInfo, ClassInfo, ComponentInfo, ServiceInfo, PipeInfo, LocationInfo};
use crate::utils::read_file_content;

pub struct CodeSummarizer;

impl CodeSummarizer {
    pub fn new() -> Self {
        CodeSummarizer
    }

    pub fn summarize_file(&self, path: &Path) -> Result<CodeSummary> {
        let content = read_file_content(path)?;
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let file_type = self.determine_file_type(path);

        let summary = CodeSummary {
            file_name,
            file_type,
            exports: self.extract_exports(&content)?,
            imports: self.extract_imports(&content)?,
            functions: self.extract_functions(&content)?,
            classes: self.extract_classes(&content)?,
            components: self.extract_components(&content)?,
            services: self.extract_services(&content)?,
            pipes: self.extract_pipes(&content)?,
            modules: self.extract_modules(&content)?,
            key_patterns: self.extract_key_patterns(&content)?,
            dependencies: self.extract_dependencies(&content)?,
            scss_variables: self.extract_scss_variables(&content)?,
            scss_mixins: self.extract_scss_mixins(&content)?,
        };

        Ok(summary)
    }

    fn determine_file_type(&self, path: &Path) -> String {
        match path.extension().and_then(|s| s.to_str()) {
            Some("ts") => "typescript".to_string(),
            Some("js") => "javascript".to_string(),
            Some("scss") => "scss".to_string(),
            Some("css") => "css".to_string(),
            Some("json") => "json".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn extract_exports(&self, content: &str) -> Result<Vec<String>> {
        let mut exports = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("export ") {
                if let Some(export_name) = self.parse_export_statement(trimmed) {
                    exports.push(export_name);
                }
            }
        }
        
        Ok(exports)
    }

    fn extract_imports(&self, content: &str) -> Result<Vec<String>> {
        let mut imports = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                if let Some(import_name) = self.parse_import_statement(trimmed) {
                    imports.push(import_name);
                }
            }
        }
        
        Ok(imports)
    }

    fn extract_functions(&self, content: &str) -> Result<Vec<FunctionInfo>> {
        let mut functions = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("function ") || trimmed.contains("=> ") {
                if let Some(func_info) = self.parse_function(trimmed) {
                    functions.push(func_info);
                }
            }
        }
        
        Ok(functions)
    }

    fn extract_classes(&self, content: &str) -> Result<Vec<ClassInfo>> {
        let mut classes = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("class ") || trimmed.contains("export class ") {
                if let Some(class_info) = self.parse_class(trimmed) {
                    classes.push(class_info);
                }
            }
        }
        
        Ok(classes)
    }

    fn extract_components(&self, content: &str) -> Result<Vec<ComponentInfo>> {
        let mut components = Vec::new();
        
        if content.contains("@Component") {
            if let Some(component_info) = self.parse_component(content) {
                components.push(component_info);
            }
        }
        
        Ok(components)
    }

    fn extract_services(&self, content: &str) -> Result<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        
        if content.contains("@Injectable") {
            if let Some(service_info) = self.parse_service(content) {
                services.push(service_info);
            }
        }
        
        Ok(services)
    }

    fn extract_pipes(&self, content: &str) -> Result<Vec<PipeInfo>> {
        let mut pipes = Vec::new();
        
        if content.contains("@Pipe") {
            if let Some(pipe_info) = self.parse_pipe(content) {
                pipes.push(pipe_info);
            }
        }
        
        Ok(pipes)
    }

    fn extract_modules(&self, content: &str) -> Result<Vec<crate::types::ModuleInfo>> {
        let mut modules = Vec::new();
        
        // Check for @NgModule decorator
        if content.contains("@NgModule") {
            if let Some(module_match) = self.extract_module_decorator(content) {
                modules.push(module_match);
            }
        }
        
        Ok(modules)
    }

    fn extract_module_decorator(&self, content: &str) -> Option<crate::types::ModuleInfo> {
        // Extract basic module information
        let name = self.extract_class_name_from_content(content)?;
        
        // Extract NgModule metadata
        let imports = self.extract_module_imports(content);
        let exports = self.extract_module_exports(content);  
        let declarations = self.extract_module_declarations(content);
        let providers = self.extract_module_providers(content);
        let bootstrap = self.extract_module_bootstrap(content);
        
        Some(crate::types::ModuleInfo {
            name,
            imports,
            exports,
            declarations,
            providers,
            bootstrap,
            schemas: vec![],
            is_root_module: content.contains("bootstrap:"),
            is_feature_module: !content.contains("bootstrap:") && !content.contains("CommonModule"),
            is_shared_module: content.contains("CommonModule"),
            lazy_routes: vec![],
            location: crate::types::LocationInfo { line: 1, column: 1 },
        })
    }

    fn extract_module_imports(&self, content: &str) -> Vec<String> {
        // Extract imports from @NgModule({ imports: [...] })
        if let Some(start) = content.find("imports:") {
            if let Some(bracket_start) = content[start..].find('[') {
                if let Some(bracket_end) = content[start + bracket_start..].find(']') {
                    let imports_str = &content[start + bracket_start + 1..start + bracket_start + bracket_end];
                    return imports_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        vec![]
    }

    fn extract_module_exports(&self, content: &str) -> Vec<String> {
        // Extract exports from @NgModule({ exports: [...] })
        if let Some(start) = content.find("exports:") {
            if let Some(bracket_start) = content[start..].find('[') {
                if let Some(bracket_end) = content[start + bracket_start..].find(']') {
                    let exports_str = &content[start + bracket_start + 1..start + bracket_start + bracket_end];
                    return exports_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        vec![]
    }

    fn extract_module_declarations(&self, content: &str) -> Vec<String> {
        // Extract declarations from @NgModule({ declarations: [...] })
        if let Some(start) = content.find("declarations:") {
            if let Some(bracket_start) = content[start..].find('[') {
                if let Some(bracket_end) = content[start + bracket_start..].find(']') {
                    let declarations_str = &content[start + bracket_start + 1..start + bracket_start + bracket_end];
                    return declarations_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        vec![]
    }

    fn extract_module_providers(&self, content: &str) -> Vec<String> {
        // Extract providers from @NgModule({ providers: [...] })
        if let Some(start) = content.find("providers:") {
            if let Some(bracket_start) = content[start..].find('[') {
                if let Some(bracket_end) = content[start + bracket_start..].find(']') {
                    let providers_str = &content[start + bracket_start + 1..start + bracket_start + bracket_end];
                    return providers_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        vec![]
    }

    fn extract_module_bootstrap(&self, content: &str) -> Vec<String> {
        // Extract bootstrap from @NgModule({ bootstrap: [...] })
        if let Some(start) = content.find("bootstrap:") {
            if let Some(bracket_start) = content[start..].find('[') {
                if let Some(bracket_end) = content[start + bracket_start..].find(']') {
                    let bootstrap_str = &content[start + bracket_start + 1..start + bracket_start + bracket_end];
                    return bootstrap_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        vec![]
    }

    fn extract_class_name_from_content(&self, content: &str) -> Option<String> {
        // Extract class name from "export class ClassName"
        if let Some(start) = content.find("export class ") {
            let after_class = &content[start + 13..];
            if let Some(end) = after_class.find(' ').or_else(|| after_class.find('{')) {
                return Some(after_class[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_key_patterns(&self, content: &str) -> Result<Vec<String>> {
        let mut patterns = Vec::new();
        let keywords = ["async", "await", "Promise", "Observable", "Subject", "BehaviorSubject"];
        
        for keyword in keywords {
            if content.contains(keyword) {
                patterns.push(keyword.to_string());
            }
        }
        
        Ok(patterns)
    }

    fn extract_dependencies(&self, content: &str) -> Result<Vec<String>> {
        let mut dependencies = Vec::new();
        
        for line in content.lines() {
            if line.trim().starts_with("import ") && line.contains("from ") {
                if let Some(dep) = self.parse_import_statement(line.trim()) {
                    dependencies.push(dep);
                }
            }
        }
        
        Ok(dependencies)
    }

    fn extract_scss_variables(&self, content: &str) -> Result<Option<Vec<String>>> {
        if !content.contains("$") {
            return Ok(None);
        }
        
        let mut variables = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("$") && trimmed.contains(":") {
                if let Some(var_name) = trimmed.split(':').next() {
                    variables.push(var_name.trim().to_string());
                }
            }
        }
        
        Ok(Some(variables))
    }

    fn extract_scss_mixins(&self, content: &str) -> Result<Option<Vec<String>>> {
        if !content.contains("@mixin") {
            return Ok(None);
        }
        
        let mut mixins = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@mixin ") {
                if let Some(mixin_name) = trimmed.split_whitespace().nth(1) {
                    mixins.push(mixin_name.split('(').next().unwrap_or("").to_string());
                }
            }
        }
        
        Ok(Some(mixins))
    }

    fn parse_export_statement(&self, line: &str) -> Option<String> {
        if line.contains("export class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "class") {
                if pos + 1 < parts.len() {
                    return Some(parts[pos + 1].to_string());
                }
            }
        } else if line.contains("export function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "function") {
                if pos + 1 < parts.len() {
                    return Some(parts[pos + 1].split('(').next().unwrap_or("").to_string());
                }
            }
        }
        
        None
    }

    fn parse_import_statement(&self, line: &str) -> Option<String> {
        if let Some(from_pos) = line.find("from ") {
            let module_part = &line[from_pos + 5..];
            let module_name = module_part.trim().trim_matches('"').trim_matches('\'');
            return Some(module_name.to_string());
        }
        
        None
    }

    fn parse_function(&self, line: &str) -> Option<FunctionInfo> {
        // Simplified function parsing
        if line.contains("function ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "function") {
                if pos + 1 < parts.len() {
                    let func_name = parts[pos + 1].split('(').next().unwrap_or("").to_string();
                    return Some(FunctionInfo {
                        name: func_name,
                        parameters: Vec::new(), // Simplified
                        return_type: "any".to_string(), // Simplified
                        is_async: line.contains("async"),
                        modifiers: Vec::new(),
                        location: LocationInfo { line: 1, column: 1 }, // Simplified
                        description: None,
                    });
                }
            }
        }
        
        None
    }

    fn parse_class(&self, line: &str) -> Option<ClassInfo> {
        if line.contains("class ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "class") {
                if pos + 1 < parts.len() {
                    let class_name = parts[pos + 1].to_string();
                    return Some(ClassInfo {
                        name: class_name,
                        methods: Vec::new(), // Simplified
                        properties: Vec::new(), // Simplified
                        extends: None, // Simplified
                        implements: Vec::new(), // Simplified
                        modifiers: Vec::new(),
                        location: LocationInfo { line: 1, column: 1 }, // Simplified
                    });
                }
            }
        }
        
        None
    }

    fn parse_component(&self, content: &str) -> Option<ComponentInfo> {
        // Simplified component parsing
        if let Some(selector_start) = content.find("selector: ") {
            let selector_line = &content[selector_start..];
            if let Some(selector_end) = selector_line.find('\n') {
                let selector_part = &selector_line[..selector_end];
                if let Some(selector_value) = selector_part.split('"').nth(1) {
                    return Some(ComponentInfo {
                        name: "Component".to_string(), // Simplified
                        selector: selector_value.to_string(),
                        inputs: Vec::new(), // Simplified
                        outputs: Vec::new(), // Simplified
                        lifecycle: Vec::new(), // Simplified
                        template_summary: "Angular Component".to_string(), // Simplified
                        location: LocationInfo { line: 1, column: 1 }, // Simplified
                    });
                }
            }
        }
        
        None
    }

    fn parse_service(&self, content: &str) -> Option<ServiceInfo> {
        // Simplified service parsing
        if content.contains("@Injectable") {
            return Some(ServiceInfo {
                name: "Service".to_string(), // Simplified
                injectable: true,
                provided_in: self.extract_provided_in(content),
                scope: self.determine_service_scope(content),
                dependencies: Vec::new(), // Simplified
                methods: Vec::new(), // Simplified
                location: LocationInfo { line: 1, column: 1 }, // Simplified
            });
        }
        
        None
    }

    fn extract_provided_in(&self, content: &str) -> Option<String> {
        // Extract providedIn from @Injectable({ providedIn: '...' })
        if let Some(start) = content.find("providedIn:") {
            let after_provided_in = &content[start + 11..];
            if let Some(quote_start) = after_provided_in.find('\'').or_else(|| after_provided_in.find('\"')) {
                let quote_char = after_provided_in.chars().nth(quote_start).unwrap();
                let after_quote = &after_provided_in[quote_start + 1..];
                if let Some(quote_end) = after_quote.find(quote_char) {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
        None
    }

    fn determine_service_scope(&self, content: &str) -> crate::types::ServiceScope {
        use crate::types::ServiceScope;
        
        if let Some(provided_in) = self.extract_provided_in(content) {
            match provided_in.as_str() {
                "root" => ServiceScope::Root,
                "platform" => ServiceScope::Platform,
                _ => ServiceScope::Module,
            }
        } else {
            ServiceScope::Module
        }
    }

    fn parse_pipe(&self, content: &str) -> Option<PipeInfo> {
        // Simplified pipe parsing
        if content.contains("@Pipe") {
            use crate::types::{FunctionInfo, ParameterInfo};
            
            let pipe_name = if let Some(name_start) = content.find("name: ") {
                let name_part = &content[name_start + 6..];
                if let Some(quote_start) = name_part.find('\'').or_else(|| name_part.find('\"')) {
                    let quote_char = name_part.chars().nth(quote_start).unwrap();
                    let name_content = &name_part[quote_start + 1..];
                    if let Some(quote_end) = name_content.find(quote_char) {
                        name_content[..quote_end].to_string()
                    } else {
                        "UnknownPipe".to_string()
                    }
                } else {
                    "UnknownPipe".to_string()
                }
            } else {
                "UnknownPipe".to_string()
            };

            let transform_method = FunctionInfo {
                name: "transform".to_string(),
                parameters: vec![
                    ParameterInfo {
                        name: "value".to_string(),
                        param_type: "any".to_string(),
                        is_optional: false,
                        default_value: None,
                    }
                ],
                return_type: "any".to_string(),
                is_async: false,
                modifiers: vec![],
                location: LocationInfo { line: 1, column: 1 },
                description: Some("Pipe transform method".to_string()),
            };

            return Some(PipeInfo {
                name: pipe_name,
                transform_method,
                is_pure: !content.contains("pure: false"),
                is_standalone: content.contains("standalone: true"),
                dependencies: Vec::new(),
                location: LocationInfo { line: 1, column: 1 },
            });
        }
        
        None
    }
}
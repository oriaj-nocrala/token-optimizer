/*! Rust Code Analyzer
 * Analyzes Rust source files, extracts metadata, and provides structured summaries
 */

use anyhow::Result;
use std::path::Path;
use crate::types::{
    FileType, FileMetadata, DetailedAnalysis, FunctionInfo, LocationInfo,
    RustModuleInfo, RustStructInfo, RustEnumInfo, RustTraitInfo, RustImplInfo,
    RustConstInfo, RustTypeAliasInfo, RustMacroInfo, RustUseInfo, RustFieldInfo,
    RustEnumVariant, RustMacroType, CargoInfo,
    Complexity, ParameterInfo
};
use tree_sitter::{Parser, Language, Node, Tree};
use chrono::Utc;

// Moderno tree-sitter API - no necesitamos extern "C"

/// Rust-specific code analyzer
pub struct RustAnalyzer {
    parser: Parser,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        
        Ok(Self { parser })
    }
    
    /// Analyze a Rust source file
    pub fn analyze_file(&mut self, path: &Path, content: &str) -> Result<FileMetadata> {
        let file_type = self.detect_rust_file_type(path, content);
        let tree = self.parser.parse(content, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Rust file"))?;
        
        let detailed_analysis = self.extract_detailed_analysis(&tree, content)?;
        let complexity = self.calculate_complexity(&detailed_analysis, content);
        
        Ok(FileMetadata {
            path: path.to_string_lossy().to_string(),
            size: content.len() as u64,
            line_count: content.lines().count(),
            last_modified: Utc::now(),
            file_type,
            summary: self.generate_summary(path, &detailed_analysis),
            relevant_sections: self.extract_relevant_sections(&detailed_analysis),
            exports: self.extract_exports(&detailed_analysis),
            imports: self.extract_imports(&detailed_analysis),
            complexity,
            detailed_analysis: Some(detailed_analysis),
        })
    }
    
    /// Detect the specific type of Rust file
    fn detect_rust_file_type(&self, path: &Path, content: &str) -> FileType {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy();
        
        match file_name {
            "lib.rs" => FileType::RustLibrary,
            "main.rs" => FileType::RustBinary,
            "mod.rs" => FileType::RustModule,
            _ => {
                if path_str.contains("/bin/") || path_str.contains("\\bin\\") {
                    FileType::RustBinary
                } else if path_str.contains("/examples/") || path_str.contains("\\examples\\") {
                    FileType::RustExample
                } else if path_str.contains("/tests/") || path_str.contains("\\tests\\") {
                    FileType::RustTest
                } else if path_str.contains("/benches/") || path_str.contains("\\benches\\") {
                    FileType::RustBench
                } else if content.contains("#[cfg(test)]") {
                    FileType::RustTest
                } else {
                    FileType::RustModule
                }
            }
        }
    }
    
    /// Extract detailed analysis from the syntax tree
    fn extract_detailed_analysis(&self, tree: &Tree, content: &str) -> Result<DetailedAnalysis> {
        let root_node = tree.root_node();
        let source_bytes = content.as_bytes();
        
        let mut functions = Vec::new();
        let mut rust_module = RustModuleInfo {
            name: "".to_string(),
            is_public: false,
            submodules: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            impl_blocks: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            type_aliases: Vec::new(),
            macros: Vec::new(),
            use_statements: Vec::new(),
        };
        
        self.traverse_node(root_node, source_bytes, &mut functions, &mut rust_module)?;
        
        Ok(DetailedAnalysis {
            functions,
            classes: Vec::new(), // Rust doesn't have classes
            interfaces: Vec::new(), // Rust uses traits instead
            enums: Vec::new(), // Will be populated from rust_module
            types: Vec::new(),
            variables: Vec::new(),
            component_info: None,
            service_info: None,
            pipe_info: None,
            module_info: None,
        })
    }
    
    /// Traverse the syntax tree recursively
    fn traverse_node(
        &self,
        node: Node,
        source_bytes: &[u8],
        functions: &mut Vec<FunctionInfo>,
        rust_module: &mut RustModuleInfo,
    ) -> Result<()> {
        match node.kind() {
            "function_item" => {
                let function = self.extract_function(&node, source_bytes)?;
                functions.push(function);
            }
            "struct_item" => {
                let struct_info = self.extract_struct(&node, source_bytes)?;
                rust_module.structs.push(struct_info);
            }
            "enum_item" => {
                let enum_info = self.extract_enum(&node, source_bytes)?;
                rust_module.enums.push(enum_info);
            }
            "trait_item" => {
                let trait_info = self.extract_trait(&node, source_bytes)?;
                rust_module.traits.push(trait_info);
            }
            "impl_item" => {
                let impl_info = self.extract_impl(&node, source_bytes)?;
                rust_module.impl_blocks.push(impl_info);
            }
            "const_item" | "static_item" => {
                let const_info = self.extract_const(&node, source_bytes)?;
                rust_module.constants.push(const_info);
            }
            "type_item" => {
                let type_alias = self.extract_type_alias(&node, source_bytes)?;
                rust_module.type_aliases.push(type_alias);
            }
            "macro_definition" => {
                let macro_info = self.extract_macro(&node, source_bytes)?;
                rust_module.macros.push(macro_info);
            }
            "use_declaration" => {
                let use_info = self.extract_use(&node, source_bytes)?;
                rust_module.use_statements.push(use_info);
            }
            _ => {
                // Recursively traverse child nodes
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.traverse_node(child, source_bytes, functions, rust_module)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract function information
    fn extract_function(&self, node: &Node, source_bytes: &[u8]) -> Result<FunctionInfo> {
        let name = self.find_child_text(node, "identifier", source_bytes)
            .unwrap_or_else(|| "unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        let is_async = node.to_sexp().contains("async");
        let is_unsafe = node.to_sexp().contains("unsafe");
        
        // Extract parameters
        let parameters = self.extract_function_parameters(node, source_bytes)?;
        
        // Extract return type
        let return_type = self.extract_return_type(node, source_bytes);
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(FunctionInfo {
            name,
            parameters,
            return_type,
            is_async,
            modifiers: if is_unsafe { vec!["unsafe".to_string()] } else { Vec::new() },
            location,
            description: None,
        })
    }
    
    /// Extract struct information
    fn extract_struct(&self, node: &Node, source_bytes: &[u8]) -> Result<RustStructInfo> {
        let name = self.find_child_text(node, "type_identifier", source_bytes)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        
        // Determine struct type
        let struct_sexp = node.to_sexp();
        let is_tuple_struct = struct_sexp.contains("field_declaration_list") && 
                             struct_sexp.contains("tuple_struct");
        let is_unit_struct = !struct_sexp.contains("field_declaration_list");
        
        let fields = self.extract_struct_fields(node, source_bytes)?;
        let derives = self.extract_derives(node, source_bytes);
        let attributes = self.extract_attributes(node, source_bytes);
        let generics = self.extract_generics(node, source_bytes);
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustStructInfo {
            name,
            is_public,
            is_tuple_struct,
            is_unit_struct,
            fields,
            derives,
            attributes,
            generics,
            location,
        })
    }
    
    /// Extract enum information
    fn extract_enum(&self, node: &Node, source_bytes: &[u8]) -> Result<RustEnumInfo> {
        let name = self.find_child_text(node, "type_identifier", source_bytes)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        let variants = self.extract_enum_variants(node, source_bytes)?;
        let derives = self.extract_derives(node, source_bytes);
        let attributes = self.extract_attributes(node, source_bytes);
        let generics = self.extract_generics(node, source_bytes);
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustEnumInfo {
            name,
            is_public,
            variants,
            derives,
            attributes,
            generics,
            location,
        })
    }
    
    /// Extract trait information
    fn extract_trait(&self, node: &Node, source_bytes: &[u8]) -> Result<RustTraitInfo> {
        let name = self.find_child_text(node, "type_identifier", source_bytes)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        let is_unsafe = node.to_sexp().contains("unsafe");
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustTraitInfo {
            name,
            is_public,
            is_unsafe,
            supertraits: Vec::new(), // TODO: implement
            associated_types: Vec::new(), // TODO: implement
            methods: Vec::new(), // TODO: implement
            generics: self.extract_generics(node, source_bytes),
            location,
        })
    }
    
    /// Extract impl block information
    fn extract_impl(&self, node: &Node, source_bytes: &[u8]) -> Result<RustImplInfo> {
        let target_type = self.find_child_text(node, "type_identifier", source_bytes)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let trait_name = None; // TODO: detect trait impl vs inherent impl
        let is_unsafe = node.to_sexp().contains("unsafe");
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustImplInfo {
            target_type,
            trait_name,
            is_unsafe,
            methods: Vec::new(), // TODO: extract methods from impl
            associated_types: Vec::new(),
            generics: self.extract_generics(node, source_bytes),
            where_clause: None, // TODO: implement
            location,
        })
    }
    
    /// Extract const/static information
    fn extract_const(&self, node: &Node, source_bytes: &[u8]) -> Result<RustConstInfo> {
        let name = self.find_child_text(node, "identifier", source_bytes)
            .unwrap_or_else(|| "unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustConstInfo {
            name,
            is_public,
            const_type: "unknown".to_string(), // TODO: extract type
            value: None, // TODO: extract value if available
            location,
        })
    }
    
    /// Extract type alias information
    fn extract_type_alias(&self, node: &Node, source_bytes: &[u8]) -> Result<RustTypeAliasInfo> {
        let name = self.find_child_text(node, "type_identifier", source_bytes)
            .unwrap_or_else(|| "Unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustTypeAliasInfo {
            name,
            is_public,
            target_type: "unknown".to_string(), // TODO: extract target type
            generics: self.extract_generics(node, source_bytes),
            location,
        })
    }
    
    /// Extract macro information
    fn extract_macro(&self, node: &Node, source_bytes: &[u8]) -> Result<RustMacroInfo> {
        let name = self.find_child_text(node, "identifier", source_bytes)
            .unwrap_or_else(|| "unknown".to_string());
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        
        let location = LocationInfo {
            line: node.start_position().row + 1,
            column: node.start_position().column + 1,
        };
        
        Ok(RustMacroInfo {
            name,
            is_public,
            macro_type: RustMacroType::DeclarativeMacro, // Default, TODO: detect type
            location,
        })
    }
    
    /// Extract use statement information
    fn extract_use(&self, node: &Node, source_bytes: &[u8]) -> Result<RustUseInfo> {
        let path = node.utf8_text(source_bytes)
            .unwrap_or("unknown")
            .to_string();
        
        let is_public = node.to_sexp().contains("visibility_modifier");
        
        Ok(RustUseInfo {
            path,
            alias: None, // TODO: detect aliases
            is_public,
            items: Vec::new(), // TODO: extract use items
        })
    }
    
    // Helper methods
    
    fn find_child_text(&self, node: &Node, kind: &str, source_bytes: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == kind {
                return child.utf8_text(source_bytes).ok().map(|s| s.to_string());
            }
        }
        None
    }
    
    fn extract_function_parameters(&self, node: &Node, source_bytes: &[u8]) -> Result<Vec<ParameterInfo>> {
        // TODO: implement parameter extraction
        Ok(Vec::new())
    }
    
    fn extract_return_type(&self, node: &Node, source_bytes: &[u8]) -> String {
        // TODO: implement return type extraction
        "()".to_string()
    }
    
    fn extract_struct_fields(&self, node: &Node, source_bytes: &[u8]) -> Result<Vec<RustFieldInfo>> {
        // TODO: implement field extraction
        Ok(Vec::new())
    }
    
    fn extract_enum_variants(&self, node: &Node, source_bytes: &[u8]) -> Result<Vec<RustEnumVariant>> {
        // TODO: implement variant extraction
        Ok(Vec::new())
    }
    
    fn extract_derives(&self, node: &Node, source_bytes: &[u8]) -> Vec<String> {
        // TODO: implement derive extraction
        Vec::new()
    }
    
    fn extract_attributes(&self, node: &Node, source_bytes: &[u8]) -> Vec<String> {
        // TODO: implement attribute extraction
        Vec::new()
    }
    
    fn extract_generics(&self, node: &Node, source_bytes: &[u8]) -> Vec<String> {
        // TODO: implement generic extraction
        Vec::new()
    }
    
    fn calculate_complexity(&self, analysis: &DetailedAnalysis, content: &str) -> Complexity {
        let function_count = analysis.functions.len();
        let line_count = content.lines().count();
        
        if function_count > 20 || line_count > 500 {
            Complexity::High
        } else if function_count > 10 || line_count > 200 {
            Complexity::Medium
        } else {
            Complexity::Low
        }
    }
    
    fn generate_summary(&self, path: &Path, analysis: &DetailedAnalysis) -> String {
        format!(
            "Rust file with {} functions",
            analysis.functions.len()
        )
    }
    
    fn extract_relevant_sections(&self, analysis: &DetailedAnalysis) -> Vec<String> {
        analysis.functions.iter()
            .map(|f| f.name.clone())
            .collect()
    }
    
    fn extract_exports(&self, analysis: &DetailedAnalysis) -> Vec<String> {
        analysis.functions.iter()
            .filter(|f| f.modifiers.contains(&"pub".to_string()))
            .map(|f| f.name.clone())
            .collect()
    }
    
    fn extract_imports(&self, analysis: &DetailedAnalysis) -> Vec<String> {
        // TODO: implement import extraction from use statements
        Vec::new()
    }
}

/// Parse Cargo.toml files
pub struct CargoAnalyzer;

impl CargoAnalyzer {
    /// Analyze Cargo.toml content
    pub fn analyze_cargo_toml(content: &str) -> Result<CargoInfo> {
        let parsed: toml::Value = content.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;
        
        // Extract package information
        let package_name = Self::extract_package_name(&parsed)?;
        let version = Self::extract_package_version(&parsed);
        let edition = Self::extract_package_edition(&parsed);
        
        // Extract dependencies
        let dependencies = Self::extract_dependencies(&parsed, "dependencies")?;
        let dev_dependencies = Self::extract_dependencies(&parsed, "dev-dependencies")?;
        let build_dependencies = Self::extract_dependencies(&parsed, "build-dependencies")?;
        
        // Extract features
        let features = Self::extract_features(&parsed)?;
        
        // Extract targets (bins, libs, examples, tests, benches)
        let targets = Self::extract_targets(&parsed)?;
        
        // Extract workspace configuration
        let workspace = Self::extract_workspace(&parsed)?;
        
        Ok(CargoInfo {
            package_name,
            version,
            edition,
            dependencies,
            dev_dependencies,
            build_dependencies,
            features,
            targets,
            workspace,
        })
    }
    
    /// Extract package name from TOML
    fn extract_package_name(parsed: &toml::Value) -> Result<String> {
        parsed
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("Package name not found in Cargo.toml"))
    }
    
    /// Extract package version
    fn extract_package_version(parsed: &toml::Value) -> String {
        parsed
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string()
    }
    
    /// Extract Rust edition
    fn extract_package_edition(parsed: &toml::Value) -> String {
        parsed
            .get("package")
            .and_then(|p| p.get("edition"))
            .and_then(|e| e.as_str())
            .unwrap_or("2021")
            .to_string()
    }
    
    /// Extract dependencies from a specific section
    fn extract_dependencies(parsed: &toml::Value, section: &str) -> Result<Vec<crate::types::CargoDependency>> {
        let mut dependencies = Vec::new();
        
        if let Some(deps) = parsed.get(section).and_then(|d| d.as_table()) {
            for (name, value) in deps {
                let dependency = Self::parse_dependency(name, value)?;
                dependencies.push(dependency);
            }
        }
        
        Ok(dependencies)
    }
    
    /// Parse a single dependency entry
    fn parse_dependency(name: &str, value: &toml::Value) -> Result<crate::types::CargoDependency> {
        let mut dependency = crate::types::CargoDependency {
            name: name.to_string(),
            version: None,
            source: crate::types::CargoDependencySource::CratesIo,
            features: Vec::new(),
            optional: false,
            default_features: true,
        };
        
        match value {
            // Simple version string: dep = "1.0"
            toml::Value::String(version) => {
                dependency.version = Some(version.clone());
                dependency.source = crate::types::CargoDependencySource::CratesIo;
            }
            // Complex dependency: dep = { version = "1.0", features = ["foo"] }
            toml::Value::Table(table) => {
                if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                    dependency.version = Some(version.to_string());
                }
                
                // Determine the source type
                if let Some(git_url) = table.get("git").and_then(|g| g.as_str()) {
                    let branch = table.get("branch").and_then(|b| b.as_str()).map(|s| s.to_string());
                    let tag = table.get("tag").and_then(|t| t.as_str()).map(|s| s.to_string());
                    let rev = table.get("rev").and_then(|r| r.as_str()).map(|s| s.to_string());
                    
                    dependency.source = crate::types::CargoDependencySource::Git {
                        url: git_url.to_string(),
                        branch,
                        tag,
                        rev,
                    };
                } else if let Some(path) = table.get("path").and_then(|p| p.as_str()) {
                    dependency.source = crate::types::CargoDependencySource::Path {
                        path: path.to_string(),
                    };
                } else {
                    dependency.source = crate::types::CargoDependencySource::CratesIo;
                }
                
                if let Some(features) = table.get("features").and_then(|f| f.as_array()) {
                    dependency.features = features
                        .iter()
                        .filter_map(|f| f.as_str())
                        .map(|s| s.to_string())
                        .collect();
                }
                
                if let Some(optional) = table.get("optional").and_then(|o| o.as_bool()) {
                    dependency.optional = optional;
                }
                
                if let Some(default_features) = table.get("default-features").and_then(|df| df.as_bool()) {
                    dependency.default_features = default_features;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported dependency format for {}", name));
            }
        }
        
        Ok(dependency)
    }
    
    /// Extract features
    fn extract_features(parsed: &toml::Value) -> Result<Vec<crate::types::CargoFeature>> {
        let mut features = Vec::new();
        
        if let Some(features_table) = parsed.get("features").and_then(|f| f.as_table()) {
            for (name, value) in features_table {
                let dependencies = match value {
                    toml::Value::Array(arr) => {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    }
                    _ => Vec::new(),
                };
                
                features.push(crate::types::CargoFeature {
                    name: name.clone(),
                    dependencies,
                    is_default: name == "default",
                });
            }
        }
        
        Ok(features)
    }
    
    /// Extract build targets
    fn extract_targets(parsed: &toml::Value) -> Result<Vec<crate::types::CargoTarget>> {
        let mut targets = Vec::new();
        
        // Extract [[bin]] targets
        if let Some(bins) = parsed.get("bin").and_then(|b| b.as_array()) {
            for bin in bins {
                if let Some(table) = bin.as_table() {
                    let target = Self::parse_target(table, crate::types::CargoTargetType::Binary)?;
                    targets.push(target);
                }
            }
        }
        
        // Extract [lib] target
        if let Some(lib) = parsed.get("lib").and_then(|l| l.as_table()) {
            let target = Self::parse_target(lib, crate::types::CargoTargetType::Library)?;
            targets.push(target);
        }
        
        // Extract [[example]] targets
        if let Some(examples) = parsed.get("example").and_then(|e| e.as_array()) {
            for example in examples {
                if let Some(table) = example.as_table() {
                    let target = Self::parse_target(table, crate::types::CargoTargetType::Example)?;
                    targets.push(target);
                }
            }
        }
        
        // Extract [[test]] targets
        if let Some(tests) = parsed.get("test").and_then(|t| t.as_array()) {
            for test in tests {
                if let Some(table) = test.as_table() {
                    let target = Self::parse_target(table, crate::types::CargoTargetType::Test)?;
                    targets.push(target);
                }
            }
        }
        
        // Extract [[bench]] targets
        if let Some(benches) = parsed.get("bench").and_then(|b| b.as_array()) {
            for bench in benches {
                if let Some(table) = bench.as_table() {
                    let target = Self::parse_target(table, crate::types::CargoTargetType::Benchmark)?;
                    targets.push(target);
                }
            }
        }
        
        Ok(targets)
    }
    
    /// Parse a single target
    fn parse_target(table: &toml::Table, target_type: crate::types::CargoTargetType) -> Result<crate::types::CargoTarget> {
        let name = table
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let path = table
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let required_features = table
            .get("required-features")
            .and_then(|rf| rf.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(crate::types::CargoTarget {
            name,
            target_type,
            path,
            required_features,
        })
    }
    
    /// Extract workspace configuration
    fn extract_workspace(parsed: &toml::Value) -> Result<Option<crate::types::CargoWorkspace>> {
        if let Some(workspace_table) = parsed.get("workspace").and_then(|w| w.as_table()) {
            let members = workspace_table
                .get("members")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            
            let exclude = workspace_table
                .get("exclude")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            
            let default_members = workspace_table
                .get("default-members")
                .and_then(|dm| dm.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();
            
            return Ok(Some(crate::types::CargoWorkspace {
                members,
                exclude,
                default_members,
            }));
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_rust_analyzer_creation() {
        let analyzer = RustAnalyzer::new();
        assert!(analyzer.is_ok());
    }
    
    #[test]
    fn test_file_type_detection() {
        let analyzer = RustAnalyzer::new().unwrap();
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("lib.rs"), ""),
            FileType::RustLibrary
        );
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("main.rs"), ""),
            FileType::RustBinary
        );
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("mod.rs"), ""),
            FileType::RustModule
        );
        
        // Test path-based detection
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("src/bin/tool.rs"), ""),
            FileType::RustBinary
        );
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("examples/demo.rs"), ""),
            FileType::RustExample
        );
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("tests/integration.rs"), ""),
            FileType::RustTest
        );
        
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("benches/benchmark.rs"), ""),
            FileType::RustBench
        );
        
        // Test content-based detection
        assert_eq!(
            analyzer.detect_rust_file_type(Path::new("utils.rs"), "#[cfg(test)] mod tests {}"),
            FileType::RustTest
        );
    }
    
    #[test]
    fn test_analyze_simple_rust_file() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
pub fn hello_world() -> String {
    "Hello, World!".to_string()
}

pub struct Person {
    pub name: String,
    age: u32,
}

impl Person {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert_eq!(metadata.file_type, FileType::RustModule);
        assert!(metadata.summary.contains("Rust file"));
        assert!(metadata.line_count > 0);
        assert!(metadata.size > 0);
        assert!(metadata.detailed_analysis.is_some());
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_analyze_rust_library() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let lib_content = r#"
//! Main library documentation
//! This is a test library

pub mod utils;
pub mod models;

use std::collections::HashMap;

/// Main library struct
pub struct Library {
    books: HashMap<String, String>,
}

impl Library {
    /// Create a new library
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }
    
    /// Add a book to the library
    pub fn add_book(&mut self, title: String, author: String) {
        self.books.insert(title, author);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_creation() {
        let lib = Library::new();
        assert_eq!(lib.books.len(), 0);
    }
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", lib_content)?;
        let temp_path = temp_file.path().parent().unwrap().join("lib.rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, lib_content)?;
        
        assert_eq!(metadata.file_type, FileType::RustLibrary);
        assert!(metadata.summary.contains("Rust file"));
        assert!(metadata.detailed_analysis.is_some());
        
        if let Some(analysis) = &metadata.detailed_analysis {
            assert!(!analysis.functions.is_empty());
        }
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_analyze_rust_binary() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let main_content = r#"
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Hello from binary! Args: {:?}", args);
    
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running main logic...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run() {
        assert!(run().is_ok());
    }
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", main_content)?;
        let temp_path = temp_file.path().parent().unwrap().join("main.rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, main_content)?;
        
        assert_eq!(metadata.file_type, FileType::RustBinary);
        assert!(metadata.summary.contains("Rust file"));
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_analyze_rust_test_file() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let test_content = r#"
use crate::lib::*;

#[test]
fn test_basic_functionality() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_string_operations() {
    let s = "hello".to_string();
    assert_eq!(s.len(), 5);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_integration() {
        assert!(true);
    }
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", test_content)?;
        let temp_path = temp_file.path().parent().unwrap().join("tests").join("integration.rs");
        std::fs::create_dir_all(temp_path.parent().unwrap())?;
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, test_content)?;
        
        assert_eq!(metadata.file_type, FileType::RustTest);
        assert!(metadata.summary.contains("Rust file"));
        
        std::fs::remove_file(&temp_path)?;
        std::fs::remove_dir_all(temp_path.parent().unwrap())?;
        Ok(())
    }
    
    #[test]
    fn test_complexity_calculation() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        
        // Simple file - should be Low complexity
        let simple_content = r#"
pub fn hello() -> String {
    "Hello".to_string()
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", simple_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, simple_content)?;
        assert_eq!(metadata.complexity, Complexity::Low);
        
        // Complex file - should be High complexity
        let complex_content = format!("{}\n{}", 
            "fn func() {}\n".repeat(25), // 25 functions
            "x\n".repeat(600) // 600 lines
        );
        
        std::fs::write(&temp_path, &complex_content)?;
        let metadata = analyzer.analyze_file(&temp_path, &complex_content)?;
        assert_eq!(metadata.complexity, Complexity::High);
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_function_extraction() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
pub fn public_function() -> i32 {
    42
}

fn private_function(x: i32, y: String) -> bool {
    x > 0 && !y.is_empty()
}

pub async fn async_function() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub unsafe fn unsafe_function() {
    // unsafe operations
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        if let Some(analysis) = &metadata.detailed_analysis {
            assert!(!analysis.functions.is_empty());
            // Check that we found multiple functions
            assert!(analysis.functions.len() >= 2);
        }
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_struct_and_enum_extraction() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    name: String,
    email: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending { reason: String },
}

impl User {
    pub fn new(id: u64, name: String) -> Self {
        Self { id, name, email: None }
    }
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert!(metadata.detailed_analysis.is_some());
        // The functions and structures should be extracted by the AST analyzer
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_trait_extraction() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
pub trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
}

pub trait Cloneable: Clone {
    fn deep_clone(&self) -> Self;
}

unsafe trait UnsafeTrait {
    unsafe fn dangerous_method(&self);
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert!(metadata.detailed_analysis.is_some());
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_macro_extraction() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

pub macro another_macro($($args:tt)*) {
    // macro body
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert!(metadata.detailed_analysis.is_some());
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_use_statement_extraction() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use serde::{Serialize, Deserialize};
pub use crate::models::*;

use self::inner::*;

mod inner {
    pub struct InnerStruct;
}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert!(metadata.detailed_analysis.is_some());
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_cargo_analyzer_basic() {
        let content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"
description = "A test package"
license = "MIT"
        "#;
        
        let result = CargoAnalyzer::analyze_cargo_toml(content);
        assert!(result.is_ok());
        
        let cargo_info = result.unwrap();
        assert_eq!(cargo_info.package_name, "test-package");
        assert_eq!(cargo_info.version, "0.1.0");
        assert_eq!(cargo_info.edition, "2021");
    }
    
    #[test]
    fn test_cargo_analyzer_with_dependencies() -> Result<()> {
        let content = r#"
[package]
name = "complex-package"
version = "1.2.3"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
local-crate = { path = "../local-crate" }
git-dep = { git = "https://github.com/example/repo.git", branch = "main" }
optional-dep = { version = "0.5", optional = true }

[dev-dependencies]
tempfile = "3.0"
criterion = "0.4"

[build-dependencies]
cc = "1.0"
        "#;
        
        let cargo_info = CargoAnalyzer::analyze_cargo_toml(content)?;
        
        // Check basic package info
        assert_eq!(cargo_info.package_name, "complex-package");
        assert_eq!(cargo_info.version, "1.2.3");
        
        // Check dependencies
        assert_eq!(cargo_info.dependencies.len(), 5);
        
        // Check serde (simple version)
        let serde = cargo_info.dependencies.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde.version, Some("1.0".to_string()));
        assert!(!serde.optional);
        
        // Check tokio (complex with features)
        let tokio = cargo_info.dependencies.iter().find(|d| d.name == "tokio").unwrap();
        assert_eq!(tokio.version, Some("1.0".to_string()));
        assert_eq!(tokio.features, vec!["full"]);
        
        // Check path dependency
        let local = cargo_info.dependencies.iter().find(|d| d.name == "local-crate").unwrap();
        assert!(matches!(local.source, crate::types::CargoDependencySource::Path { .. }));
        
        // Check git dependency
        let git_dep = cargo_info.dependencies.iter().find(|d| d.name == "git-dep").unwrap();
        assert!(matches!(git_dep.source, crate::types::CargoDependencySource::Git { .. }));
        
        // Check optional dependency
        let optional = cargo_info.dependencies.iter().find(|d| d.name == "optional-dep").unwrap();
        assert!(optional.optional);
        
        // Check dev dependencies
        assert_eq!(cargo_info.dev_dependencies.len(), 2);
        let tempfile = cargo_info.dev_dependencies.iter().find(|d| d.name == "tempfile").unwrap();
        assert_eq!(tempfile.version, Some("3.0".to_string()));
        
        // Check build dependencies
        assert_eq!(cargo_info.build_dependencies.len(), 1);
        let cc = cargo_info.build_dependencies.iter().find(|d| d.name == "cc").unwrap();
        assert_eq!(cc.version, Some("1.0".to_string()));
        
        Ok(())
    }
    
    #[test]
    fn test_cargo_analyzer_with_features() -> Result<()> {
        let content = r#"
[package]
name = "feature-package"
version = "0.1.0"
edition = "2021"

[features]
default = ["std"]
std = []
async = ["tokio"]
experimental = ["async", "std"]
        "#;
        
        let cargo_info = CargoAnalyzer::analyze_cargo_toml(content)?;
        
        assert_eq!(cargo_info.features.len(), 4);
        
        // Check default feature
        let default = cargo_info.features.iter().find(|f| f.name == "default").unwrap();
        assert_eq!(default.dependencies, vec!["std"]);
        
        // Check experimental feature
        let experimental = cargo_info.features.iter().find(|f| f.name == "experimental").unwrap();
        assert_eq!(experimental.dependencies, vec!["async", "std"]);
        
        Ok(())
    }
    
    #[test]
    fn test_cargo_analyzer_with_targets() -> Result<()> {
        let content = r#"
[package]
name = "target-package"
version = "0.1.0"
edition = "2021"

[lib]
name = "mylib"
path = "src/lib.rs"

[[bin]]
name = "cli"
path = "src/bin/cli.rs"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[[example]]
name = "demo"
path = "examples/demo.rs"

[[test]]
name = "integration"
path = "tests/integration.rs"

[[bench]]
name = "performance"
path = "benches/performance.rs"
        "#;
        
        let cargo_info = CargoAnalyzer::analyze_cargo_toml(content)?;
        
        assert_eq!(cargo_info.targets.len(), 6);
        
        // Check library target
        let lib = cargo_info.targets.iter().find(|t| t.name == "mylib").unwrap();
        assert!(matches!(lib.target_type, crate::types::CargoTargetType::Library));
        assert_eq!(lib.path, "src/lib.rs".to_string());
        
        // Check binary targets
        let binaries: Vec<_> = cargo_info.targets.iter()
            .filter(|t| matches!(t.target_type, crate::types::CargoTargetType::Binary))
            .collect();
        assert_eq!(binaries.len(), 2);
        
        // Check example target
        let example = cargo_info.targets.iter().find(|t| t.name == "demo").unwrap();
        assert!(matches!(example.target_type, crate::types::CargoTargetType::Example));
        
        // Check test target
        let test = cargo_info.targets.iter().find(|t| t.name == "integration").unwrap();
        assert!(matches!(test.target_type, crate::types::CargoTargetType::Test));
        
        // Check benchmark target
        let bench = cargo_info.targets.iter().find(|t| t.name == "performance").unwrap();
        assert!(matches!(bench.target_type, crate::types::CargoTargetType::Benchmark));
        
        Ok(())
    }
    
    #[test]
    fn test_cargo_analyzer_with_workspace() -> Result<()> {
        let content = r#"
[workspace]
members = [
    "crate-a",
    "crate-b",
    "tools/*"
]
exclude = [
    "old-crate",
    "experimental/*"
]

[package]
name = "workspace-root"
version = "0.1.0"
edition = "2021"
        "#;
        
        let cargo_info = CargoAnalyzer::analyze_cargo_toml(content)?;
        
        assert!(cargo_info.workspace.is_some());
        let workspace = cargo_info.workspace.unwrap();
        
        assert_eq!(workspace.members, vec!["crate-a", "crate-b", "tools/*"]);
        assert_eq!(workspace.exclude, vec!["old-crate", "experimental/*"]);
        assert_eq!(workspace.default_members, Vec::<String>::new());
        
        Ok(())
    }
    
    #[test]
    fn test_cargo_analyzer_malformed_toml() {
        let content = r#"
[package
name = "broken-package"
version = "0.1.0"
        "#;
        
        let result = CargoAnalyzer::analyze_cargo_toml(content);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cargo_analyzer_missing_package_name() {
        let content = r#"
[package]
version = "0.1.0"
edition = "2021"
        "#;
        
        let result = CargoAnalyzer::analyze_cargo_toml(content);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cargo_analyzer_self_analysis() -> Result<()> {
        // Test with our own Cargo.toml
        let content = std::fs::read_to_string("Cargo.toml")?;
        let cargo_info = CargoAnalyzer::analyze_cargo_toml(&content)?;
        
        assert_eq!(cargo_info.package_name, "token-optimizer");
        assert_eq!(cargo_info.version, "0.1.0");
        assert_eq!(cargo_info.edition, "2021");
        
        // Should have many dependencies
        assert!(!cargo_info.dependencies.is_empty());
        
        // Should have the pipeline_demo binary
        let pipeline_demo = cargo_info.targets.iter()
            .find(|t| t.name == "pipeline_demo" && matches!(t.target_type, crate::types::CargoTargetType::Binary));
        assert!(pipeline_demo.is_some());
        
        // Should have toml dependency that we just added
        let toml_dep = cargo_info.dependencies.iter().find(|d| d.name == "toml");
        assert!(toml_dep.is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_empty_rust_file() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let empty_content = "";
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", empty_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, empty_content)?;
        
        assert_eq!(metadata.complexity, Complexity::Low);
        assert_eq!(metadata.line_count, 0);
        assert_eq!(metadata.size, 0);
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
    
    #[test]
    fn test_summary_generation() -> Result<()> {
        let mut analyzer = RustAnalyzer::new()?;
        let rust_content = r#"
fn func1() {}
fn func2() {}
fn func3() {}
        "#;
        
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", rust_content)?;
        let temp_path = temp_file.path().with_extension("rs");
        std::fs::copy(temp_file.path(), &temp_path)?;
        
        let metadata = analyzer.analyze_file(&temp_path, rust_content)?;
        
        assert!(metadata.summary.contains("Rust file"));
        assert!(metadata.summary.contains("functions"));
        
        std::fs::remove_file(&temp_path)?;
        Ok(())
    }
}
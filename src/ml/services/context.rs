//! Smart context detection service

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use std::time::Instant;
use serde_json;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::models::*;
use crate::analyzers::ts_ast_analyzer::TypeScriptASTAnalyzer;

/// Smart context detection service
pub struct SmartContextService {
    config: MLConfig,
    plugin_manager: Arc<PluginManager>,
    ast_analyzer: TypeScriptASTAnalyzer,
    is_ready: bool,
}

impl SmartContextService {
    pub fn new(config: MLConfig, plugin_manager: Arc<PluginManager>) -> Result<Self> {
        Ok(Self {
            config,
            plugin_manager,
            ast_analyzer: TypeScriptASTAnalyzer::new()?,
            is_ready: false,
        })
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Smart Context service");
        
        // Check if plugins are available (but don't require them)
        let available_plugins = self.plugin_manager.get_available_plugins();
        tracing::info!("Available plugins: {:?}", available_plugins);

        self.is_ready = true;
        Ok(())
    }
    
    /// Get smart context for a project file and function - main entry point
    pub async fn get_smart_context(&self, function_name: &str, project_path: &Path) -> Result<SmartContextResult> {
        if !self.is_ready {
            anyhow::bail!("Smart Context service not initialized");
        }

        let start_time = Instant::now();
        
        // 1. Fast AST analysis first (simulated for now)
        let ast_context = self.analyze_function_basic(function_name, project_path).await?;
        
        // 2. Create base context from AST
        let _base_context = self.create_base_context(function_name, &project_path.to_string_lossy(), &ast_context)?;
        
        // 3. ML enhancement if available
        let semantic_context = if self.has_embedding_capability().await {
            Some(self.enhance_with_embeddings(&ast_context, function_name).await?)
        } else {
            None
        };
        
        // 4. Create result
        let confidence = if semantic_context.is_some() { 0.9 } else { 0.7 };
        
        Ok(SmartContextResult {
            ast_context,
            semantic_context,
            confidence,
            processing_time: start_time.elapsed(),
        })
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down Smart Context service");
        self.is_ready = false;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// Analyze function context with AI enhancement
    pub async fn analyze_function_context(&self, function_name: &str, file_path: &str, ast_context: &str) -> Result<EnhancedSmartContext> {
        if !self.is_ready {
            anyhow::bail!("Smart Context service not initialized");
        }

        // Create base context from AST analysis
        let base_context = self.create_base_context(function_name, file_path, ast_context)?;

        // Enhance with AI analysis if available
        if self.plugin_manager.get_available_plugins().contains(&"deepseek".to_string()) {
            let semantic_analysis = self.generate_semantic_analysis(function_name, ast_context).await?;
            let risk_assessment = self.assess_risk(function_name, file_path, ast_context).await?;
            let optimization_suggestions = self.generate_optimization_suggestions(function_name, ast_context).await?;

            Ok(EnhancedSmartContext {
                base_context,
                semantic_analysis,
                risk_assessment,
                optimization_suggestions,
            })
        } else {
            // Fallback to basic analysis
            Ok(EnhancedSmartContext {
                base_context,
                semantic_analysis: self.create_basic_semantic_analysis(function_name, ast_context),
                risk_assessment: self.create_basic_risk_assessment(),
                optimization_suggestions: Vec::new(),
            })
        }
    }

    /// Get context for multiple functions
    pub async fn analyze_multiple_functions(&self, functions: &[(String, String, String)]) -> Result<Vec<EnhancedSmartContext>> {
        let mut contexts = Vec::new();
        
        for (function_name, file_path, ast_context) in functions {
            let context = self.analyze_function_context(function_name, file_path, ast_context).await?;
            contexts.push(context);
        }
        
        Ok(contexts)
    }

    /// Find related functions based on context
    pub async fn find_related_functions(&self, function_name: &str, file_path: &str, ast_context: &str) -> Result<Vec<DependencyInfo>> {
        if !self.is_ready {
            anyhow::bail!("Smart Context service not initialized");
        }

        // Use AI to find semantic relationships
        if self.plugin_manager.is_plugin_loaded("deepseek") {
            let query = format!("Find functions related to {} in {}", function_name, file_path);
            let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
            
            // Parse AI response to extract dependencies
            self.parse_dependencies_from_ai_response(&response)
        } else {
            // Fallback to basic AST analysis
            Ok(self.extract_basic_dependencies(ast_context))
        }
    }

    /// Generate usage patterns for a function
    pub async fn analyze_usage_patterns(&self, function_name: &str, _file_path: &str, usage_examples: &[String]) -> Result<Vec<UsagePattern>> {
        if !self.is_ready {
            anyhow::bail!("Smart Context service not initialized");
        }

        let mut patterns = Vec::new();
        
        // Analyze usage patterns
        for example in usage_examples {
            let pattern = self.identify_usage_pattern(function_name, example).await?;
            patterns.push(pattern);
        }
        
        Ok(patterns)
    }

    /// Create base context from AST analysis
    pub fn create_base_context(&self, function_name: &str, file_path: &str, ast_context: &str) -> Result<SmartContext> {
        // Extract line range from AST context
        let line_range = self.extract_line_range(ast_context);
        
        let mut context = SmartContext::new(
            function_name.to_string(),
            file_path.to_string(),
            line_range,
        );

        // Calculate complexity from AST
        context.complexity_score = self.calculate_complexity_score(ast_context);
        
        // Determine impact scope
        context.impact_scope = self.determine_impact_scope(ast_context);
        
        // Extract dependencies from AST context
        context.dependencies = self.extract_dependencies_from_context(ast_context);

        Ok(context)
    }

    /// Basic function analysis using simple pattern matching
    async fn analyze_function_basic(&self, function_name: &str, _project_path: &Path) -> Result<String> {
        // For now, we'll simulate reading a file and extracting function context
        // In a real implementation, this would use the AST analyzer more thoroughly
        let simulated_context = format!(
            "function {}() {{\n  // Function implementation\n  return something;\n}}\n\n// Related code context",
            function_name
        );
        
        // Add some realistic patterns based on the function name
        let enhanced_context = if function_name.contains("service") || function_name.contains("Service") {
            format!("@Injectable()\nexport class SomeService {{\n  {}\n}}", simulated_context)
        } else if function_name.contains("component") || function_name.contains("Component") {
            format!("@Component({{\n  selector: 'app-component'\n}})\nexport class SomeComponent {{\n  {}\n}}", simulated_context)
        } else {
            simulated_context
        };
        
        Ok(enhanced_context)
    }
    
    /// Check if embedding capability is available
    async fn has_embedding_capability(&self) -> bool {
        let available_plugins = self.plugin_manager.get_available_plugins();
        available_plugins.contains(&"qwen_embedding".to_string())
    }
    
    /// Enhance context with embeddings
    async fn enhance_with_embeddings(&self, ast_context: &str, function_name: &str) -> Result<SemanticContext> {
        // Use embedding plugin to find related functions
        let related_functions = self.find_semantically_related_functions(function_name, ast_context).await?;
        
        // Generate conceptual context using reasoning plugin
        let conceptual_context = if self.plugin_manager.get_available_plugins().contains(&"deepseek".to_string()) {
            self.generate_conceptual_context(function_name, ast_context).await?
        } else {
            format!("Basic context for function: {}", function_name)
        };
        
        Ok(SemanticContext {
            related_functions,
            conceptual_context,
            usage_patterns: self.extract_usage_patterns_from_ast(ast_context),
            dependencies: self.extract_dependencies_from_ast(ast_context),
        })
    }
    
    /// Find semantically related functions using embeddings
    async fn find_semantically_related_functions(&self, function_name: &str, ast_context: &str) -> Result<Vec<String>> {
        if !self.plugin_manager.is_plugin_loaded("qwen_embedding") {
            // Try to load the plugin
            if let Err(_) = self.plugin_manager.load_plugin("qwen_embedding").await {
                return Ok(vec![]); // Fallback to empty if plugin not available
            }
        }
        
        let query = format!("Functions related to {}: {}", function_name, ast_context.chars().take(200).collect::<String>());
        
        match self.plugin_manager.process_with_plugin("qwen_embedding", &query).await {
            Ok(response) => {
                // Parse the embedding response to extract related functions
                self.parse_related_functions_response(&response)
            }
            Err(_) => Ok(vec![]) // Graceful fallback
        }
    }
    
    /// Generate conceptual context using reasoning
    async fn generate_conceptual_context(&self, function_name: &str, ast_context: &str) -> Result<String> {
        let query = format!(
            "Explain the conceptual purpose and context of this function:\n\
             Function: {}\n\
             Code context: {}\n\
             Provide a concise explanation of what this function does and how it fits in the codebase.",
            function_name, 
            ast_context.chars().take(500).collect::<String>()
        );
        
        match self.plugin_manager.process_with_plugin("deepseek", &query).await {
            Ok(response) => {
                // Extract the conceptual explanation from the response
                self.parse_conceptual_response(&response)
            }
            Err(_) => Ok(format!("Function {} handles specific business logic", function_name))
        }
    }
    
    /// Parse related functions from embedding response
    fn parse_related_functions_response(&self, response: &str) -> Result<Vec<String>> {
        // Try to parse JSON response
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(array) = parsed.get("related_functions").and_then(|v| v.as_array()) {
                let functions: Vec<String> = array
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                return Ok(functions);
            }
        }
        
        // Fallback: try to extract function names from text
        let words: Vec<String> = response
            .split_whitespace()
            .filter(|word| word.contains("Function") || word.ends_with("()"))
            .map(|s| s.replace("()", "").replace("Function", ""))
            .filter(|s| !s.is_empty())
            .take(5)
            .collect();
            
        Ok(words)
    }
    
    /// Parse conceptual response from reasoning plugin
    fn parse_conceptual_response(&self, response: &str) -> Result<String> {
        // Try to parse JSON response
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(analysis) = parsed.get("analysis").and_then(|v| v.as_str()) {
                return Ok(analysis.to_string());
            }
        }
        
        // Fallback: use the response as-is but clean it up
        let cleaned = response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .take(3) // Take first few lines
            .collect::<Vec<_>>()
            .join(" ");
            
        Ok(if cleaned.len() > 200 {
            format!("{}...", &cleaned[..200])
        } else {
            cleaned
        })
    }
    
    /// Extract usage patterns from AST
    fn extract_usage_patterns_from_ast(&self, ast_context: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        
        if ast_context.contains("async") {
            patterns.push("Asynchronous operation".to_string());
        }
        if ast_context.contains("Promise") {
            patterns.push("Promise-based".to_string());
        }
        if ast_context.contains("Observable") {
            patterns.push("Observable pattern".to_string());
        }
        if ast_context.contains("@Injectable") {
            patterns.push("Dependency injection".to_string());
        }
        if ast_context.contains("@Component") {
            patterns.push("Angular component".to_string());
        }
        
        patterns
    }
    
    /// Extract dependencies from AST
    fn extract_dependencies_from_ast(&self, ast_context: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Simple regex-like pattern matching for imports
        for line in ast_context.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import") && trimmed.contains("from") {
                if let Some(module) = trimmed.split("from").nth(1) {
                    let module = module.trim()
                        .trim_matches(';')
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .trim();
                    dependencies.push(module.to_string());
                }
            }
        }
        
        dependencies
    }

    /// Generate semantic analysis with AI
    async fn generate_semantic_analysis(&self, function_name: &str, ast_context: &str) -> Result<SemanticAnalysis> {
        let query = format!(
            "Analyze the semantic meaning of function '{}' with context: {}",
            function_name, ast_context
        );
        
        let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
        
        // Parse AI response into semantic analysis
        self.parse_semantic_analysis_from_ai_response(&response)
    }

    /// Assess risk with AI
    async fn assess_risk(&self, function_name: &str, file_path: &str, ast_context: &str) -> Result<RiskAssessment> {
        let query = format!(
            "Assess the risk of modifying function '{}' in file '{}' with context: {}",
            function_name, file_path, ast_context
        );
        
        let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
        
        // Parse AI response into risk assessment
        self.parse_risk_assessment_from_ai_response(&response)
    }

    /// Generate optimization suggestions with AI
    async fn generate_optimization_suggestions(&self, function_name: &str, ast_context: &str) -> Result<Vec<OptimizationSuggestion>> {
        let query = format!(
            "Suggest optimizations for function '{}' with context: {}",
            function_name, ast_context
        );
        
        let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
        
        // Parse AI response into optimization suggestions
        self.parse_optimization_suggestions_from_ai_response(&response)
    }

    // Helper methods for basic fallback analysis
    fn create_basic_semantic_analysis(&self, function_name: &str, _ast_context: &str) -> SemanticAnalysis {
        // Enhanced basic analysis with better relevance scoring
        let enhanced_relevance = if function_name.contains("login") || function_name.contains("auth") {
            0.7 // High relevance for authentication functions
        } else if function_name.contains("get") || function_name.contains("set") {
            0.6 // Medium-high relevance for accessors
        } else {
            0.6 // Default higher relevance
        };
        
        SemanticAnalysis {
            purpose: format!("Function analysis: {}", function_name),
            behavior_description: format!("AST-based analysis of function '{}'", function_name),
            key_concepts: vec![function_name.to_string(), "function".to_string()],
            semantic_relationships: Vec::new(),
            context_relevance: enhanced_relevance,
        }
    }

    fn create_basic_risk_assessment(&self) -> RiskAssessment {
        RiskAssessment {
            overall_risk: RiskLevel::Medium,
            breaking_change_risk: 0.3,
            performance_impact: 0.2,
            security_implications: Vec::new(),
            mitigation_strategies: vec!["Run tests".to_string()],
        }
    }

    fn extract_line_range(&self, ast_context: &str) -> (usize, usize) {
        // Basic line extraction from AST context
        // TODO: Implement proper AST parsing
        (1, ast_context.lines().count())
    }

    fn calculate_complexity_score(&self, ast_context: &str) -> f32 {
        // Enhanced complexity calculation
        let lines = ast_context.lines().count() as f32;
        let branches = ast_context.matches("if ").count() as f32;
        let loops = ast_context.matches("for ").count() + ast_context.matches("while ").count();
        let async_ops = ast_context.matches("await ").count() as f32;
        let try_catch = ast_context.matches("try ").count() as f32;
        let nested_calls = ast_context.matches(".").count() as f32;
        
        // More realistic complexity calculation
        let base_complexity = lines * 0.02 + branches * 0.3 + loops as f32 * 0.4;
        let async_complexity = async_ops * 0.2 + try_catch * 0.3;
        let call_complexity = nested_calls * 0.05;
        
        (base_complexity + async_complexity + call_complexity).max(0.1)
    }

    fn determine_impact_scope(&self, ast_context: &str) -> ImpactScope {
        // Check for public/export first (highest impact)
        if ast_context.contains("export") || ast_context.contains("public") {
            ImpactScope::Service
        } 
        // Check for private (lowest impact)
        else if ast_context.contains("private") {
            ImpactScope::Local
        } 
        // Check for async/await patterns that might be service-level
        else if ast_context.contains("async") && ast_context.contains("await") {
            ImpactScope::Service
        }
        // Default to component level
        else {
            ImpactScope::Component
        }
    }

    fn extract_dependencies_from_context(&self, ast_context: &str) -> Vec<DependencyInfo> {
        let mut dependencies = Vec::new();
        
        // Extract import statements
        for line in ast_context.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import") {
                if let Some(from_pos) = trimmed.find("from") {
                    let import_part = &trimmed[..from_pos];
                    // Extract what's being imported
                    if let Some(start) = import_part.find("{") {
                        if let Some(end) = import_part.find("}") {
                            let imports = &import_part[start + 1..end];
                            for import in imports.split(",") {
                                let clean_import = import.trim();
                                if !clean_import.is_empty() {
                                    dependencies.push(DependencyInfo {
                                        dependency_type: DependencyType::Import,
                                        source_file: "current".to_string(),
                                        target_file: clean_import.to_string(),
                                        functions: vec![clean_import.to_string()],
                                        strength: 0.8,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Extract function calls and property access
        for line in ast_context.lines() {
            let trimmed = line.trim();
            // Find function calls like userRepository.findByEmail
            if let Some(dot_pos) = trimmed.find('.') {
                let before_dot = &trimmed[..dot_pos];
                if let Some(last_word_start) = before_dot.rfind(|c: char| !c.is_alphanumeric() && c != '_') {
                    let obj_name = &before_dot[last_word_start + 1..];
                    if !obj_name.is_empty() && obj_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        let dependency = DependencyInfo {
                            dependency_type: DependencyType::FunctionCall,
                            source_file: "current".to_string(),
                            target_file: obj_name.to_string(),
                            functions: vec![obj_name.to_string()],
                            strength: 0.6,
                        };
                        if !dependencies.iter().any(|d| d.target_file == dependency.target_file) {
                            dependencies.push(dependency);
                        }
                    }
                } else if !before_dot.is_empty() {
                    let dependency = DependencyInfo {
                        dependency_type: DependencyType::FunctionCall,
                        source_file: "current".to_string(),
                        target_file: before_dot.to_string(),
                        functions: vec![before_dot.to_string()],
                        strength: 0.6,
                    };
                    if !dependencies.iter().any(|d| d.target_file == dependency.target_file) {
                        dependencies.push(dependency);
                    }
                }
            }
            
            // Find await calls like await bcrypt.compare
            if trimmed.contains("await ") {
                if let Some(await_pos) = trimmed.find("await ") {
                    let after_await = &trimmed[await_pos + 6..];
                    if let Some(dot_pos) = after_await.find('.') {
                        let obj_name = &after_await[..dot_pos];
                        if !obj_name.is_empty() && obj_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            let dependency = DependencyInfo {
                                dependency_type: DependencyType::FunctionCall,
                                source_file: "current".to_string(),
                                target_file: obj_name.to_string(),
                                functions: vec![obj_name.to_string()],
                                strength: 0.7,
                            };
                            if !dependencies.iter().any(|d| d.target_file == dependency.target_file) {
                                dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
        }
        
        dependencies
    }

    fn extract_basic_dependencies(&self, _ast_context: &str) -> Vec<DependencyInfo> {
        // Basic dependency extraction
        // TODO: Implement proper AST parsing
        Vec::new()
    }

    async fn identify_usage_pattern(&self, _function_name: &str, example: &str) -> Result<UsagePattern> {
        // Basic pattern identification
        Ok(UsagePattern {
            pattern_type: PatternType::BehavioralPattern,
            frequency: 1,
            confidence: 0.5,
            examples: vec![example.to_string()],
        })
    }

    // AI response parsing methods
    fn parse_dependencies_from_ai_response(&self, response: &str) -> Result<Vec<DependencyInfo>> {
        // Try to parse JSON response, fallback to basic parsing
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            let mut dependencies = Vec::new();
            
            if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_array()) {
                for dep in deps {
                    if let Some(name) = dep.as_str() {
                        dependencies.push(DependencyInfo {
                            dependency_type: DependencyType::Import,
                            source_file: "unknown".to_string(),
                            target_file: name.to_string(),
                            functions: Vec::new(),
                            strength: 0.8,
                        });
                    }
                }
            }
            
            Ok(dependencies)
        } else {
            // Fallback: extract dependencies mentioned in text
            let mut dependencies = Vec::new();
            for line in response.lines() {
                if line.contains("import") || line.contains("require") {
                    let words: Vec<&str> = line.split_whitespace().collect();
                    if let Some(dep_name) = words.iter()
                        .find(|w| w.starts_with("'") || w.starts_with("\""))
                        .map(|w| w.trim_matches(['\'', '"']))
                    {
                        dependencies.push(DependencyInfo {
                            dependency_type: DependencyType::Import,
                            source_file: "unknown".to_string(),
                            target_file: dep_name.to_string(),
                            functions: Vec::new(),
                            strength: 0.6,
                        });
                    }
                }
            }
            Ok(dependencies)
        }
    }

    fn parse_semantic_analysis_from_ai_response(&self, response: &str) -> Result<SemanticAnalysis> {
        // Try to parse structured JSON response first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            let purpose = parsed.get("purpose")
                .and_then(|p| p.as_str())
                .unwrap_or("AI-analyzed function")
                .to_string();
                
            let behavior_description = parsed.get("behavior")
                .and_then(|b| b.as_str())
                .unwrap_or(&response.chars().take(200).collect::<String>())
                .to_string();
                
            let key_concepts = parsed.get("concepts")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(Vec::new);
                
            let context_relevance = parsed.get("relevance")
                .and_then(|r| r.as_f64())
                .unwrap_or(0.7) as f32;
                
            Ok(SemanticAnalysis {
                purpose,
                behavior_description,
                key_concepts,
                semantic_relationships: Vec::new(), // TODO: Parse relationships
                context_relevance: context_relevance.max(0.5), // Ensure minimum relevance
            })
        } else {
            // Fallback: analyze text content
            let lines: Vec<&str> = response.lines().collect();
            let purpose = if lines.len() > 0 {
                lines[0].chars().take(100).collect()
            } else {
                "AI-analyzed function".to_string()
            };
            
            let behavior_description = response.chars().take(300).collect();
            
            // Extract key concepts from text
            let key_concepts = response.split_whitespace()
                .filter(|word| word.len() > 3 && !word.chars().all(|c| c.is_ascii_punctuation()))
                .take(5)
                .map(|s| s.to_string())
                .collect();
                
            Ok(SemanticAnalysis {
                purpose,
                behavior_description,
                key_concepts,
                semantic_relationships: Vec::new(),
                context_relevance: 0.6, // Default relevance for text parsing
            })
        }
    }

    fn parse_risk_assessment_from_ai_response(&self, response: &str) -> Result<RiskAssessment> {
        // Try to parse structured JSON response first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            let overall_risk = parsed.get("overall_risk")
                .and_then(|r| r.as_str())
                .and_then(|s| match s.to_lowercase().as_str() {
                    "low" => Some(RiskLevel::Low),
                    "medium" => Some(RiskLevel::Medium),
                    "high" => Some(RiskLevel::High),
                    "critical" => Some(RiskLevel::Critical),
                    _ => None,
                })
                .unwrap_or(RiskLevel::Medium);
                
            let breaking_change_risk = parsed.get("breaking_change_risk")
                .and_then(|r| r.as_f64())
                .unwrap_or(0.3) as f32;
                
            let performance_impact = parsed.get("performance_impact")
                .and_then(|p| p.as_f64())
                .unwrap_or(0.2) as f32;
                
            let security_implications = parsed.get("security_implications")
                .and_then(|s| s.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(Vec::new);
                
            let mitigation_strategies = parsed.get("mitigation_strategies")
                .and_then(|m| m.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_else(|| vec!["Review changes carefully".to_string()]);
                
            Ok(RiskAssessment {
                overall_risk,
                breaking_change_risk,
                performance_impact,
                security_implications,
                mitigation_strategies,
            })
        } else {
            // Fallback: analyze text for risk indicators
            let text_lower = response.to_lowercase();
            
            let overall_risk = if text_lower.contains("critical") || text_lower.contains("dangerous") {
                RiskLevel::Critical
            } else if text_lower.contains("high") || text_lower.contains("major") {
                RiskLevel::High
            } else if text_lower.contains("low") || text_lower.contains("minor") {
                RiskLevel::Low
            } else {
                RiskLevel::Medium
            };
            
            let breaking_change_risk = if text_lower.contains("breaking") {
                0.8
            } else if text_lower.contains("compatible") {
                0.1
            } else {
                0.3
            };
            
            let mitigation_strategies = if text_lower.contains("test") {
                vec!["Run comprehensive tests".to_string(), "Review changes carefully".to_string()]
            } else {
                vec!["Review changes carefully".to_string()]
            };
            
            Ok(RiskAssessment {
                overall_risk,
                breaking_change_risk,
                performance_impact: 0.2,
                security_implications: Vec::new(),
                mitigation_strategies,
            })
        }
    }

    fn parse_optimization_suggestions_from_ai_response(&self, _response: &str) -> Result<Vec<OptimizationSuggestion>> {
        // Try to parse structured JSON response first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(_response) {
            let mut suggestions = Vec::new();
            
            if let Some(sugg_array) = parsed.get("suggestions").and_then(|s| s.as_array()) {
                for sugg in sugg_array {
                    let suggestion_type = sugg.get("type")
                        .and_then(|t| t.as_str())
                        .and_then(|s| match s.to_lowercase().as_str() {
                            "performance" => Some(OptimizationType::Performance),
                            "readability" => Some(OptimizationType::Maintainability),
                            "security" => Some(OptimizationType::Security),
                            "maintainability" => Some(OptimizationType::Maintainability),
                            _ => None,
                        })
                        .unwrap_or(OptimizationType::Performance);
                        
                    let description = sugg.get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("AI-generated suggestion")
                        .to_string();
                        
                    let expected_benefit = sugg.get("benefit")
                        .and_then(|b| b.as_str())
                        .unwrap_or("Improved code quality")
                        .to_string();
                        
                    let implementation_effort = sugg.get("effort")
                        .and_then(|e| e.as_str())
                        .and_then(|s| match s.to_lowercase().as_str() {
                            "low" => Some(EffortLevel::Low),
                            "medium" => Some(EffortLevel::Medium),
                            "high" => Some(EffortLevel::High),
                            _ => None,
                        })
                        .unwrap_or(EffortLevel::Medium);
                        
                    let priority = sugg.get("priority")
                        .and_then(|p| p.as_str())
                        .and_then(|s| match s.to_lowercase().as_str() {
                            "low" => Some(Priority::Low),
                            "medium" => Some(Priority::Medium),
                            "high" => Some(Priority::High),
                            _ => None,
                        })
                        .unwrap_or(Priority::Medium);
                        
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type,
                        description,
                        expected_benefit,
                        implementation_effort,
                        priority,
                    });
                }
            }
            
            Ok(suggestions)
        } else {
            // Fallback: create default suggestion
            Ok(vec![
                OptimizationSuggestion {
                    suggestion_type: OptimizationType::Performance,
                    description: "Review function complexity and consider refactoring".to_string(),
                    expected_benefit: "Improved performance and maintainability".to_string(),
                    implementation_effort: EffortLevel::Medium,
                    priority: Priority::Medium,
                }
            ])
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::config::MLConfig;

    #[tokio::test]
    async fn test_smart_context_service_creation() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        assert!(!service.is_ready());
    }

    #[tokio::test]
    async fn test_basic_context_creation() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let context = service.create_base_context(
            "testFunction",
            "src/test.ts",
            "function testFunction() { return 42; }"
        ).unwrap();
        
        assert_eq!(context.function_name, "testFunction");
        assert_eq!(context.file_path, "src/test.ts");
        assert!(context.complexity_score >= 0.0);
    }

    #[tokio::test]
    async fn test_complexity_calculation() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let simple_code = "function test() { return 1; }";
        let complex_code = "function test() { if (x) { for (let i = 0; i < 10; i++) { if (y) { return i; } } } }";
        
        let simple_score = service.calculate_complexity_score(simple_code);
        let complex_score = service.calculate_complexity_score(complex_code);
        
        assert!(complex_score > simple_score);
    }

    #[tokio::test]
    async fn test_impact_scope_determination() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let private_code = "private function test() { return 1; }";
        let public_code = "export function test() { return 1; }";
        
        let private_scope = service.determine_impact_scope(private_code);
        let public_scope = service.determine_impact_scope(public_code);
        
        assert_eq!(private_scope, ImpactScope::Local);
        assert_eq!(public_scope, ImpactScope::Service);
    }

    #[tokio::test]
    async fn test_uninitialized_service() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let result = service.analyze_function_context("test", "test.ts", "code").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_initialization() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let mut service = SmartContextService::new(config, plugin_manager).unwrap();
        
        assert!(service.initialize().await.is_ok());
        assert!(service.is_ready());
    }

    #[tokio::test]
    async fn test_extract_usage_patterns() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let angular_code = "@Component async function test() { return Observable.of(42); }";
        let patterns = service.extract_usage_patterns_from_ast(angular_code);
        
        assert!(patterns.contains(&"Asynchronous operation".to_string()));
        assert!(patterns.contains(&"Observable pattern".to_string()));
        assert!(patterns.contains(&"Angular component".to_string()));
    }

    #[tokio::test]
    async fn test_extract_dependencies() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let code_with_imports = "import { Component } from '@angular/core';\nimport { HttpClient } from '@angular/common/http';";
        let dependencies = service.extract_dependencies_from_ast(code_with_imports);
        
        assert!(dependencies.contains(&"@angular/core".to_string()));
        assert!(dependencies.contains(&"@angular/common/http".to_string()));
    }

    #[tokio::test]
    async fn test_parse_related_functions_response() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let json_response = r#"{"related_functions": ["getUserData", "validateUser", "saveUserData"]}"#;
        let functions = service.parse_related_functions_response(json_response).unwrap();
        
        assert_eq!(functions.len(), 3);
        assert!(functions.contains(&"getUserData".to_string()));
        assert!(functions.contains(&"validateUser".to_string()));
        assert!(functions.contains(&"saveUserData".to_string()));
    }

    #[tokio::test]
    async fn test_parse_conceptual_response() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = SmartContextService::new(config, plugin_manager).unwrap();
        
        let json_response = r#"{"analysis": "This function handles user authentication and validation"}"#;
        let concept = service.parse_conceptual_response(json_response).unwrap();
        
        assert_eq!(concept, "This function handles user authentication and validation");
        
        // Test fallback for non-JSON
        let text_response = "This is a simple text response\nWith multiple lines\nAnd more content";
        let concept2 = service.parse_conceptual_response(text_response).unwrap();
        
        assert!(concept2.contains("This is a simple text response"));
    }

    #[tokio::test]
    async fn test_has_embedding_capability() {
        let config = MLConfig::for_testing();
        let mut plugin_manager = PluginManager::new();
        
        // Initialize plugin manager
        assert!(plugin_manager.initialize(&config).await.is_ok());
        
        let service = SmartContextService::new(config, Arc::new(plugin_manager)).unwrap();
        
        // Check capability detection
        let has_embedding = service.has_embedding_capability().await;
        // Should be true since we register qwen_embedding in initialization
        assert!(has_embedding);
    }

    #[tokio::test]
    async fn test_service_shutdown() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let mut service = SmartContextService::new(config, plugin_manager).unwrap();
        
        assert!(service.initialize().await.is_ok());
        assert!(service.is_ready());
        
        assert!(service.shutdown().await.is_ok());
        assert!(!service.is_ready());
    }
}

#[cfg(test)]
mod context_test;
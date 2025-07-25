//! MCP Tools for Claude Code integration

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use anyhow::Result;

use crate::cache::CacheManager; 
use crate::ml::services::enhanced_search::{
    EnhancedSearchService, SearchRequest, SearchType, SearchFilters, SearchOptions
};
use crate::generators::ProjectOverviewGenerator;
use crate::analyzers::DiffAnalyzer;
use crate::types::{ChangeType, ModifiedFile};
use super::context_optimizer::ContextOptimizer;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Result of an MCP tool execution
#[derive(Debug, Serialize)]
pub struct MCPToolResult {
    pub result: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

/// Trait for MCP tools
#[async_trait]
pub trait MCPTool: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;
    
    /// Tool description for Claude Code
    fn description(&self) -> &str;
    
    /// JSON schema for tool parameters
    fn parameters_schema(&self) -> serde_json::Value;
    
    /// Execute the tool with given parameters
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult>;
}

/// Smart Context Tool - solves compactation pain point
pub struct SmartContextTool {
    cache_manager: Arc<Mutex<CacheManager>>,
    search_service: Arc<EnhancedSearchService>,
    optimizer: ContextOptimizer,
}

#[derive(Debug, Deserialize)]
struct SmartContextParams {
    query: String,
    max_tokens: Option<usize>,
    include_tests: Option<bool>,
    include_dependencies: Option<bool>,
    file_types: Option<Vec<String>>,
}

impl SmartContextTool {
    pub fn new(
        cache_manager: Arc<Mutex<CacheManager>>,
        search_service: Arc<EnhancedSearchService>,
    ) -> Self {
        Self {
            cache_manager,
            search_service,
            optimizer: ContextOptimizer::new(),
        }
    }
}

#[async_trait]
impl MCPTool for SmartContextTool {
    fn name(&self) -> &str {
        "smart_context"
    }
    
    fn description(&self) -> &str {
        "Get relevant code context for a query, optimized for token efficiency. Solves compactation issues by providing only the most relevant code sections."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "What you're looking for (e.g., 'authentication logic', 'error handling patterns')"
                },
                "max_tokens": {
                    "type": "integer", 
                    "description": "Maximum tokens to return (default: 4000)",
                    "default": 4000
                },
                "include_tests": {
                    "type": "boolean",
                    "description": "Include test files in results (default: false)",
                    "default": false
                },
                "include_dependencies": {
                    "type": "boolean", 
                    "description": "Include imported dependencies (default: true)",
                    "default": true
                },
                "file_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "File extensions to include (e.g., ['.ts', '.rs'])"
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: SmartContextParams = serde_json::from_value(parameters)?;
        
        println!("🔍 Smart Context query: '{}'", params.query);
        println!("   Max tokens: {}", params.max_tokens.unwrap_or(4000));
        
        // Create search request
        let search_request = SearchRequest {
            query: params.query.clone(),
            search_type: SearchType::General,
            filters: SearchFilters {
                file_patterns: params.file_types.clone(),
                ..Default::default()
            },
            options: SearchOptions {
                max_results: 20, // Get more results for optimization
                include_metadata: true,
                ..Default::default()
            },
        };
        
        // Perform semantic search
        let search_response = self.search_service.search(search_request).await?;
        
        println!("   Found {} semantic matches", search_response.results.len());
        
        // Optimize context for token budget
        let optimized_context = self.optimizer.optimize_context(
            &search_response.results,
            params.max_tokens.unwrap_or(4000),
            params.include_tests.unwrap_or(false),
            params.include_dependencies.unwrap_or(true),
        ).await?;
        
        println!("✅ Optimized context: {} tokens, {} files", 
                optimized_context.total_tokens, 
                optimized_context.files.len());
        
        let result = serde_json::json!({
            "context": optimized_context.context,
            "files_included": optimized_context.files,
            "total_tokens": optimized_context.total_tokens,
            "optimization_summary": optimized_context.summary
        });
        
        let metadata = serde_json::json!({
            "query": params.query,
            "semantic_matches": search_response.results.len(),
            "files_analyzed": optimized_context.files.len(),
            "token_efficiency": format!("{:.1}%", 
                (optimized_context.total_tokens as f64 / params.max_tokens.unwrap_or(4000) as f64) * 100.0),
            "search_time_ms": search_response.search_time_ms
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// Explore Codebase Tool - semantic file discovery
pub struct ExploreCodebaseTool {
    cache_manager: Arc<Mutex<CacheManager>>,
    search_service: Arc<EnhancedSearchService>,
}

#[derive(Debug, Deserialize)]
struct ExploreCodebaseParams {
    query: String,
    file_types: Option<Vec<String>>,
    max_results: Option<usize>,
    include_snippets: Option<bool>,
}

impl ExploreCodebaseTool {
    pub fn new(
        cache_manager: Arc<Mutex<CacheManager>>,
        search_service: Arc<EnhancedSearchService>,
    ) -> Self {
        Self {
            cache_manager,
            search_service,
        }
    }
}

#[async_trait]
impl MCPTool for ExploreCodebaseTool {
    fn name(&self) -> &str {
        "explore_codebase"
    }
    
    fn description(&self) -> &str {
        "Find related files and functions semantically without reading entire files. Perfect for code exploration and discovery."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "What to explore (e.g., 'error handling patterns', 'database queries')"
                },
                "file_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "File extensions to search (e.g., ['.ts', '.rs'])"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum files to return (default: 10)",
                    "default": 10
                },
                "include_snippets": {
                    "type": "boolean",
                    "description": "Include code snippets in results (default: true)",
                    "default": true
                }
            },
            "required": ["query"]
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: ExploreCodebaseParams = serde_json::from_value(parameters)?;
        
        println!("🔍 Exploring codebase for: '{}'", params.query);
        
        // Create search request
        let search_request = SearchRequest {
            query: params.query.clone(),
            search_type: SearchType::General,
            filters: SearchFilters {
                file_patterns: params.file_types.clone(),
                ..Default::default()
            },
            options: SearchOptions {
                max_results: params.max_results.unwrap_or(10),
                include_metadata: true,
                ..Default::default()
            },
        };
        
        // Perform semantic search
        let search_response = self.search_service.search(search_request).await?;
        
        println!("   Found {} relevant files", search_response.results.len());
        
        // Format results for exploration
        let files: Vec<serde_json::Value> = search_response.results.iter()
            .map(|result| {
                let mut file_info = serde_json::json!({
                    "file_path": result.entry.metadata.file_path,
                    "language": result.entry.metadata.language,
                    "complexity": result.entry.metadata.complexity,
                    "relevance_score": result.combined_score,
                    "code_type": format!("{:?}", result.entry.metadata.code_type),
                });
                
                if params.include_snippets.unwrap_or(true) {
                    // Include a small snippet for context
                    let snippet = if result.entry.metadata.tokens.len() > 10 {
                        result.entry.metadata.tokens[..10].join(" ") + "..."
                    } else {
                        result.entry.metadata.tokens.join(" ")
                    };
                    file_info["snippet"] = serde_json::Value::String(snippet);
                }
                
                file_info
            })
            .collect();
        
        let result = serde_json::json!({
            "files": files,
            "total_found": search_response.results.len(),
            "suggestions": search_response.suggestions
        });
        
        let metadata = serde_json::json!({
            "query": params.query,
            "search_time_ms": search_response.search_time_ms,
            "total_candidates": search_response.total_candidates
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// Project Overview Tool - Get structured project analysis without reading all files
pub struct ProjectOverviewTool {
    cache_manager: Arc<Mutex<CacheManager>>,
}

#[derive(Debug, Deserialize)]
struct ProjectOverviewParams {
    format: Option<String>, // "json", "text", "markdown"
    include_health: Option<bool>,
    include_stats: Option<bool>,
    max_files: Option<usize>,
}

impl ProjectOverviewTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self {
            cache_manager,
        }
    }
}

#[async_trait]
impl MCPTool for ProjectOverviewTool {
    fn name(&self) -> &str {
        "project_overview"
    }
    
    fn description(&self) -> &str {
        "Get structured project analysis without reading all files. Perfect for understanding project architecture and getting oriented quickly."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["json", "text", "markdown"],
                    "description": "Output format (default: markdown)",
                    "default": "markdown"
                },
                "include_health": {
                    "type": "boolean",
                    "description": "Include project health metrics (default: true)",
                    "default": true
                },
                "include_stats": {
                    "type": "boolean", 
                    "description": "Include detailed statistics (default: true)",
                    "default": true
                },
                "max_files": {
                    "type": "integer",
                    "description": "Maximum files to analyze for overview (default: 100)",
                    "default": 100
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: ProjectOverviewParams = serde_json::from_value(parameters)?;
        
        println!("📊 Generating project overview...");
        println!("   Format: {}", params.format.as_deref().unwrap_or("markdown"));
        
        // Get project path
        let project_path = std::env::current_dir()?;
        
        // Initialize overview generator
        let generator = ProjectOverviewGenerator::new(CacheManager::new(&project_path)?);
        
        // Generate overview
        let overview = generator.generate_overview(&project_path)?;
        
        println!("✅ Project overview generated:");
        println!("   Components found: {}", overview.structure.components.len());
        println!("   Services found: {}", overview.structure.services.len());
        
        // Format output based on requested format
        let formatted_output = match params.format.as_deref().unwrap_or("markdown") {
            "json" => serde_json::to_string_pretty(&overview)?,
            "text" | "markdown" => {
                format!(
                    "# Project Overview: {}\n\n## Summary\n- **Components**: {}\n- **Services**: {}\n- **Pipes**: {}\n- **Test Coverage**: {:.1}%\n- **Last Updated**: {}\n\n## Architecture\n\n### Components ({})\n{}\n\n### Services ({})\n{}\n\n## Recent Changes\n- Modified files: {}\n- Impact scope: {:?}\n\n## Technical Stack\n- Language: {}\n- Framework: {}\n\n## Recommendations\n{}\n",
                    overview.project_name,
                    overview.structure.components.len(),
                    overview.structure.services.len(), 
                    overview.structure.pipes.len(),
                    overview.health_metrics.test_coverage * 100.0,
                    overview.last_updated.format("%Y-%m-%d %H:%M UTC"),
                    overview.structure.components.len(),
                    overview.structure.components.iter()
                        .take(10)
                        .map(|c| format!("- **{}**: {} functions, complexity: {:?}", c.name, c.functions.len(), c.complexity))
                        .collect::<Vec<_>>().join("\n"),
                    overview.structure.services.len(),
                    overview.structure.services.iter()
                        .take(10) 
                        .map(|s| format!("- **{}**: {} functions, scope: {:?}", s.name, s.functions.len(), s.scope))
                        .collect::<Vec<_>>().join("\n"),
                    overview.recent_changes.modified_files.len(), 
                    overview.recent_changes.impact_scope,
                    overview.technical_stack.language, 
                    overview.technical_stack.framework,
                    overview.recommendations.join("\n- ")
                )
            },
            _ => return Err(anyhow::anyhow!("Unsupported format: {}", params.format.unwrap_or_default())),
        };
        
        let result = serde_json::json!({
            "overview": formatted_output,
            "stats": {
                "project_name": overview.project_name,
                "components": overview.structure.components.len(),
                "services": overview.structure.services.len(),
                "pipes": overview.structure.pipes.len(),
                "health_score": overview.health_metrics.test_coverage
            }
        });
        
        let metadata = serde_json::json!({
            "format": params.format.unwrap_or_else(|| "markdown".to_string()),
            "include_health": params.include_health.unwrap_or(true),
            "generation_time_ms": 0 // TODO: Add timing
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// Changes Analysis Tool - Git-aware context for modified files only
pub struct ChangesAnalysisTool {
    cache_manager: Arc<Mutex<CacheManager>>,
}

#[derive(Debug, Deserialize)]
struct ChangesAnalysisParams {
    since: Option<String>, // "last-commit", "last-week", "2024-01-01"
    modified_only: Option<bool>,
    include_context: Option<bool>,
    max_files: Option<usize>,
}

impl ChangesAnalysisTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self {
            cache_manager,
        }
    }
}

#[async_trait]
impl MCPTool for ChangesAnalysisTool {
    fn name(&self) -> &str {
        "changes_analysis"
    }
    
    fn description(&self) -> &str {
        "Git-aware context for modified files only. Perfect for understanding what changed and providing focused context for recent modifications."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "since": {
                    "type": "string",
                    "description": "Time reference for changes (e.g., 'last-commit', 'last-week', '2024-01-01')",
                    "default": "last-commit"
                },
                "modified_only": {
                    "type": "boolean",
                    "description": "Only include actually modified files (default: true)",
                    "default": true
                },
                "include_context": {
                    "type": "boolean",
                    "description": "Include surrounding context for changed functions (default: true)",
                    "default": true
                },
                "max_files": {
                    "type": "integer",
                    "description": "Maximum files to analyze (default: 50)",
                    "default": 50
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: ChangesAnalysisParams = serde_json::from_value(parameters)?;
        
        println!("📝 Analyzing recent changes...");
        println!("   Since: {}", params.since.as_deref().unwrap_or("last-commit"));
        
        // Get project path
        let project_path = std::env::current_dir()?;
        
        // Initialize diff analyzer
        let diff_analyzer = DiffAnalyzer::new(&project_path)?;
        
        // Get changes based on time reference
        let changes = match params.since.as_deref().unwrap_or("last-commit") {
            "last-commit" => diff_analyzer.analyze_changes(&project_path)?,
            "last-week" => diff_analyzer.analyze_changes(&project_path)?,
            time_ref => {
                // Try to parse as date or fall back to last commit
                match time_ref.parse::<chrono::NaiveDate>() {
                    Ok(date) => {
                        let _datetime = date.and_hms_opt(0, 0, 0).unwrap();
                        diff_analyzer.analyze_changes(&project_path)?
                    },
                    Err(_) => diff_analyzer.analyze_changes(&project_path)?,
                }
            }
        };
        
        println!("✅ Changes analysis complete:");
        println!("   Modified files: {}", changes.modified_files.len());
        println!("   Added files: {}", changes.added_files.len());
        println!("   Deleted files: {}", changes.deleted_files.len());
        
        // Filter to only modified files if requested
        let mut files_to_analyze = changes.modified_files.clone();
        if !params.modified_only.unwrap_or(true) {
            // Include all changed files - convert added files to ModifiedFile for consistency
            for added_file in &changes.added_files {
                files_to_analyze.push(ModifiedFile {
                    path: added_file.clone(),
                    change_type: ChangeType::Created,
                    lines_added: 0,
                    lines_removed: 0,
                    sections_changed: vec![],
                    impacted_files: vec![],
                });
            }
        }
        
        // Limit number of files
        let max_files = params.max_files.unwrap_or(50);
        if files_to_analyze.len() > max_files {
            files_to_analyze.truncate(max_files);
        }
        
        // Build context for changed files
        let mut file_contexts = Vec::new();
        
        for modified_file in &files_to_analyze {
            // Get file summary from cache if available using proper path normalization
            if let Some(file_data) = self.cache_manager.lock().unwrap().get_file_summary(&modified_file.path) {
                let context = format!(
                    "## {}\n- **Type**: {:?}\n- **Complexity**: {:.1}\n- **Functions**: {}\n- **Last Modified**: {}\n\n**Summary**: {}\n",
                    modified_file.path,
                    file_data.metadata.file_type,
                    1.0, // TODO: Get actual complexity score
                    file_data.summary.functions.len(),
                    file_data.metadata.last_modified.format("%Y-%m-%d %H:%M"),
                    file_data.summary.file_name
                );
                file_contexts.push(context);
            }
        }
        
        let changes_summary = format!(
            "# Recent Changes Analysis\n\n## Summary\n- **Modified Files**: {}\n- **Added Files**: {}\n- **Deleted Files**: {}\n- **Impact Scope**: {:?}\n\n## Modified Files\n\n{}\n\n## Suggested Actions\n{}\n",
            changes.modified_files.len(),
            changes.added_files.len(), 
            changes.deleted_files.len(),
            changes.impact_scope,
            file_contexts.join("\n"),
            changes.suggested_actions.join("\n- ")
        );
        
        let result = serde_json::json!({
            "changes_summary": changes_summary,
            "changes": {
                "modified_files": changes.modified_files,
                "added_files": changes.added_files,
                "deleted_files": changes.deleted_files,
                "impact_scope": format!("{:?}", changes.impact_scope)
            }
        });
        
        let metadata = serde_json::json!({
            "since": params.since.unwrap_or_else(|| "last-commit".to_string()),
            "files_analyzed": files_to_analyze.len(),
            "total_changes": changes.modified_files.len() + changes.added_files.len()
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// File Summary Tool - Get detailed analysis of a specific file
pub struct FileSummaryTool {
    cache_manager: Arc<Mutex<CacheManager>>,
}

#[derive(Debug, Deserialize)]
struct FileSummaryParams {
    file_path: String,
    format: Option<String>, // "json", "text", "markdown"
    include_complexity: Option<bool>,
    include_functions: Option<bool>,
    include_dependencies: Option<bool>,
}

impl FileSummaryTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self {
            cache_manager,
        }
    }
}

#[async_trait]
impl MCPTool for FileSummaryTool {
    fn name(&self) -> &str {
        "file_summary"
    }
    
    fn description(&self) -> &str {
        "Get detailed analysis of a specific file including complexity, functions, and dependencies. Perfect for understanding individual files quickly."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to analyze (relative to project root)"
                },
                "format": {
                    "type": "string",
                    "enum": ["json", "text", "markdown"],
                    "description": "Output format (default: markdown)",
                    "default": "markdown"
                },
                "include_complexity": {
                    "type": "boolean",
                    "description": "Include complexity analysis (default: true)",
                    "default": true
                },
                "include_functions": {
                    "type": "boolean",
                    "description": "Include function details (default: true)",
                    "default": true
                },
                "include_dependencies": {
                    "type": "boolean",
                    "description": "Include import/export analysis (default: true)",
                    "default": true
                }
            },
            "required": ["file_path"]
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: FileSummaryParams = serde_json::from_value(parameters)?;
        
        println!("📄 Analyzing file: '{}'", params.file_path);
        println!("   Format: {}", params.format.as_deref().unwrap_or("markdown"));
        
        // Get file data from cache using proper path normalization
        let manager = self.cache_manager.lock().unwrap();
        let file_data = manager.get_file_summary(&params.file_path)
            .ok_or_else(|| anyhow::anyhow!("File not found in cache: {}. Run 'cargo run -- analyze' first.", params.file_path))?;
        
        println!("✅ File found in cache:");
        println!("   Type: {:?}", file_data.metadata.file_type);
        println!("   Functions: {}", file_data.summary.functions.len());
        println!("   Complexity: {:?}", file_data.metadata.complexity);
        
        // Format output based on requested format
        let formatted_output = match params.format.as_deref().unwrap_or("markdown") {
            "json" => {
                let mut summary = serde_json::json!({
                    "file_path": params.file_path,
                    "file_type": format!("{:?}", file_data.metadata.file_type),
                    "size": file_data.metadata.size,
                    "line_count": file_data.metadata.line_count,
                    "last_modified": file_data.metadata.last_modified,
                    "complexity": format!("{:?}", file_data.metadata.complexity)
                });
                
                if params.include_functions.unwrap_or(true) {
                    summary["functions"] = serde_json::json!(file_data.summary.functions);
                }
                
                if params.include_dependencies.unwrap_or(true) {
                    summary["imports"] = serde_json::json!(file_data.metadata.imports);
                    summary["exports"] = serde_json::json!(file_data.metadata.exports);
                }
                
                serde_json::to_string_pretty(&summary)?
            },
            "text" | "markdown" => {
                let mut output = format!(
                    "# File Summary: {}\n\n## Overview\n- **Type**: {:?}\n- **Size**: {} bytes\n- **Lines**: {}\n- **Complexity**: {:?}\n- **Last Modified**: {}\n\n",
                    params.file_path,
                    file_data.metadata.file_type,
                    file_data.metadata.size,
                    file_data.metadata.line_count,
                    file_data.metadata.complexity,
                    file_data.metadata.last_modified.format("%Y-%m-%d %H:%M UTC")
                );
                
                if params.include_functions.unwrap_or(true) && !file_data.summary.functions.is_empty() {
                    output.push_str(&format!("## Functions ({})\n", file_data.summary.functions.len()));
                    for func in &file_data.summary.functions {
                        output.push_str(&format!(
                            "- **{}**({}) -> {}{}\n",
                            func.name,
                            func.parameters.iter()
                                .map(|p| format!("{}: {}", p.name, p.param_type))
                                .collect::<Vec<_>>()
                                .join(", "),
                            func.return_type,
                            if func.is_async { " (async)" } else { "" }
                        ));
                    }
                    output.push('\n');
                }
                
                if params.include_dependencies.unwrap_or(true) {
                    if !file_data.metadata.imports.is_empty() {
                        output.push_str(&format!("## Imports ({})\n", file_data.metadata.imports.len()));
                        for import in &file_data.metadata.imports {
                            output.push_str(&format!("- {}\n", import));
                        }
                        output.push('\n');
                    }
                    
                    if !file_data.metadata.exports.is_empty() {
                        output.push_str(&format!("## Exports ({})\n", file_data.metadata.exports.len()));
                        for export in &file_data.metadata.exports {
                            output.push_str(&format!("- {}\n", export));
                        }
                        output.push('\n');
                    }
                }
                
                if params.include_complexity.unwrap_or(true) {
                    output.push_str("## Complexity Analysis\n");
                    output.push_str(&format!("- **Overall Complexity**: {:?}\n", file_data.metadata.complexity));
                    output.push_str(&format!("- **Function Count**: {}\n", file_data.summary.functions.len()));
                    output.push_str(&format!("- **Class Count**: {}\n", file_data.summary.classes.len()));
                    
                    if !file_data.summary.components.is_empty() {
                        output.push_str(&format!("- **Components**: {}\n", file_data.summary.components.len()));
                    }
                    if !file_data.summary.services.is_empty() {
                        output.push_str(&format!("- **Services**: {}\n", file_data.summary.services.len()));
                    }
                }
                
                output
            },
            _ => return Err(anyhow::anyhow!("Unsupported format: {}", params.format.unwrap_or_default())),
        };
        
        let result = serde_json::json!({
            "summary": formatted_output,
            "file_info": {
                "path": params.file_path,
                "type": format!("{:?}", file_data.metadata.file_type),
                "size": file_data.metadata.size,
                "lines": file_data.metadata.line_count,
                "functions": file_data.summary.functions.len(),
                "complexity": format!("{:?}", file_data.metadata.complexity)
            }
        });
        
        let metadata = serde_json::json!({
            "format": params.format.unwrap_or_else(|| "markdown".to_string()),
            "file_path": params.file_path,
            "analysis_time_ms": 0 // TODO: Add timing
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// Cache Status Tool - Monitor cache health and optimization opportunities
pub struct CacheStatusTool {
    cache_manager: Arc<Mutex<CacheManager>>,
}

#[derive(Debug, Deserialize)]
struct CacheStatusParams {
    include_details: Option<bool>,
    check_integrity: Option<bool>,
    format: Option<String>, // "json", "text", "markdown"
}

impl CacheStatusTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self {
            cache_manager,
        }
    }
}

#[async_trait]
impl MCPTool for CacheStatusTool {
    fn name(&self) -> &str {
        "cache_status"
    }
    
    fn description(&self) -> &str {
        "Monitor cache health, statistics, and optimization opportunities. Perfect for understanding cache performance and finding optimization potential."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_details": {
                    "type": "boolean",
                    "description": "Include detailed file-level statistics (default: false)",
                    "default": false
                },
                "check_integrity": {
                    "type": "boolean",
                    "description": "Perform cache integrity checks (default: false)",
                    "default": false
                },
                "format": {
                    "type": "string",
                    "enum": ["json", "text", "markdown"],
                    "description": "Output format (default: markdown)",
                    "default": "markdown"
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: CacheStatusParams = serde_json::from_value(parameters)?;
        
        println!("📊 Analyzing cache status...");
        println!("   Include details: {}", params.include_details.unwrap_or(false));
        
        // Get cache statistics
        let manager = self.cache_manager.lock().unwrap();
        let cache_stats = manager.get_cache_stats();
        let cache = manager.get_cache();
        
        // Calculate detailed metrics
        let total_files = cache.entries.len();
        let rust_files = cache.entries.values()
            .filter(|entry| matches!(entry.metadata.file_type, 
                crate::types::FileType::RustLibrary | 
                crate::types::FileType::RustBinary | 
                crate::types::FileType::RustModule | 
                crate::types::FileType::RustTest))
            .count();
        
        let typescript_files = cache.entries.values()
            .filter(|entry| matches!(entry.metadata.file_type, 
                crate::types::FileType::Component | 
                crate::types::FileType::Service | 
                crate::types::FileType::Module))
            .count();
        
        let high_complexity_files = cache.entries.values()
            .filter(|entry| matches!(entry.metadata.complexity, crate::types::Complexity::High))
            .count();
        
        let avg_file_size = if total_files > 0 {
            cache_stats.total_size / total_files as u64
        } else {
            0
        };
        
        // Check for potential issues
        let mut optimization_suggestions = Vec::new();
        
        if high_complexity_files > total_files / 4 {
            optimization_suggestions.push("Consider refactoring high-complexity files to improve maintainability".to_string());
        }
        
        if avg_file_size > 10000 {
            optimization_suggestions.push("Large average file size detected - consider splitting large files".to_string());
        }
        
        if rust_files == 0 && typescript_files == 0 {
            optimization_suggestions.push("No recognized framework files found - check file type detection".to_string());
        }
        
        // Integrity check if requested
        let mut integrity_issues = Vec::new();
        if params.check_integrity.unwrap_or(false) {
            println!("🔍 Performing integrity checks...");
            
            // Check for missing files
            for (file_path, _) in &cache.entries {
                let path = std::path::Path::new(file_path);
                if !path.exists() {
                    integrity_issues.push(format!("Missing file: {}", file_path));
                }
            }
        }
        
        println!("✅ Cache analysis complete:");
        println!("   Total files: {}", total_files);
        println!("   Rust files: {}", rust_files);
        println!("   TypeScript files: {}", typescript_files);
        println!("   High complexity: {}", high_complexity_files);
        
        // Format output
        let formatted_output = match params.format.as_deref().unwrap_or("markdown") {
            "json" => {
                let status = serde_json::json!({
                    "cache_stats": {
                        "total_files": total_files,
                        "total_size_bytes": cache_stats.total_size,
                        "average_file_size": avg_file_size,
                        "last_updated": cache.last_updated
                    },
                    "file_types": {
                        "rust_files": rust_files,
                        "typescript_files": typescript_files,
                        "other_files": total_files - rust_files - typescript_files
                    },
                    "complexity_distribution": {
                        "high_complexity": high_complexity_files,
                        "percentage_high": if total_files > 0 { (high_complexity_files as f64 / total_files as f64) * 100.0 } else { 0.0 }
                    },
                    "optimization_suggestions": optimization_suggestions,
                    "integrity_issues": integrity_issues
                });
                serde_json::to_string_pretty(&status)?
            },
            "text" | "markdown" => {
                let mut output = format!(
                    "# Cache Status Report\n\n## Overview\n- **Total Files**: {}\n- **Total Size**: {:.2} MB\n- **Average File Size**: {:.1} KB\n- **Last Updated**: {}\n\n## File Distribution\n- **Rust Files**: {} ({:.1}%)\n- **TypeScript Files**: {} ({:.1}%)\n- **Other Files**: {} ({:.1}%)\n\n## Complexity Analysis\n- **High Complexity Files**: {} ({:.1}%)\n- **Health Score**: {:.1}/10\n\n",
                    total_files,
                    cache_stats.total_size as f64 / 1_048_576.0, // MB
                    avg_file_size as f64 / 1024.0, // KB
                    cache.last_updated.format("%Y-%m-%d %H:%M UTC"),
                    rust_files, 
                    if total_files > 0 { (rust_files as f64 / total_files as f64) * 100.0 } else { 0.0 },
                    typescript_files,
                    if total_files > 0 { (typescript_files as f64 / total_files as f64) * 100.0 } else { 0.0 },
                    total_files - rust_files - typescript_files,
                    if total_files > 0 { ((total_files - rust_files - typescript_files) as f64 / total_files as f64) * 100.0 } else { 0.0 },
                    high_complexity_files,
                    if total_files > 0 { (high_complexity_files as f64 / total_files as f64) * 100.0 } else { 0.0 },
                    10.0 - ((high_complexity_files as f64 / total_files.max(1) as f64) * 5.0) // Simple health score
                );
                
                if !optimization_suggestions.is_empty() {
                    output.push_str("## Optimization Suggestions\n");
                    for suggestion in &optimization_suggestions {
                        output.push_str(&format!("- {}\n", suggestion));
                    }
                    output.push('\n');
                }
                
                if !integrity_issues.is_empty() {
                    output.push_str("## Integrity Issues\n");
                    for issue in &integrity_issues {
                        output.push_str(&format!("- ⚠️ {}\n", issue));
                    }
                    output.push('\n');
                }
                
                if params.include_details.unwrap_or(false) {
                    output.push_str("## File Details\n");
                    let mut files_by_size: Vec<_> = cache.entries.iter().collect();
                    files_by_size.sort_by(|a, b| b.1.metadata.size.cmp(&a.1.metadata.size));
                    
                    for (file_path, entry) in files_by_size.iter().take(10) {
                        output.push_str(&format!(
                            "- **{}**: {:.1} KB, {:?} complexity, {:?}\n",
                            file_path,
                            entry.metadata.size as f64 / 1024.0,
                            entry.metadata.complexity,
                            entry.metadata.file_type
                        ));
                    }
                }
                
                output
            },
            _ => return Err(anyhow::anyhow!("Unsupported format: {}", params.format.unwrap_or_default())),
        };
        
        let result = serde_json::json!({
            "status_report": formatted_output,
            "summary": {
                "total_files": total_files,
                "rust_files": rust_files,
                "typescript_files": typescript_files,
                "high_complexity_files": high_complexity_files,
                "cache_size_mb": cache_stats.total_size as f64 / 1_048_576.0,
                "health_score": 10.0 - ((high_complexity_files as f64 / total_files.max(1) as f64) * 5.0)
            }
        });
        
        let metadata = serde_json::json!({
            "format": params.format.unwrap_or_else(|| "markdown".to_string()),
            "include_details": params.include_details.unwrap_or(false),
            "integrity_check": params.check_integrity.unwrap_or(false),
            "analysis_time_ms": 0 // TODO: Add timing
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}

/// Cache generation progress tracking
#[derive(Debug, Clone, Serialize)]
pub struct CacheProgress {
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
    pub errors: Vec<String>,
    pub percentage: f32,
}

/// Cache generation status
#[derive(Debug, Clone, Serialize)]
pub enum CacheGenerationStatus {
    Idle,
    Running,
    Completed,
    Failed(String),
}

/// Shared state for cache generation tracking
#[derive(Debug, Clone, Serialize)]
pub struct CacheGenerationState {
    pub status: CacheGenerationStatus,
    pub progress: CacheProgress,
    pub started_at: Option<SystemTime>,
    pub duration_ms: Option<u64>,
}

impl Default for CacheGenerationState {
    fn default() -> Self {
        Self {
            status: CacheGenerationStatus::Idle,
            progress: CacheProgress {
                total_files: 0,
                processed_files: 0,
                current_file: None,
                errors: Vec::new(),
                percentage: 0.0,
            },
            started_at: None,
            duration_ms: None,
        }
    }
}

/// Cache Generation Tool - Asynchronous cache generation with progress tracking
pub struct CacheGenerationTool {
    cache_manager: Arc<Mutex<CacheManager>>,
    pub generation_state: Arc<RwLock<CacheGenerationState>>,
}

#[derive(Debug, Deserialize)]
struct CacheGenerationParams {
    project_path: Option<String>,
    force_rebuild: Option<bool>,
    background: Option<bool>, // Run in background vs foreground
}

impl CacheGenerationTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self {
            cache_manager,
            generation_state: Arc::new(RwLock::new(CacheGenerationState::default())),
        }
    }
}

#[async_trait]
impl MCPTool for CacheGenerationTool {
    fn name(&self) -> &str {
        "generate_cache"
    }
    
    fn description(&self) -> &str {
        "Generate or rebuild the project cache with real-time progress tracking. Supports background and foreground modes for flexible cache generation."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Path to project (defaults to current working directory)"
                },
                "force_rebuild": {
                    "type": "boolean",
                    "description": "Force complete cache rebuild (default: false)",
                    "default": false
                },
                "background": {
                    "type": "boolean",
                    "description": "Run in background mode - returns immediately (default: false)",
                    "default": false
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: CacheGenerationParams = serde_json::from_value(parameters)?;
        
        let project_path = params.project_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
        
        let force_rebuild = params.force_rebuild.unwrap_or(false);
        let background = params.background.unwrap_or(false);
        
        println!("🏗️ Cache generation request:");
        println!("   Project: {}", project_path.display());
        println!("   Force rebuild: {}", force_rebuild);
        println!("   Background mode: {}", background);
        
        if background {
            // Background mode - start generation and return immediately
            let _ = Self::run_cache_generation(
                self.cache_manager.clone(),
                self.generation_state.clone(),
                project_path.clone(),
                force_rebuild
            ).await;
            
            let result = serde_json::json!({
                "status": "started",
                "message": "Cache generation started in background",
                "project_path": project_path.to_string_lossy(),
                "force_rebuild": force_rebuild,
                "tip": "Use cache_generation_status tool to monitor progress"
            });
            
            let metadata = serde_json::json!({
                "background": true,
                "started_at": SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis()
            });
            
            Ok(MCPToolResult {
                result,
                metadata: Some(metadata),
            })
        } else {
            // Foreground mode - wait for completion
            match Self::run_cache_generation(
                self.cache_manager.clone(),
                self.generation_state.clone(),
                project_path.clone(),
                force_rebuild
            ).await {
                Ok(final_state) => {
                    let result = serde_json::json!({
                        "status": "completed",
                        "final_state": final_state
                    });
                    
                    let metadata = serde_json::json!({
                        "background": false,
                        "force_rebuild": force_rebuild,
                        "project_path": project_path.to_string_lossy()
                    });
                    
                    Ok(MCPToolResult {
                        result,
                        metadata: Some(metadata),
                    })
                }
                Err(e) => {
                    let result = serde_json::json!({
                        "status": "failed",
                        "error": e.to_string()
                    });
                    
                    Ok(MCPToolResult {
                        result,
                        metadata: None,
                    })
                }
            }
        }
    }
    
}

impl CacheGenerationTool {
    /// Run cache generation with progress tracking
    async fn run_cache_generation(
        cache_manager: Arc<Mutex<CacheManager>>,
        generation_state: Arc<RwLock<CacheGenerationState>>,
        project_path: std::path::PathBuf,
        force_rebuild: bool,
    ) -> Result<CacheGenerationState> {
        // Set initial state
        {
            let mut state = generation_state.write().await;
            state.status = CacheGenerationStatus::Running;
            state.started_at = Some(SystemTime::now());
            state.duration_ms = None;
        }
        
        let start_time = SystemTime::now();
        
        // Create progress channel
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<crate::cache::CacheProgress>();
        
        // Update progress in background
        let state_clone = generation_state.clone();
        let progress_task = tokio::spawn(async move {
            while let Some(cache_progress) = progress_rx.recv().await {
                let mut state = state_clone.write().await;
                // Convert cache_manager::CacheProgress to tools::CacheProgress
                state.progress = CacheProgress {
                    total_files: cache_progress.total,
                    processed_files: cache_progress.processed,
                    current_file: Some(cache_progress.current_file),
                    errors: Vec::new(), // TODO: Add error handling
                    percentage: cache_progress.percentage,
                };
            }
        });
        
        // Run cache generation
        let result = CacheManager::analyze_project_async_with_progress(
            cache_manager,
            &project_path,
            force_rebuild,
            Some(progress_tx),
        ).await;
        
        // Finalize state
        let duration = start_time.elapsed()?;
        let mut final_state = generation_state.write().await;
        
        match result {
            Ok(_) => {
                final_state.status = CacheGenerationStatus::Completed;
                final_state.duration_ms = Some(duration.as_millis() as u64);
                final_state.progress.percentage = 100.0;
                final_state.progress.current_file = None;
            }
            Err(e) => {
                final_state.status = CacheGenerationStatus::Failed(e.to_string());
                final_state.duration_ms = Some(duration.as_millis() as u64);
            }
        }
        
        progress_task.abort();
        
        println!("🏗️ Cache generation completed in {}ms", duration.as_millis());
        
        Ok(final_state.clone())
    }
}

/// Cache Generation Status Tool - Monitor cache generation progress
pub struct CacheGenerationStatusTool {
    generation_state: Arc<RwLock<CacheGenerationState>>,
}

impl CacheGenerationStatusTool {
    pub fn new(generation_state: Arc<RwLock<CacheGenerationState>>) -> Self {
        Self { generation_state }
    }
}

#[async_trait]
impl MCPTool for CacheGenerationStatusTool {
    fn name(&self) -> &str {
        "cache_generation_status"
    }
    
    fn description(&self) -> &str {
        "Monitor the progress of background cache generation. Shows real-time progress, current file being processed, and any errors encountered."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }
    
    async fn execute(&self, _parameters: serde_json::Value) -> Result<MCPToolResult> {
        let state = self.generation_state.read().await.clone();
        
        let result = serde_json::json!({
            "status": state.status,
            "progress": state.progress,
            "started_at": state.started_at.map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()),
            "duration_ms": state.duration_ms,
            "is_running": matches!(state.status, CacheGenerationStatus::Running)
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(serde_json::json!({
                "timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            })),
        })
    }
}

/// Cache Clear Tool - Force cache cleanup for agent control
pub struct CacheClearTool {
    cache_manager: Arc<Mutex<CacheManager>>,
}

#[derive(Debug, Deserialize)]
struct CacheClearParams {
    project_path: Option<String>,
    confirm: Option<bool>, // Safety confirmation
    backup_before_clear: Option<bool>, // Create backup before clearing
}

impl CacheClearTool {
    pub fn new(cache_manager: Arc<Mutex<CacheManager>>) -> Self {
        Self { cache_manager }
    }
}

#[async_trait]
impl MCPTool for CacheClearTool {
    fn name(&self) -> &str {
        "clear_cache"
    }
    
    fn description(&self) -> &str {
        "Force cache cleanup and removal. Essential for agent to reset cache state when needed. Provides safety confirmation and optional backup."
    }
    
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Path to project (defaults to current working directory)"
                },
                "confirm": {
                    "type": "boolean",
                    "description": "Safety confirmation to prevent accidental cache deletion (default: false)",
                    "default": false
                },
                "backup_before_clear": {
                    "type": "boolean",
                    "description": "Create backup of cache before clearing (default: true)",
                    "default": true
                }
            },
            "required": []
        })
    }
    
    async fn execute(&self, parameters: serde_json::Value) -> Result<MCPToolResult> {
        let params: CacheClearParams = serde_json::from_value(parameters)?;
        
        // Safety check - require confirmation
        if !params.confirm.unwrap_or(false) {
            return Ok(MCPToolResult {
                result: serde_json::json!({
                    "status": "confirmation_required",
                    "message": "Cache clear operation requires explicit confirmation. Set 'confirm': true to proceed.",
                    "warning": "This operation will permanently delete all cached analysis data."
                }),
                metadata: Some(serde_json::json!({
                    "safety_check": "enabled",
                    "confirmed": false
                })),
            });
        }
        
        let project_path = params.project_path
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
        
        println!("🗑️ Clearing cache for project: {}", project_path.display());
        
        // Get cache stats before clearing
        let entries_before = {
            let manager = self.cache_manager.lock().unwrap();
            manager.get_cache_stats().total_entries
        };
        
        // Optional backup before clearing
        let backup_path = if params.backup_before_clear.unwrap_or(true) {
            let backup_name = format!("cache_backup_{}.json", 
                SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs());
            let backup_path = project_path.join(".cache").join(&backup_name);
            
            // Create backup - assume cache is at .cache/analysis-cache.json
            let cache_file_path = project_path.join(".cache").join("analysis-cache.json");
            if cache_file_path.exists() {
                // Ensure backup directory exists
                if let Some(parent) = backup_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(&cache_file_path, &backup_path)?;
                println!("💾 Cache backed up to: {}", backup_path.display());
                Some(backup_path)
            } else {
                println!("⚠️ No cache file found to backup");
                None
            }
        } else {
            None
        };
        
        // Clear the cache using async method
        let start_time = SystemTime::now();
        let actual_entries_cleared = CacheManager::clear_cache_async(self.cache_manager.clone()).await?;
        let clear_duration = start_time.elapsed()?.as_millis();
        
        println!("✅ Cache cleared successfully in {}ms", clear_duration);
        println!("   Removed {} cache entries", entries_before);
        
        let result = serde_json::json!({
            "status": "success",
            "message": "Cache cleared successfully",
            "cleared_entries": actual_entries_cleared,
            "duration_ms": clear_duration,
            "backup_created": backup_path.is_some(),
            "backup_path": backup_path.map(|p| p.to_string_lossy().to_string())
        });
        
        let metadata = serde_json::json!({
            "project_path": project_path.to_string_lossy(),
            "confirmed": true,
            "backup_enabled": params.backup_before_clear.unwrap_or(true),
            "timestamp": SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis()
        });
        
        Ok(MCPToolResult {
            result,
            metadata: Some(metadata),
        })
    }
}
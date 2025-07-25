//! MCP Server implementation for Claude Code integration

use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, Mutex}};
use tower_http::cors::{Any, CorsLayer};
use anyhow::Result;

use crate::cache::CacheManager;
use crate::ml::services::enhanced_search::EnhancedSearchService;
use super::tools::{SmartContextTool, ExploreCodebaseTool, ProjectOverviewTool, ChangesAnalysisTool, FileSummaryTool, CacheStatusTool, CacheGenerationTool, CacheGenerationStatusTool, CacheClearTool, MCPTool};

/// MCP Server for Claude Code integration
pub struct MCPServer {
    cache_manager: Arc<Mutex<CacheManager>>,
    search_service: Arc<EnhancedSearchService>,
    tools: Arc<HashMap<String, Box<dyn MCPTool>>>,
}

/// MCP Tool definition for Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// MCP Tool call request
#[derive(Debug, Deserialize)]
pub struct MCPToolCall {
    pub tool: String,
    pub parameters: serde_json::Value,
}

/// MCP Tool response
#[derive(Debug, Serialize)]
pub struct MCPToolResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl MCPServer {
    /// Create new MCP server
    pub async fn new() -> Result<Self> {
        println!("🚀 Initializing MCP Server for Claude Code...");
        
        // Initialize components
        let project_path = std::env::current_dir()?;
        let cache_manager = Arc::new(Mutex::new(CacheManager::new(&project_path)?));
        
        // Initialize ML search service with production config
        let ml_config = crate::ml::config::MLConfig {
            model_cache_dir: std::path::PathBuf::from(".cache/ml-models"),
            memory_budget: 8_000_000_000, // 8GB
            quantization: crate::ml::config::QuantizationLevel::Q6_K,
            reasoning_timeout: 60,
            embedding_timeout: 30,
            operation_timeout: 15,
            ..Default::default()
        };
        
        let search_service = Arc::new(
            EnhancedSearchService::new(ml_config).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize search service: {}", e))?
        );
        
        // Initialize tools
        let mut tools: HashMap<String, Box<dyn MCPTool>> = HashMap::new();
        
        // Smart Context Tool - solves compactation pain point
        tools.insert(
            "smart_context".to_string(),
            Box::new(SmartContextTool::new(
                cache_manager.clone(),
                search_service.clone(),
            )),
        );
        
        // Explore Codebase Tool - semantic file discovery
        tools.insert(
            "explore_codebase".to_string(),
            Box::new(ExploreCodebaseTool::new(
                cache_manager.clone(),
                search_service.clone(),
            )),
        );
        
        // Project Overview Tool - structured project analysis
        tools.insert(
            "project_overview".to_string(),
            Box::new(ProjectOverviewTool::new(
                cache_manager.clone(),
            )),
        );
        
        // Changes Analysis Tool - Git-aware context for modified files
        tools.insert(
            "changes_analysis".to_string(),
            Box::new(ChangesAnalysisTool::new(
                cache_manager.clone(),
            )),
        );
        
        // File Summary Tool - Detailed file analysis
        tools.insert(
            "file_summary".to_string(),
            Box::new(FileSummaryTool::new(
                cache_manager.clone(),
            )),
        );
        
        // Cache Status Tool - Cache health and optimization monitoring
        tools.insert(
            "cache_status".to_string(),
            Box::new(CacheStatusTool::new(
                cache_manager.clone(),
            )),
        );
        
        // Cache Generation Tool - Background cache generation
        let cache_generation_tool = CacheGenerationTool::new(cache_manager.clone());
        let generation_state = cache_generation_tool.generation_state.clone();
        
        tools.insert(
            "generate_cache".to_string(),
            Box::new(cache_generation_tool),
        );
        
        // Cache Generation Status Tool - Monitor cache generation progress
        tools.insert(
            "cache_generation_status".to_string(),
            Box::new(CacheGenerationStatusTool::new(generation_state)),
        );
        
        // Cache Clear Tool - Force cache cleanup for agent control
        tools.insert(
            "clear_cache".to_string(),
            Box::new(CacheClearTool::new(cache_manager.clone())),
        );
        
        
        println!("✅ MCP Server initialized with {} tools", tools.len());
        println!("   - smart_context: Optimized context for Claude Code");
        println!("   - explore_codebase: Semantic file discovery");
        println!("   - project_overview: Structured project analysis");
        println!("   - changes_analysis: Git-aware context for modifications");
        println!("   - file_summary: Detailed file analysis with complexity metrics");
        println!("   - cache_status: Cache health and optimization monitoring");
        println!("   - generate_cache: Background cache generation with progress tracking");
        println!("   - cache_generation_status: Monitor cache generation progress");
        println!("   - clear_cache: Force cache cleanup with safety confirmation");
        
        Ok(Self {
            cache_manager,
            search_service,
            tools: Arc::new(tools),
        })
    }
    
    /// Start the MCP server
    pub async fn start(&self, port: u16) -> Result<()> {
        println!("🌐 Starting MCP Server on port {}...", port);
        
        let app_state = MCPServerState {
            tools: self.tools.clone(),
        };
        
        let app = Router::new()
            .route("/", get(health_check))
            .route("/tools", get(list_tools))
            .route("/tools/:tool", post(call_tool))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers(Any),
            )
            .with_state(app_state);
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("✅ MCP Server listening on http://0.0.0.0:{}", port);
        println!("📋 Available endpoints:");
        println!("   GET  / - Health check");
        println!("   GET  /tools - List available tools");
        println!("   POST /tools/:tool - Call a specific tool");
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

/// Server state for Axum
#[derive(Clone)]
struct MCPServerState {
    tools: Arc<HashMap<String, Box<dyn MCPTool>>>,
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "token-optimizer-mcp",
        "version": "0.1.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// List available tools
async fn list_tools(State(state): State<MCPServerState>) -> Json<Vec<MCPToolDefinition>> {
    let tools: Vec<MCPToolDefinition> = state.tools.iter()
        .map(|(name, tool)| MCPToolDefinition {
            name: name.clone(),
            description: tool.description().to_string(),
            parameters: tool.parameters_schema(),
        })
        .collect();
    
    Json(tools)
}

/// Call a specific tool
async fn call_tool(
    Path(tool_name): Path<String>,
    State(state): State<MCPServerState>,
    Json(request): Json<MCPToolCall>,
) -> Result<Json<MCPToolResponse>, StatusCode> {
    println!("🔧 MCP Tool call: {} with params: {}", tool_name, request.parameters);
    
    match state.tools.get(&tool_name) {
        Some(tool) => {
            match tool.execute(request.parameters).await {
                Ok(result) => {
                    println!("✅ Tool {} executed successfully", tool_name);
                    Ok(Json(MCPToolResponse {
                        success: true,
                        result: Some(result.result),
                        error: None,
                        metadata: result.metadata,
                    }))
                }
                Err(e) => {
                    println!("❌ Tool {} failed: {}", tool_name, e);
                    Ok(Json(MCPToolResponse {
                        success: false,
                        result: None,
                        error: Some(e.to_string()),
                        metadata: None,
                    }))
                }
            }
        }
        None => {
            println!("❌ Tool not found: {}", tool_name);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
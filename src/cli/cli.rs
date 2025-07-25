use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "token-optimizer")]
#[command(about = "A CLI tool for optimizing token usage in code analysis")]
#[command(version = "1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze project files and generate metadata
    Analyze {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Force re-analysis of all files
        #[arg(short, long)]
        force: bool,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Generate code summary for files
    Summary {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Specific file to summarize
        #[arg(long)]
        file: Option<PathBuf>,
        
        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Show files changed since last analysis
    Changes {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Show only modified files
        #[arg(short, long)]
        modified_only: bool,
    },
    
    /// Generate project overview
    Overview {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Output format (json, text, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// Include health metrics
        #[arg(long)]
        include_health: bool,
    },
    
    /// Cache management commands
    Cache {
        #[command(subcommand)]
        action: CacheCommands,
    },
    
    /// ML-enhanced analysis commands
    ML {
        #[command(subcommand)]
        action: MLCommands,
    },
    
    /// Start MCP server for Claude Code integration
    Mcp {
        /// Port to run the MCP server on
        #[arg(short, long, default_value = "4080")]
        port: u16,
        
        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache status
    Status {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Clean cache (remove outdated entries)
    Clean {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Rebuild entire cache
    Rebuild {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// Clear entire cache
    Clear {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum MLCommands {
    /// Smart context analysis for functions
    Context {
        /// Function name to analyze
        #[arg(short, long)]
        function: String,
        
        /// File path containing the function
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Enable AI-enhanced analysis
        #[arg(long)]
        ai_enhanced: bool,
        
        /// Output format (json, text)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Impact analysis for code changes
    Impact {
        /// Changed file path
        #[arg(short, long)]
        changed_file: PathBuf,
        
        /// Changed function names
        #[arg(long)]
        changed_functions: Vec<String>,
        
        /// Enable AI-enhanced analysis
        #[arg(long)]
        ai_analysis: bool,
        
        /// Output format (json, text)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Pattern detection and analysis
    Patterns {
        /// Path to analyze
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Detect code duplicates
        #[arg(long)]
        detect_duplicates: bool,
        
        /// Use ML similarity matching
        #[arg(long)]
        ml_similarity: bool,
        
        /// Minimum similarity threshold (0.0 to 1.0)
        #[arg(long, default_value = "0.8")]
        min_similarity: f32,
        
        /// Output format (json, text)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Semantic code search
    Search {
        /// Search query
        #[arg(short, long)]
        query: String,
        
        /// Path to search in
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        
        /// Use semantic search
        #[arg(long)]
        semantic: bool,
        
        /// Include context information
        #[arg(long)]
        include_context: bool,
        
        /// Maximum number of results
        #[arg(long, default_value = "10")]
        max_results: usize,
        
        /// Output format (json, text)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Token usage optimization
    Optimize {
        /// Task description
        #[arg(short, long)]
        task: String,
        
        /// Maximum token budget
        #[arg(long, default_value = "5000")]
        max_tokens: usize,
        
        /// Enable AI-enhanced optimization
        #[arg(long)]
        ai_enhanced: bool,
        
        /// Output format (json, text)
        #[arg(long, default_value = "json")]
        format: String,
    },
    
    /// Model management commands
    Models {
        #[command(subcommand)]
        action: ModelCommands,
    },
}

#[derive(Subcommand)]
pub enum ModelCommands {
    /// List available models
    List {
        /// Show local models only
        #[arg(long)]
        local_only: bool,
    },
    
    /// Download a model
    Download {
        /// Model name to download
        #[arg(short, long)]
        model: String,
        
        /// Download all models
        #[arg(long)]
        all: bool,
    },
    
    /// Delete a model from cache
    Delete {
        /// Model name to delete
        #[arg(short, long)]
        model: String,
    },
    
    /// Show model cache status
    Status,
    
    /// Clean model cache
    Clean,
}
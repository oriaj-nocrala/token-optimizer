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
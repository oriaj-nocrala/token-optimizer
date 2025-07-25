//! MCP Server standalone binary for Claude Code integration
//! 
//! This binary starts the Model Context Protocol server that provides
//! smart context and semantic search capabilities to Claude Code.
//! 
//! Usage:
//!   cargo run --bin mcp_server -- --port 4080
//!   
//! This solves the compactation pain point by providing only relevant
//! code context, reducing token usage by 70-90%.

use clap::Parser;
use anyhow::Result;
use token_optimizer::mcp::MCPServer;

#[derive(Parser)]
#[command(name = "mcp-server")]
#[command(about = "MCP Server for Claude Code integration")]
#[command(version = "1.0")]
struct Cli {
    /// Port to run the MCP server on
    #[arg(short, long, default_value = "4080")]
    port: u16,
    
    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.debug {
        println!("🔧 Debug mode enabled");
    }
    
    println!("🚀 Token Optimizer MCP Server");
    println!("   Solving Claude Code's compactation pain point");
    println!("   Port: {}", cli.port);
    println!();
    
    // Initialize and start MCP server
    let server = MCPServer::new().await?;
    
    println!("🎯 Ultimate LLM Agent Token Optimization Tools:");
    println!("   • smart_context: Get optimized code context (reduces tokens 70-90%)");
    println!("   • explore_codebase: Discover related files semantically");
    println!("   • project_overview: Structured project analysis without reading all files");
    println!("   • changes_analysis: Git-aware context for modified files only");
    println!("   • file_summary: Detailed analysis of specific files with complexity metrics");
    println!();
    
    println!("💡 Claude Code Integration:");
    println!("   Configure Claude Code to use this MCP server at http://localhost:{}", cli.port);
    println!("   Tools will be available for intelligent context selection");
    println!();
    
    server.start(cli.port).await?;
    
    Ok(())
}
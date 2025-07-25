//! MCP Server command implementation

use anyhow::Result;
use clap::Args;

use crate::mcp::MCPServer;

#[derive(Args)]
pub struct MCPCommand {
    /// Port to run the MCP server on
    #[clap(short, long, default_value = "4080")]
    pub port: u16,
    
    /// Enable debug logging
    #[clap(long)]
    pub debug: bool,
}

impl MCPCommand {
    pub async fn execute(&self) -> Result<()> {
        if self.debug {
            println!("🔧 Debug mode enabled");
        }
        
        println!("🚀 Starting token-optimizer MCP Server...");
        println!("   Purpose: Provide smart context to Claude Code");
        println!("   Port: {}", self.port);
        println!("   This server solves the compactation pain point by providing");
        println!("   only relevant code context, reducing token usage by 70-90%");
        println!();
        
        // Initialize and start MCP server
        let server = MCPServer::new().await?;
        
        println!("📋 Available MCP tools for Claude Code:");
        println!("   • smart_context: Get optimized code context for queries");
        println!("   • explore_codebase: Discover related files semantically");
        println!();
        
        println!("💡 Usage in Claude Code:");
        println!("   Instead of loading entire files, Claude Code can now call:");
        println!("   - smart_context(\"authentication logic\", max_tokens=3000)");
        println!("   - explore_codebase(\"error handling patterns\")");
        println!();
        
        server.start(self.port).await?;
        
        Ok(())
    }
}
//! Model Context Protocol (MCP) Server for Claude Code integration
//! 
//! This module provides an MCP server that allows Claude Code to access
//! intelligent context and semantic search capabilities, solving the
//! compactation pain point by providing only relevant code context.

pub mod server;
pub mod tools;
pub mod context_optimizer;

pub use server::MCPServer;
pub use tools::*;
pub use context_optimizer::*;
mod types;
mod utils;
mod analyzers;
mod cache;
mod cli;
mod generators;

use clap::Parser;
use cli::{Cli, Commands, CacheCommands};
use cli::commands::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Analyze { path, force, verbose } => {
            run_analyze(path, *force, *verbose)?;
        }
        
        Commands::Summary { path, file, format } => {
            run_summary(path, file.as_deref(), format)?;
        }
        
        Commands::Changes { path, modified_only } => {
            run_changes(path, *modified_only)?;
        }
        
        Commands::Overview { path, format, include_health } => {
            run_overview(path, format, *include_health)?;
        }
        
        Commands::Cache { action } => {
            match action {
                CacheCommands::Status { path } => {
                    run_cache_status(path)?;
                }
                
                CacheCommands::Clean { path } => {
                    run_cache_clean(path)?;
                }
                
                CacheCommands::Rebuild { path } => {
                    run_cache_rebuild(path)?;
                }
                
                CacheCommands::Clear { path } => {
                    run_cache_clear(path)?;
                }
            }
        }
    }

    Ok(())
}

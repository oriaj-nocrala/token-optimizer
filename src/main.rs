mod types;
mod utils;
mod analyzers;
mod cache;
mod cli;
mod generators;
mod ml;

use clap::Parser;
use cli::{Cli, Commands, CacheCommands, MLCommands, ModelCommands};
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
        
        Commands::ML { action } => {
            match action {
                MLCommands::Context { function, file, ai_enhanced, format } => {
                    run_ml_context(function, file.as_deref(), *ai_enhanced, format).await?;
                }
                
                MLCommands::Impact { changed_file, changed_functions, ai_analysis, format } => {
                    run_ml_impact(changed_file, changed_functions, *ai_analysis, format).await?;
                }
                
                MLCommands::Patterns { path, detect_duplicates, ml_similarity, min_similarity, format } => {
                    run_ml_patterns(path, *detect_duplicates, *ml_similarity, *min_similarity, format).await?;
                }
                
                MLCommands::Search { query, path, semantic, include_context, max_results, format } => {
                    run_ml_search(query, path, *semantic, *include_context, *max_results, format).await?;
                }
                
                MLCommands::Optimize { task, max_tokens, ai_enhanced, format } => {
                    run_ml_optimize(task, *max_tokens, *ai_enhanced, format).await?;
                }
                
                MLCommands::Models { action } => {
                    match action {
                        ModelCommands::List { local_only } => {
                            run_model_list(*local_only).await?;
                        }
                        
                        ModelCommands::Download { model, all } => {
                            run_model_download(Some(model.as_str()), *all).await?;
                        }
                        
                        ModelCommands::Delete { model } => {
                            run_model_delete(model).await?;
                        }
                        
                        ModelCommands::Status => {
                            run_model_status().await?;
                        }
                        
                        ModelCommands::Clean => {
                            run_model_clean().await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

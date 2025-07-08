use anyhow::Result;
use std::path::Path;
use crate::cache::CacheManager;

pub fn run_analyze(path: &Path, force: bool, verbose: bool) -> Result<()> {
    if verbose {
        println!("Starting analysis of project at: {}", path.display());
    }
    
    let mut cache_manager = CacheManager::new(path)?;
    cache_manager.analyze_project(path, force)?;
    
    let stats = cache_manager.get_cache_stats();
    
    println!("Analysis complete!");
    println!("- Files analyzed: {}", stats.total_entries);
    println!("- Total size: {:.2} MB", stats.total_size as f64 / 1024.0 / 1024.0);
    
    if let Some(oldest) = stats.oldest_entry {
        println!("- Oldest entry: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }
    
    if let Some(newest) = stats.newest_entry {
        println!("- Newest entry: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }
    
    Ok(())
}
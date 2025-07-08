use anyhow::Result;
use std::path::Path;
use crate::cache::CacheManager;

pub fn run_cache_status(path: &Path) -> Result<()> {
    let cache_manager = CacheManager::new(path)?;
    let stats = cache_manager.get_cache_stats();
    
    println!("Cache Status");
    println!("============");
    println!("Total entries: {}", stats.total_entries);
    println!("Total size: {:.2} MB", stats.total_size as f64 / 1024.0 / 1024.0);
    println!("Last updated: {}", stats.last_updated.format("%Y-%m-%d %H:%M:%S"));
    
    if let Some(oldest) = stats.oldest_entry {
        println!("Oldest entry: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
    }
    
    if let Some(newest) = stats.newest_entry {
        println!("Newest entry: {}", newest.format("%Y-%m-%d %H:%M:%S"));
    }
    
    let outdated = cache_manager.get_outdated_files(path)?;
    if !outdated.is_empty() {
        println!("\nOutdated files: {}", outdated.len());
        for file in outdated.iter().take(10) {
            println!("  - {}", file);
        }
        if outdated.len() > 10 {
            println!("  ... and {} more", outdated.len() - 10);
        }
    }
    
    Ok(())
}

pub fn run_cache_clean(path: &Path) -> Result<()> {
    let mut cache_manager = CacheManager::new(path)?;
    let deleted_count = cache_manager.clean_cache(path)?;
    
    println!("Cache cleaned!");
    println!("Removed {} outdated entries", deleted_count);
    
    Ok(())
}

pub fn run_cache_rebuild(path: &Path) -> Result<()> {
    let mut cache_manager = CacheManager::new(path)?;
    
    println!("Rebuilding cache...");
    cache_manager.rebuild_cache(path)?;
    
    let stats = cache_manager.get_cache_stats();
    println!("Cache rebuilt!");
    println!("Total entries: {}", stats.total_entries);
    println!("Total size: {:.2} MB", stats.total_size as f64 / 1024.0 / 1024.0);
    
    Ok(())
}

pub fn run_cache_clear(path: &Path) -> Result<()> {
    let mut cache_manager = CacheManager::new(path)?;
    cache_manager.clear_cache()?;
    
    println!("Cache cleared!");
    
    Ok(())
}
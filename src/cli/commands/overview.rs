use anyhow::Result;
use std::path::Path;
use crate::generators::{ProjectOverviewGenerator, ReportGenerator};
use crate::cache::CacheManager;

pub fn run_overview(path: &Path, format: &str, include_health: bool) -> Result<()> {
    // Ensure we analyze the project first to have cache data
    let mut cache_manager = CacheManager::new(path)?;
    
    // Check if cache exists and is populated, if not analyze project
    if cache_manager.get_cache().entries.is_empty() {
        cache_manager.analyze_project(path, false)?;
    }
    
    let generator = ProjectOverviewGenerator::new(cache_manager);
    let report_generator = ReportGenerator::new();
    
    let mut overview = generator.generate_overview(path)?;
    
    if !include_health {
        // Simplified health metrics if not requested
        overview.health_metrics.test_coverage = 0.0;
        overview.health_metrics.performance.load_time = 0.0;
        overview.health_metrics.performance.memory_usage = 0;
    }
    
    match format {
        "json" => {
            let json = report_generator.generate_json_report(&overview)?;
            println!("{}", json);
        }
        "markdown" => {
            let markdown = report_generator.generate_markdown_report(&overview)?;
            println!("{}", markdown);
        }
        _ => {
            let text = report_generator.generate_text_report(&overview)?;
            println!("{}", text);
        }
    }
    
    Ok(())
}
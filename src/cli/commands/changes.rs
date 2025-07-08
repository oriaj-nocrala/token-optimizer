use anyhow::Result;
use std::path::Path;
use crate::analyzers::DiffAnalyzer;

pub fn run_changes(path: &Path, modified_only: bool) -> Result<()> {
    let diff_analyzer = DiffAnalyzer::new(path)?;
    let changes = diff_analyzer.analyze_changes(path)?;
    
    println!("Change Analysis - Session: {}", changes.session_id);
    println!("Timestamp: {}", changes.timestamp.format("%Y-%m-%d %H:%M:%S"));
    println!("Impact Scope: {:?}", changes.impact_scope);
    println!();
    
    if !changes.modified_files.is_empty() {
        println!("Modified Files:");
        for file in &changes.modified_files {
            println!("  - {} ({:?})", file.path, file.change_type);
            println!("    Lines: +{} -{}", file.lines_added, file.lines_removed);
            if !file.sections_changed.is_empty() {
                println!("    Sections: {}", file.sections_changed.join(", "));
            }
        }
        println!();
    }
    
    if !modified_only {
        if !changes.added_files.is_empty() {
            println!("Added Files:");
            for file in &changes.added_files {
                println!("  + {}", file);
            }
            println!();
        }
        
        if !changes.deleted_files.is_empty() {
            println!("Deleted Files:");
            for file in &changes.deleted_files {
                println!("  - {}", file);
            }
            println!();
        }
        
        if !changes.renamed_files.is_empty() {
            println!("Renamed Files:");
            for file in &changes.renamed_files {
                println!("  {} -> {}", file.old_path, file.new_path);
            }
            println!();
        }
    }
    
    if !changes.relevant_context.is_empty() {
        println!("Relevant Context:");
        for context in &changes.relevant_context {
            println!("  - {}", context);
        }
        println!();
    }
    
    if !changes.suggested_actions.is_empty() {
        println!("Suggested Actions:");
        for action in &changes.suggested_actions {
            println!("  - {}", action);
        }
    }
    
    Ok(())
}
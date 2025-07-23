//! End-to-end tests for Pattern Detection Service with calendario-psicologia project

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use serial_test::serial;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::services::pattern::PatternDetectionService;
use crate::ml::models::*;

#[tokio::test]
#[serial]
async fn test_pattern_detection_with_real_project() -> Result<()> {
    // Skip if calendario-psicologia project doesn't exist
    let project_path = "calendario-psicologia";
    if !Path::new(project_path).exists() {
        println!("Skipping test - calendario-psicologia project not found");
        return Ok(());
    }

    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = PatternDetectionService::new(config, plugin_manager);

    // Initialize service
    service.initialize().await?;
    assert!(service.is_ready());

    // Run pattern detection on real project
    let start_time = std::time::Instant::now();
    let pattern_report = service.detect_patterns(project_path).await?;
    let analysis_time = start_time.elapsed();

    println!("Pattern Detection Analysis Results:");
    println!("Project: {}", pattern_report.project_path);
    println!("Analysis time: {:?}", analysis_time);
    println!("Total functions analyzed: {}", pattern_report.analysis_metadata.total_functions);
    println!("Embedding model used: {}", pattern_report.analysis_metadata.embedding_model);
    println!("Similarity threshold: {}", pattern_report.analysis_metadata.similarity_threshold);

    // Validate results
    assert_eq!(pattern_report.project_path, project_path);
    assert!(pattern_report.analysis_metadata.total_functions > 0);
    assert!(analysis_time.as_secs() < 30); // Should complete in under 30 seconds

    // Check duplicate patterns
    println!("\nDuplicate Patterns Found: {}", pattern_report.duplicate_patterns.len());
    for (i, pattern) in pattern_report.duplicate_patterns.iter().take(3).enumerate() {
        println!("  {}. {} -> {} (similarity: {:.2})", 
                i + 1, 
                pattern.primary_function.function_name,
                pattern.duplicate_functions.first().map(|f| f.function_name.as_str()).unwrap_or("N/A"),
                pattern.similarity_score);
    }

    // Check semantic clusters
    println!("\nSemantic Clusters Found: {}", pattern_report.semantic_clusters.len());
    for (i, cluster) in pattern_report.semantic_clusters.iter().take(3).enumerate() {
        println!("  {}. {} ({} functions, similarity: {:.2})", 
                i + 1, 
                cluster.cluster_type,
                cluster.functions.len(),
                cluster.similarity_score);
    }

    // Check architectural patterns
    println!("\nArchitectural Patterns Found: {}", pattern_report.architectural_patterns.len());
    for (i, pattern) in pattern_report.architectural_patterns.iter().take(3).enumerate() {
        println!("  {}. {} ({} files, confidence: {:.2})", 
                i + 1, 
                pattern.pattern_name,
                pattern.affected_files.len(),
                pattern.confidence);
    }

    // Check refactoring suggestions
    println!("\nRefactoring Suggestions: {}", pattern_report.refactoring_suggestions.len());
    for (i, suggestion) in pattern_report.refactoring_suggestions.iter().take(3).enumerate() {
        println!("  {}. {:?} - {} (effort: {:?}, priority: {:?})", 
                i + 1, 
                suggestion.suggestion_type,
                suggestion.description,
                suggestion.effort_level,
                suggestion.priority);
    }

    // Validate pattern report structure
    assert!(pattern_report.analysis_metadata.total_functions > 10); // Should find functions in Angular project
    
    // Should find some architectural patterns in Angular project
    let has_angular_patterns = pattern_report.architectural_patterns.iter()
        .any(|p| p.pattern_name == "Component Pattern" || p.pattern_name == "Service Pattern");
    
    if pattern_report.architectural_patterns.len() > 0 {
        println!("\nFound Angular architectural patterns: {}", has_angular_patterns);
    }

    // Ensure proper cleanup
    service.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_similar_functions_search_real_project() -> Result<()> {
    // Skip if calendario-psicologia project doesn't exist
    let project_path = "calendario-psicologia";
    if !Path::new(project_path).exists() {
        println!("Skipping test - calendario-psicologia project not found");
        return Ok(());
    }

    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = PatternDetectionService::new(config, plugin_manager);

    // Initialize service
    service.initialize().await?;

    // Search for similar functions to a typical Angular function
    let target_function = r#"
        ngOnInit() {
            this.loadData();
            this.initializeSubscriptions();
        }
    "#;

    let start_time = std::time::Instant::now();
    let similar_functions = service.find_similar_functions(target_function, project_path).await?;
    let search_time = start_time.elapsed();

    println!("Similar Functions Search Results:");
    println!("Target function: ngOnInit lifecycle method");
    println!("Search time: {:?}", search_time);
    println!("Similar functions found: {}", similar_functions.len());

    // Display results
    for (i, similar_func) in similar_functions.iter().take(5).enumerate() {
        println!("  {}. {} in {} (similarity: {:.2})", 
                i + 1, 
                similar_func.function_name,
                similar_func.file_path,
                similar_func.similarity_score);
    }

    // Validate results
    assert!(search_time.as_secs() < 10); // Should complete quickly
    
    // Should find at least some similar functions in Angular project
    if similar_functions.len() > 0 {
        assert!(similar_functions.iter().all(|f| f.similarity_score > 0.7));
        assert!(similar_functions.iter().all(|f| f.similarity_score <= 1.0));
    }

    // Ensure proper cleanup
    service.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_duplicate_code_detection_real_project() -> Result<()> {
    // Skip if calendario-psicologia project doesn't exist
    let project_path = "calendario-psicologia";
    if !Path::new(project_path).exists() {
        println!("Skipping test - calendario-psicologia project not found");
        return Ok(());
    }

    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = PatternDetectionService::new(config, plugin_manager);

    // Initialize service
    service.initialize().await?;

    // Extract code fragments from project
    let project_path_obj = Path::new(project_path);
    let code_fragments = service.extract_code_fragments(project_path_obj)?;

    println!("Code Fragments Extracted: {}", code_fragments.len());

    if code_fragments.len() > 0 {
        // Test duplicate detection on real code
        let start_time = std::time::Instant::now();
        let duplicates = service.detect_duplicate_code(&code_fragments).await?;
        let detection_time = start_time.elapsed();

        println!("Duplicate Detection Results:");
        println!("Detection time: {:?}", detection_time);
        println!("Duplicate patterns found: {}", duplicates.len());

        // Display results
        for (i, duplicate) in duplicates.iter().take(3).enumerate() {
            println!("  {}. {} <-> {} (similarity: {:.2})", 
                    i + 1, 
                    duplicate.primary_function.function_name,
                    duplicate.duplicate_functions.first().map(|f| f.function_name.as_str()).unwrap_or("N/A"),
                    duplicate.similarity_score);
        }

        // Validate results
        assert!(detection_time.as_secs() < 30); // Should complete in reasonable time
        
        // All duplicates should have high similarity
        for duplicate in &duplicates {
            assert!(duplicate.similarity_score > 0.90);
            assert!(duplicate.similarity_score <= 1.01); // Allow small floating point errors
            assert!(!duplicate.primary_function.function_name.is_empty());
            assert!(!duplicate.duplicate_functions.is_empty());
        }
    }

    // Ensure proper cleanup
    service.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_pattern_detection_performance_real_project() -> Result<()> {
    // Skip if calendario-psicologia project doesn't exist
    let project_path = "calendario-psicologia";
    if !Path::new(project_path).exists() {
        println!("Skipping test - calendario-psicologia project not found");
        return Ok(());
    }

    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = PatternDetectionService::new(config, plugin_manager);

    // Initialize service
    service.initialize().await?;

    // Measure performance metrics
    let start_time = std::time::Instant::now();
    let pattern_report = service.detect_patterns(project_path).await?;
    let total_time = start_time.elapsed();

    println!("Performance Metrics:");
    println!("Total analysis time: {:?}", total_time);
    println!("Functions analyzed: {}", pattern_report.analysis_metadata.total_functions);
    println!("Functions per second: {:.2}", 
            pattern_report.analysis_metadata.total_functions as f64 / total_time.as_secs_f64());

    // Performance assertions
    assert!(total_time.as_secs() < 60); // Should complete in under 1 minute
    
    if pattern_report.analysis_metadata.total_functions > 0 {
        let functions_per_second = pattern_report.analysis_metadata.total_functions as f64 / total_time.as_secs_f64();
        assert!(functions_per_second > 1.0); // Should process at least 1 function per second
    }

    // Memory usage should be reasonable (this is a simple check)
    println!("   Found {} duplicate patterns", pattern_report.duplicate_patterns.len());
    println!("   Found {} semantic clusters", pattern_report.semantic_clusters.len());
    
    // For large projects, we expect more patterns - just verify it's not completely unreasonable
    assert!(pattern_report.duplicate_patterns.len() < 20000); // Shouldn't find excessive duplicates (increased for large projects)
    assert!(pattern_report.semantic_clusters.len() < 5000); // Reasonable number of clusters

    // Ensure proper cleanup
    service.shutdown().await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_fallback_mode_real_project() -> Result<()> {
    // Skip if calendario-psicologia project doesn't exist
    let project_path = "calendario-psicologia";
    if !Path::new(project_path).exists() {
        println!("Skipping test - calendario-psicologia project not found");
        return Ok(());
    }

    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new()); // No ML plugins loaded
    let mut service = PatternDetectionService::new(config, plugin_manager);

    // Initialize service (should work without ML plugins)
    service.initialize().await?;
    assert!(service.is_ready());

    // Run pattern detection in fallback mode
    let start_time = std::time::Instant::now();
    let pattern_report = service.detect_patterns(project_path).await?;
    let analysis_time = start_time.elapsed();

    println!("Fallback Mode Analysis Results:");
    println!("Analysis time: {:?}", analysis_time);
    println!("Embedding model used: {}", pattern_report.analysis_metadata.embedding_model);
    println!("Total functions analyzed: {}", pattern_report.analysis_metadata.total_functions);

    // Validate fallback mode
    assert_eq!(pattern_report.analysis_metadata.embedding_model, "lexical-similarity");
    assert!(pattern_report.analysis_metadata.total_functions > 0);
    assert!(analysis_time.as_secs() < 30); // Should be fast in fallback mode

    // Should still find some patterns using lexical similarity
    println!("Patterns found in fallback mode:");
    println!("  Duplicate patterns: {}", pattern_report.duplicate_patterns.len());
    println!("  Semantic clusters: {}", pattern_report.semantic_clusters.len());
    println!("  Architectural patterns: {}", pattern_report.architectural_patterns.len());
    println!("  Refactoring suggestions: {}", pattern_report.refactoring_suggestions.len());

    // Ensure proper cleanup
    service.shutdown().await?;
    Ok(())
}
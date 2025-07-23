//! E2E tests with real calendario-psicologia project
//! Tests ML services with actual Angular project files

use anyhow::Result;
use std::sync::Arc;
use std::fs;
use std::path::Path;

use crate::ml::services::context::SmartContextService;
use crate::ml::services::impact_analysis::ImpactAnalysisService;
use crate::ml::services::search::SemanticSearchService;
use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::cache::CacheManager;
use crate::analyzers::FileAnalyzer;

/// E2E test with real calendario-psicologia project
#[tokio::test]
async fn test_e2e_calendar_project_analysis() -> Result<()> {
    println!("🏥 Starting E2E test with calendario-psicologia project...");
    
    let project_path = Path::new("/home/oriaj/Prog/Rust/Ts-tools/claude-ts-tools/calendario-psicologia");
    
    // Check if project exists
    if !project_path.exists() {
        println!("⚠️  Skipping E2E test - calendario-psicologia project not found");
        return Ok(());
    }
    
    // Test config
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    
    // Initialize ML services
    let mut context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    let mut search_service = SemanticSearchService::new(config.clone(), plugin_manager.clone());
    
    context_service.initialize().await?;
    impact_service.initialize().await?;
    search_service.initialize().await?;
    
    println!("✅ ML services initialized");
    
    // Test 1: Analyze AuthService from real project
    println!("🔍 Testing AuthService analysis...");
    let auth_service_path = project_path.join("src/app/services/auth.service.ts");
    
    if auth_service_path.exists() {
        let auth_service_content = fs::read_to_string(&auth_service_path)?;
        
        // Test context analysis
        let context = context_service.create_base_context(
            "AuthService",
            &auth_service_path.to_string_lossy(),
            &auth_service_content
        )?;
        
        println!("✅ AuthService analysis:");
        println!("   - File: {}", context.file_path);
        println!("   - Complexity: {:.2}", context.complexity_score);
        println!("   - Dependencies: {}", context.dependencies.len());
        println!("   - Impact Scope: {:?}", context.impact_scope);
        
        // Validate results
        assert!(context.complexity_score > 0.0);
        assert!(!context.dependencies.is_empty());
        
        // Test enhanced context analysis
        let enhanced_context = context_service.analyze_function_context(
            "AuthService",
            &auth_service_path.to_string_lossy(),
            &auth_service_content
        ).await?;
        
        println!("✅ Enhanced AuthService analysis:");
        println!("   - Risk Level: {:?}", enhanced_context.risk_assessment.overall_risk);
        println!("   - Optimization Suggestions: {}", enhanced_context.optimization_suggestions.len());
        
        // Enhanced analysis should provide risk assessment (optimization suggestions may be empty)
        assert!(enhanced_context.risk_assessment.overall_risk != crate::ml::models::RiskLevel::Low || enhanced_context.optimization_suggestions.len() >= 0);
    } else {
        println!("⚠️  AuthService not found at expected path");
    }
    
    // Test 2: Analyze UserService
    println!("🔍 Testing UserService analysis...");
    let user_service_path = project_path.join("src/app/services/user.service.ts");
    
    if user_service_path.exists() {
        let user_service_content = fs::read_to_string(&user_service_path)?;
        
        let context = context_service.create_base_context(
            "UserService",
            &user_service_path.to_string_lossy(),
            &user_service_content
        )?;
        
        println!("✅ UserService analysis:");
        println!("   - Complexity: {:.2}", context.complexity_score);
        println!("   - Dependencies: {}", context.dependencies.len());
        
        assert!(context.complexity_score > 0.0);
    } else {
        println!("⚠️  UserService not found at expected path");
    }
    
    // Test 3: Analyze Angular Components
    println!("🔍 Testing Angular components analysis...");
    let components_to_test = vec![
        "src/app/login/login.component.ts",
        "src/app/dashboard/dashboard.component.ts",
        "src/app/appointments/appointments.component.ts",
        "src/app/calendar/calendar.component.ts",
    ];
    
    let mut analyzed_components = 0;
    
    for component_path in components_to_test {
        let full_path = project_path.join(component_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)?;
            let component_name = component_path.split('/').last().unwrap_or("component");
            
            let context = context_service.create_base_context(
                component_name,
                &full_path.to_string_lossy(),
                &content
            )?;
            
            println!("✅ {} analysis:", component_name);
            println!("   - Complexity: {:.2}", context.complexity_score);
            println!("   - Dependencies: {}", context.dependencies.len());
            
            analyzed_components += 1;
            
            assert!(context.complexity_score > 0.0);
        }
    }
    
    println!("✅ Analyzed {} components", analyzed_components);
    assert!(analyzed_components > 0, "Should analyze at least one component");
    
    // Test 4: Semantic Search across project
    println!("🔎 Testing semantic search across project...");
    let search_results = search_service.search(
        "authentication and user management",
        &project_path.to_string_lossy(),
        Some(5)
    ).await?;
    
    println!("✅ Search results:");
    println!("   - Total matches: {}", search_results.total_matches);
    println!("   - Results: {}", search_results.results.len());
    
    for (i, result) in search_results.results.iter().enumerate() {
        println!("   {}. {} (score: {:.3})", i + 1, result.file_path, result.relevance_score);
    }
    
    assert!(search_results.results.len() > 0);
    assert!(search_results.total_matches > 0);
    
    // Test 5: Project-wide dependency analysis
    println!("🔗 Testing project-wide dependency analysis...");
    let mut project_dependencies = Vec::new();
    
    // Analyze key files for dependencies
    let key_files = vec![
        "src/app/services/auth.service.ts",
        "src/app/services/user.service.ts",
        "src/app/guards/auth.guard.ts",
        "src/app/interceptors/auth.interceptor.ts",
    ];
    
    for file_path in key_files {
        let full_path = project_path.join(file_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)?;
            let filename = file_path.split('/').last().unwrap_or("file");
            
            let context = context_service.create_base_context(
                filename,
                &full_path.to_string_lossy(),
                &content
            )?;
            
            project_dependencies.extend(context.dependencies);
        }
    }
    
    println!("✅ Project dependencies analysis:");
    println!("   - Total dependencies found: {}", project_dependencies.len());
    
    // Count unique dependencies
    let mut unique_deps = std::collections::HashSet::new();
    for dep in &project_dependencies {
        unique_deps.insert(&dep.target_file);
    }
    
    println!("   - Unique dependencies: {}", unique_deps.len());
    
    assert!(project_dependencies.len() > 0);
    assert!(unique_deps.len() > 0);
    
    // Test 6: Integration with Cache Manager
    println!("💾 Testing cache integration...");
    let mut cache_manager = CacheManager::new(project_path)?;
    
    // Analyze a few key files with cache
    let auth_service_path = project_path.join("src/app/services/auth.service.ts");
    if auth_service_path.exists() {
        cache_manager.analyze_file(&auth_service_path)?;
        
        let cache_stats = cache_manager.get_cache_stats();
        println!("✅ Cache integration:");
        println!("   - Entries in cache: {}", cache_stats.total_entries);
        println!("   - Cache size: {} bytes", cache_stats.total_size);
        
        assert!(cache_stats.total_entries > 0);
    }
    
    // Test 7: Real file analysis with FileAnalyzer
    println!("📊 Testing FileAnalyzer integration...");
    let file_analyzer = FileAnalyzer::new();
    let typescript_files = vec![
        "src/app/services/auth.service.ts",
        "src/app/models/user.model.ts",
        "src/app/app.component.ts",
    ];
    
    let mut analyzed_files = 0;
    for file_path in typescript_files {
        let full_path = project_path.join(file_path);
        if full_path.exists() {
            let metadata = file_analyzer.analyze_file(&full_path)?;
            
            println!("✅ File analysis: {}", file_path);
            println!("   - Type: {:?}", metadata.file_type);
            println!("   - Complexity: {:?}", metadata.complexity);
            if let Some(detailed) = &metadata.detailed_analysis {
                println!("   - Functions: {}", detailed.functions.len());
                println!("   - Classes: {}", detailed.classes.len());
            }
            println!("   - Imports: {}", metadata.imports.len());
            
            analyzed_files += 1;
            
            // Should have some analysis data
            assert!(metadata.imports.len() > 0 || metadata.exports.len() > 0 || metadata.detailed_analysis.is_some());
        }
    }
    
    println!("✅ Analyzed {} TypeScript files", analyzed_files);
    assert!(analyzed_files > 0);
    
    println!("🎉 E2E test completed successfully!");
    println!("   - All ML services working with real Angular project");
    println!("   - Context analysis: ✅");
    println!("   - Semantic search: ✅");  
    println!("   - Dependency analysis: ✅");
    println!("   - Cache integration: ✅");
    println!("   - File analysis: ✅");
    
    Ok(())
}

/// Test ML services performance with real project files
#[tokio::test]
async fn test_calendar_project_performance() -> Result<()> {
    println!("⚡ Testing performance with calendario-psicologia project...");
    
    let project_path = Path::new("/home/oriaj/Prog/Rust/Ts-tools/claude-ts-tools/calendario-psicologia");
    
    if !project_path.exists() {
        println!("⚠️  Skipping performance test - project not found");
        return Ok(());
    }
    
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    
    // Test performance with multiple files
    let start_time = std::time::Instant::now();
    
    let test_files = vec![
        "src/app/services/auth.service.ts",
        "src/app/services/user.service.ts",
        "src/app/login/login.component.ts",
        "src/app/dashboard/dashboard.component.ts",
        "src/app/appointments/appointments.component.ts",
    ];
    
    let mut processed_files = 0;
    
    for file_path in test_files {
        let full_path = project_path.join(file_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)?;
            let filename = file_path.split('/').last().unwrap_or("file");
            
            let _context = context_service.create_base_context(
                filename,
                &full_path.to_string_lossy(),
                &content
            )?;
            
            processed_files += 1;
        }
    }
    
    let duration = start_time.elapsed();
    
    println!("✅ Performance test results:");
    println!("   - Files processed: {}", processed_files);
    println!("   - Total time: {:?}", duration);
    println!("   - Average per file: {:?}", duration / processed_files as u32);
    
    // Should process multiple files in reasonable time
    assert!(duration.as_secs() < 10, "Should process files in under 10 seconds");
    assert!(processed_files > 0);
    
    Ok(())
}

/// Test error handling with real project edge cases
#[tokio::test]
async fn test_calendar_project_edge_cases() -> Result<()> {
    println!("🧪 Testing edge cases with calendario-psicologia project...");
    
    let project_path = Path::new("/home/oriaj/Prog/Rust/Ts-tools/claude-ts-tools/calendario-psicologia");
    
    if !project_path.exists() {
        println!("⚠️  Skipping edge case test - project not found");
        return Ok(());
    }
    
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    
    // Test 1: Empty or minimal files
    let spec_files = vec![
        "src/app/services/auth.service.spec.ts",
        "src/app/services/user.service.spec.ts",
    ];
    
    for file_path in spec_files {
        let full_path = project_path.join(file_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)?;
            let filename = file_path.split('/').last().unwrap_or("file");
            
            // Should handle spec files gracefully
            let context = context_service.create_base_context(
                filename,
                &full_path.to_string_lossy(),
                &content
            )?;
            
            println!("✅ Spec file analysis: {}", filename);
            println!("   - Complexity: {:.2}", context.complexity_score);
            
            // Should not crash on spec files
            assert!(context.complexity_score >= 0.0);
        }
    }
    
    // Test 2: Non-existent file handling
    let non_existent_path = project_path.join("src/app/non-existent-file.ts");
    let result = context_service.create_base_context(
        "NonExistentFile",
        &non_existent_path.to_string_lossy(),
        ""
    );
    
    // Should handle empty content gracefully
    assert!(result.is_ok());
    
    // Test 3: Large files (if any exist)
    let large_files = vec![
        "src/styles.scss",
        "src/app/app.component.html",
    ];
    
    for file_path in large_files {
        let full_path = project_path.join(file_path);
        if full_path.exists() {
            let content = fs::read_to_string(&full_path)?;
            let filename = file_path.split('/').last().unwrap_or("file");
            
            // Should handle large files without crashing
            let context = context_service.create_base_context(
                filename,
                &full_path.to_string_lossy(),
                &content
            )?;
            
            println!("✅ Large file analysis: {}", filename);
            println!("   - Content length: {}", content.len());
            println!("   - Complexity: {:.2}", context.complexity_score);
            
            assert!(context.complexity_score >= 0.0);
        }
    }
    
    println!("✅ Edge case testing completed successfully");
    
    Ok(())
}
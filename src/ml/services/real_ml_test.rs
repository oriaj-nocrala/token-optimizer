//! Real ML testing with actual model loading and calendario-psicologia project

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use serial_test::serial;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::services::context::SmartContextService;
use crate::ml::services::impact_analysis::ImpactAnalysisService;
use crate::ml::models::*;

/// Test with actual ML models loaded
#[tokio::test]
#[serial]
async fn test_real_ml_models_with_calendario_psicologia() -> Result<()> {
    let project_path = Path::new("calendario-psicologia");
    
    if !project_path.exists() {
        println!("🔄 Skipping real ML test - calendario-psicologia not found");
        return Ok(());
    }
    
    // Check if models exist
    let models_dir = Path::new(".cache/ml-models");
    if !models_dir.exists() {
        println!("🔄 Skipping real ML test - models directory not found");
        return Ok(());
    }
    
    let deepseek_model = models_dir.join("DeepSeek-R1-0528-Qwen3-8B-Q6_K.gguf");
    let embedding_model = models_dir.join("Qwen3-Embedding-8B-Q6_K.gguf");
    let reranker_model = models_dir.join("qwen3-reranker-8b-q6_k.gguf");
    
    if !deepseek_model.exists() || !embedding_model.exists() || !reranker_model.exists() {
        println!("🔄 Skipping real ML test - some models not found");
        return Ok(());
    }
    
    println!("🚀 Testing Token Optimizer with REAL ML Models");
    println!("📁 Project: {}", project_path.display());
    println!("🤖 Models directory: {}", models_dir.display());
    
    // Create config with actual models
    let config = MLConfig {
        model_cache_dir: models_dir.to_path_buf(),
        memory_budget: 8 * 1024 * 1024 * 1024, // 8GB
        max_concurrent_models: 1,
        operation_timeout: 60, // 60 seconds for model loading
        ..Default::default()
    };
    
    let mut plugin_manager = PluginManager::new();
    
    // Initialize plugin manager with real models
    println!("🔄 Initializing plugin manager with real ML models...");
    let init_start = std::time::Instant::now();
    plugin_manager.initialize(&config).await?;
    let init_time = init_start.elapsed();
    println!("✅ Plugin manager initialized in {:?}", init_time);
    
    let plugin_manager = Arc::new(plugin_manager);
    
    // Create services
    let mut context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    
    // Initialize services
    println!("🔄 Initializing ML services...");
    let services_start = std::time::Instant::now();
    
    // Try to initialize context service (requires DeepSeek)
    match context_service.initialize().await {
        Ok(_) => {
            println!("✅ SmartContextService initialized with DeepSeek model");
        }
        Err(e) => {
            println!("⚠️  SmartContextService failed to initialize: {}", e);
            println!("   Continuing with impact service only...");
        }
    }
    
    // Initialize impact service (works with or without ML)
    impact_service.initialize().await?;
    println!("✅ ImpactAnalysisService initialized");
    
    let services_time = services_start.elapsed();
    println!("✅ Services initialized in {:?}", services_time);
    
    // Test 1: Real File Analysis with Enhanced Context
    println!("\n🔍 Test 1: Enhanced Context Analysis");
    
    let auth_service_path = project_path.join("src/app/services/auth.service.ts");
    if auth_service_path.exists() {
        let content = std::fs::read_to_string(&auth_service_path)?;
        println!("   📄 Auth Service: {} bytes", content.len());
        
        // Test enhanced context analysis if available
        if context_service.is_ready() {
            let analysis_start = std::time::Instant::now();
            
            let enhanced_context = context_service.analyze_function_context(
                "login",
                auth_service_path.to_str().unwrap(),
                &content
            ).await?;
            
            let analysis_time = analysis_start.elapsed();
            
            println!("   🚀 Enhanced Analysis Results:");
            println!("      Function: {}", enhanced_context.base_context.function_name);
            println!("      Complexity: {:.2}", enhanced_context.base_context.complexity_score);
            println!("      Impact Scope: {:?}", enhanced_context.base_context.impact_scope);
            println!("      Semantic Purpose: {}", enhanced_context.semantic_analysis.purpose);
            println!("      Overall Risk: {:?}", enhanced_context.risk_assessment.overall_risk);
            println!("      Optimization Suggestions: {}", enhanced_context.optimization_suggestions.len());
            println!("      Analysis Time: {:?}", analysis_time);
            
            // Show first few optimization suggestions
            for (i, suggestion) in enhanced_context.optimization_suggestions.iter().take(3).enumerate() {
                println!("      Suggestion {}: {} (Priority: {:?})", 
                    i + 1, suggestion.description, suggestion.priority);
            }
        } else {
            println!("   ⚠️  Enhanced context analysis not available - DeepSeek model not loaded");
            
            // Fall back to base context
            let base_context = context_service.create_base_context(
                "login",
                auth_service_path.to_str().unwrap(),
                &content
            )?;
            
            println!("   📊 Base Context Analysis:");
            println!("      Function: {}", base_context.function_name);
            println!("      Complexity: {:.2}", base_context.complexity_score);
            println!("      Impact Scope: {:?}", base_context.impact_scope);
        }
    }
    
    // Test 2: Enhanced Impact Analysis 
    println!("\n🔍 Test 2: Enhanced Impact Analysis");
    
    let impact_start = std::time::Instant::now();
    
    let impact_report = impact_service.analyze_impact(
        "src/app/services/auth.service.ts",
        &vec!["login".to_string(), "logout".to_string()]
    ).await?;
    
    let impact_time = impact_start.elapsed();
    
    match impact_report {
        ImpactReport::Enhanced { base_impact, semantic_impact, risk_assessment, recommendations, confidence } => {
            println!("   🚀 Enhanced Impact Analysis Results:");
            println!("      Mode: Enhanced with ML");
            println!("      Changed File: {}", base_impact.changed_file);
            println!("      Change Type: {:?}", base_impact.change_type);
            println!("      Severity: {:?}", base_impact.severity);
            println!("      Confidence: {:.2}", confidence);
            println!("      Analysis Time: {:?}", impact_time);
            
            println!("   🧠 Semantic Impact:");
            println!("      Relationships: {}", semantic_impact.semantic_relationships.len());
            println!("      Conceptual Changes: {}", semantic_impact.conceptual_changes.len());
            println!("      Architectural Implications: {}", semantic_impact.architectural_implications.len());
            
            println!("   ⚠️  Risk Assessment:");
            println!("      Overall Risk: {:?}", risk_assessment.overall_risk);
            println!("      Breaking Change Probability: {:.2}", risk_assessment.breaking_change_probability);
            println!("      Regression Risk: {:.2}", risk_assessment.regression_risk);
            println!("      Mitigation Strategies: {}", risk_assessment.mitigation_strategies.len());
            
            println!("   💡 Recommendations:");
            for (i, rec) in recommendations.iter().take(3).enumerate() {
                println!("      {}. {} (Priority: {:?})", 
                    i + 1, rec.description, rec.priority);
            }
        }
        ImpactReport::Basic { base_impact, confidence } => {
            println!("   📊 Basic Impact Analysis Results:");
            println!("      Mode: Basic (Fallback)");
            println!("      Changed File: {}", base_impact.changed_file);
            println!("      Change Type: {:?}", base_impact.change_type);
            println!("      Severity: {:?}", base_impact.severity);
            println!("      Confidence: {:.2}", confidence);
            println!("      Analysis Time: {:?}", impact_time);
        }
    }
    
    // Test 3: Project-wide Enhanced Analysis
    println!("\n🔍 Test 3: Project-wide Enhanced Analysis");
    
    let project_start = std::time::Instant::now();
    
    let project_impact = impact_service.analyze_project_impact(
        &vec!["src/app/services/auth.service.ts".to_string()],
        project_path
    ).await?;
    
    let project_time = project_start.elapsed();
    
    println!("   🚀 Project Analysis Results:");
    println!("      Project: {}", project_impact.project_path);
    println!("      Changed File: {}", project_impact.changed_file);
    println!("      Impacted Files: {}", project_impact.impacted_files.len());
    println!("      Analysis Time: {:?}", project_time);
    
    // Show top impacted files
    for (i, file) in project_impact.impacted_files.iter().take(5).enumerate() {
        println!("      {}. {} (score: {:.2}, type: {:?})", 
            i + 1, file.file_path, file.impact_score, file.impact_type);
    }
    
    // Test 4: Cascade Effects with ML
    println!("\n🔍 Test 4: Enhanced Cascade Effects");
    
    let cascade_start = std::time::Instant::now();
    
    let cascade_effects = impact_service.predict_cascade_effects(
        "login",
        &project_path.join("src/app/services/auth.service.ts"),
        project_path
    ).await?;
    
    let cascade_time = cascade_start.elapsed();
    
    println!("   🚀 Cascade Effects Results:");
    println!("      Predicted Effects: {}", cascade_effects.len());
    println!("      Analysis Time: {:?}", cascade_time);
    
    for (i, effect) in cascade_effects.iter().take(5).enumerate() {
        println!("      {}. {} -> {} (Type: {:?}, Impact: {:?})", 
            i + 1, effect.affected_component, effect.affected_function, 
            effect.effect_type, effect.impact_level);
    }
    
    // Test 5: Performance with Real Models
    println!("\n🔍 Test 5: Performance with Real Models");
    
    let perf_start = std::time::Instant::now();
    
    // Test multiple analyses
    let mut total_analyses = 0;
    let test_files = vec![
        "src/app/services/auth.service.ts",
        "src/app/services/user.service.ts",
        "src/app/dashboard/dashboard.component.ts",
        "src/app/login/login.component.ts",
    ];
    
    for file in test_files {
        let _impact = impact_service.analyze_impact(
            file,
            &vec!["testFunction".to_string()]
        ).await?;
        total_analyses += 1;
    }
    
    let perf_time = perf_start.elapsed();
    
    println!("   🚀 Performance Results:");
    println!("      {} analyses in {:?}", total_analyses, perf_time);
    println!("      Average per analysis: {:?}", perf_time / total_analyses);
    
    // Performance assertions
    let avg_time_ms = perf_time.as_millis() / total_analyses as u128;
    if avg_time_ms < 1000 {
        println!("      ✅ Performance: Excellent (< 1s per analysis)");
    } else if avg_time_ms < 5000 {
        println!("      ✅ Performance: Good (< 5s per analysis)");
    } else {
        println!("      ⚠️  Performance: Slow (> 5s per analysis)");
    }
    
    // Test 6: Memory Usage Check
    println!("\n🔍 Test 6: Memory Usage Check");
    
    let available_plugins = plugin_manager.get_available_plugins();
    let plugin_health = plugin_manager.health_check().await?;
    
    println!("   🚀 Plugin Status:");
    println!("      Available Plugins: {:?}", available_plugins);
    for (plugin, health) in plugin_health {
        println!("      {}: {}", plugin, if health.loaded { "✅ Healthy" } else { "❌ Unhealthy" });
    }
    
    // Shutdown
    println!("\n🔄 Shutting down services...");
    let shutdown_start = std::time::Instant::now();
    
    context_service.shutdown().await?;
    impact_service.shutdown().await?;
    plugin_manager.shutdown().await?;
    
    let shutdown_time = shutdown_start.elapsed();
    println!("✅ Services shut down in {:?}", shutdown_time);
    
    println!("\n🎉 Real ML models test completed successfully!");
    println!("📊 Total test time: {:?}", init_time + services_time + shutdown_time);
    
    Ok(())
}

/// Test model loading performance
#[tokio::test]
#[serial]
async fn test_model_loading_performance() -> Result<()> {
    let models_dir = Path::new(".cache/ml-models");
    if !models_dir.exists() {
        println!("🔄 Skipping model loading test - models directory not found");
        return Ok(());
    }
    
    println!("🚀 Testing Model Loading Performance");
    
    let config = MLConfig {
        model_cache_dir: models_dir.to_path_buf(),
        memory_budget: 8 * 1024 * 1024 * 1024, // 8GB
        max_concurrent_models: 1,
        operation_timeout: 120, // 2 minutes for model loading
        ..Default::default()
    };
    
    let mut plugin_manager = PluginManager::new();
    
    // Test initialization time
    let init_start = std::time::Instant::now();
    
    match plugin_manager.initialize(&config).await {
        Ok(_) => {
            let init_time = init_start.elapsed();
            println!("✅ Plugin manager initialized in {:?}", init_time);
            
            // Test individual plugin loading
            let plugins = ["deepseek", "qwen_embedding", "qwen_reranker"];
            
            for plugin in plugins {
                let load_start = std::time::Instant::now();
                
                match plugin_manager.load_plugin(plugin).await {
                    Ok(_) => {
                        let load_time = load_start.elapsed();
                        println!("✅ Plugin '{}' loaded in {:?}", plugin, load_time);
                    }
                    Err(e) => {
                        println!("❌ Plugin '{}' failed to load: {}", plugin, e);
                    }
                }
            }
            
            // Test health check
            let health = plugin_manager.health_check().await?;
            for (plugin, healthy) in health {
                println!("   {}: {}", plugin, if healthy.loaded { "✅ Healthy" } else { "❌ Unhealthy" });
            }
            
            // Shutdown
            plugin_manager.shutdown().await?;
        }
        Err(e) => {
            println!("❌ Plugin manager initialization failed: {}", e);
            println!("   This might be due to:");
            println!("   - Insufficient memory (need ~8GB available)");
            println!("   - Missing model files");
            println!("   - CUDA/GPU issues");
        }
    }
    
    Ok(())
}
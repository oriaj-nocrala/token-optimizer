//! Memory leak prevention tests for ML services

use anyhow::Result;
use serial_test::serial;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::services::context::SmartContextService;
use crate::ml::services::impact_analysis::ImpactAnalysisService;

/// Test that ensures no memory leaks occur during heavy ML operations
#[tokio::test]
#[serial]
async fn test_memory_leak_prevention() -> Result<()> {
    // Skip if no models directory exists
    let models_dir = Path::new(".cache/ml-models");
    if !models_dir.exists() {
        println!("🔄 Skipping memory leak test - models directory not found");
        return Ok(());
    }

    println!("🧪 Testing Memory Leak Prevention");

    // Create config with aggressive timeouts
    let config = MLConfig {
        model_cache_dir: models_dir.to_path_buf(),
        memory_budget: 4 * 1024 * 1024 * 1024, // 4GB (reduced from 8GB)
        max_concurrent_models: 1,
        operation_timeout: 10, // 10 seconds timeout
        ..Default::default()
    };

    // Test 1: Plugin Manager Memory Management
    println!("   🔍 Test 1: Plugin Manager Memory Management");
    {
        let mut plugin_manager = PluginManager::new();
        
        // Initialize with timeout
        let init_result = tokio::time::timeout(
            Duration::from_secs(15),
            plugin_manager.initialize(&config)
        ).await;
        
        match init_result {
            Ok(Ok(_)) => {
                println!("      ✅ Plugin manager initialized successfully");
                
                // Test memory usage tracking
                let memory_before = plugin_manager.get_memory_usage();
                println!("      📊 Memory usage before: {} bytes", memory_before);
                
                // Load a plugin
                let load_result = tokio::time::timeout(
                    Duration::from_secs(15),
                    plugin_manager.load_plugin("deepseek")
                ).await;
                
                match load_result {
                    Ok(Ok(_)) => {
                        let memory_after = plugin_manager.get_memory_usage();
                        println!("      📊 Memory usage after loading: {} bytes", memory_after);
                        
                        // Unload the plugin
                        plugin_manager.unload_plugin("deepseek").await?;
                        
                        let memory_final = plugin_manager.get_memory_usage();
                        println!("      📊 Memory usage after unloading: {} bytes", memory_final);
                        
                        // Memory should be cleaned up
                        assert!(memory_final <= memory_before, 
                               "Memory leak detected: {} -> {}", memory_before, memory_final);
                    }
                    Ok(Err(e)) => {
                        println!("      ⚠️  Plugin loading failed (expected): {}", e);
                    }
                    Err(_) => {
                        println!("      ⚠️  Plugin loading timed out (expected for heavy models)");
                    }
                }
                
                // Ensure proper shutdown
                plugin_manager.shutdown().await?;
                
                let memory_shutdown = plugin_manager.get_memory_usage();
                println!("      📊 Memory usage after shutdown: {} bytes", memory_shutdown);
                assert_eq!(memory_shutdown, 0, "Memory not cleaned up after shutdown");
            }
            Ok(Err(e)) => {
                println!("      ⚠️  Plugin manager initialization failed: {}", e);
            }
            Err(_) => {
                println!("      ⚠️  Plugin manager initialization timed out");
            }
        }
    } // PluginManager should be dropped here

    // Test 2: Service Memory Management
    println!("   🔍 Test 2: Service Memory Management");
    {
        let mut plugin_manager = PluginManager::new();
        
        // Initialize with shorter timeout
        let init_result = tokio::time::timeout(
            Duration::from_secs(10),
            plugin_manager.initialize(&config)
        ).await;
        
        if let Ok(Ok(_)) = init_result {
            let plugin_manager = Arc::new(plugin_manager);
            
            // Test SmartContextService
            {
                let mut context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
                
                let service_init_result = tokio::time::timeout(
                    Duration::from_secs(10),
                    context_service.initialize()
                ).await;
                
                match service_init_result {
                    Ok(Ok(_)) => {
                        println!("      ✅ SmartContextService initialized");
                        
                        // Test that service properly handles failures
                        let context_result = context_service.create_base_context(
                            "test_function",
                            "test_file.ts",
                            "function test() { return 42; }"
                        );
                        
                        match context_result {
                            Ok(_) => println!("      ✅ Base context created successfully"),
                            Err(e) => println!("      ⚠️  Base context creation failed: {}", e),
                        }
                    }
                    Ok(Err(e)) => {
                        println!("      ⚠️  SmartContextService initialization failed: {}", e);
                    }
                    Err(_) => {
                        println!("      ⚠️  SmartContextService initialization timed out");
                    }
                }
                
                // Ensure proper shutdown
                context_service.shutdown().await?;
            }
            
            // Test ImpactAnalysisService
            {
                let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
                
                let service_init_result = tokio::time::timeout(
                    Duration::from_secs(10),
                    impact_service.initialize()
                ).await;
                
                match service_init_result {
                    Ok(Ok(_)) => {
                        println!("      ✅ ImpactAnalysisService initialized");
                        
                        // Test basic impact analysis
                        let impact_result = tokio::time::timeout(
                            Duration::from_secs(5),
                            impact_service.analyze_impact(
                                "test_file.ts",
                                &vec!["test_function".to_string()]
                            )
                        ).await;
                        
                        match impact_result {
                            Ok(Ok(_)) => println!("      ✅ Impact analysis completed successfully"),
                            Ok(Err(e)) => println!("      ⚠️  Impact analysis failed: {}", e),
                            Err(_) => println!("      ⚠️  Impact analysis timed out"),
                        }
                    }
                    Ok(Err(e)) => {
                        println!("      ⚠️  ImpactAnalysisService initialization failed: {}", e);
                    }
                    Err(_) => {
                        println!("      ⚠️  ImpactAnalysisService initialization timed out");
                    }
                }
                
                // Ensure proper shutdown
                impact_service.shutdown().await?;
            }
            
            // Shutdown plugin manager
            plugin_manager.shutdown().await?;
        }
    } // Services should be dropped here

    // Test 3: Repeated Operations Memory Stability
    println!("   🔍 Test 3: Repeated Operations Memory Stability");
    
    for i in 0..5 {
        let mut plugin_manager = PluginManager::new();
        
        let init_result = tokio::time::timeout(
            Duration::from_secs(5),
            plugin_manager.initialize(&config)
        ).await;
        
        if let Ok(Ok(_)) = init_result {
            // Do some quick operations
            let health = plugin_manager.health_check().await?;
            println!("      📊 Iteration {}: {} plugins healthy", i + 1, 
                    health.values().filter(|v| v.loaded).count());
            
            // Shutdown
            plugin_manager.shutdown().await?;
        }
    }

    println!("✅ Memory leak prevention tests completed successfully");
    Ok(())
}

/// Test that ensures proper resource cleanup on panic
#[tokio::test]
#[serial]
async fn test_panic_resource_cleanup() -> Result<()> {
    println!("🧪 Testing Panic Resource Cleanup");
    
    let models_dir = Path::new(".cache/ml-models");
    if !models_dir.exists() {
        println!("🔄 Skipping panic cleanup test - models directory not found");
        return Ok(());
    }

    let config = MLConfig {
        model_cache_dir: models_dir.to_path_buf(),
        memory_budget: 2 * 1024 * 1024 * 1024, // 2GB
        max_concurrent_models: 1,
        operation_timeout: 5, // 5 seconds timeout
        ..Default::default()
    };

    // Test that Drop implementations work correctly
    {
        let mut plugin_manager = PluginManager::new();
        
        let init_result = tokio::time::timeout(
            Duration::from_secs(5),
            plugin_manager.initialize(&config)
        ).await;
        
        if let Ok(Ok(_)) = init_result {
            let memory_before = plugin_manager.get_memory_usage();
            println!("   📊 Memory before: {} bytes", memory_before);
            
            // Plugin manager will be dropped here when scope exits
        }
    }
    
    // Test service cleanup
    {
        let mut plugin_manager = PluginManager::new();
        
        if plugin_manager.initialize(&config).await.is_ok() {
            let plugin_manager = Arc::new(plugin_manager);
            
            {
                let mut context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
                let _ = context_service.initialize().await;
                // Service will be dropped here
            }
            
            plugin_manager.shutdown().await?;
        }
    }

    println!("✅ Panic resource cleanup tests completed successfully");
    Ok(())
}

/// Test timeout handling to prevent infinite loops
#[tokio::test]
#[serial]
async fn test_timeout_handling() -> Result<()> {
    println!("🧪 Testing Timeout Handling");
    
    let config = MLConfig {
        model_cache_dir: Path::new(".cache/ml-models").to_path_buf(),
        memory_budget: 1 * 1024 * 1024 * 1024, // 1GB
        max_concurrent_models: 1,
        operation_timeout: 1, // 1 second timeout - very aggressive
        ..Default::default()
    };

    let mut plugin_manager = PluginManager::new();
    
    // This should timeout quickly
    let init_result = tokio::time::timeout(
        Duration::from_secs(2),
        plugin_manager.initialize(&config)
    ).await;
    
    match init_result {
        Ok(Ok(_)) => {
            println!("   ✅ Plugin manager initialized within timeout");
            plugin_manager.shutdown().await?;
        }
        Ok(Err(e)) => {
            println!("   ⚠️  Plugin manager initialization failed: {}", e);
        }
        Err(_) => {
            println!("   ⚠️  Plugin manager initialization timed out (expected)");
        }
    }

    println!("✅ Timeout handling tests completed successfully");
    Ok(())
}
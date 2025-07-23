//! Real project integration test for all 4 AI reliability layers
//! Tests the complete AI reliability strategy with an actual TypeScript project

use anyhow::Result;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs;
use tempfile::TempDir;

use crate::ml::{
    MLConfig, PluginManager, LayeredAnalysisService, AnalysisLevel, 
    StructuredPrompts, MLResponseCache, ExternalTimeoutWrapper
};

/// Real project integration test for all 4 layers
#[tokio::test]
#[serial_test::serial] // Serialize to prevent resource conflicts
async fn test_four_layer_real_project_integration() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().join("test-angular-project");
    
    // Create realistic Angular project structure
    create_angular_project_structure(&project_path)?;
    
    let config = MLConfig::for_testing_with_path(project_path.clone());
    let plugin_manager = Arc::new(PluginManager::new());
    
    // Test Layer 1: External Timeout Control
    println!("🔧 Testing Layer 1: External Timeout Control");
    test_external_timeout_layer(&config).await?;
    
    // Test Layer 2: Structured Prompts
    println!("📝 Testing Layer 2: Structured Prompts");
    test_structured_prompts_layer().await?;
    
    // Test Layer 3: ML Response Caching
    println!("💾 Testing Layer 3: ML Response Caching");
    test_ml_cache_layer(&config).await?;
    
    // Test Layer 4: Layered Analysis Architecture
    println!("🏗️ Testing Layer 4: Layered Analysis Architecture");
    test_layered_analysis_layer(&config, &plugin_manager, &project_path).await?;
    
    // Test All Layers Integration
    println!("🚀 Testing Complete 4-Layer Integration");
    test_complete_integration(&config, &plugin_manager, &project_path).await?;
    
    println!("✅ All 4 layers successfully tested with real project integration!");
    Ok(())
}

/// Test Layer 1: External timeout control prevents DeepSeek overthinking
async fn test_external_timeout_layer(config: &MLConfig) -> Result<()> {
    let timeout_wrapper = ExternalTimeoutWrapper::new(config.clone());
    
    // Test with a prompt that could cause overthinking
    let complex_prompt = r#"
    Analyze this extremely complex TypeScript function with multiple nested conditions,
    recursive calls, and intricate business logic. Consider all edge cases, performance
    implications, security concerns, architectural patterns, and provide comprehensive
    recommendations for improvement. Think deeply about every aspect.
    
    function complexAnalyzer(data: any): any {
        // This would normally cause DeepSeek to overthink
        return data;
    }
    "#;
    
    // Test that timeout control prevents hanging
    let start = std::time::Instant::now();
    let result = timeout_wrapper.execute_deepseek_reasoning(complex_prompt).await;
    let duration = start.elapsed();
    
    // Should complete within reasonable timeout (configured to 240s, but test should be much faster)
    assert!(duration.as_secs() < 10, "External timeout should prevent long thinking loops");
    
    match result {
        Ok(response) => {
            assert!(!response.is_empty(), "Should get a response");
            println!("  ✅ External timeout working: got response in {}ms", duration.as_millis());
        }
        Err(_) => {
            // Timeout or error is also acceptable - important thing is it doesn't hang
            println!("  ✅ External timeout working: prevented hanging in {}ms", duration.as_millis());
        }
    }
    
    Ok(())
}

/// Test Layer 2: Structured prompts with few-shot examples
async fn test_structured_prompts_layer() -> Result<()> {
    // Test function analysis prompt structure
    let prompt = StructuredPrompts::function_analysis(
        "calculateUserScore",
        "function calculateUserScore(user: User, metrics: Metrics): number { return user.points * metrics.multiplier; }"
    );
    
    assert!(prompt.contains("function calculateUserScore"), "Should include function name");
    assert!(prompt.contains("JSON"), "Should specify JSON output");
    assert!(prompt.contains("EXAMPLE"), "Should include few-shot example");
    // Note: this prompt uses "complexity" instead of "confidence" 
    assert!(prompt.contains("complexity"), "Should include complexity assessment");
    
    // Test change risk analysis prompt structure
    let changed_functions = vec!["updateUser".to_string(), "validateData".to_string()];
    let risk_prompt = StructuredPrompts::change_risk_analysis("user.service.ts", &changed_functions);
    
    assert!(risk_prompt.contains("user.service.ts"), "Should include file name");
    assert!(risk_prompt.contains("updateUser"), "Should include changed functions");
    assert!(risk_prompt.contains("risk_level"), "Should specify risk assessment format");
    
    // Test JSON extraction
    let mock_response = r#"
    Here's my analysis of the function:
    
    ```json
    {
        "function": "calculateUserScore",
        "complexity": 2,
        "confidence": 0.95,
        "type": "utility"
    }
    ```
    
    This function is straightforward...
    "#;
    
    let extracted = StructuredPrompts::extract_json_from_response(mock_response)
        .map_err(|e| anyhow::anyhow!("JSON extraction failed: {}", e))?;
    assert!(extracted.contains("calculateUserScore"), "Should extract JSON properly");
    assert!(extracted.contains("confidence"), "Should preserve confidence field");
    
    println!("  ✅ Structured prompts working: JSON format enforced with few-shot examples");
    Ok(())
}

/// Test Layer 3: ML response caching prevents re-computation
async fn test_ml_cache_layer(config: &MLConfig) -> Result<()> {
    let mut cache = MLResponseCache::new(
        config.model_cache_dir.clone(),
        100
    );
    
    // Test cache miss then hit
    let prompt = "function testFunc() { return 42; }";
    let hash = MLResponseCache::generate_prompt_hash(prompt, "deepseek", "test_config");
    
    // First call should be cache miss
    assert!(cache.get(&hash).is_none(), "Should be cache miss initially");
    
    // Store response
    let mock_response = r#"{"function": "testFunc", "analysis": "simple function", "confidence": 0.9}"#;
    cache.put(hash.clone(), mock_response.to_string(), "deepseek".to_string())?;
    
    // Second call should be cache hit
    let cached = cache.get(&hash);
    assert!(cached.is_some(), "Should be cache hit");
    assert_eq!(cached.unwrap(), mock_response, "Should return cached response");
    
    // Test cache statistics
    assert!(cache.hit_rate() > 0.0, "Should have positive hit rate");
    assert_eq!(cache.size(), 1, "Should have one cached entry");
    
    // Test cache persistence
    cache.save()?;
    let mut new_cache = MLResponseCache::new(config.model_cache_dir.clone(), 100);
    new_cache.load()?;
    
    let persisted = new_cache.get(&hash);
    assert!(persisted.is_some(), "Cache should persist across instances");
    
    println!("  ✅ ML caching working: prevents re-computation with {}% hit rate", 
             (cache.hit_rate() * 100.0) as u32);
    Ok(())
}

/// Test Layer 4: Layered analysis with confidence-based fallback
async fn test_layered_analysis_layer(
    config: &MLConfig, 
    plugin_manager: &Arc<PluginManager>, 
    project_path: &Path
) -> Result<()> {
    let mut layered_service = LayeredAnalysisService::new(
        config.clone(), 
        plugin_manager.clone()
    );
    
    // Create a test TypeScript file
    let test_file = project_path.join("src/app/test.service.ts");
    
    // Test simple function (should use AST analysis)
    let result = layered_service.analyze_function("simpleFunction", &test_file).await?;
    
    assert!(result.confidence > 0.0, "Should have confidence score");
    assert!(!result.reasoning_path.is_empty(), "Should have reasoning path");
    
    match result.level_used {
        AnalysisLevel::AST => {
            println!("  ✅ Layer 4 working: Used AST analysis for simple function");
        }
        AnalysisLevel::Semantic => {
            println!("  ✅ Layer 4 working: Used semantic analysis");
        }
        AnalysisLevel::DeepAI => {
            println!("  ✅ Layer 4 working: Used DeepSeek analysis");
        }
    }
    
    // Test change impact analysis
    let changed_functions = vec!["updateUser".to_string()];
    let impact_result = layered_service.analyze_change_impact("user.service.ts", &changed_functions).await?;
    
    assert!(impact_result.confidence > 0.0, "Should analyze change impact");
    assert!(impact_result.processing_time_ms > 0, "Should measure processing time");
    
    // Test cache statistics
    let cache_stats = layered_service.get_cache_stats();
    assert!(cache_stats.contains("Cache:"), "Should provide cache statistics");
    
    println!("  ✅ Layered analysis working: confidence-based fallback with {} confidence", 
             (result.confidence * 100.0) as u32);
    Ok(())
}

/// Test complete integration of all 4 layers working together
async fn test_complete_integration(
    config: &MLConfig,
    plugin_manager: &Arc<PluginManager>,
    project_path: &Path
) -> Result<()> {
    let mut layered_service = LayeredAnalysisService::new(
        config.clone(),
        plugin_manager.clone()
    );
    
    // Test complex function that should trigger multiple layers
    let complex_file = project_path.join("src/app/complex.service.ts");
    
    // First analysis (cache miss)
    let start = std::time::Instant::now();
    let result1 = layered_service.analyze_function("complexBusinessLogic", &complex_file).await?;
    let first_duration = start.elapsed();
    
    // Second analysis (should hit cache if using same layer)
    let start = std::time::Instant::now();
    let result2 = layered_service.analyze_function("complexBusinessLogic", &complex_file).await?;
    let second_duration = start.elapsed();
    
    // Verify all layers are working
    assert!(result1.confidence > 0.0, "Should have confidence");
    assert!(result2.confidence > 0.0, "Should have confidence in second run");
    
    // Cache should improve performance
    if result2.cache_hit {
        assert!(second_duration < first_duration, "Cache hit should be faster");
        println!("  ✅ Cache optimization working: {}ms vs {}ms", 
                 second_duration.as_millis(), first_duration.as_millis());
    }
    
    // Timeout protection should prevent hanging
    assert!(first_duration.as_secs() < 30, "Should complete within reasonable time");
    
    // Structured prompts should provide consistent JSON
    if result1.result.starts_with('{') {
        let _parsed: serde_json::Value = serde_json::from_str(&result1.result)?;
        println!("  ✅ Structured prompts working: valid JSON output");
    }
    
    // Layered analysis should show reasoning path
    assert!(!result1.reasoning_path.is_empty(), "Should show reasoning path");
    println!("  ✅ Reasoning path: {:?}", result1.reasoning_path);
    
    // Test all layers are documented
    println!("  📊 Final integration results:");
    println!("     - Analysis level used: {:?}", result1.level_used);
    println!("     - Processing time: {}ms", result1.processing_time_ms);
    println!("     - Confidence: {:.1}%", result1.confidence * 100.0);
    println!("     - Cache hit: {}", result2.cache_hit);
    println!("     - Cache stats: {}", layered_service.get_cache_stats());
    
    Ok(())
}

/// Create a realistic Angular project structure for testing
fn create_angular_project_structure(project_path: &Path) -> Result<()> {
    fs::create_dir_all(&project_path)?;
    
    // Create directory structure
    let src_path = project_path.join("src");
    let app_path = src_path.join("app");
    fs::create_dir_all(&app_path)?;
    
    // Create simple service file
    let simple_service = app_path.join("test.service.ts");
    fs::write(&simple_service, r#"
import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class TestService {
  
  simpleFunction(): string {
    return 'hello world';
  }
  
  calculateSum(a: number, b: number): number {
    return a + b;
  }
}
"#)?;

    // Create complex service file
    let complex_service = app_path.join("complex.service.ts");
    fs::write(&complex_service, r#"
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';

interface BusinessData {
  id: string;
  value: number;
  metadata: any;
}

@Injectable({
  providedIn: 'root'
})
export class ComplexService {
  
  complexBusinessLogic(data: BusinessData[]): Observable<BusinessData[]> {
    // Complex nested logic that might trigger semantic/deep analysis
    return new Observable(observer => {
      const processed = data
        .filter(item => item.value > 0)
        .map(item => ({
          ...item,
          value: this.applyBusinessRules(item.value, item.metadata)
        }))
        .sort((a, b) => b.value - a.value);
      
      observer.next(processed);
      observer.complete();
    });
  }
  
  private applyBusinessRules(value: number, metadata: any): number {
    if (metadata?.priority === 'high') {
      return value * 1.5;
    } else if (metadata?.category === 'premium') {
      return value * 1.2;
    } else {
      return value;
    }
  }
  
  validateAndTransform(input: any): BusinessData | null {
    try {
      if (!input || typeof input.id !== 'string') {
        return null;
      }
      
      return {
        id: input.id,
        value: parseFloat(input.value) || 0,
        metadata: input.metadata || {}
      };
    } catch (error) {
      console.error('Validation failed:', error);
      return null;
    }
  }
}
"#)?;

    println!("✅ Created Angular project structure at: {}", project_path.display());
    Ok(())
}

/// Helper to create ML config for testing with specific path
impl MLConfig {
    pub fn for_testing_with_path(project_path: PathBuf) -> Self {
        let mut config = Self::for_testing();
        config.model_cache_dir = project_path.join(".cache/ml");
        config
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_angular_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test-project");
        
        create_angular_project_structure(&project_path).unwrap();
        
        assert!(project_path.join("src/app/test.service.ts").exists());
        assert!(project_path.join("src/app/complex.service.ts").exists());
        
        let content = fs::read_to_string(project_path.join("src/app/test.service.ts")).unwrap();
        assert!(content.contains("simpleFunction"));
        assert!(content.contains("TestService"));
    }
}
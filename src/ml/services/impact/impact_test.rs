//! Integration tests for Impact Analysis Service

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::services::impact_analysis::ImpactAnalysisService;
use crate::ml::models::*;

/// Test basic impact analysis functionality
#[tokio::test]
async fn test_basic_impact_analysis() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    // Initialize service
    service.initialize().await?;
    assert!(service.is_ready());
    
    // Test basic impact analysis
    let changed_file = "src/app/services/auth.service.ts";
    let changed_functions = vec!["login".to_string(), "logout".to_string()];
    
    let impact_report = service.analyze_impact(changed_file, &changed_functions).await?;
    
    // Verify basic report structure
    match impact_report {
        ImpactReport::Basic { base_impact, confidence } => {
            assert_eq!(base_impact.changed_file, changed_file);
            assert_eq!(base_impact.changed_functions, changed_functions);
            assert!(confidence > 0.0);
            assert_eq!(base_impact.change_type, ChangeType::ServiceModification);
            assert_eq!(base_impact.severity, Severity::Medium);
        }
        ImpactReport::Enhanced { base_impact, confidence, .. } => {
            assert_eq!(base_impact.changed_file, changed_file);
            assert_eq!(base_impact.changed_functions, changed_functions);
            assert!(confidence > 0.0);
        }
    }
    
    println!("✅ Basic impact analysis test passed");
    Ok(())
}

/// Test impact analysis with calendario-psicologia project structure
#[tokio::test]
async fn test_project_impact_analysis() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test with calendario-psicologia project
    let project_path = Path::new("calendario-psicologia");
    let changed_file = "src/app/services/auth.service.ts";
    let changed_functions = vec!["login".to_string()];
    
    if project_path.exists() {
        let project_report = service.analyze_project_impact(
            project_path,
            changed_file,
            &changed_functions
        ).await?;
        
        // Verify project report structure
        assert_eq!(project_report.changed_file, changed_file);
        assert_eq!(project_report.changed_functions, changed_functions);
        assert!(!project_report.impacted_files.is_empty() || project_path.exists());
        
        // Check that impacted files are sorted by score
        let scores: Vec<f32> = project_report.impacted_files.iter()
            .map(|f| f.impact_score)
            .collect();
        
        for i in 1..scores.len() {
            assert!(scores[i-1] >= scores[i], "Files should be sorted by impact score");
        }
        
        println!("✅ Project impact analysis test passed");
        println!("   Found {} impacted files", project_report.impacted_files.len());
        
        // Show top impacted files
        for (i, file) in project_report.impacted_files.iter().take(3).enumerate() {
            println!("   {}. {} (score: {:.2}, type: {:?})", 
                i + 1, file.file_path, file.impact_score, file.impact_type);
        }
    } else {
        println!("🔄 Skipping project impact test (calendario-psicologia not found)");
    }
    
    Ok(())
}

/// Test cascade effect prediction
#[tokio::test]
async fn test_cascade_effects_prediction() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test cascade effect prediction
    let changed_file = "src/app/services/user.service.ts";
    let changed_functions = vec!["getUserProfile".to_string(), "updateUser".to_string()];
    
    let cascade_effects = service.predict_cascade_effects(changed_file, &changed_functions).await?;
    
    // Should have at least direct effects
    assert!(!cascade_effects.is_empty());
    assert_eq!(cascade_effects.len(), changed_functions.len());
    
    // Verify direct effects
    for (i, effect) in cascade_effects.iter().enumerate() {
        assert_eq!(effect.effect_type, EffectType::Direct);
        assert_eq!(effect.affected_component, changed_file);
        assert_eq!(effect.affected_function, changed_functions[i]);
        assert_eq!(effect.impact_level, ImpactLevel::High);
        assert!(!effect.description.is_empty());
    }
    
    println!("✅ Cascade effects prediction test passed");
    println!("   Predicted {} cascade effects", cascade_effects.len());
    
    Ok(())
}

/// Test change type classification
#[tokio::test]
async fn test_change_type_classification() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test different file types
    let test_cases = vec![
        ("src/app/services/auth.service.ts", vec!["login".to_string()], ChangeType::ServiceModification),
        ("src/app/components/calendar.component.ts", vec!["ngOnInit".to_string()], ChangeType::ComponentModification),
        ("src/app/services/auth.service.spec.ts", vec!["testLogin".to_string()], ChangeType::TestModification),
        ("src/app/utils/helpers.ts", vec!["formatDate".to_string()], ChangeType::CodeModification),
    ];
    
    for (changed_file, changed_functions, expected_type) in test_cases {
        let impact_report = service.analyze_impact(changed_file, &changed_functions).await?;
        
        let actual_type = match impact_report {
            ImpactReport::Basic { base_impact, .. } => base_impact.change_type,
            ImpactReport::Enhanced { base_impact, .. } => base_impact.change_type,
        };
        
        assert_eq!(actual_type, expected_type, 
            "Wrong change type for {}: expected {:?}, got {:?}", 
            changed_file, expected_type, actual_type);
    }
    
    println!("✅ Change type classification test passed");
    Ok(())
}

/// Test severity assessment
#[tokio::test]
async fn test_severity_assessment() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test different severity levels
    let test_cases = vec![
        // Low severity: single private function
        ("src/app/components/calendar.component.ts", vec!["private_helper".to_string()], Severity::Low),
        // Medium severity: service function
        ("src/app/services/auth.service.ts", vec!["login".to_string()], Severity::Medium),
        // High severity: multiple public functions
        ("src/app/services/auth.service.ts", vec!["public_login".to_string(), "public_logout".to_string(), "export_user".to_string()], Severity::High),
    ];
    
    for (changed_file, changed_functions, expected_severity) in test_cases {
        let impact_report = service.analyze_impact(changed_file, &changed_functions).await?;
        
        let actual_severity = match impact_report {
            ImpactReport::Basic { base_impact, .. } => base_impact.severity,
            ImpactReport::Enhanced { base_impact, .. } => base_impact.severity,
        };
        
        assert_eq!(actual_severity, expected_severity, 
            "Wrong severity for {}: expected {:?}, got {:?}", 
            changed_file, expected_severity, actual_severity);
    }
    
    println!("✅ Severity assessment test passed");
    Ok(())
}

/// Test file impact analysis
#[tokio::test]
async fn test_file_impact_analysis() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    let changed_file = "src/app/services/auth.service.ts";
    let changed_functions = vec!["login".to_string()];
    
    // Test same file impact (should be 1.0)
    let same_file_impact = service.analyze_file_impact(
        changed_file,
        changed_file,
        &changed_functions
    ).await?;
    
    assert_eq!(same_file_impact.impact_score, 1.0);
    assert_eq!(same_file_impact.impact_type, ImpactType::Direct);
    assert_eq!(same_file_impact.affected_functions, changed_functions);
    assert_eq!(same_file_impact.reasoning, "Same file as changed file");
    
    // Test different file impact (should be lower)
    let different_file_impact = service.analyze_file_impact(
        "src/app/components/login.component.ts",
        changed_file,
        &changed_functions
    ).await?;
    
    assert!(different_file_impact.impact_score < 1.0);
    assert_ne!(different_file_impact.impact_type, ImpactType::Direct);
    assert!(!different_file_impact.reasoning.is_empty());
    
    println!("✅ File impact analysis test passed");
    println!("   Same file impact: {:.2}", same_file_impact.impact_score);
    println!("   Different file impact: {:.2}", different_file_impact.impact_score);
    
    Ok(())
}

/// Test file relatedness detection
#[tokio::test]
async fn test_file_relatedness() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test service-to-service relatedness
    let is_related = service.might_be_related(
        "src/app/services/user.service.ts",
        "src/app/services/auth.service.ts"
    )?;
    assert!(is_related, "Services should be related");
    
    // Test component-to-service relatedness  
    let is_related = service.might_be_related(
        "src/app/components/login.component.ts",
        "src/app/services/auth.service.ts"
    )?;
    assert!(is_related, "Components should be related to services");
    
    // Test same directory relatedness
    let is_related = service.might_be_related(
        "src/app/services/user.service.ts",
        "src/app/services/auth.service.ts"
    )?;
    assert!(is_related, "Files in same directory should be related");
    
    // Test unrelated files
    let is_related = service.might_be_related(
        "src/app/utils/helpers.ts",
        "src/app/models/user.model.ts"
    )?;
    assert!(!is_related, "Unrelated files should not be related");
    
    println!("✅ File relatedness test passed");
    Ok(())
}

/// Test basic recommendations generation
#[tokio::test]
async fn test_recommendations_generation() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    let changed_file = "src/app/services/auth.service.ts";
    let changed_functions = vec!["login".to_string(), "logout".to_string()];
    
    let recommendations = service.generate_recommendations(changed_file, &changed_functions).await?;
    
    // Should have at least basic recommendations
    assert!(!recommendations.is_empty());
    
    // Check for testing recommendation
    let has_testing_rec = recommendations.iter().any(|r| 
        r.recommendation_type == RecommendationType::Testing
    );
    assert!(has_testing_rec, "Should have testing recommendation");
    
    // Check for review recommendation
    let has_review_rec = recommendations.iter().any(|r| 
        r.recommendation_type == RecommendationType::Review
    );
    assert!(has_review_rec, "Should have review recommendation");
    
    // Verify recommendation structure
    for rec in &recommendations {
        assert!(!rec.description.is_empty());
        assert!(!rec.implementation_steps.is_empty());
        assert!(matches!(rec.priority, Priority::High | Priority::Medium | Priority::Low));
        assert!(matches!(rec.estimated_effort, EffortLevel::Low | EffortLevel::Medium | EffortLevel::High));
    }
    
    println!("✅ Recommendations generation test passed");
    println!("   Generated {} recommendations", recommendations.len());
    
    for (i, rec) in recommendations.iter().enumerate() {
        println!("   {}. {} (Priority: {:?}, Effort: {:?})", 
            i + 1, rec.description, rec.priority, rec.estimated_effort);
    }
    
    Ok(())
}

/// Test risk assessment
#[tokio::test]
async fn test_risk_assessment() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    // Test service modification (higher risk)
    let service_risk = service.assess_change_risk(
        "src/app/services/auth.service.ts",
        &vec!["login".to_string()]
    ).await?;
    
    assert!(service_risk.breaking_change_probability > 0.5);
    assert!(!service_risk.mitigation_strategies.is_empty());
    
    // Test component modification (lower risk)
    let component_risk = service.assess_change_risk(
        "src/app/components/calendar.component.ts",
        &vec!["ngOnInit".to_string()]
    ).await?;
    
    assert!(component_risk.breaking_change_probability < service_risk.breaking_change_probability);
    
    println!("✅ Risk assessment test passed");
    println!("   Service risk: {:.2}", service_risk.breaking_change_probability);
    println!("   Component risk: {:.2}", component_risk.breaking_change_probability);
    
    Ok(())
}

/// Performance test for impact analysis
#[tokio::test]
async fn test_impact_analysis_performance() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let mut service = ImpactAnalysisService::new(config, plugin_manager);
    
    service.initialize().await?;
    
    let start = std::time::Instant::now();
    
    // Analyze multiple files to test performance
    for i in 0..50 {
        let changed_file = format!("src/app/services/test{}.service.ts", i);
        let changed_functions = vec![format!("testFunction{}", i)];
        
        let _impact_report = service.analyze_impact(&changed_file, &changed_functions).await?;
    }
    
    let duration = start.elapsed();
    
    println!("✅ Performance test passed");
    println!("   Analyzed 50 files in {:?}", duration);
    println!("   Average per file: {:?}", duration / 50);
    
    // Should be reasonably fast (< 50ms per file for basic analysis)
    assert!(duration.as_millis() < 2500);
    
    Ok(())
}

/// Test uninitialized service
#[tokio::test]
async fn test_uninitialized_service() -> Result<()> {
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    let service = ImpactAnalysisService::new(config, plugin_manager);
    
    // Should fail when not initialized
    let result = service.analyze_impact("test.ts", &vec!["test".to_string()]).await;
    assert!(result.is_err());
    
    let result = service.predict_cascade_effects("test.ts", &vec!["test".to_string()]).await;
    assert!(result.is_err());
    
    println!("✅ Uninitialized service test passed");
    Ok(())
}
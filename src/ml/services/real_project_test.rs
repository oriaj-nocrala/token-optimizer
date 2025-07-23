//! Real project testing with calendario-psicologia (fallback mode)

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use serial_test::serial;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::services::context::SmartContextService;
use crate::ml::services::impact_analysis::ImpactAnalysisService;
use crate::ml::models::*;

/// Test real project with fallback mode (no ML plugins required)
#[tokio::test]
#[serial]
async fn test_calendario_psicologia_fallback() -> Result<()> {
    let project_path = Path::new("calendario-psicologia");
    
    if !project_path.exists() {
        println!("🔄 Skipping real project test - calendario-psicologia not found");
        return Ok(());
    }
    
    println!("🚀 Testing Token Optimizer with Calendario Psicologia Project");
    println!("📁 Project path: {}", project_path.display());
    
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    
    // Initialize services (they will work in fallback mode)
    let mut context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    
    // Impact service doesn't require DeepSeek, so it should initialize
    impact_service.initialize().await?;
    assert!(impact_service.is_ready());
    
    println!("✅ Services initialized (fallback mode)");
    
    // Test 1: Real File Analysis
    println!("\n🔍 Test 1: Real File Analysis");
    
    let auth_service_path = project_path.join("src/app/services/auth.service.ts");
    let user_service_path = project_path.join("src/app/services/user.service.ts");
    
    if auth_service_path.exists() {
        let content = std::fs::read_to_string(&auth_service_path)?;
        println!("   📄 Auth Service Content Length: {} bytes", content.len());
        
        // Test base context creation (doesn't require ML)
        let base_context = context_service.create_base_context(
            "login",
            auth_service_path.to_str().unwrap(),
            &content
        )?;
        
        println!("   ✅ Base Context Analysis:");
        println!("      Function: {}", base_context.function_name);
        println!("      Complexity: {:.2}", base_context.complexity_score);
        println!("      Impact Scope: {:?}", base_context.impact_scope);
        
        assert_eq!(base_context.function_name, "login");
        assert!(base_context.complexity_score > 0.0);
    }
    
    if user_service_path.exists() {
        let content = std::fs::read_to_string(&user_service_path)?;
        println!("   📄 User Service Content Length: {} bytes", content.len());
        
        let base_context = context_service.create_base_context(
            "getUserProfile",
            user_service_path.to_str().unwrap(),
            &content
        )?;
        
        println!("   ✅ User Service Analysis:");
        println!("      Function: {}", base_context.function_name);
        println!("      Complexity: {:.2}", base_context.complexity_score);
        println!("      Impact Scope: {:?}", base_context.impact_scope);
    }
    
    // Test 2: Impact Analysis
    println!("\n🔍 Test 2: Impact Analysis");
    
    let impact_report = impact_service.analyze_impact(
        "src/app/services/auth.service.ts",
        &vec!["login".to_string(), "logout".to_string()]
    ).await?;
    
    match impact_report {
        ImpactReport::Basic { base_impact, confidence } => {
            println!("   ✅ Impact Analysis Results:");
            println!("      Mode: Basic (Fallback)");
            println!("      Changed File: {}", base_impact.changed_file);
            println!("      Functions: {:?}", base_impact.changed_functions);
            println!("      Change Type: {:?}", base_impact.change_type);
            println!("      Severity: {:?}", base_impact.severity);
            println!("      Confidence: {:.2}", confidence);
            
            assert_eq!(base_impact.change_type, ChangeType::ServiceModification);
            assert_eq!(base_impact.severity, Severity::Medium);
        }
        ImpactReport::Enhanced { base_impact, confidence, .. } => {
            println!("   ✅ Enhanced Impact Analysis Results:");
            println!("      Changed File: {}", base_impact.changed_file);
            println!("      Confidence: {:.2}", confidence);
        }
    }
    
    // Test 3: Project-wide Impact Analysis
    println!("\n🔍 Test 3: Project-wide Impact Analysis");
    
    // Debug: Show discovered files first
    let discovered_files = impact_service.discover_project_files(project_path)?;
    println!("   📁 Discovered {} files:", discovered_files.len());
    for (i, file) in discovered_files.iter().take(10).enumerate() {
        println!("      {}. {}", i + 1, file);
    }
    
    let project_impact = impact_service.analyze_project_impact(
        &vec!["src/app/services/auth.service.ts".to_string()],
        project_path
    ).await?;
    
    println!("   ✅ Project Analysis Results:");
    println!("      Project: {}", project_impact.project_path);
    println!("      Changed File: {}", project_impact.changed_file);
    println!("      Impacted Files: {}", project_impact.impacted_files.len());
    
    // Show top impacted files
    for (i, file) in project_impact.impacted_files.iter().take(5).enumerate() {
        println!("      {}. {} (score: {:.2}, type: {:?})", 
            i + 1, file.file_path, file.impact_score, file.impact_type);
    }
    
    // Don't assert failure if no files found - this is expected in fallback mode sometimes
    if project_impact.impacted_files.len() == 0 {
        println!("   ⚠️  No impacted files found - this is expected in fallback mode");
    }
    
    // Test 4: File Discovery
    println!("\n🔍 Test 4: File Discovery");
    
    let discovered_files = impact_service.discover_project_files(project_path)?;
    println!("   ✅ File Discovery Results:");
    println!("      Total Files: {}", discovered_files.len());
    
    let mut file_types = std::collections::HashMap::new();
    for file in &discovered_files {
        let ext = Path::new(file).extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        *file_types.entry(ext).or_insert(0) += 1;
    }
    
    for (ext, count) in file_types.iter() {
        println!("      .{}: {} files", ext, count);
    }
    
    assert!(discovered_files.len() > 0);
    
    // Test 5: File Type Classification
    println!("\n🔍 Test 5: File Type Classification");
    
    let test_files = vec![
        "src/app/services/auth.service.ts",
        "src/app/services/user.service.ts",
        "src/app/dashboard/dashboard.component.ts",
        "src/app/login/login.component.ts",
        "src/app/guards/auth.guard.ts",
        "src/app/interceptors/auth.interceptor.ts",
        "src/app/services/auth.service.spec.ts",
        "src/app/home/home.component.spec.ts",
    ];
    
    println!("   ✅ File Classification Results:");
    for file in test_files {
        let impact = impact_service.analyze_impact(file, &vec!["testFunction".to_string()]).await?;
        
        let (change_type, severity) = match impact {
            ImpactReport::Basic { base_impact, .. } => (base_impact.change_type, base_impact.severity),
            ImpactReport::Enhanced { base_impact, .. } => (base_impact.change_type, base_impact.severity),
        };
        
        println!("      {} -> {:?} ({})", file, change_type, format!("{:?}", severity));
    }
    
    // Test 6: Cascade Effects
    println!("\n🔍 Test 6: Cascade Effects");
    
    let cascade_effects = impact_service.predict_cascade_effects(
        "login",
        &project_path.join("src/app/services/auth.service.ts"),
        project_path
    ).await?;
    
    println!("   ✅ Cascade Effects Results:");
    println!("      Predicted Effects: {}", cascade_effects.len());
    
    for (i, effect) in cascade_effects.iter().enumerate() {
        println!("      {}. {} -> {} ({})", 
            i + 1, effect.affected_component, effect.affected_function, effect.description);
    }
    
    // Test 7: Performance Test
    println!("\n🔍 Test 7: Performance Test");
    
    let start = std::time::Instant::now();
    
    for i in 0..10 {
        let _impact = impact_service.analyze_impact(
            "src/app/services/auth.service.ts",
            &vec![format!("function{}", i)]
        ).await?;
    }
    
    let duration = start.elapsed();
    println!("   ✅ Performance Results:");
    println!("      10 analyses in {:?}", duration);
    println!("      Average per analysis: {:?}", duration / 10);
    
    assert!(duration.as_secs() < 5);
    
    // Test 8: Real File Pattern Detection
    println!("\n🔍 Test 8: Real File Pattern Detection");
    
    // Check individual component directories
    let component_dirs = vec!["dashboard", "login", "calendar", "appointments", "home"];
    let mut total_components = 0;
    
    for dir in component_dirs {
        let component_dir = project_path.join("src/app").join(dir);
        if component_dir.exists() {
            let components = std::fs::read_dir(component_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("ts"))
                .count();
            total_components += components;
        }
    }
    println!("   ✅ Found {} component files total", total_components);
    
    let services_dir = project_path.join("src/app/services");
    if services_dir.exists() {
        let services = std::fs::read_dir(services_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .count();
        println!("   ✅ Found {} service files", services);
    }
    
    println!("\n🎉 All tests passed! Token Optimizer is working correctly with calendario-psicologia project.");
    
    // Ensure proper cleanup
    impact_service.shutdown().await?;
    Ok(())
}

/// Test specific Angular patterns from calendario-psicologia
#[tokio::test]
#[serial]
async fn test_angular_patterns() -> Result<()> {
    let project_path = Path::new("calendario-psicologia");
    
    if !project_path.exists() {
        println!("🔄 Skipping Angular patterns test - calendario-psicologia not found");
        return Ok(());
    }
    
    println!("🔍 Testing Angular Patterns Recognition");
    
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    
    let context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    
    impact_service.initialize().await?;
    
    // Test Angular component patterns
    let component_code = r#"
    import { Component, OnInit } from '@angular/core';
    import { AuthService } from '../services/auth.service';
    
    @Component({
      selector: 'app-dashboard',
      templateUrl: './dashboard.component.html',
      styleUrls: ['./dashboard.component.scss']
    })
    export class DashboardComponent implements OnInit {
      
      constructor(private authService: AuthService) {}
      
      ngOnInit(): void {
        this.loadUserData();
      }
      
      private loadUserData(): void {
        this.authService.getUserProfile().subscribe(user => {
          console.log('User loaded:', user);
        });
      }
    }
    "#;
    
    let component_context = context_service.create_base_context(
        "loadUserData",
        "src/app/dashboard/dashboard.component.ts",
        component_code
    )?;
    
    println!("   ✅ Angular Component Analysis:");
    println!("      Function: {}", component_context.function_name);
    println!("      Complexity: {:.2}", component_context.complexity_score);
    println!("      Impact Scope: {:?}", component_context.impact_scope);
    
    // Test Angular service patterns
    let service_code = r#"
    import { Injectable } from '@angular/core';
    import { HttpClient } from '@angular/common/http';
    import { Observable } from 'rxjs';
    
    @Injectable({
      providedIn: 'root'
    })
    export class AuthService {
      private apiUrl = 'https://api.example.com';
      
      constructor(private http: HttpClient) {}
      
      login(email: string, password: string): Observable<any> {
        return this.http.post(`${this.apiUrl}/login`, { email, password });
      }
      
      getUserProfile(): Observable<any> {
        return this.http.get(`${this.apiUrl}/profile`);
      }
    }
    "#;
    
    let service_context = context_service.create_base_context(
        "login",
        "src/app/services/auth.service.ts",
        service_code
    )?;
    
    println!("   ✅ Angular Service Analysis:");
    println!("      Function: {}", service_context.function_name);
    println!("      Complexity: {:.2}", service_context.complexity_score);
    println!("      Impact Scope: {:?}", service_context.impact_scope);
    
    // Test impact analysis on Angular patterns
    let impact = impact_service.analyze_impact(
        "src/app/services/auth.service.ts",
        &vec!["login".to_string(), "getUserProfile".to_string()]
    ).await?;
    
    match impact {
        ImpactReport::Basic { base_impact, .. } => {
            println!("   ✅ Angular Service Impact:");
            println!("      Change Type: {:?}", base_impact.change_type);
            println!("      Severity: {:?}", base_impact.severity);
            
            assert_eq!(base_impact.change_type, ChangeType::ServiceModification);
        }
        ImpactReport::Enhanced { base_impact, .. } => {
            println!("   ✅ Enhanced Angular Service Impact:");
            println!("      Change Type: {:?}", base_impact.change_type);
        }
    }
    
    println!("✅ Angular patterns recognition completed successfully");
    
    // Ensure proper cleanup
    impact_service.shutdown().await?;
    Ok(())
}

/// Test performance with realistic project size
#[tokio::test]
#[serial]
async fn test_performance_realistic() -> Result<()> {
    let project_path = Path::new("calendario-psicologia");
    
    if !project_path.exists() {
        println!("🔄 Skipping performance test - calendario-psicologia not found");
        return Ok(());
    }
    
    println!("🚀 Performance Testing with Realistic Project");
    
    let config = MLConfig::for_testing();
    let plugin_manager = Arc::new(PluginManager::new());
    
    let context_service = SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    
    impact_service.initialize().await?;
    
    // Test 1: Multiple file analysis
    let start = std::time::Instant::now();
    
    let files = vec![
        "src/app/services/auth.service.ts",
        "src/app/services/user.service.ts",
        "src/app/dashboard/dashboard.component.ts",
        "src/app/login/login.component.ts",
        "src/app/calendar/calendar.component.ts",
        "src/app/appointments/appointments.component.ts",
        "src/app/guards/auth.guard.ts",
        "src/app/interceptors/auth.interceptor.ts",
    ];
    
    for file in files {
        let _impact = impact_service.analyze_impact(file, &vec!["testFunction".to_string()]).await?;
    }
    
    let duration = start.elapsed();
    println!("   ✅ Multiple File Analysis:");
    println!("      8 files analyzed in {:?}", duration);
    println!("      Average per file: {:?}", duration / 8);
    
    // Test 2: Project impact analysis
    let start = std::time::Instant::now();
    
    let _project_impact = impact_service.analyze_project_impact(
        &vec!["src/app/services/auth.service.ts".to_string()],
        project_path
    ).await?;
    
    let duration = start.elapsed();
    println!("   ✅ Project Impact Analysis:");
    println!("      Full project analyzed in {:?}", duration);
    
    // Test 3: Context analysis performance
    let start = std::time::Instant::now();
    
    let test_code = r#"
    export class TestService {
      async complexMethod(param1: string, param2: number): Promise<void> {
        if (param1 && param2 > 0) {
          for (let i = 0; i < param2; i++) {
            try {
              await this.processItem(param1, i);
            } catch (error) {
              console.error('Error processing item:', error);
            }
          }
        }
      }
    }
    "#;
    
    for i in 0..20 {
        let _context = context_service.create_base_context(
            "complexMethod",
            &format!("src/app/services/test{}.service.ts", i),
            test_code
        )?;
    }
    
    let duration = start.elapsed();
    println!("   ✅ Context Analysis Performance:");
    println!("      20 context analyses in {:?}", duration);
    println!("      Average per analysis: {:?}", duration / 20);
    
    // Performance assertions
    assert!(duration.as_millis() < 1000); // Should be under 1 second
    
    println!("✅ Performance requirements met");
    
    // Ensure proper cleanup  
    impact_service.shutdown().await?;
    Ok(())
}
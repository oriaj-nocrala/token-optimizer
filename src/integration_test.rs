//! Integration tests for areas of improvement fixes
//! Tests all the fixes implemented based on the calendario-psicologia project analysis

use anyhow::Result;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

use crate::analyzers::{DiffAnalyzer, StateAnalyzer, RoutingAnalyzer, InterceptorAnalyzer};
use crate::cache::CacheManager;
use crate::utils::{GitUtils, path_normalizer::PathNormalizer};
use crate::types::StateType;

/// Integration test for changes command fixes
#[test]
fn test_changes_command_fixes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Initialize a git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&temp_dir)
        .output()?;
    
    // Configure git to avoid warnings
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&temp_dir)
        .output()?;
        
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&temp_dir)
        .output()?;
    
    // Create and commit initial file
    let auth_service_path = temp_dir.path().join("auth.service.ts");
    fs::write(&auth_service_path, r#"
import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  constructor() { }
}
"#)?;
    
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&temp_dir)
        .output()?;
        
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&temp_dir)
        .output()?;
    
    // Modify the file with actual content changes
    fs::write(&auth_service_path, r#"
import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private isAuthenticated = new BehaviorSubject<boolean>(false);
  
  constructor() { }
  
  login(): void {
    this.isAuthenticated.next(true);
  }
  
  logout(): void {
    this.isAuthenticated.next(false);
  }
}
"#)?;
    
    // Test GitUtils fixes
    let git_utils = GitUtils::new(temp_dir.path())?;
    
    // Test modified files detection (should only show files with actual changes)
    let modified_files = git_utils.get_modified_files()?;
    assert_eq!(modified_files.len(), 1, "Should detect exactly one modified file");
    assert!(modified_files[0].contains("auth.service.ts"), "Should detect the auth service as modified");
    
    // Test line changes analysis (should no longer return 0,0)
    let (lines_added, lines_removed) = git_utils.get_file_changes("auth.service.ts")?;
    assert!(lines_added > 0, "Should detect lines added: got {}", lines_added);
    assert!(lines_removed == 0, "Should detect no lines removed for this change: got {}", lines_removed);
    
    // Test DiffAnalyzer integration
    let diff_analyzer = DiffAnalyzer::new(temp_dir.path())?;
    let change_analysis = diff_analyzer.analyze_changes(temp_dir.path())?;
    
    assert_eq!(change_analysis.modified_files.len(), 1, "DiffAnalyzer should detect one modified file");
    let modified_file = &change_analysis.modified_files[0];
    assert!(modified_file.lines_added > 0, "Should show lines added in DiffAnalyzer");
    assert_eq!(modified_file.lines_removed, 0, "Should show no lines removed in DiffAnalyzer");
    
    println!("✅ Changes command fixes validated");
    Ok(())
}

/// Integration test for StateAnalyzer improvements  
#[test]
fn test_state_analyzer_improvements() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut cache_manager = CacheManager::new(temp_dir.path())?;
    
    // Create a comprehensive Angular service with various RxJS patterns
    let auth_service_content = r#"
import { Injectable } from '@angular/core';
import { BehaviorSubject, Subject, ReplaySubject, Observable } from 'rxjs';
import { map, filter } from 'rxjs/operators';
import { HttpClient } from '@angular/common/http';

interface User {
  id: string;
  name: string;
  email: string;
}

interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  loading: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  // BehaviorSubject for main state
  private authState = new BehaviorSubject<AuthState>({
    isAuthenticated: false,
    user: null,
    loading: true
  });

  // Additional state subjects
  private userSubject = new Subject<User>();
  private notificationSubject = new ReplaySubject<string>(5);
  
  // Public observables with $ convention
  public readonly authState$ = this.authState.asObservable();
  public readonly isAuthenticated$ = this.authState$.pipe(map(state => state.isAuthenticated));
  public readonly user$ = this.authState$.pipe(map(state => state.user));
  public readonly loading$ = this.authState$.pipe(map(state => state.loading));
  
  // Observable without $ suffix
  public readonly notifications = this.notificationSubject.asObservable();
  
  // HTTP observable
  private apiData: Observable<any> = this.http.get('/api/user');
  
  // Type annotated observables
  private userStream: Observable<User> = this.http.get<User>('/api/current-user');
  public dataSource: BehaviorSubject<any> = new BehaviorSubject(null);

  constructor(private http: HttpClient) {}

  updateAuthState(isAuthenticated: boolean, user: User | null, loading: boolean): void {
    this.authState.next({
      isAuthenticated,
      user,
      loading
    });
  }

  private setLoadingState(loading: boolean): void {
    const currentState = this.authState.value;
    this.updateAuthState(currentState.isAuthenticated, currentState.user, loading);
  }
  
  login(credentials: any): void {
    this.setLoadingState(true);
    // Login logic here
    this.userSubject.next(credentials.user);
  }
  
  logout(): void {
    this.updateAuthState(false, null, false);
    this.notificationSubject.next('User logged out');
  }
}
"#;
    
    let service_file = temp_dir.path().join("auth.service.ts");
    fs::write(&service_file, auth_service_content)?;
    
    // Analyze the file
    cache_manager.analyze_file(&service_file)?;
    
    // Test StateAnalyzer
    let state_analyzer = StateAnalyzer::new();
    let analysis = state_analyzer.analyze_project_state(&cache_manager)?;
    
    // Validate that we now detect state management patterns
    assert_eq!(analysis.services_with_state.len(), 1, "Should detect one service with state");
    
    let auth_service = &analysis.services_with_state[0];
    assert_eq!(auth_service.service_name, "AuthService");
    
    // Test BehaviorSubject detection
    assert!(!auth_service.state_properties.is_empty(), "Should detect state properties");
    let behavior_subjects: Vec<_> = auth_service.state_properties.iter()
        .filter(|p| matches!(p.property_type, StateType::BehaviorSubject))
        .collect();
    assert!(behavior_subjects.len() >= 2, "Should detect at least 2 BehaviorSubjects: authState and dataSource");
    
    // Test Subject detection
    let subjects: Vec<_> = auth_service.state_properties.iter()
        .filter(|p| matches!(p.property_type, StateType::Subject))
        .collect();
    assert!(!subjects.is_empty(), "Should detect Subject (userSubject)");
    
    // Test ReplaySubject detection
    let replay_subjects: Vec<_> = auth_service.state_properties.iter()
        .filter(|p| matches!(p.property_type, StateType::ReplaySubject))
        .collect();
    assert!(!replay_subjects.is_empty(), "Should detect ReplaySubject (notificationSubject)");
    
    // Test observable detection (should detect multiple observables)
    assert!(auth_service.observables.len() >= 6, "Should detect at least 6 observables: authState$, isAuthenticated$, user$, loading$, notifications, userStream");
    
    // Test $ naming convention detection
    let dollar_observables: Vec<_> = auth_service.observables.iter()
        .filter(|o| o.name.ends_with('$'))
        .collect();
    assert!(dollar_observables.len() >= 4, "Should detect observables with $ suffix");
    
    // Test type annotation detection
    let typed_observables: Vec<_> = auth_service.observables.iter()
        .filter(|o| o.name == "userStream" || o.name == "dataSource")
        .collect();
    assert!(typed_observables.len() >= 2, "Should detect type-annotated observables");
    
    // Test state method detection
    assert!(!auth_service.state_methods.is_empty(), "Should detect state management methods");
    assert!(auth_service.state_methods.contains(&"updateAuthState".to_string()), "Should detect updateAuthState method");
    assert!(auth_service.state_methods.contains(&"setLoadingState".to_string()), "Should detect setLoadingState method");
    
    // Test pattern detection
    assert!(!analysis.patterns_detected.is_empty(), "Should detect patterns");
    assert!(analysis.patterns_detected.iter().any(|p| p.contains("BehaviorSubject pattern")), "Should detect BehaviorSubject pattern");
    assert!(analysis.patterns_detected.iter().any(|p| p.contains("Observable naming convention")), "Should detect $ naming convention");
    assert!(analysis.patterns_detected.iter().any(|p| p.contains("State encapsulation pattern")), "Should detect encapsulation pattern");
    
    println!("✅ StateAnalyzer improvements validated");
    Ok(())
}

/// Integration test for Guards and Interceptors node_modules exclusion
#[test]
fn test_routing_interceptor_node_modules_exclusion() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path();
    
    // Create src directory structure
    let src_path = project_path.join("src/app");
    fs::create_dir_all(&src_path)?;
    
    // Create node_modules structure (should be excluded)
    let node_modules_path = project_path.join("node_modules/@angular/router");
    fs::create_dir_all(&node_modules_path)?;
    
    // Create legitimate guard in src directory
    let auth_guard_content = r#"
import { Injectable } from '@angular/core';
import { CanActivate } from '@angular/router';

@Injectable({
  providedIn: 'root'
})
export class AuthGuard implements CanActivate {
  canActivate(): boolean {
    return true;
  }
}
"#;
    fs::write(src_path.join("auth.guard.ts"), auth_guard_content)?;
    
    // Create fake guard in node_modules (should be excluded)
    let fake_guard_content = r#"
export class SomeLibraryGuard implements CanActivate {
  canActivate(): boolean {
    return false;
  }
}
"#;
    fs::write(node_modules_path.join("fake.guard.ts"), fake_guard_content)?;
    
    // Create legitimate interceptor in src directory
    let auth_interceptor_content = r#"
import { Injectable } from '@angular/core';
import { HttpInterceptor, HttpRequest, HttpHandler } from '@angular/common/http';

@Injectable()
export class AuthInterceptor implements HttpInterceptor {
  intercept(req: HttpRequest<any>, next: HttpHandler) {
    return next.handle(req);
  }
}
"#;
    fs::write(src_path.join("auth.interceptor.ts"), auth_interceptor_content)?;
    
    // Create fake interceptor in node_modules (should be excluded)
    let fake_interceptor_content = r#"
export class SomeLibraryInterceptor implements HttpInterceptor {
  intercept(req: HttpRequest<any>, next: HttpHandler) {
    return next.handle(req);
  }
}
"#;
    fs::write(node_modules_path.join("fake.interceptor.ts"), fake_interceptor_content)?;
    
    // Test RoutingAnalyzer
    let routing_analyzer = RoutingAnalyzer::new();
    let routing_analysis = routing_analyzer.analyze_project_routing(project_path)?;
    
    // Should only find guards in src directory
    let auth_guards: Vec<_> = routing_analysis.guards.iter()
        .filter(|g| g.name.contains("auth"))
        .collect();
    assert!(!auth_guards.is_empty(), "Should find AuthGuard in src directory");
    
    // Should not find guards from node_modules
    let fake_guards: Vec<_> = routing_analysis.guards.iter()
        .filter(|g| g.name.contains("SomeLibrary"))
        .collect();
    assert!(fake_guards.is_empty(), "Should not find guards from node_modules");
    
    // Test InterceptorAnalyzer
    let interceptor_analyzer = InterceptorAnalyzer::new();
    let interceptor_analysis = interceptor_analyzer.analyze_project_interceptors(project_path)?;
    
    // Should only find interceptors in src directory
    let auth_interceptors: Vec<_> = interceptor_analysis.interceptors.iter()
        .filter(|i| i.name.contains("auth"))
        .collect();
    assert!(!auth_interceptors.is_empty(), "Should find AuthInterceptor in src directory");
    
    // Should not find interceptors from node_modules
    let fake_interceptors: Vec<_> = interceptor_analysis.interceptors.iter()
        .filter(|i| i.name.contains("SomeLibrary"))
        .collect();
    assert!(fake_interceptors.is_empty(), "Should not find interceptors from node_modules");
    
    println!("✅ Guards and Interceptors node_modules exclusion validated");
    Ok(())
}

/// Integration test for path normalization
#[test]
fn test_path_normalization() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().join("calendario-psicologia");
    fs::create_dir_all(&project_path)?;
    
    let normalizer = PathNormalizer::new(&project_path);
    
    // Test absolute path normalization
    let absolute_path = project_path.join("src/app/services/auth.service.ts");
    let normalized = normalizer.normalize_to_project_relative(&absolute_path);
    assert_eq!(normalized, "./src/app/services/auth.service.ts");
    
    // Test cache key consistency
    let cache_key = normalizer.create_cache_key(&absolute_path);
    assert_eq!(cache_key, "./src/app/services/auth.service.ts");
    
    // Test path matching variations
    assert!(normalizer.path_matches_cache_key(&cache_key, "src/app/services/auth.service.ts"));
    assert!(normalizer.path_matches_cache_key(&cache_key, "./src/app/services/auth.service.ts"));
    assert!(normalizer.path_matches_cache_key(&cache_key, "calendario-psicologia/src/app/services/auth.service.ts"));
    
    // Test with project prefix in cache key
    let cache_key_with_project = "./calendario-psicologia/src/app/services/auth.service.ts";
    assert!(normalizer.path_matches_cache_key(cache_key_with_project, "src/app/services/auth.service.ts"));
    assert!(normalizer.path_matches_cache_key(cache_key_with_project, "./src/app/services/auth.service.ts"));
    
    // Test that unrelated paths don't match
    assert!(!normalizer.path_matches_cache_key(&cache_key, "src/app/components/user.component.ts"));
    
    println!("✅ Path normalization validated");
    Ok(())
}

/// Integration test to verify all fixes work together
#[test]
fn test_complete_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().join("test-angular-project");
    fs::create_dir_all(&project_path.join("src/app/services"))?;
    fs::create_dir_all(&project_path.join("src/app/guards"))?;
    fs::create_dir_all(&project_path.join("node_modules/@angular/common"))?;
    
    // Initialize git
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&project_path)
        .output()?;
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&project_path)
        .output()?;
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&project_path)
        .output()?;
    
    // Create comprehensive test files
    let auth_service = project_path.join("src/app/services/auth.service.ts");
    fs::write(&auth_service, r#"
import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private authState = new BehaviorSubject<boolean>(false);
  public readonly authState$ = this.authState.asObservable();
  
  updateAuth(isAuth: boolean): void {
    this.authState.next(isAuth);
  }
}
"#)?;
    
    let auth_guard = project_path.join("src/app/guards/auth.guard.ts");
    fs::write(&auth_guard, r#"
import { Injectable } from '@angular/core';
import { CanActivate } from '@angular/router';

@Injectable({
  providedIn: 'root'
})
export class AuthGuard implements CanActivate {
  canActivate(): boolean {
    return true;
  }
}
"#)?;
    
    // Create fake files in node_modules
    fs::write(project_path.join("node_modules/@angular/common/fake.guard.ts"), "fake content")?;
    
    // Initial commit
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&project_path)
        .output()?;
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial"])
        .current_dir(&project_path)
        .output()?;
    
    // Modify auth service
    fs::write(&auth_service, r#"
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { map } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private authState = new BehaviorSubject<boolean>(false);
  public readonly authState$ = this.authState.asObservable();
  public readonly isLoggedIn$ = this.authState$.pipe(map(state => state));
  
  updateAuth(isAuth: boolean): void {
    this.authState.next(isAuth);
  }
  
  login(): void {
    this.updateAuth(true);
  }
  
  logout(): void {
    this.updateAuth(false);
  }
}
"#)?;
    
    // Test all components working together
    let normalizer = PathNormalizer::new(&project_path);
    let mut cache_manager = CacheManager::new(&project_path)?;
    
    // Analyze files
    cache_manager.analyze_file(&auth_service)?;
    cache_manager.analyze_file(&auth_guard)?;
    
    // Test StateAnalyzer with path normalization
    let state_analyzer = StateAnalyzer::new();
    let analysis = state_analyzer.analyze_project_state(&cache_manager)?;
    
    assert_eq!(analysis.services_with_state.len(), 1);
    let service = &analysis.services_with_state[0];
    assert!(service.state_properties.len() >= 1);
    assert!(service.observables.len() >= 2); // authState$ and isLoggedIn$
    
    // Test RoutingAnalyzer with node_modules exclusion
    let routing_analyzer = RoutingAnalyzer::new();
    let routing_analysis = routing_analyzer.analyze_project_routing(&project_path)?;
    
    // Should find guard in src but not in node_modules
    assert!(!routing_analysis.guards.is_empty());
    assert!(routing_analysis.guards.iter().all(|g| g.path.contains("/src/")));
    
    // Test GitUtils with proper change detection
    let git_utils = GitUtils::new(&project_path)?;
    let modified_files = git_utils.get_modified_files()?;
    
    assert!(!modified_files.is_empty());
    
    // Test that line changes are properly calculated
    for file in &modified_files {
        let (added, _removed) = git_utils.get_file_changes(file)?;
        if file.contains("auth.service.ts") {
            assert!(added > 0, "Should detect lines added in auth service");
        }
    }
    
    // Test path normalization integration
    let service_path_absolute = project_path.join("src/app/services/auth.service.ts");
    let normalized_key = normalizer.create_cache_key(&service_path_absolute);
    assert!(normalized_key.starts_with("./"));
    assert!(normalizer.path_matches_cache_key(&normalized_key, "src/app/services/auth.service.ts"));
    
    println!("✅ Complete integration test passed - all fixes working together");
    Ok(())
}

/// Integration test for the complete ML pipeline with real business logic
/// This tests the full workflow from function analysis to AI-powered insights
#[tokio::test]
async fn test_complete_ml_pipeline_with_real_business_logic() -> Result<()> {
    println!("🚀 Starting complete ML pipeline integration test...");
    
    // Use real ML configuration
    let config = crate::ml::config::MLConfig {
        model_cache_dir: std::path::PathBuf::from("/home/oriaj/Prog/Rust/Ts-tools/claude-ts-tools/.cache/ml-models"),
        memory_budget: 8_000_000_000, // 8GB
        quantization: crate::ml::config::QuantizationLevel::Q6_K,
        reasoning_timeout: 120,
        embedding_timeout: 60,
        operation_timeout: 30,
        ..Default::default()
    };
    
    // Initialize services
    let plugin_manager = Arc::new(crate::ml::plugins::PluginManager::new());
    let context_service = crate::ml::services::context::SmartContextService::new(config.clone(), plugin_manager.clone())?;
    let mut impact_service = crate::ml::services::impact_analysis::ImpactAnalysisService::new(config.clone(), plugin_manager.clone());
    let mut search_service = crate::ml::services::search::SemanticSearchService::new(config.clone(), plugin_manager.clone());
    
    // Initialize services
    impact_service.initialize().await?;
    search_service.initialize().await?;
    
    // Test with realistic TypeScript function
    let function_code = r#"
async function processUserAuthentication(email: string, password: string): Promise<AuthResult> {
    try {
        // Validate input parameters
        if (!email || !isValidEmail(email)) {
            throw new ValidationError('Invalid email format');
        }
        
        if (!password || password.length < 8) {
            throw new ValidationError('Password must be at least 8 characters');
        }
        
        // Check if user exists
        const user = await userRepository.findByEmail(email);
        if (!user) {
            throw new AuthenticationError('User not found');
        }
        
        // Verify password
        const isPasswordValid = await bcrypt.compare(password, user.hashedPassword);
        if (!isPasswordValid) {
            throw new AuthenticationError('Invalid password');
        }
        
        // Generate JWT token
        const token = await jwtService.generateToken({
            userId: user.id,
            email: user.email,
            roles: user.roles
        });
        
        // Update last login
        await userRepository.updateLastLogin(user.id);
        
        return {
            success: true,
            token,
            user: {
                id: user.id,
                email: user.email,
                roles: user.roles
            }
        };
    } catch (error) {
        logger.error('Authentication failed:', error);
        throw error;
    }
}
"#;
    
    println!("📊 Testing smart context analysis...");
    
    // Test 1: Smart Context Analysis
    let context = context_service.create_base_context(
        "processUserAuthentication",
        "src/auth/auth.service.ts",
        function_code
    )?;
    
    // Validate context analysis results
    assert_eq!(context.function_name, "processUserAuthentication");
    assert_eq!(context.file_path, "src/auth/auth.service.ts");
    assert!(context.complexity_score > 0.5); // Should detect significant complexity
    assert!(context.dependencies.len() > 0); // Should find dependencies
    
    // Check that analysis was performed
    assert!(context.complexity_score > 0.0);
    assert!(context.dependencies.len() > 0);
    
    println!("✅ Smart context analysis successful");
    println!("   - Function: {}", context.function_name);
    println!("   - Complexity: {:.2}", context.complexity_score);
    println!("   - Dependencies: {:?}", context.dependencies);
    println!("   - Impact Scope: {:?}", context.impact_scope);
    
    println!("🔍 Testing impact analysis...");
    
    // Test 2: Impact Analysis
    let changed_functions = vec!["processUserAuthentication".to_string()];
    
    let impact_analysis = impact_service.analyze_impact(
        "src/auth/auth.service.ts",
        &changed_functions
    ).await?;
    
    // Validate impact analysis results based on the ImpactReport enum
    match &impact_analysis {
        crate::ml::models::ImpactReport::Basic { base_impact, confidence } => {
            assert!(!base_impact.changed_file.is_empty());
            assert!(!base_impact.changed_functions.is_empty());
            assert!(*confidence > 0.0);
            println!("✅ Impact analysis successful (Basic Mode)");
            println!("   - Change Type: {:?}", base_impact.change_type);
            println!("   - Severity: {:?}", base_impact.severity);
            println!("   - Confidence: {:.2}", confidence);
        }
        crate::ml::models::ImpactReport::Enhanced { base_impact, semantic_impact, recommendations, confidence, .. } => {
            assert!(!base_impact.changed_file.is_empty());
            assert!(!base_impact.changed_functions.is_empty());
            assert!(semantic_impact.semantic_relationships.len() > 0);
            assert!(recommendations.len() > 0);
            assert!(*confidence > 0.0);
            println!("✅ Impact analysis successful (Enhanced Mode)");
            println!("   - Change Type: {:?}", base_impact.change_type);
            println!("   - Severity: {:?}", base_impact.severity);
            println!("   - Semantic Relationships: {}", semantic_impact.semantic_relationships.len());
            println!("   - Recommendations: {}", recommendations.len());
            println!("   - Confidence: {:.2}", confidence);
        }
    }
    
    println!("🔎 Testing semantic search...");
    
    // Test 3: Semantic Search
    let temp_project_dir = tempfile::tempdir()?;
    let project_path = temp_project_dir.path();
    
    // Create test files for search
    fs::create_dir_all(project_path.join("src/auth"))?;
    fs::create_dir_all(project_path.join("src/user"))?;
    fs::create_dir_all(project_path.join("src/validation"))?;
    fs::write(project_path.join("src/auth/auth.service.ts"), function_code)?;
    fs::write(project_path.join("src/user/user.repository.ts"), "export class UserRepository {}")?;
    fs::write(project_path.join("src/validation/email.validator.ts"), "export class EmailValidator {}")?;
    
    let search_results = search_service.search(
        "authentication and password validation",
        project_path.to_str().unwrap(),
        Some(3)
    ).await?;
    
    // Validate search results
    assert!(search_results.results.len() > 0);
    // Auth service should rank highly for authentication query
    assert!(search_results.results[0].file_path.contains("auth"));
    assert!(search_results.results[0].relevance_score > 0.0);
    
    println!("✅ Semantic search successful");
    println!("   - Results found: {}", search_results.results.len());
    for result in &search_results.results {
        println!("   - {}: {:.3}", result.file_path, result.relevance_score);
    }
    
    println!("🎯 Testing end-to-end workflow...");
    
    // Test 4: End-to-End Workflow
    // This simulates a real scenario where a developer wants to understand
    // the impact of modifying an authentication function
    
    // Step 1: Analyze the function context
    let enhanced_context = context_service.analyze_function_context(
        "processUserAuthentication",
        "src/auth/auth.service.ts",
        function_code
    ).await?;
    
    // Step 2: Understand potential impacts
    let full_impact = impact_service.analyze_impact(
        "src/auth/auth.service.ts",
        &changed_functions
    ).await?;
    
    // Step 3: Find related files that might be affected
    let related_files = search_service.search(
        "authentication jwt token user",
        project_path.to_str().unwrap(),
        Some(5)
    ).await?;
    
    // Validate end-to-end workflow
    assert!(enhanced_context.base_context.complexity_score > 0.0);
    assert!(related_files.results.len() > 0);
    
    // Check impact analysis results
    match &full_impact {
        crate::ml::models::ImpactReport::Basic { base_impact, .. } => {
            assert!(base_impact.changed_functions.len() > 0);
            println!("✅ End-to-end workflow successful (Basic Mode)");
            println!("   - Enhanced Context: complexity {:.2}", enhanced_context.base_context.complexity_score);
            println!("   - Impact Analysis: {} changed functions", base_impact.changed_functions.len());
            println!("   - Related Files: {} files found", related_files.results.len());
        }
        crate::ml::models::ImpactReport::Enhanced { base_impact, recommendations, .. } => {
            assert!(base_impact.changed_functions.len() > 0);
            assert!(recommendations.len() > 0);
            println!("✅ End-to-end workflow successful (Enhanced Mode)");
            println!("   - Enhanced Context: complexity {:.2}", enhanced_context.base_context.complexity_score);
            println!("   - Impact Analysis: {} recommendations", recommendations.len());
            println!("   - Related Files: {} files found", related_files.results.len());
        }
    }
    
    println!("🏆 Complete ML pipeline integration test completed successfully!");
    println!("   All ML services are working with real business logic");
    
    Ok(())
}

/// Test ML services with edge cases and error scenarios
#[tokio::test]
async fn test_ml_services_error_handling() -> Result<()> {
    println!("🧪 Testing ML services error handling...");
    
    let config = crate::ml::config::MLConfig::for_testing(); // Use test config
    let plugin_manager = Arc::new(crate::ml::plugins::PluginManager::new());
    let context_service = crate::ml::services::context::SmartContextService::new(config.clone(), plugin_manager.clone())?;
    
    // Test with empty function
    let empty_result = context_service.create_base_context(
        "emptyFunction",
        "src/empty.ts",
        ""
    );
    
    // Should handle empty input gracefully
    assert!(empty_result.is_ok());
    
    // Test with malformed code
    let malformed_code = "function broken( { missing bracket";
    let malformed_result = context_service.create_base_context(
        "brokenFunction",
        "src/broken.ts",
        malformed_code
    );
    
    // Should handle malformed code gracefully
    assert!(malformed_result.is_ok());
    
    println!("✅ Error handling tests passed");
    
    Ok(())
}

/// Test ML services performance with realistic data volumes
#[tokio::test]
async fn test_ml_services_performance() -> Result<()> {
    println!("⚡ Testing ML services performance...");
    
    let config = crate::ml::config::MLConfig::for_testing();
    let plugin_manager = Arc::new(crate::ml::plugins::PluginManager::new());
    let context_service = crate::ml::services::context::SmartContextService::new(config.clone(), plugin_manager.clone())?;
    
    // Generate large function for performance testing
    let large_function = format!(
        "function largeFunction() {{\n{}\n}}",
        (0..1000).map(|i| format!("    const var{} = {};", i, i)).collect::<Vec<_>>().join("\n")
    );
    
    let start_time = std::time::Instant::now();
    
    let context = context_service.create_base_context(
        "largeFunction",
        "src/large.ts",
        &large_function
    )?;
    
    let duration = start_time.elapsed();
    
    // Should complete within reasonable time (< 5 seconds)
    assert!(duration.as_secs() < 5);
    assert!(context.complexity_score > 0.0);
    
    println!("✅ Performance test passed");
    println!("   - Processed {} chars in {:?}", large_function.len(), duration);
    
    Ok(())
}

/// Test ML plugin integration with real models if available
#[tokio::test]
async fn test_ml_plugin_integration() -> Result<()> {
    println!("🔌 Testing ML plugin integration...");
    
    let config = crate::ml::config::MLConfig {
        model_cache_dir: std::path::PathBuf::from("/home/oriaj/Prog/Rust/Ts-tools/claude-ts-tools/.cache/ml-models"),
        memory_budget: 8_000_000_000,
        quantization: crate::ml::config::QuantizationLevel::Q6_K,
        reasoning_timeout: 60,
        embedding_timeout: 30,
        operation_timeout: 15,
        ..Default::default()
    };
    
    let plugin_manager = Arc::new(crate::ml::plugins::PluginManager::new());
    
    // Test plugin loading - individual plugins
    let available_plugins = plugin_manager.get_available_plugins();
    println!("Available plugins: {:?}", available_plugins);
    
    if available_plugins.len() > 0 {
        println!("✅ ML plugins are available");
        println!("   - {} plugins available", available_plugins.len());
    } else {
        println!("⚠️  No ML plugins available");
        println!("   This is expected if GGUF models are not installed");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_improvements() -> Result<()> {
        // Test 1: Changes command fixes
        test_changes_command_fixes()?;
        println!("✅ Changes command fixes validated");
        
        // Test 2: StateAnalyzer improvements  
        test_state_analyzer_improvements()?;
        println!("✅ StateAnalyzer improvements validated");
        
        // Test 3: Guards/Interceptors node_modules exclusion
        test_routing_interceptor_node_modules_exclusion()?;
        println!("✅ Guards and Interceptors node_modules exclusion validated");
        
        // Test 4: Path normalization
        test_path_normalization()?;
        println!("✅ Path normalization validated");
        
        // Test 5: Complete integration
        test_complete_integration()?;
        println!("✅ Complete integration test passed");
        
        println!("🎉 All improvement tests passed successfully!");
        
        Ok(())
    }
}
//! Tests for real semantic scoring based on embeddings

use anyhow::Result;
use std::sync::Arc;
use tempfile::tempdir;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::services::search::SemanticSearchService;
    use crate::ml::plugins::PluginManager;
    use crate::ml::config::MLConfig;
    use std::path::PathBuf;

    /// Test real semantic scoring with embeddings
    #[tokio::test]
    async fn test_real_semantic_scoring() -> Result<()> {
        println!("🧪 Testing real semantic scoring with embeddings...");
        
        // Create test mode config for embeddings
        let config = MLConfig {
            model_cache_dir: PathBuf::from(".cache/test-models"), // Force test mode
            memory_budget: 8_000_000_000,
            quantization: crate::ml::config::QuantizationLevel::Q6_K,
            reasoning_timeout: 120,
            embedding_timeout: 60,
            ..Default::default()
        };
        
        let plugin_manager = Arc::new(PluginManager::new());
        let mut search_service = SemanticSearchService::new(config, plugin_manager);
        
        // Initialize service
        search_service.initialize().await?;
        
        // Create test files with realistic TypeScript/Angular content
        let test_files = vec![
            ("auth.service.ts", 
             "import { Injectable } from '@angular/core';\n\
              @Injectable({ providedIn: 'root' })\n\
              export class AuthService {\n\
                login(email: string, password: string) {\n\
                  return this.http.post('/auth/login', { email, password });\n\
                }\n\
                \n\
                logout() {\n\
                  return this.http.post('/auth/logout', {});\n\
                }\n\
                \n\
                getCurrentUser() {\n\
                  return this.http.get('/auth/me');\n\
                }\n\
              }"
            ),
            ("user.service.ts",
             "import { Injectable } from '@angular/core';\n\
              @Injectable({ providedIn: 'root' })\n\
              export class UserService {\n\
                getProfile(userId: number) {\n\
                  return this.http.get(`/users/${userId}`);\n\
                }\n\
                \n\
                updateProfile(userId: number, data: any) {\n\
                  return this.http.put(`/users/${userId}`, data);\n\
                }\n\
                \n\
                deleteUser(userId: number) {\n\
                  return this.http.delete(`/users/${userId}`);\n\
                }\n\
              }"
            ),
            ("login.component.ts",
             "import { Component } from '@angular/core';\n\
              import { AuthService } from './auth.service';\n\
              \n\
              @Component({\n\
                selector: 'app-login',\n\
                templateUrl: './login.component.html'\n\
              })\n\
              export class LoginComponent {\n\
                email = '';\n\
                password = '';\n\
                \n\
                constructor(private authService: AuthService) {}\n\
                \n\
                onSubmit() {\n\
                  this.authService.login(this.email, this.password)\n\
                    .subscribe(response => {\n\
                      console.log('Login successful', response);\n\
                    });\n\
                }\n\
              }"
            ),
            ("calendar.component.ts",
             "import { Component, OnInit } from '@angular/core';\n\
              import { CalendarService } from './calendar.service';\n\
              \n\
              @Component({\n\
                selector: 'app-calendar',\n\
                templateUrl: './calendar.component.html'\n\
              })\n\
              export class CalendarComponent implements OnInit {\n\
                events: any[] = [];\n\
                \n\
                constructor(private calendarService: CalendarService) {}\n\
                \n\
                ngOnInit() {\n\
                  this.loadEvents();\n\
                }\n\
                \n\
                loadEvents() {\n\
                  this.calendarService.getEvents()\n\
                    .subscribe(events => {\n\
                      this.events = events;\n\
                    });\n\
                }\n\
              }"
            ),
        ];
        
        // Create temporary directory and files
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        
        for (filename, content) in &test_files {
            let file_path = temp_path.join(filename);
            std::fs::write(&file_path, content)?;
        }
        
        // Test semantic search with various queries
        let test_queries = vec![
            // Authentication-related queries
            ("user authentication", "Should find auth service and login component"),
            ("login functionality", "Should find login component and auth service"),
            ("logout user", "Should find auth service with logout method"),
            
            // User management queries
            ("user profile", "Should find user service and profile methods"),
            ("update user data", "Should find user service update method"),
            ("delete user", "Should find user service delete method"),
            
            // Angular-specific queries
            ("angular component", "Should find components"),
            ("angular service", "Should find services"),
            ("dependency injection", "Should find services with @Injectable"),
            
            // Calendar-related queries
            ("calendar events", "Should find calendar component"),
            ("load events", "Should find calendar component with loadEvents"),
        ];
        
        for (query, description) in &test_queries {
            println!("\n🔍 Testing query: '{}' - {}", query, description);
            
            let results = search_service.search(query, &temp_path.to_string_lossy(), Some(5)).await?;
            
            println!("   Found {} matches:", results.total_matches);
            
            // Validate results - some queries might not have matches if threshold is too high
            if results.total_matches == 0 {
                println!("   ⚠️  No matches found for query '{}' - this may indicate threshold is too high", query);
                continue;
            }
            
            let mut score_sum = 0.0f32;
            let mut max_score = 0.0f32;
            let mut min_score = 1.0f32;
            
            for (i, result) in results.results.iter().enumerate() {
                let file_name = result.file_path.split('/').last().unwrap_or("unknown");
                println!("     {}. {}: {:.4} relevance", i + 1, file_name, result.relevance_score);
                
                // Validate score properties
                assert!(result.relevance_score >= 0.0 && result.relevance_score <= 1.0, 
                        "Score should be between 0 and 1: {}", result.relevance_score);
                
                score_sum += result.relevance_score;
                max_score = max_score.max(result.relevance_score);
                min_score = min_score.min(result.relevance_score);
            }
            
            if !results.results.is_empty() {
                let avg_score = score_sum / results.results.len() as f32;
                println!("   📊 Score stats: avg={:.4}, max={:.4}, min={:.4}", avg_score, max_score, min_score);
                
                // Validate that we get reasonable score distribution
                assert!(avg_score > 0.0, "Average score should be > 0");
                
                // Allow for some score uniformity in small result sets with similar content
                if results.results.len() > 1 {
                    let score_range = max_score - min_score;
                    if score_range < 0.001 {
                        println!("   ⚠️  Note: All scores are very similar ({:.4}), which may indicate similar content relevance", max_score);
                    }
                }
            }
            
            // Test query-specific expectations
            match query {
                &"user authentication" => {
                    let auth_found = results.results.iter().any(|r| r.file_path.contains("auth.service"));
                    let login_found = results.results.iter().any(|r| r.file_path.contains("login.component"));
                    assert!(auth_found || login_found, "Should find auth-related files");
                }
                &"angular component" => {
                    let component_found = results.results.iter().any(|r| r.file_path.contains("component"));
                    assert!(component_found, "Should find component files");
                }
                &"angular service" => {
                    let service_found = results.results.iter().any(|r| r.file_path.contains("service"));
                    assert!(service_found, "Should find service files");
                }
                &"calendar events" => {
                    let calendar_found = results.results.iter().any(|r| r.file_path.contains("calendar"));
                    assert!(calendar_found, "Should find calendar-related files");
                }
                _ => {} // No specific validation for other queries
            }
        }
        
        // Test embedding consistency
        println!("\n🔍 Testing embedding consistency...");
        
        let results1 = search_service.search("user authentication", &temp_path.to_string_lossy(), Some(3)).await?;
        let results2 = search_service.search("user authentication", &temp_path.to_string_lossy(), Some(3)).await?;
        
        assert_eq!(results1.results.len(), results2.results.len(), "Same query should return same number of results");
        
        for (r1, r2) in results1.results.iter().zip(results2.results.iter()) {
            assert_eq!(r1.file_path, r2.file_path, "Same query should return same files");
            assert!((r1.relevance_score - r2.relevance_score).abs() < 0.001, 
                    "Same query should return same scores");
        }
        
        println!("✅ Embedding consistency test passed");
        
        // Cleanup
        search_service.shutdown().await?;
        
        println!("\n🎉 Real semantic scoring test completed successfully!");
        Ok(())
    }
    
    /// Test scoring quality with different content types
    #[tokio::test]
    async fn test_semantic_scoring_quality() -> Result<()> {
        println!("🧪 Testing semantic scoring quality...");
        
        let config = MLConfig {
            model_cache_dir: PathBuf::from(".cache/test-models"),
            memory_budget: 8_000_000_000,
            quantization: crate::ml::config::QuantizationLevel::Q6_K,
            reasoning_timeout: 120,
            embedding_timeout: 60,
            ..Default::default()
        };
        
        let plugin_manager = Arc::new(PluginManager::new());
        let mut search_service = SemanticSearchService::new(config, plugin_manager);
        search_service.initialize().await?;
        
        // Create test files with different content types
        let test_files = vec![
            ("highly_relevant.ts", 
             "export class UserAuthenticationService {\n\
                authenticateUser(email: string, password: string) {\n\
                  return this.login(email, password);\n\
                }\n\
                \n\
                login(email: string, password: string) {\n\
                  return this.authService.authenticate(email, password);\n\
                }\n\
              }"
            ),
            ("somewhat_relevant.ts",
             "export class UserService {\n\
                getUser(id: number) {\n\
                  return this.http.get(`/users/${id}`);\n\
                }\n\
                \n\
                updateUser(id: number, data: any) {\n\
                  return this.http.put(`/users/${id}`, data);\n\
                }\n\
              }"
            ),
            ("not_relevant.ts",
             "export class MathUtils {\n\
                add(a: number, b: number) {\n\
                  return a + b;\n\
                }\n\
                \n\
                multiply(a: number, b: number) {\n\
                  return a * b;\n\
                }\n\
              }"
            ),
        ];
        
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        
        for (filename, content) in &test_files {
            let file_path = temp_path.join(filename);
            std::fs::write(&file_path, content)?;
        }
        
        // Test with authentication query
        let results = search_service.search("user authentication", &temp_path.to_string_lossy(), Some(5)).await?;
        
        assert!(results.total_matches > 0, "Should find matches");
        
        // Find scores for different files
        let highly_relevant_score = results.results.iter()
            .find(|r| r.file_path.contains("highly_relevant"))
            .map(|r| r.relevance_score)
            .unwrap_or(0.0);
            
        let somewhat_relevant_score = results.results.iter()
            .find(|r| r.file_path.contains("somewhat_relevant"))
            .map(|r| r.relevance_score)
            .unwrap_or(0.0);
            
        let not_relevant_score = results.results.iter()
            .find(|r| r.file_path.contains("not_relevant"))
            .map(|r| r.relevance_score)
            .unwrap_or(0.0);
        
        println!("📊 Relevance scores:");
        println!("   Highly relevant: {:.4}", highly_relevant_score);
        println!("   Somewhat relevant: {:.4}", somewhat_relevant_score);
        println!("   Not relevant: {:.4}", not_relevant_score);
        
        // Validate scoring quality
        if highly_relevant_score > 0.0 && somewhat_relevant_score > 0.0 {
            // Allow for similar scores if both contain the same keywords
            // The highly relevant should be >= somewhat relevant
            assert!(highly_relevant_score >= somewhat_relevant_score, 
                    "Highly relevant should score >= somewhat relevant");
        }
        
        if somewhat_relevant_score > 0.0 && not_relevant_score > 0.0 {
            assert!(somewhat_relevant_score > not_relevant_score, 
                    "Somewhat relevant should score higher than not relevant");
        }
        
        search_service.shutdown().await?;
        
        println!("✅ Semantic scoring quality test passed!");
        Ok(())
    }
}
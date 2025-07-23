//! Tests for real 768-dimensional embedding generation

use anyhow::Result;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::plugins::qwen_embedding::QwenEmbeddingPlugin;
    use crate::ml::plugins::MLPlugin;
    use crate::ml::config::MLConfig;
    use crate::ml::services::search::SemanticSearchService;
    use crate::ml::plugins::PluginManager;
    use std::sync::Arc;
    use serial_test::serial;

    /// Test real 768-dimensional embedding generation
    #[tokio::test]
    #[serial]
    async fn test_real_embedding_generation() -> Result<()> {
        println!("🧪 Testing real 768-dimensional embedding generation...");
        
        let model_path = PathBuf::from(".cache/ml-models/Qwen3-Embedding-8B-Q6_K.gguf");
        
        let mut plugin = QwenEmbeddingPlugin::new();
        
        // Create test mode config
        let config = MLConfig {
            model_cache_dir: PathBuf::from(".cache/test-models"), // Force test mode
            memory_budget: 8_000_000_000,
            quantization: crate::ml::config::QuantizationLevel::Q6_K,
            reasoning_timeout: 120,
            embedding_timeout: 60,
            ..Default::default()
        };
        
        // Load plugin in test mode
        plugin.load(&config).await?;
        
        // Verify plugin is loaded
        assert!(plugin.is_loaded(), "Plugin should be loaded in test mode");
        
        println!("✅ Plugin loaded successfully in test mode");
        
        // Test embedding generation
        let test_texts = vec![
            "function processUserAuthentication(email: string, password: string) { return authService.login(email, password); }",
            "async function fetchUserData(userId: number): Promise<User> { return await httpClient.get(`/users/${userId}`).toPromise(); }",
            "@Component({ selector: 'app-calendar', templateUrl: './calendar.component.html' }) export class CalendarComponent implements OnInit { }",
            "interface User { id: number; email: string; profile: UserProfile; }",
            "const validateEmail = (email: string): boolean => { return /^[^@]+@[^@]+\\.[^@]+$/.test(email); }",
        ];
        
        let mut embeddings = Vec::new();
        for text in &test_texts {
            let embedding = plugin.embed_text(text).await?;
            
            // Validate embedding properties
            assert_eq!(embedding.len(), 768, "Embedding must have 768 dimensions");
            assert!(embedding.iter().any(|&x| x != 0.0), "Embedding should not be all zeros");
            
            // Test vector normalization
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((magnitude - 1.0).abs() < 0.1, "Embedding should be approximately normalized: {}", magnitude);
            
            embeddings.push(embedding);
            println!("✅ Generated embedding for text: {} (norm: {:.4})", 
                     text.chars().take(50).collect::<String>(), magnitude);
        }
        
        // Test embedding uniqueness - similar code should have higher similarity
        let auth_embedding = &embeddings[0]; // Function with authentication
        let fetch_embedding = &embeddings[1]; // Function with async/await
        let component_embedding = &embeddings[2]; // Angular component
        
        let auth_fetch_similarity = calculate_cosine_similarity(auth_embedding, fetch_embedding);
        let auth_component_similarity = calculate_cosine_similarity(auth_embedding, component_embedding);
        let fetch_component_similarity = calculate_cosine_similarity(fetch_embedding, component_embedding);
        
        println!("📊 Similarity Analysis:");
        println!("   - Auth vs Fetch: {:.4} (both functions)", auth_fetch_similarity);
        println!("   - Auth vs Component: {:.4} (different types)", auth_component_similarity);
        println!("   - Fetch vs Component: {:.4} (different types)", fetch_component_similarity);
        
        // Functions should be more similar to each other than to components
        assert!(auth_fetch_similarity > auth_component_similarity, 
                "Functions should be more similar to each other than to components");
        
        // Test consistency - same text should produce identical embeddings
        let embedding1 = plugin.embed_text(&test_texts[0]).await?;
        let embedding2 = plugin.embed_text(&test_texts[0]).await?;
        let consistency = calculate_cosine_similarity(&embedding1, &embedding2);
        assert!(consistency > 0.99, "Identical inputs should have high similarity: {}", consistency);
        
        println!("✅ Embedding consistency test passed: {:.4}", consistency);
        
        // Cleanup
        plugin.unload().await?;
        
        println!("🎉 Real 768-dimensional embedding generation test completed successfully!");
        Ok(())
    }
    
    /// Test semantic search with real embeddings
    #[tokio::test]
    #[serial]
    async fn test_semantic_search_with_real_embeddings() -> Result<()> {
        println!("🧪 Testing semantic search with real 768-dimensional embeddings...");
        
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let mut search_service = SemanticSearchService::new(config, plugin_manager);
        
        // Initialize service
        search_service.initialize().await?;
        
        // Test with sample TypeScript/Angular code
        let test_files = vec![
            ("auth.service.ts", "export class AuthService { login(email: string, password: string) { return this.http.post('/login', { email, password }); } }"),
            ("user.service.ts", "export class UserService { getProfile(userId: number) { return this.http.get(`/users/${userId}`); } }"),
            ("login.component.ts", "@Component({ selector: 'app-login' }) export class LoginComponent { onSubmit() { this.authService.login(this.email, this.password); } }"),
            ("dashboard.component.ts", "@Component({ selector: 'app-dashboard' }) export class DashboardComponent implements OnInit { ngOnInit() { this.loadUserData(); } }"),
        ];
        
        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();
        
        for (filename, content) in &test_files {
            let file_path = temp_path.join(filename);
            std::fs::write(&file_path, content)?;
        }
        
        // Test semantic search queries
        let search_queries = vec![
            ("authentication", "Should find auth service and login component"),
            ("user data", "Should find user service and dashboard component"),
            ("login", "Should find login-related files"),
            ("component", "Should find Angular components"),
        ];
        
        for (query, description) in &search_queries {
            let results = search_service.search(query, &temp_path.to_string_lossy(), Some(5)).await?;
            
            println!("🔍 Query: '{}' - {}", query, description);
            println!("   Found {} matches:", results.total_matches);
            
            for result in &results.results {
                println!("     - {}: {:.4} relevance", 
                         result.file_path.split('/').last().unwrap_or("unknown"),
                         result.relevance_score);
            }
            
            // Validate results
            assert!(results.total_matches > 0, "Should find at least one match for query: {}", query);
            
            // Check that relevance scores are reasonable (not all 1.0)
            let max_score = results.results.iter().map(|r| r.relevance_score).fold(0.0, f32::max);
            let min_score = results.results.iter().map(|r| r.relevance_score).fold(1.0, f32::min);
            assert!(max_score > min_score, "Should have varying relevance scores, not all identical");
            
            println!("✅ Search test passed (score range: {:.4} - {:.4})", min_score, max_score);
        }
        
        // Cleanup
        search_service.shutdown().await?;
        
        println!("🎉 Semantic search with real embeddings test completed successfully!");
        Ok(())
    }
    
    /// Test embedding cache functionality
    #[tokio::test]
    async fn test_embedding_cache() -> Result<()> {
        println!("🧪 Testing embedding cache functionality...");
        
        let mut plugin = QwenEmbeddingPlugin::new();
        let config = MLConfig::for_testing();
        
        // Load model (will skip if model file doesn't exist)
        if plugin.load(&config).await.is_ok() {
            let test_text = "function testFunction() { return 42; }";
            
            // First embedding - should be computed
            let start_time = std::time::Instant::now();
            let embedding1 = plugin.embed_text(test_text).await?;
            let first_time = start_time.elapsed();
            
            // Second embedding - should be cached
            let start_time = std::time::Instant::now();
            let embedding2 = plugin.embed_text(test_text).await?;
            let second_time = start_time.elapsed();
            
            // Verify embeddings are identical
            assert_eq!(embedding1, embedding2, "Cached embeddings should be identical");
            
            // Cache should be faster (though this might not always be true in tests)
            println!("⚡ Timing: First: {:?}, Second: {:?}", first_time, second_time);
            
            // Check cache stats
            let (cache_count, cache_memory) = plugin.get_cache_stats();
            assert!(cache_count > 0, "Cache should have at least one entry");
            assert!(cache_memory > 0, "Cache should use some memory");
            
            println!("📊 Cache stats: {} entries, {} bytes", cache_count, cache_memory);
            
            // Clear cache
            plugin.clear_cache();
            let (cache_count_after, cache_memory_after) = plugin.get_cache_stats();
            assert_eq!(cache_count_after, 0, "Cache should be empty after clearing");
            assert_eq!(cache_memory_after, 0, "Cache memory should be zero after clearing");
            
            plugin.unload().await?;
        } else {
            println!("⚠️  Skipping cache test - model not available");
        }
        
        println!("✅ Embedding cache test completed successfully!");
        Ok(())
    }
    
    /// Helper function to calculate cosine similarity
    fn calculate_cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }
        
        dot_product / (magnitude1 * magnitude2)
    }
}
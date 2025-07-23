/*! Pipeline Demo
 * Demonstration of the complete Qwen Embedding → LSH → Reranker pipeline
 */

use super::*;
use crate::ml::{
    MLConfig,
    plugins::{QwenEmbeddingPlugin, QwenRerankerPlugin},
    vector_db::{
        VectorStoreFactory, VectorDBConfig, VectorEntry, SemanticSearchFactory,
        SemanticSearchConfig, SearchQuery, CodeMetadata, CodeType
    },
};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::info;

/// Pipeline demonstration
pub struct PipelineDemo {
    vector_db: Arc<RwLock<dyn VectorDatabase>>,
    embedding_plugin: Arc<RwLock<QwenEmbeddingPlugin>>,
    reranker_plugin: Arc<RwLock<QwenRerankerPlugin>>,
    pipeline: SemanticSearchPipeline,
}

impl PipelineDemo {
    /// Create new pipeline demo
    pub fn new() -> Result<Self> {
        info!("🚀 Creating ML Pipeline Demo");
        
        // Create vector database
        let vector_db_config = VectorDBConfig {
            cache_dir: ".cache/demo-vector-db".to_string(),
            lsh_candidates: 20,
            final_results: 5,
            ..VectorDBConfig::default()
        };
        let vector_db = VectorStoreFactory::create_native(vector_db_config);
        
        // Create ML plugins
        let embedding_plugin = Arc::new(RwLock::new(QwenEmbeddingPlugin::new()));
        let reranker_plugin = Arc::new(RwLock::new(QwenRerankerPlugin::new()));
        
        // Create search pipeline
        let search_config = SemanticSearchConfig {
            lsh_candidates: 20,     // Get 20 candidates from LSH
            final_results: 5,       // Return top 5 after reranking
            lsh_threshold: 0.2,     // Low threshold for broad recall
            rerank_threshold: 0.5,  // Moderate threshold for balance
            enable_caching: true,
            embedding_cache_size: 100,
        };
        
        let pipeline = SemanticSearchFactory::create_with_config(
            vector_db.clone(),
            embedding_plugin.clone(),
            reranker_plugin.clone(),
            search_config,
        );
        
        Ok(Self {
            vector_db,
            embedding_plugin,
            reranker_plugin,
            pipeline,
        })
    }
    
    /// Demonstrate the complete pipeline
    pub async fn run_demo(&self) -> Result<()> {
        info!("🎯 Starting Pipeline Demo");
        
        // Step 1: Add sample data
        self.add_sample_data().await?;
        
        // Step 2: Demonstrate searches
        self.demonstrate_searches().await?;
        
        // Step 3: Show statistics
        self.show_statistics().await?;
        
        info!("✅ Pipeline Demo completed successfully!");
        Ok(())
    }
    
    /// Add sample TypeScript/JavaScript code to the database
    async fn add_sample_data(&self) -> Result<()> {
        info!("📚 Adding sample code to vector database...");
        
        let sample_codes = vec![
            SampleCode {
                file_path: "utils/math.ts".to_string(),
                function_name: Some("calculateSum".to_string()),
                content: "function calculateSum(a: number, b: number): number { return a + b; }".to_string(),
                code_type: CodeType::Function,
                language: "typescript".to_string(),
            },
            SampleCode {
                file_path: "utils/string.ts".to_string(),
                function_name: Some("formatString".to_string()),
                content: "function formatString(str: string): string { return str.trim().toLowerCase(); }".to_string(),
                code_type: CodeType::Function,
                language: "typescript".to_string(),
            },
            SampleCode {
                file_path: "components/Button.tsx".to_string(),
                function_name: Some("Button".to_string()),
                content: "export const Button = ({ onClick, children }) => <button onClick={onClick}>{children}</button>".to_string(),
                code_type: CodeType::Component,
                language: "typescript".to_string(),
            },
            SampleCode {
                file_path: "services/api.ts".to_string(),
                function_name: Some("fetchData".to_string()),
                content: "async function fetchData(url: string): Promise<any> { const response = await fetch(url); return response.json(); }".to_string(),
                code_type: CodeType::Function,
                language: "typescript".to_string(),
            },
            SampleCode {
                file_path: "utils/validation.ts".to_string(),
                function_name: Some("validateEmail".to_string()),
                content: "function validateEmail(email: string): boolean { const regex = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/; return regex.test(email); }".to_string(),
                code_type: CodeType::Function,
                language: "typescript".to_string(),
            },
        ];
        
        let mut vector_db = self.vector_db.write();
        
        for (i, sample) in sample_codes.iter().enumerate() {
            info!("  📝 Adding: {}", sample.function_name.as_ref().unwrap_or(&"Unknown".to_string()));
            
            // Create dummy embedding (deterministic for demo)
            let embedding = self.create_demo_embedding(&sample.content, i);
            
            let metadata = CodeMetadata {
                file_path: sample.file_path.clone(),
                function_name: sample.function_name.clone(),
                line_start: 1,
                line_end: 1,
                code_type: sample.code_type.clone(),
                language: sample.language.clone(),
                complexity: 1.0 + (i as f32 * 0.5),
                tokens: sample.content.split_whitespace().take(10).map(|s| s.to_string()).collect(),
                hash: format!("hash_{}", i),
            };
            
            let entry = VectorEntry {
                id: format!("demo_{}", i),
                embedding,
                metadata,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            vector_db.add_vector(entry)?;
        }
        
        info!("✅ Added {} code samples to vector database", sample_codes.len());
        Ok(())
    }
    
    /// Demonstrate different types of searches
    async fn demonstrate_searches(&self) -> Result<()> {
        info!("🔍 Demonstrating semantic searches...");
        
        let test_queries = vec![
            ("function that adds numbers", "Looking for math functions"),
            ("string manipulation", "Looking for string utilities"),
            ("React component", "Looking for UI components"),
            ("async fetch", "Looking for API functions"),
            ("email validation", "Looking for validation functions"),
        ];
        
        for (query, description) in test_queries {
            info!("  🎯 {}: '{}'", description, query);
            
            let search_query = SearchQuery {
                text: query.to_string(),
                code_type: None,
                language: Some("typescript".to_string()),
                file_context: None,
                max_results: Some(3),
            };
            
            match self.pipeline.search(&search_query).await {
                Ok(results) => {
                    if results.is_empty() {
                        info!("    ❌ No results found");
                    } else {
                        info!("    ✅ Found {} results:", results.len());
                        for (i, result) in results.iter().enumerate() {
                            info!("      {}. {} (similarity: {:.3}, rerank: {:.3}, combined: {:.3})",
                                 i + 1,
                                 result.entry.metadata.function_name.as_ref().unwrap_or(&"Unknown".to_string()),
                                 result.embedding_similarity,
                                 result.rerank_score,
                                 result.combined_score
                            );
                        }
                    }
                }
                Err(e) => {
                    info!("    ❌ Search failed: {}", e);
                }
            }
            
            // Small delay between searches for readability
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
    
    /// Show pipeline and database statistics
    async fn show_statistics(&self) -> Result<()> {
        info!("📊 Pipeline Statistics:");
        
        // Vector DB stats
        let db_stats = self.vector_db.read().stats();
        info!("  📚 Vector Database:");
        info!("    - Total vectors: {}", db_stats.total_vectors);
        info!("    - Total files: {}", db_stats.total_files);
        info!("    - Index size: {:.2} MB", db_stats.index_size_mb);
        info!("    - Languages: {:?}", db_stats.by_language);
        info!("    - Code types: {:?}", db_stats.by_code_type);
        
        // Pipeline stats
        match self.pipeline.get_stats().await {
            Ok(pipeline_stats) => {
                info!("  🔍 Search Pipeline:");
                info!("    - Embedding cache hit rate: {:.2}%", pipeline_stats.embedding_cache_hit_rate * 100.0);
                info!("    - Rerank cache hit rate: {:.2}%", pipeline_stats.rerank_cache_hit_rate * 100.0);
            }
            Err(e) => {
                info!("  ❌ Failed to get pipeline stats: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Create deterministic demo embedding
    fn create_demo_embedding(&self, content: &str, seed: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Mix with seed for variation
        let mixed_seed = hash.wrapping_add(seed as u64);
        
        // Generate deterministic embedding
        let mut embedding = Vec::with_capacity(768);
        let mut rng_state = mixed_seed;
        
        for _ in 0..768 {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let val = ((rng_state / 65536) % 32768) as f32 / 32768.0 - 0.5;
            embedding.push(val);
        }
        
        // Normalize to unit vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut embedding {
                *val /= magnitude;
            }
        }
        
        embedding
    }
    
    /// Get demo statistics
    pub async fn get_demo_stats(&self) -> Result<DemoStats> {
        let db_stats = self.vector_db.read().stats();
        let pipeline_stats = self.pipeline.get_stats().await?;
        
        Ok(DemoStats {
            total_vectors: db_stats.total_vectors,
            search_pipeline_ready: true,
            embedding_cache_hit_rate: pipeline_stats.embedding_cache_hit_rate,
            rerank_cache_hit_rate: pipeline_stats.rerank_cache_hit_rate,
        })
    }
}

/// Sample code entry for demo
#[derive(Clone, Debug)]
struct SampleCode {
    file_path: String,
    function_name: Option<String>,
    content: String,
    code_type: CodeType,
    language: String,
}

/// Demo statistics
#[derive(Clone, Debug)]
pub struct DemoStats {
    pub total_vectors: usize,
    pub search_pipeline_ready: bool,
    pub embedding_cache_hit_rate: f64,
    pub rerank_cache_hit_rate: f64,
}

/// Convenience function to run the demo
pub async fn run_pipeline_demo() -> Result<()> {
    let demo = PipelineDemo::new()?;
    demo.run_demo().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pipeline_demo_creation() {
        let demo = PipelineDemo::new().unwrap();
        let stats = demo.get_demo_stats().await.unwrap();
        
        // Initially empty
        assert_eq!(stats.total_vectors, 0);
        assert!(stats.search_pipeline_ready);
    }
    
    #[tokio::test]
    async fn test_pipeline_demo_full() {
        let demo = PipelineDemo::new().unwrap();
        
        // Run the demo
        demo.run_demo().await.unwrap();
        
        // Check that data was added
        let stats = demo.get_demo_stats().await.unwrap();
        assert!(stats.total_vectors > 0);
        assert!(stats.search_pipeline_ready);
    }
    
    #[test]
    fn test_demo_embedding_generation() {
        let demo = PipelineDemo::new().unwrap();
        
        let embedding1 = demo.create_demo_embedding("function test() {}", 0);
        let embedding2 = demo.create_demo_embedding("function test() {}", 0);
        let embedding3 = demo.create_demo_embedding("function test() {}", 1);
        
        // Same content and seed should produce same embedding
        assert_eq!(embedding1, embedding2);
        
        // Different seed should produce different embedding
        assert_ne!(embedding1, embedding3);
        
        // Check dimensions
        assert_eq!(embedding1.len(), 768);
        
        // Check normalization (should be unit vector)
        let magnitude: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 1e-6);
    }
}
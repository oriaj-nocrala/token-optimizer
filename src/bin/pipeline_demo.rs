/*! Pipeline Demo Binary
 * Demonstration of the complete Qwen Embedding → LSH → Reranker pipeline
 */

use token_optimizer::ml::{
    MLConfig,
    plugins::{QwenEmbeddingPlugin, QwenRerankerPlugin, MLPlugin},
    vector_db::{
        VectorStoreFactory, VectorDBConfig, VectorEntry, SemanticSearchFactory,
        SemanticSearchConfig, SearchQuery, CodeMetadata, CodeType
    },
};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🚀 Starting ML Pipeline Demo");
    info!("📋 Pipeline: Qwen Embedding → LSH Index → Qwen Reranker");

    // Create vector database
    let vector_db_config = VectorDBConfig {
        cache_dir: ".cache/demo-vector-db".to_string(),
        num_hash_functions: 16,
        hash_bits: 10,
        similarity_threshold: 0.7,
        max_results: 20,
        enable_persistence: true,
    };
    let vector_db = VectorStoreFactory::create_native(vector_db_config);
    
    // Create ML plugins with real models
    let ml_config = MLConfig::for_8gb_vram();

    info!("🔧 Loading Qwen Embedding model...");
    let mut embedding_plugin = QwenEmbeddingPlugin::new();
    if let Err(e) = embedding_plugin.load(&ml_config).await {
        info!("⚠️  Failed to load embedding model: {}", e);
        info!("📝 Using dummy embeddings for demo");
    } else {
        info!("✅ Qwen Embedding model loaded successfully");
    }
    let embedding_plugin = Arc::new(RwLock::new(embedding_plugin));

    info!("🔧 Loading Qwen Reranker model...");
    let mut reranker_plugin = QwenRerankerPlugin::new();
    if let Err(e) = reranker_plugin.load(&ml_config).await {
        info!("⚠️  Failed to load reranker model: {}", e);
        info!("📝 Using dummy reranking for demo");
    } else {
        info!("✅ Qwen Reranker model loaded successfully");
    }
    let reranker_plugin = Arc::new(RwLock::new(reranker_plugin));
    
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

    info!("✅ Pipeline created successfully");

    // Add sample data
    info!("📚 Adding sample TypeScript code to vector database...");
    add_sample_data(&vector_db, &embedding_plugin).await?;

    // Demonstrate searches
    info!("🔍 Demonstrating semantic searches...");
    demonstrate_searches(&pipeline).await?;

    // Show statistics
    info!("📊 Pipeline Statistics:");
    show_statistics(&vector_db, &pipeline).await?;

    info!("🎉 Pipeline demo completed successfully!");
    Ok(())
}

async fn add_sample_data(
    vector_db: &Arc<RwLock<dyn token_optimizer::ml::vector_db::VectorDatabase>>,
    embedding_plugin: &Arc<RwLock<QwenEmbeddingPlugin>>
) -> Result<()> {
    let sample_codes = vec![
        ("utils/math.ts", "calculateSum", "function calculateSum(a: number, b: number): number { return a + b; }", CodeType::Function),
        ("utils/string.ts", "formatString", "function formatString(str: string): string { return str.trim().toLowerCase(); }", CodeType::Function),
        ("components/Button.tsx", "Button", "export const Button = ({ onClick, children }) => <button onClick={onClick}>{children}</button>", CodeType::Component),
        ("services/api.ts", "fetchData", "async function fetchData(url: string): Promise<any> { const response = await fetch(url); return response.json(); }", CodeType::Function),
        ("utils/validation.ts", "validateEmail", "function validateEmail(email: string): boolean { const regex = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/; return regex.test(email); }", CodeType::Function),
    ];
    
    let mut vector_db = vector_db.write();
    
    for (i, (file_path, function_name, content, code_type)) in sample_codes.iter().enumerate() {
        info!("  📝 Adding: {}", function_name);
        
        // Try to create real embedding, fallback to dummy
        let embedding = if embedding_plugin.read().is_loaded() {
            match embedding_plugin.read().embed_text(content).await {
                Ok(real_embedding) => {
                    info!("    ✅ Generated real embedding ({}D)", real_embedding.len());
                    real_embedding
                }
                Err(e) => {
                    info!("    ⚠️  Real embedding failed: {}, using dummy", e);
                    create_demo_embedding(content, i)
                }
            }
        } else {
            info!("    📝 Using dummy embedding (plugin not loaded)");
            create_demo_embedding(content, i)
        };
        
        let metadata = CodeMetadata {
            file_path: file_path.to_string(),
            function_name: Some(function_name.to_string()),
            line_start: 1,
            line_end: 1,
            code_type: code_type.clone(),
            language: "typescript".to_string(),
            complexity: 1.0 + (i as f32 * 0.5),
            tokens: content.split_whitespace().take(10).map(|s| s.to_string()).collect(),
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

async fn demonstrate_searches(pipeline: &token_optimizer::ml::vector_db::SemanticSearchPipeline) -> Result<()> {
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
        
        match pipeline.search(&search_query).await {
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

async fn show_statistics(
    vector_db: &Arc<RwLock<dyn token_optimizer::ml::vector_db::VectorDatabase>>, 
    pipeline: &token_optimizer::ml::vector_db::SemanticSearchPipeline
) -> Result<()> {
    // Vector DB stats
    let db_stats = vector_db.read().stats();
    info!("  📚 Vector Database:");
    info!("    - Total vectors: {}", db_stats.total_vectors);
    info!("    - Total files: {}", db_stats.total_files);
    info!("    - Index size: {:.2} MB", db_stats.index_size_mb);
    info!("    - Languages: {:?}", db_stats.by_language);
    info!("    - Code types: {:?}", db_stats.by_code_type);
    
    // Pipeline stats
    match pipeline.get_stats().await {
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

fn create_demo_embedding(content: &str, seed: usize) -> Vec<f32> {
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
/*! Semantic Search Pipeline
 * Advanced search combining embeddings, LSH indexing, and reranking
 */

use super::*;
use crate::ml::plugins::{QwenEmbeddingPlugin, QwenRerankerPlugin};
use crate::ml::vector_db::{VectorDatabase, SearchResult};
use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Semantic search pipeline combining multiple ML techniques
pub struct SemanticSearchPipeline {
    /// Vector database for LSH-based fast search
    vector_db: Arc<RwLock<dyn VectorDatabase>>,
    /// Embedding model for query vectorization
    embedding_plugin: Arc<RwLock<QwenEmbeddingPlugin>>,
    /// Reranker for result refinement
    reranker_plugin: Arc<RwLock<QwenRerankerPlugin>>,
    /// Pipeline configuration
    config: SemanticSearchConfig,
}

/// Configuration for semantic search pipeline
#[derive(Clone, Debug)]
pub struct SemanticSearchConfig {
    /// Number of candidates to retrieve from LSH
    pub lsh_candidates: usize,
    /// Final number of results after reranking
    pub final_results: usize,
    /// Minimum similarity threshold for LSH
    pub lsh_threshold: f32,
    /// Reranking threshold
    pub rerank_threshold: f32,
    /// Enable semantic caching
    pub enable_caching: bool,
    /// Cache size for embeddings
    pub embedding_cache_size: usize,
}

impl Default for SemanticSearchConfig {
    fn default() -> Self {
        Self {
            lsh_candidates: 100,     // Get 100 candidates from LSH
            final_results: 10,       // Return top 10 after reranking
            lsh_threshold: 0.1,      // Even lower threshold for very broad recall
            rerank_threshold: 0.001, // Ultra-low threshold for debugging
            enable_caching: true,
            embedding_cache_size: 1000,
        }
    }
}

/// Enhanced search result with reranking score
#[derive(Clone, Debug)]
pub struct EnhancedSearchResult {
    pub entry: VectorEntry,
    pub embedding_similarity: f32,
    pub rerank_score: f32,
    pub combined_score: f32,
    pub confidence: f32,
}

/// Search query with metadata
#[derive(Clone, Debug)]
pub struct SearchQuery {
    pub text: String,
    pub code_type: Option<CodeType>,
    pub language: Option<String>,
    pub file_context: Option<String>,
    pub max_results: Option<usize>,
}

impl SemanticSearchPipeline {
    /// Create new semantic search pipeline
    pub fn new(
        vector_db: Arc<RwLock<dyn VectorDatabase>>,
        embedding_plugin: Arc<RwLock<QwenEmbeddingPlugin>>,
        reranker_plugin: Arc<RwLock<QwenRerankerPlugin>>,
        config: SemanticSearchConfig,
    ) -> Self {
        Self {
            vector_db,
            embedding_plugin,
            reranker_plugin,
            config,
        }
    }
    
    /// Perform enhanced semantic search
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<EnhancedSearchResult>> {
        info!("Starting semantic search for: '{}'", query.text);
        
        // Step 1: Generate query embedding
        let query_embedding = self.generate_query_embedding(&query.text).await?;
        debug!("Generated query embedding: {} dimensions", query_embedding.len());
        
        // Step 2: LSH-based candidate retrieval
        let candidates = self.retrieve_candidates(&query_embedding, query).await?;
        info!("Retrieved {} candidates from LSH index", candidates.len());
        
        if candidates.is_empty() {
            warn!("No candidates found for query: '{}'", query.text);
            return Ok(Vec::new());
        }
        
        // Step 3: Rerank candidates
        let reranked_results = self.rerank_candidates(&query.text, candidates).await?;
        info!("Reranked {} results", reranked_results.len());
        
        // Step 4: Apply final filtering and scoring
        let final_results = self.finalize_results(reranked_results, query).await?;
        info!("Returning {} final results", final_results.len());
        
        Ok(final_results)
    }
    
    /// Search for similar code snippets
    pub async fn search_similar_code(&self, code: &str, language: &str) -> Result<Vec<EnhancedSearchResult>> {
        let query = SearchQuery {
            text: code.to_string(),
            code_type: None,
            language: Some(language.to_string()),
            file_context: None,
            max_results: Some(self.config.final_results),
        };
        
        self.search(&query).await
    }
    
    /// Search for functions with similar functionality
    pub async fn search_similar_functions(&self, function_signature: &str, function_body: &str) -> Result<Vec<EnhancedSearchResult>> {
        let combined_text = format!("{}\n{}", function_signature, function_body);
        
        let query = SearchQuery {
            text: combined_text,
            code_type: Some(CodeType::Function),
            language: None,
            file_context: None,
            max_results: Some(self.config.final_results),
        };
        
        self.search(&query).await
    }
    
    /// Search for components with similar patterns
    pub async fn search_similar_components(&self, component_code: &str, framework: &str) -> Result<Vec<EnhancedSearchResult>> {
        let query = SearchQuery {
            text: component_code.to_string(),
            code_type: Some(CodeType::Component),
            language: Some(framework.to_string()),
            file_context: None,
            max_results: Some(self.config.final_results),
        };
        
        self.search(&query).await
    }
    
    /// Generate embedding for query text
    pub async fn generate_query_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Strategy: Extract the necessary data from the plugin without holding the lock across await
        let text_clone = text.to_string();
        let embedding_plugin = Arc::clone(&self.embedding_plugin);
        
        // Spawn a task that handles the embedding generation
        let embeddings = tokio::task::spawn_blocking(move || {
            // This runs in a blocking thread where Send is not required
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let plugin = embedding_plugin.read();
                plugin.embed_texts(&[text_clone]).await
            })
        }).await??;
        
        if embeddings.is_empty() {
            anyhow::bail!("Failed to generate embedding for query text");
        }
        
        Ok(embeddings[0].clone())
    }
    
    /// Retrieve candidates using LSH index
    async fn retrieve_candidates(&self, query_embedding: &[f32], query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let vector_db = self.vector_db.read();
        let db_stats = vector_db.stats();
        println!("🔍 Vector DB stats at search time:");
        println!("   Total vectors: {}", db_stats.total_vectors);
        println!("   Total files: {}", db_stats.total_files);
        println!("🔍 Searching with query embedding len: {}, lsh_candidates: {}", 
                 query_embedding.len(), self.config.lsh_candidates);
        
        let mut candidates = vector_db.search(query_embedding, self.config.lsh_candidates)?;
        println!("🔍 Vector DB search returned {} raw candidates", candidates.len());
        
        // Apply query-specific filtering
        println!("🔍 Applying query-specific filtering...");
        println!("   Query code_type: {:?}", query.code_type);
        println!("   Query language: {:?}", query.language);
        println!("   LSH threshold: {}", self.config.lsh_threshold);
        
        self.filter_candidates(&mut candidates, query);
        
        // Sort by similarity (already done by search, but ensure consistency)
        candidates.sort_by(|a, b| {
            b.similarity.partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        println!("🔍 After filtering and sorting: {} candidates", candidates.len());
        
        Ok(candidates)
    }
    
    /// Filter candidates based on query criteria
    fn filter_candidates(&self, candidates: &mut Vec<SearchResult>, query: &SearchQuery) {
        candidates.retain(|candidate| {
            // Filter by code type if specified
            if let Some(ref query_type) = query.code_type {
                if candidate.entry.metadata.code_type != *query_type {
                    return false;
                }
            }
            
            // Filter by language if specified
            if let Some(ref query_lang) = query.language {
                if !candidate.entry.metadata.language.eq_ignore_ascii_case(query_lang) {
                    return false;
                }
            }
            
            // Filter by similarity threshold
            if candidate.similarity < self.config.lsh_threshold {
                return false;
            }
            
            true
        });
    }
    
    /// Rerank candidates using the reranker model
    async fn rerank_candidates(&self, query: &str, candidates: Vec<SearchResult>) -> Result<Vec<EnhancedSearchResult>> {
        if candidates.is_empty() {
            println!("🔍 Reranker: No candidates to rerank");
            return Ok(Vec::new());
        }
        
        println!("🔍 Reranker: Processing {} candidates", candidates.len());
        
        // Prepare documents for reranking
        let documents: Vec<String> = candidates.iter()
            .map(|c| self.prepare_document_for_reranking(&c.entry))
            .collect();
        
        println!("🔍 Reranker: Prepared {} documents for reranking", documents.len());
        for (i, doc) in documents.iter().enumerate() {
            println!("🔍 Document {}: {} chars", i, doc.len());
        }
        
        // Get reranking scores
        println!("🔍 Reranker: Calling rank_documents with query: '{}'", query);
        let query_clone = query.to_string();
        let documents_clone = documents.clone();
        let reranker_plugin = Arc::clone(&self.reranker_plugin);
        
        let rerank_results = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let reranker = reranker_plugin.read();
                reranker.rank_documents(&query_clone, &documents_clone).await
            })
        }).await??;
        println!("🔍 Reranker: Got {} rerank results", rerank_results.len());
        
        // Combine LSH similarity with reranking scores
        let mut enhanced_results = Vec::new();
        
        for (candidate_idx, rerank_score) in rerank_results {
            println!("🔍 Processing rerank result: candidate_idx={}, rerank_score={:.6}", candidate_idx, rerank_score);
            
            if candidate_idx < candidates.len() {
                let candidate = &candidates[candidate_idx];
                
                println!("🔍 Candidate {}: embedding_similarity={:.6}", candidate_idx, candidate.similarity);
                
                // Calculate combined score
                let combined_score = self.calculate_combined_score(
                    candidate.similarity,
                    rerank_score,
                );
                
                // Calculate confidence based on agreement between methods
                let confidence = self.calculate_confidence(
                    candidate.similarity,
                    rerank_score,
                );
                
                println!("🔍 Calculated scores: combined={:.6}, confidence={:.6}", combined_score, confidence);
                
                enhanced_results.push(EnhancedSearchResult {
                    entry: candidate.entry.clone(),
                    embedding_similarity: candidate.similarity,
                    rerank_score,
                    combined_score,
                    confidence,
                });
            }
        }
        
        // Sort by combined score
        enhanced_results.sort_by(|a, b| {
            b.combined_score.partial_cmp(&a.combined_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(enhanced_results)
    }
    
    /// Prepare document text for reranking
    fn prepare_document_for_reranking(&self, entry: &VectorEntry) -> String {
        let mut doc = String::new();
        
        // Add function name if available
        if let Some(ref func_name) = entry.metadata.function_name {
            doc.push_str(&format!("Function: {}\n", func_name));
        }
        
        // Add file context
        doc.push_str(&format!("File: {}\n", entry.metadata.file_path));
        doc.push_str(&format!("Language: {}\n", entry.metadata.language));
        doc.push_str(&format!("Type: {:?}\n", entry.metadata.code_type));
        
        // Add tokens as context
        if !entry.metadata.tokens.is_empty() {
            doc.push_str("Context: ");
            doc.push_str(&entry.metadata.tokens.join(" "));
        }
        
        doc
    }
    
    /// Calculate combined score from embedding similarity and rerank score
    fn calculate_combined_score(&self, embedding_sim: f32, rerank_score: f32) -> f32 {
        // Weighted average with slight preference for reranking
        let embedding_weight = 0.4;
        let rerank_weight = 0.6;
        
        (embedding_sim * embedding_weight) + (rerank_score * rerank_weight)
    }
    
    /// Calculate confidence based on agreement between methods
    fn calculate_confidence(&self, embedding_sim: f32, rerank_score: f32) -> f32 {
        // High confidence when both methods agree
        let agreement = 1.0 - (embedding_sim - rerank_score).abs();
        let base_quality = (embedding_sim + rerank_score) / 2.0;
        
        // Combine agreement and base quality
        (agreement * 0.3) + (base_quality * 0.7)
    }
    
    /// Apply final filtering and result limiting
    async fn finalize_results(&self, mut results: Vec<EnhancedSearchResult>, query: &SearchQuery) -> Result<Vec<EnhancedSearchResult>> {
        println!("🔍 Finalize: Starting with {} results", results.len());
        println!("🔍 Finalize: Rerank threshold = {:.6}", self.config.rerank_threshold);
        
        // Show all scores before filtering
        for (i, result) in results.iter().enumerate() {
            println!("🔍 Result {}: rerank_score={:.6}, combined_score={:.6}", 
                     i, result.rerank_score, result.combined_score);
        }
        
        // Filter by rerank threshold
        let before_filter = results.len();
        results.retain(|r| r.rerank_score >= self.config.rerank_threshold);
        println!("🔍 Finalize: After rerank threshold filter: {} -> {} results", before_filter, results.len());
        
        // Apply max results limit
        let max_results = query.max_results.unwrap_or(self.config.final_results);
        results.truncate(max_results);
        
        // Log result quality metrics
        if !results.is_empty() {
            let avg_combined_score: f32 = results.iter().map(|r| r.combined_score).sum::<f32>() / results.len() as f32;
            let avg_confidence: f32 = results.iter().map(|r| r.confidence).sum::<f32>() / results.len() as f32;
            
            debug!("Result quality - Avg Combined Score: {:.3}, Avg Confidence: {:.3}", 
                   avg_combined_score, avg_confidence);
        }
        
        Ok(results)
    }
    
    /// Get pipeline statistics
    pub async fn get_stats(&self) -> Result<SemanticSearchStats> {
        let vector_db = self.vector_db.read();
        let db_stats = vector_db.stats();
        
        let embedding_plugin = self.embedding_plugin.read();
        let (embedding_hits, embedding_total) = embedding_plugin.get_cache_stats();
        
        let reranker_plugin = self.reranker_plugin.read();
        let (rerank_hits, rerank_total) = reranker_plugin.get_cache_stats();
        
        Ok(SemanticSearchStats {
            total_vectors: db_stats.total_vectors,
            embedding_cache_hit_rate: if embedding_total > 0 { 
                embedding_hits as f64 / embedding_total as f64 
            } else { 0.0 },
            rerank_cache_hit_rate: if rerank_total > 0 { 
                rerank_hits as f64 / rerank_total as f64 
            } else { 0.0 },
            average_candidates_per_query: 0.0, // TODO: Track this
            average_rerank_improvement: 0.0,   // TODO: Track this
        })
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: SemanticSearchConfig) {
        self.config = config;
        info!("Updated semantic search configuration");
    }
}

/// Semantic search statistics
#[derive(Clone, Debug)]
pub struct SemanticSearchStats {
    pub total_vectors: usize,
    pub embedding_cache_hit_rate: f64,
    pub rerank_cache_hit_rate: f64,
    pub average_candidates_per_query: f64,
    pub average_rerank_improvement: f64,
}

/// Factory for creating semantic search pipelines
pub struct SemanticSearchFactory;

impl SemanticSearchFactory {
    /// Create a new semantic search pipeline
    pub fn create_pipeline(
        vector_db: Arc<RwLock<dyn VectorDatabase>>,
        embedding_plugin: Arc<RwLock<QwenEmbeddingPlugin>>,
        reranker_plugin: Arc<RwLock<QwenRerankerPlugin>>,
    ) -> SemanticSearchPipeline {
        SemanticSearchPipeline::new(
            vector_db,
            embedding_plugin,
            reranker_plugin,
            SemanticSearchConfig::default(),
        )
    }
    
    /// Create with custom configuration
    pub fn create_with_config(
        vector_db: Arc<RwLock<dyn VectorDatabase>>,
        embedding_plugin: Arc<RwLock<QwenEmbeddingPlugin>>,
        reranker_plugin: Arc<RwLock<QwenRerankerPlugin>>,
        config: SemanticSearchConfig,
    ) -> SemanticSearchPipeline {
        SemanticSearchPipeline::new(
            vector_db,
            embedding_plugin,
            reranker_plugin,
            config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::vector_db::{VectorStoreFactory, VectorDBConfig};
    use crate::ml::config::MLConfig;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_semantic_search_pipeline() {
        // Create temporary directory for test
        let temp_dir = TempDir::new().unwrap();
        
        // Create components
        let ml_config = MLConfig::for_testing();
        let vector_db_config = VectorDBConfig {
            cache_dir: temp_dir.path().to_string_lossy().to_string(),
            ..VectorDBConfig::default()
        };
        
        let vector_db = VectorStoreFactory::create_native(vector_db_config);
        let embedding_plugin = Arc::new(RwLock::new(QwenEmbeddingPlugin::new()));
        let reranker_plugin = Arc::new(RwLock::new(QwenRerankerPlugin::new()));
        
        // Create pipeline
        let pipeline = SemanticSearchFactory::create_pipeline(
            vector_db,
            embedding_plugin,
            reranker_plugin,
        );
        
        // Test search (will be empty but should not fail)
        let query = SearchQuery {
            text: "function calculateSum".to_string(),
            code_type: Some(CodeType::Function),
            language: Some("typescript".to_string()),
            file_context: None,
            max_results: Some(5),
        };
        
        // Should fail when ML plugins are not loaded
        let result = pipeline.search(&query).await;
        assert!(result.is_err()); // Expect error when embedding plugin is not loaded
        
        // Test stats
        let stats = pipeline.get_stats().await.unwrap();
        assert_eq!(stats.total_vectors, 0);
    }
    
    #[test]
    fn test_combined_score_calculation() {
        let pipeline = SemanticSearchFactory::create_pipeline(
            Arc::new(RwLock::new(crate::ml::vector_db::vector_store::NativeVectorStore::new(VectorDBConfig::default()))),
            Arc::new(RwLock::new(QwenEmbeddingPlugin::new())),
            Arc::new(RwLock::new(QwenRerankerPlugin::new())),
        );
        
        // Test score combination
        let combined = pipeline.calculate_combined_score(0.8, 0.9);
        assert!(combined > 0.8);
        assert!(combined < 0.9);
        
        // Test confidence calculation
        let confidence = pipeline.calculate_confidence(0.8, 0.85);
        assert!(confidence > 0.8);
    }
}
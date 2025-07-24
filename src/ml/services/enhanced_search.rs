/*! Enhanced Search Service
 * High-level service integrating the complete ML search pipeline
 */

use crate::ml::{
    MLConfig,
    plugins::{QwenEmbeddingPlugin, QwenRerankerPlugin, MLPlugin},
    vector_db::{
        VectorDatabase, VectorStoreFactory, VectorDBConfig, VectorEntry,
        SemanticSearchPipeline, SemanticSearchFactory, SearchQuery, 
        EnhancedSearchResult, SemanticSearchConfig, CodeType, CodeMetadata
    },
};
use anyhow::Result;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Enhanced search service combining all ML components
pub struct EnhancedSearchService {
    /// Semantic search pipeline
    search_pipeline: SemanticSearchPipeline,
    /// Vector database
    vector_db: Arc<RwLock<dyn VectorDatabase>>,
    /// Configuration
    config: MLConfig,
}

/// Search request with rich context
#[derive(Clone, Debug)]
pub struct SearchRequest {
    pub query: String,
    pub search_type: SearchType,
    pub filters: SearchFilters,
    pub options: SearchOptions,
}

/// Types of searches supported
#[derive(Clone, Debug)]
pub enum SearchType {
    /// Search for similar code snippets
    SimilarCode { language: String },
    /// Search for functions with similar functionality
    SimilarFunctions,
    /// Search for components with similar patterns
    SimilarComponents { framework: String },
    /// General semantic search
    General,
    /// Search within specific file context
    FileContext { file_path: String },
}

/// Search filters
#[derive(Clone, Debug, Default)]
pub struct SearchFilters {
    pub languages: Option<Vec<String>>,
    pub code_types: Option<Vec<CodeType>>,
    pub file_patterns: Option<Vec<String>>,
    pub exclude_files: Option<Vec<String>>,
    pub min_complexity: Option<f32>,
    pub max_complexity: Option<f32>,
}

/// Search options
#[derive(Clone, Debug)]
pub struct SearchOptions {
    pub max_results: usize,
    pub include_metadata: bool,
    pub explain_ranking: bool,
    pub use_cache: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_results: 10,
            include_metadata: true,
            explain_ranking: false,
            use_cache: true,
        }
    }
}

/// Enhanced search response
#[derive(Clone, Debug)]
pub struct SearchResponse {
    pub results: Vec<EnhancedSearchResult>,
    pub total_candidates: usize,
    pub search_time_ms: u64,
    pub explanation: Option<String>,
    pub suggestions: Vec<String>,
}

impl EnhancedSearchService {
    /// Create new enhanced search service
    pub async fn new(config: MLConfig) -> Result<Self> {
        info!("Initializing Enhanced Search Service");
        
        // Create vector database
        let vector_db_config = VectorDBConfig {
            cache_dir: format!("{}/.cache/vector-db", 
                              std::env::current_dir()?.to_string_lossy()),
            similarity_threshold: 0.1, // Lower threshold for better recall with dummy embeddings
            ..VectorDBConfig::default()
        };
        let vector_db = VectorStoreFactory::create_native(vector_db_config);
        
        // Initialize plugins
        let embedding_plugin = Arc::new(RwLock::new(QwenEmbeddingPlugin::new()));
        let reranker_plugin = Arc::new(RwLock::new(QwenRerankerPlugin::new()));
        
        // Load the plugins
        println!("🔧 Loading ML plugins from: {}", config.model_cache_dir.display());
        
        println!("📥 Loading Qwen Embedding plugin...");
        match embedding_plugin.write().load(&config).await {
            Ok(_) => println!("✅ Qwen Embedding plugin loaded successfully"),
            Err(e) => {
                println!("⚠️  Failed to load Qwen Embedding plugin: {}", e);
                println!("   Semantic search will use fallback mode");
            }
        }
        
        println!("📥 Loading Qwen Reranker plugin...");
        match reranker_plugin.write().load(&config).await {
            Ok(_) => println!("✅ Qwen Reranker plugin loaded successfully"),
            Err(e) => {
                println!("⚠️  Failed to load Qwen Reranker plugin: {}", e);
                println!("   Reranking will use fallback mode");
            }
        }
        
        // Create semantic search pipeline
        let search_config = SemanticSearchConfig {
            lsh_candidates: 50,
            final_results: 20,
            lsh_threshold: 0.3,
            rerank_threshold: 0.6,
            enable_caching: true,
            embedding_cache_size: 1000,
        };
        
        let search_pipeline = SemanticSearchFactory::create_with_config(
            vector_db.clone(),
            embedding_plugin,
            reranker_plugin,
            search_config,
        );
        
        Ok(Self {
            search_pipeline,
            vector_db,
            config,
        })
    }
    
    /// Perform enhanced search
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse> {
        let start_time = std::time::Instant::now();
        println!("🔍 Performing enhanced search: {:?}", request.search_type);
        println!("🔍 Query: '{}'", request.query);
        
        // Check vector DB stats before search
        let vector_db = self.vector_db.read();
        let stats = vector_db.stats();
        println!("📊 Vector DB stats before search:");
        println!("   Total vectors: {}", stats.total_vectors);
        println!("   Total files: {}", stats.total_files);
        drop(vector_db); // Release the read lock
        
        // Convert request to internal query
        let query = self.build_search_query(&request)?;
        println!("🔍 Built internal query: {:?}", query);
        
        // Perform search
        println!("🔍 Executing search pipeline...");
        let results = self.search_pipeline.search(&query).await?;
        println!("🔍 Search pipeline returned {} results", results.len());
        
        // Apply additional filtering
        let filtered_results = self.apply_filters(results, &request.filters).await?;
        
        // Generate response
        let search_time_ms = start_time.elapsed().as_millis() as u64;
        let explanation = if request.options.explain_ranking {
            Some(self.generate_explanation(&filtered_results))
        } else {
            None
        };
        
        let suggestions = self.generate_suggestions(&request, &filtered_results).await?;
        
        Ok(SearchResponse {
            total_candidates: filtered_results.len(),
            results: filtered_results,
            search_time_ms,
            explanation,
            suggestions,
        })
    }
    
    /// Add code to the search index
    pub async fn index_code(&self, code_entries: Vec<CodeIndexEntry>) -> Result<usize> {
        println!("📝 Indexing {} code entries", code_entries.len());
        
        let mut indexed_count = 0;
        let mut vector_db = self.vector_db.write();
        
        for (i, entry) in code_entries.into_iter().enumerate() {
            println!("📝 Processing entry {}: {}", i + 1, entry.file_path);
            match self.create_vector_entry(entry).await {
                Ok(vector_entry) => {
                    println!("✅ Created vector entry with ID: {}", vector_entry.id);
                    vector_db.add_vector(vector_entry)?;
                    indexed_count += 1;
                    println!("✅ Added to vector DB, total indexed: {}", indexed_count);
                }
                Err(e) => {
                    println!("❌ Failed to create vector entry: {}", e);
                }
            }
        }
        
        // Save to disk
        println!("💾 Saving vector database to disk...");
        vector_db.save()?;
        
        // Check database stats
        let stats = vector_db.stats();
        println!("📊 Vector DB stats after indexing:");
        println!("   Total vectors: {}", stats.total_vectors);
        println!("   Total files: {}", stats.total_files);
        println!("   Index size: {:.2}MB", stats.index_size_mb);
        
        println!("✅ Successfully indexed {} entries", indexed_count);
        Ok(indexed_count)
    }
    
    /// Remove code from index
    pub async fn remove_from_index(&self, file_path: &str) -> Result<usize> {
        info!("Removing entries for file: {}", file_path);
        
        let mut vector_db = self.vector_db.write();
        let entries = vector_db.get_by_file(file_path)?;
        let remove_count = entries.len();
        
        for entry in entries {
            vector_db.delete(&entry.id)?;
        }
        
        vector_db.save()?;
        
        info!("Removed {} entries for file: {}", remove_count, file_path);
        Ok(remove_count)
    }
    
    /// Update index for changed files
    pub async fn update_index(&self, file_path: &str, code_entries: Vec<CodeIndexEntry>) -> Result<usize> {
        // Remove old entries
        self.remove_from_index(file_path).await?;
        
        // Add new entries
        self.index_code(code_entries).await
    }
    
    /// Get search statistics
    pub async fn get_stats(&self) -> Result<SearchServiceStats> {
        let pipeline_stats = self.search_pipeline.get_stats().await?;
        let vector_db = self.vector_db.read();
        let db_stats = vector_db.stats();
        
        Ok(SearchServiceStats {
            total_indexed_entries: db_stats.total_vectors,
            total_files: db_stats.total_files,
            index_size_mb: db_stats.index_size_mb,
            embedding_cache_hit_rate: pipeline_stats.embedding_cache_hit_rate,
            rerank_cache_hit_rate: pipeline_stats.rerank_cache_hit_rate,
            languages: db_stats.by_language.clone(),
            code_types: db_stats.by_code_type.clone(),
        })
    }
    
    /// Build internal search query from request
    fn build_search_query(&self, request: &SearchRequest) -> Result<SearchQuery> {
        let (code_type, language) = match &request.search_type {
            SearchType::SimilarCode { language } => (None, Some(language.clone())),
            SearchType::SimilarFunctions => (Some(CodeType::Function), None),
            SearchType::SimilarComponents { framework } => (Some(CodeType::Component), Some(framework.clone())),
            SearchType::General => (None, None),
            SearchType::FileContext { file_path } => {
                // Extract language from file extension
                let language = self.extract_language_from_path(file_path);
                (None, language)
            }
        };
        
        Ok(SearchQuery {
            text: request.query.clone(),
            code_type,
            language,
            file_context: match &request.search_type {
                SearchType::FileContext { file_path } => Some(file_path.clone()),
                _ => None,
            },
            max_results: Some(request.options.max_results),
        })
    }
    
    /// Apply additional filters to results
    async fn apply_filters(&self, mut results: Vec<EnhancedSearchResult>, filters: &SearchFilters) -> Result<Vec<EnhancedSearchResult>> {
        results.retain(|result| {
            // Language filter
            if let Some(ref languages) = filters.languages {
                if !languages.iter().any(|lang| {
                    result.entry.metadata.language.eq_ignore_ascii_case(lang)
                }) {
                    return false;
                }
            }
            
            // Code type filter
            if let Some(ref code_types) = filters.code_types {
                if !code_types.contains(&result.entry.metadata.code_type) {
                    return false;
                }
            }
            
            // File pattern filter
            if let Some(ref patterns) = filters.file_patterns {
                if !patterns.iter().any(|pattern| {
                    result.entry.metadata.file_path.contains(pattern)
                }) {
                    return false;
                }
            }
            
            // Exclude files filter
            if let Some(ref exclude_patterns) = filters.exclude_files {
                if exclude_patterns.iter().any(|pattern| {
                    result.entry.metadata.file_path.contains(pattern)
                }) {
                    return false;
                }
            }
            
            // Complexity filters
            if let Some(min_complexity) = filters.min_complexity {
                if result.entry.metadata.complexity < min_complexity {
                    return false;
                }
            }
            
            if let Some(max_complexity) = filters.max_complexity {
                if result.entry.metadata.complexity > max_complexity {
                    return false;
                }
            }
            
            true
        });
        
        Ok(results)
    }
    
    /// Generate explanation for ranking
    fn generate_explanation(&self, results: &[EnhancedSearchResult]) -> String {
        if results.is_empty() {
            return "No results found.".to_string();
        }
        
        let mut explanation = String::new();
        explanation.push_str("Search Results Explanation:\n\n");
        
        for (i, result) in results.iter().take(3).enumerate() {
            explanation.push_str(&format!(
                "Result #{}: {}\n",
                i + 1,
                result.entry.metadata.file_path
            ));
            explanation.push_str(&format!(
                "  - Embedding Similarity: {:.3}\n",
                result.embedding_similarity
            ));
            explanation.push_str(&format!(
                "  - Rerank Score: {:.3}\n",
                result.rerank_score
            ));
            explanation.push_str(&format!(
                "  - Combined Score: {:.3}\n",
                result.combined_score
            ));
            explanation.push_str(&format!(
                "  - Confidence: {:.3}\n\n",
                result.confidence
            ));
        }
        
        explanation
    }
    
    /// Generate search suggestions
    async fn generate_suggestions(&self, request: &SearchRequest, results: &[EnhancedSearchResult]) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();
        
        if results.is_empty() {
            suggestions.push("Try using different keywords".to_string());
            suggestions.push("Check spelling and syntax".to_string());
            suggestions.push("Use more general terms".to_string());
        } else if results.len() < 3 {
            suggestions.push("Try broader search terms".to_string());
            suggestions.push("Remove some filters".to_string());
        }
        
        // Add language-specific suggestions
        if let Some(ref languages) = request.filters.languages {
            if languages.len() == 1 {
                suggestions.push(format!("Try searching in other languages besides {}", languages[0]));
            }
        }
        
        Ok(suggestions)
    }
    
    /// Create vector entry from code index entry
    async fn create_vector_entry(&self, code_entry: CodeIndexEntry) -> Result<VectorEntry> {
        // Use real embedding model to generate embedding
        let embedding = self.generate_real_embedding(&code_entry.content).await?;
        
        // Create metadata
        let metadata = CodeMetadata {
            file_path: code_entry.file_path,
            function_name: code_entry.function_name,
            line_start: code_entry.line_start,
            line_end: code_entry.line_end,
            code_type: code_entry.code_type,
            language: code_entry.language,
            complexity: code_entry.complexity,
            tokens: self.extract_tokens(&code_entry.content),
            hash: self.calculate_content_hash(&code_entry.content),
        };
        
        // Create vector entry
        let entry = VectorEntry {
            id: format!("{}:{}:{}", metadata.file_path, metadata.line_start, metadata.line_end),
            embedding,
            metadata,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        Ok(entry)
    }
    
    /// Extract language from file path
    fn extract_language_from_path(&self, file_path: &str) -> Option<String> {
        let path = Path::new(file_path);
        match path.extension()?.to_str()? {
            "ts" => Some("typescript".to_string()),
            "js" => Some("javascript".to_string()),
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "java" => Some("java".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "go" => Some("go".to_string()),
            _ => None,
        }
    }
    
    /// Extract tokens from code content
    fn extract_tokens(&self, content: &str) -> Vec<String> {
        // Simple tokenization - in production, use proper language-aware tokenizer
        content
            .split_whitespace()
            .filter(|token| token.len() > 2)
            .take(50) // Limit tokens
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Calculate content hash
    fn calculate_content_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Generate real embedding using Qwen model
    async fn generate_real_embedding(&self, content: &str) -> Result<Vec<f32>> {
        // Use the semantic search pipeline's embedding generation method
        println!("🤖 Generating real embedding for content: {} chars", content.len());
        
        match self.search_pipeline.generate_query_embedding(content).await {
            Ok(embedding) => {
                println!("✅ Generated real embedding with {} dimensions", embedding.len());
                Ok(embedding)
            }
            Err(e) => {
                println!("⚠️  Failed to generate real embedding: {}", e);
                println!("   Falling back to dummy embedding");
                Ok(self.create_dummy_embedding_fallback(content))
            }
        }
    }
    
    /// Fallback dummy embedding for testing (only used if real model fails)
    fn create_dummy_embedding_fallback(&self, content: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Generate deterministic embedding from hash
        let mut embedding = Vec::with_capacity(768);
        let mut seed = hash;
        
        for _ in 0..768 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let val = ((seed / 65536) % 32768) as f32 / 32768.0 - 0.5;
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
}

/// Code entry for indexing
#[derive(Clone, Debug)]
pub struct CodeIndexEntry {
    pub file_path: String,
    pub function_name: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub code_type: CodeType,
    pub language: String,
    pub complexity: f32,
    pub content: String,
}

/// Search service statistics
#[derive(Clone, Debug)]
pub struct SearchServiceStats {
    pub total_indexed_entries: usize,
    pub total_files: usize,
    pub index_size_mb: f64,
    pub embedding_cache_hit_rate: f64,
    pub rerank_cache_hit_rate: f64,
    pub languages: std::collections::HashMap<String, usize>,
    pub code_types: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_enhanced_search_service() {
        let config = MLConfig::for_testing();
        let service = EnhancedSearchService::new(config).await.unwrap();
        
        // Test empty search
        let request = SearchRequest {
            query: "function test".to_string(),
            search_type: SearchType::General,
            filters: SearchFilters::default(),
            options: SearchOptions::default(),
        };
        
        let response = service.search(request).await.unwrap();
        assert!(response.results.is_empty());
        assert!(response.search_time_ms > 0);
    }
    
    #[tokio::test]
    async fn test_code_indexing() {
        let config = MLConfig::for_testing();
        let service = EnhancedSearchService::new(config).await.unwrap();
        
        let code_entries = vec![
            CodeIndexEntry {
                file_path: "test.ts".to_string(),
                function_name: Some("testFunction".to_string()),
                line_start: 1,
                line_end: 10,
                code_type: CodeType::Function,
                language: "typescript".to_string(),
                complexity: 1.0,
                content: "function testFunction() { return 42; }".to_string(),
            }
        ];
        
        let indexed = service.index_code(code_entries).await.unwrap();
        assert_eq!(indexed, 1);
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.total_indexed_entries, 1);
    }
}
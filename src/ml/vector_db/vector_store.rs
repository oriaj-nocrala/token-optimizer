/*! Vector Store Implementation
 * Native vector database with LSH indexing for fast similarity search
 */

use super::*;
use crate::ml::vector_db::{
    lsh_index::{LSHIndex, LSHConfig},
    similarity::{CosineSimilarity, SimilarityMetric},
};
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Native vector store implementation
pub struct NativeVectorStore {
    /// Vector entries by ID
    vectors: RwLock<HashMap<String, VectorEntry>>,
    /// LSH index for fast search
    lsh_index: RwLock<LSHIndex>,
    /// Similarity metric
    similarity_metric: Box<dyn SimilarityMetric>,
    /// Configuration
    config: VectorDBConfig,
    /// File index for quick lookups
    file_index: RwLock<HashMap<String, Vec<String>>>,
    /// Statistics
    stats: RwLock<VectorDBStats>,
}

impl NativeVectorStore {
    /// Create new vector store
    pub fn new(config: VectorDBConfig) -> Self {
        let lsh_config = LSHConfig {
            num_hash_functions: config.num_hash_functions,
            hash_bits: 5,  // Further reduced from 6 to 5 for more hash collisions
            num_tables: 12, // Increased from 8 to 12 for better coverage
            seed: 42,
        };
        
        // Initialize with 768 dimensions (standard embedding size)
        let lsh_index = LSHIndex::new(768, lsh_config);
        
        let stats = VectorDBStats {
            total_vectors: 0,
            total_files: 0,
            index_size_mb: 0.0,
            average_similarity: 0.0,
            by_language: HashMap::new(),
            by_code_type: HashMap::new(),
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        Self {
            vectors: RwLock::new(HashMap::new()),
            lsh_index: RwLock::new(lsh_index),
            similarity_metric: Box::new(CosineSimilarity),
            config,
            file_index: RwLock::new(HashMap::new()),
            stats: RwLock::new(stats),
        }
    }
    
    /// Create vector store with custom similarity metric
    pub fn with_similarity_metric(
        config: VectorDBConfig,
        metric: Box<dyn SimilarityMetric>,
    ) -> Self {
        let mut store = Self::new(config);
        store.similarity_metric = metric;
        store
    }
    
    /// Rebuild LSH index (useful after bulk operations)
    pub fn rebuild_index(&self) -> Result<()> {
        let vectors = self.vectors.read();
        let mut index = self.lsh_index.write();
        
        index.clear();
        
        for (id, entry) in vectors.iter() {
            index.add(id.clone(), &entry.embedding)?;
        }
        
        // Update stats
        self.update_stats();
        
        Ok(())
    }
    
    /// Get embedding for code using the ML pipeline
    async fn get_embedding_for_code(&self, code: &str) -> Result<Vec<f32>> {
        // TODO: Integration with QwenEmbeddingPlugin
        // For now, create a dummy embedding based on code hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        code.hash(&mut hasher);
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
        
        Ok(embedding)
    }
    
    /// Update internal statistics
    fn update_stats(&self) {
        let vectors = self.vectors.read();
        let file_index = self.file_index.read();
        let mut stats = self.stats.write();
        
        stats.total_vectors = vectors.len();
        stats.total_files = file_index.len();
        stats.last_updated = chrono::Utc::now();
        
        // Estimate index size (rough approximation)
        stats.index_size_mb = (vectors.len() * 768 * 4) as f64 / 1024.0 / 1024.0;
        
        // Language and type statistics
        stats.by_language.clear();
        stats.by_code_type.clear();
        
        for entry in vectors.values() {
            *stats.by_language.entry(entry.metadata.language.clone()).or_insert(0) += 1;
            let type_name = format!("{:?}", entry.metadata.code_type);
            *stats.by_code_type.entry(type_name).or_insert(0) += 1;
        }
    }
    
    /// Compute average similarity for a sample of vectors
    fn compute_average_similarity(&self) -> f32 {
        let vectors = self.vectors.read();
        let vec_list: Vec<_> = vectors.values().collect();
        
        if vec_list.len() < 2 {
            return 0.0;
        }
        
        let sample_size = (vec_list.len().min(100)).max(2);
        let mut total_similarity = 0.0;
        let mut count = 0;
        
        for i in 0..sample_size {
            for j in (i + 1)..sample_size {
                if let Ok(sim) = self.similarity_metric.similarity(
                    &vec_list[i].embedding,
                    &vec_list[j].embedding,
                ) {
                    total_similarity += sim;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            total_similarity / count as f32
        } else {
            0.0
        }
    }
}

impl VectorDatabase for NativeVectorStore {
    fn add_vector(&mut self, entry: VectorEntry) -> Result<()> {
        let id = entry.id.clone();
        let file_path = entry.metadata.file_path.clone();
        
        // Add to LSH index
        {
            let mut index = self.lsh_index.write();
            index.add(id.clone(), &entry.embedding)?;
        }
        
        // Add to vectors
        {
            let mut vectors = self.vectors.write();
            vectors.insert(id.clone(), entry);
        }
        
        // Update file index
        {
            let mut file_index = self.file_index.write();
            file_index.entry(file_path).or_insert_with(Vec::new).push(id);
        }
        
        self.update_stats();
        Ok(())
    }
    
    fn add_vectors(&mut self, entries: Vec<VectorEntry>) -> Result<()> {
        for entry in entries {
            self.add_vector(entry)?;
        }
        Ok(())
    }
    
    fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        // Get candidates from LSH index
        let candidates = {
            let index = self.lsh_index.read();
            index.search_candidates(query_embedding)?
        };
        
        // Compute exact similarities for candidates
        let vectors = self.vectors.read();
        let mut results = Vec::new();
        
        for candidate_id in candidates {
            if let Some(entry) = vectors.get(&candidate_id) {
                let similarity = self.similarity_metric.similarity(
                    query_embedding,
                    &entry.embedding,
                )?;
                
                if similarity >= self.config.similarity_threshold {
                    let distance = self.similarity_metric.distance(
                        query_embedding,
                        &entry.embedding,
                    )?;
                    
                    results.push(SearchResult {
                        entry: entry.clone(),
                        similarity,
                        distance,
                    });
                }
            }
        }
        
        // Sort by similarity (descending)
        results.sort_by(|a, b| {
            b.similarity.partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Limit results
        results.truncate(limit.min(self.config.max_results));
        
        Ok(results)
    }
    
    fn search_by_code(&self, code: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // This would normally use the ML pipeline to generate embeddings
        // For now, we'll create a simple hash-based embedding
        let query_embedding = tokio::runtime::Handle::current()
            .block_on(self.get_embedding_for_code(code))?;
        
        self.search(&query_embedding, limit)
    }
    
    fn get_by_id(&self, id: &str) -> Result<Option<VectorEntry>> {
        let vectors = self.vectors.read();
        Ok(vectors.get(id).cloned())
    }
    
    fn update_vector(&mut self, entry: VectorEntry) -> Result<()> {
        let id = entry.id.clone();
        
        // Remove old entry from index if it exists
        if let Some(old_entry) = self.vectors.read().get(&id) {
            let mut index = self.lsh_index.write();
            index.remove(&id, &old_entry.embedding)?;
        }
        
        // Add updated entry
        self.add_vector(entry)
    }
    
    fn delete(&mut self, id: &str) -> Result<bool> {
        let entry = {
            let mut vectors = self.vectors.write();
            vectors.remove(id)
        };
        
        if let Some(entry) = entry {
            // Remove from LSH index
            {
                let mut index = self.lsh_index.write();
                index.remove(id, &entry.embedding)?;
            }
            
            // Remove from file index
            {
                let mut file_index = self.file_index.write();
                if let Some(file_entries) = file_index.get_mut(&entry.metadata.file_path) {
                    file_entries.retain(|x| x != id);
                    if file_entries.is_empty() {
                        file_index.remove(&entry.metadata.file_path);
                    }
                }
            }
            
            self.update_stats();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn get_by_file(&self, file_path: &str) -> Result<Vec<VectorEntry>> {
        let file_index = self.file_index.read();
        let vectors = self.vectors.read();
        
        if let Some(ids) = file_index.get(file_path) {
            let mut entries = Vec::new();
            for id in ids {
                if let Some(entry) = vectors.get(id) {
                    entries.push(entry.clone());
                }
            }
            Ok(entries)
        } else {
            Ok(Vec::new())
        }
    }
    
    fn get_all_vectors(&self) -> Result<Vec<VectorEntry>> {
        let vectors = self.vectors.read();
        Ok(vectors.values().cloned().collect())
    }
    
    fn stats(&self) -> VectorDBStats {
        let mut stats = self.stats.read().clone();
        stats.average_similarity = self.compute_average_similarity();
        stats
    }
    
    fn save(&self) -> Result<()> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        let cache_dir = PathBuf::from(&self.config.cache_dir);
        std::fs::create_dir_all(&cache_dir)?;
        
        // Save vectors
        let vectors_path = cache_dir.join("vectors.json");
        let vectors = self.vectors.read();
        let vectors_json = serde_json::to_string_pretty(&*vectors)?;
        std::fs::write(vectors_path, vectors_json)?;
        
        // Save file index
        let file_index_path = cache_dir.join("file_index.json");
        let file_index = self.file_index.read();
        let file_index_json = serde_json::to_string_pretty(&*file_index)?;
        std::fs::write(file_index_path, file_index_json)?;
        
        // Save stats
        let stats_path = cache_dir.join("stats.json");
        let stats = self.stats.read();
        let stats_json = serde_json::to_string_pretty(&*stats)?;
        std::fs::write(stats_path, stats_json)?;
        
        Ok(())
    }
    
    fn load(&mut self) -> Result<()> {
        if !self.config.enable_persistence {
            return Ok(());
        }
        
        let cache_dir = PathBuf::from(&self.config.cache_dir);
        
        // Load vectors
        let vectors_path = cache_dir.join("vectors.json");
        if vectors_path.exists() {
            let vectors_json = std::fs::read_to_string(vectors_path)?;
            let vectors: HashMap<String, VectorEntry> = serde_json::from_str(&vectors_json)?;
            *self.vectors.write() = vectors;
        }
        
        // Load file index
        let file_index_path = cache_dir.join("file_index.json");
        if file_index_path.exists() {
            let file_index_json = std::fs::read_to_string(file_index_path)?;
            let file_index: HashMap<String, Vec<String>> = serde_json::from_str(&file_index_json)?;
            *self.file_index.write() = file_index;
        }
        
        // Load stats
        let stats_path = cache_dir.join("stats.json");
        if stats_path.exists() {
            let stats_json = std::fs::read_to_string(stats_path)?;
            let stats: VectorDBStats = serde_json::from_str(&stats_json)?;
            *self.stats.write() = stats;
        }
        
        // Rebuild LSH index
        self.rebuild_index()?;
        
        Ok(())
    }
    
    fn clear(&mut self) -> Result<()> {
        self.vectors.write().clear();
        self.file_index.write().clear();
        self.lsh_index.write().clear();
        
        let mut stats = self.stats.write();
        stats.total_vectors = 0;
        stats.total_files = 0;
        stats.index_size_mb = 0.0;
        stats.average_similarity = 0.0;
        stats.by_language.clear();
        stats.by_code_type.clear();
        stats.last_updated = chrono::Utc::now();
        
        Ok(())
    }
}

/// Factory for creating vector stores
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    /// Create a new native vector store
    pub fn create_native(config: VectorDBConfig) -> Arc<RwLock<dyn VectorDatabase>> {
        let store = NativeVectorStore::new(config);
        Arc::new(RwLock::new(store))
    }
    
    /// Create vector store with custom similarity metric
    pub fn create_with_metric(
        config: VectorDBConfig,
        metric: Box<dyn SimilarityMetric>,
    ) -> Arc<RwLock<dyn VectorDatabase>> {
        let store = NativeVectorStore::with_similarity_metric(config, metric);
        Arc::new(RwLock::new(store))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::vector_db::CodeType;
    
    fn create_test_entry(id: &str, embedding: Vec<f32>) -> VectorEntry {
        VectorEntry {
            id: id.to_string(),
            embedding,
            metadata: CodeMetadata {
                file_path: "test.ts".to_string(),
                function_name: Some("testFunction".to_string()),
                line_start: 1,
                line_end: 10,
                code_type: CodeType::Function,
                language: "typescript".to_string(),
                complexity: 1.0,
                tokens: vec!["test".to_string()],
                hash: "hash123".to_string(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    #[test]
    fn test_vector_store_basic_operations() {
        let config = VectorDBConfig::default();
        let mut store = NativeVectorStore::new(config);
        
        // Add vector
        let entry = create_test_entry("test1", vec![1.0; 768]);
        store.add_vector(entry.clone()).unwrap();
        
        // Get by ID
        let retrieved = store.get_by_id("test1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test1");
        
        // Search
        let results = store.search(&vec![1.0; 768], 10).unwrap();
        assert!(!results.is_empty());
        
        // Delete
        let deleted = store.delete("test1").unwrap();
        assert!(deleted);
        
        let retrieved_after_delete = store.get_by_id("test1").unwrap();
        assert!(retrieved_after_delete.is_none());
    }
    
    #[test]
    fn test_similarity_search() {
        let config = VectorDBConfig::default();
        let mut store = NativeVectorStore::new(config);
        
        // Add similar vectors
        let mut vec1 = vec![1.0; 768];
        let mut vec2 = vec![0.9; 768];
        let mut vec3 = vec![-1.0; 768];
        
        // Normalize vectors
        super::similarity::VectorNorm::l2_normalize(&mut vec1);
        super::similarity::VectorNorm::l2_normalize(&mut vec2);
        super::similarity::VectorNorm::l2_normalize(&mut vec3);
        
        store.add_vector(create_test_entry("similar1", vec1.clone())).unwrap();
        store.add_vector(create_test_entry("similar2", vec2.clone())).unwrap();
        store.add_vector(create_test_entry("different", vec3.clone())).unwrap();
        
        // Search for similar vectors
        let results = store.search(&vec1, 10).unwrap();
        
        // Should find similar vectors first
        assert!(!results.is_empty());
        if results.len() >= 2 {
            assert!(results[0].similarity >= results[1].similarity);
        }
    }
    
    #[test]
    fn test_file_based_operations() {
        let config = VectorDBConfig::default();
        let mut store = NativeVectorStore::new(config);
        
        // Add vectors from same file
        let mut entry1 = create_test_entry("func1", vec![1.0; 768]);
        let mut entry2 = create_test_entry("func2", vec![0.5; 768]);
        entry1.metadata.file_path = "file1.ts".to_string();
        entry2.metadata.file_path = "file1.ts".to_string();
        
        store.add_vector(entry1).unwrap();
        store.add_vector(entry2).unwrap();
        
        // Get by file
        let file_entries = store.get_by_file("file1.ts").unwrap();
        assert_eq!(file_entries.len(), 2);
        
        // Stats should reflect file count
        let stats = store.stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_vectors, 2);
    }
}
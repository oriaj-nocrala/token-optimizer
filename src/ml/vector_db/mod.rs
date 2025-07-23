/*! Vector Database with LSH Index
 * Native implementation optimized for code embeddings
 */

pub mod lsh_index;
pub mod vector_store;
pub mod similarity;
pub mod persistence;
pub mod semantic_search;

pub use vector_store::*;
pub use lsh_index::*;
pub use similarity::*;
pub use persistence::*;
pub use semantic_search::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector database configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorDBConfig {
    /// Number of hash functions for LSH
    pub num_hash_functions: usize,
    /// Number of bits per hash function
    pub hash_bits: usize,
    /// Similarity threshold for matches
    pub similarity_threshold: f32,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Enable persistence to disk
    pub enable_persistence: bool,
    /// Cache directory for vector index
    pub cache_dir: String,
}

impl Default for VectorDBConfig {
    fn default() -> Self {
        Self {
            num_hash_functions: 16,
            hash_bits: 10,
            similarity_threshold: 0.7,
            max_results: 50,
            enable_persistence: true,
            cache_dir: ".cache/vector-db".to_string(),
        }
    }
}

/// Code metadata for vector entries
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeMetadata {
    pub file_path: String,
    pub function_name: Option<String>,
    pub line_start: usize,
    pub line_end: usize,
    pub code_type: CodeType,
    pub language: String,
    pub complexity: f32,
    pub tokens: Vec<String>,
    pub hash: String,
}

/// Types of code snippets
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CodeType {
    Function,
    Class,
    Interface,
    Component,
    Service,
    Module,
    Test,
    Comment,
    Import,
    Config,
}

/// Vector database entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorEntry {
    pub id: String,
    pub embedding: Vec<f32>,
    pub metadata: CodeMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Search result with similarity score
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub entry: VectorEntry,
    pub similarity: f32,
    pub distance: f32,
}

/// Vector database interface
pub trait VectorDatabase: Send + Sync {
    /// Add a vector to the database
    fn add_vector(&mut self, entry: VectorEntry) -> Result<()>;
    
    /// Add multiple vectors in batch
    fn add_vectors(&mut self, entries: Vec<VectorEntry>) -> Result<()>;
    
    /// Search for similar vectors
    fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    
    /// Search by code content
    fn search_by_code(&self, code: &str, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// Get vector by ID
    fn get_by_id(&self, id: &str) -> Result<Option<VectorEntry>>;
    
    /// Update vector
    fn update_vector(&mut self, entry: VectorEntry) -> Result<()>;
    
    /// Delete vector by ID
    fn delete(&mut self, id: &str) -> Result<bool>;
    
    /// Get all vectors for a file
    fn get_by_file(&self, file_path: &str) -> Result<Vec<VectorEntry>>;
    
    /// Get all vectors in the database
    fn get_all_vectors(&self) -> Result<Vec<VectorEntry>>;
    
    /// Get statistics
    fn stats(&self) -> VectorDBStats;
    
    /// Save to disk
    fn save(&self) -> Result<()>;
    
    /// Load from disk
    fn load(&mut self) -> Result<()>;
    
    /// Clear all data
    fn clear(&mut self) -> Result<()>;
}

/// Database statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorDBStats {
    pub total_vectors: usize,
    pub total_files: usize,
    pub index_size_mb: f64,
    pub average_similarity: f32,
    pub by_language: HashMap<String, usize>,
    pub by_code_type: HashMap<String, usize>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
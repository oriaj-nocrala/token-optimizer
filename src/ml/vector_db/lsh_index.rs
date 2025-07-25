/*! LSH (Locality Sensitive Hashing) Index
 * Fast approximate nearest neighbor search for high-dimensional vectors
 */

use anyhow::Result;
use bit_vec::BitVec;
use fnv::FnvHashMap;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// LSH Index for fast similarity search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LSHIndex {
    /// Random projection matrices for hash functions
    hash_functions: Vec<Vec<Vec<f32>>>,
    /// Hash tables mapping hash values to vector IDs
    hash_tables: Vec<FnvHashMap<u64, Vec<String>>>,
    /// Configuration
    config: LSHConfig,
    /// Vector dimension
    dimension: usize,
}

/// LSH configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LSHConfig {
    pub num_hash_functions: usize,
    pub hash_bits: usize,
    pub num_tables: usize,
    pub seed: u64,
}

impl Default for LSHConfig {
    fn default() -> Self {
        Self {
            num_hash_functions: 16,
            hash_bits: 10,
            num_tables: 8,
            seed: 42,
        }
    }
}

impl LSHIndex {
    /// Create new LSH index
    pub fn new(dimension: usize, config: LSHConfig) -> Self {
        let mut rng = StdRng::seed_from_u64(config.seed);
        
        // Generate random projection matrices
        let mut hash_functions = Vec::new();
        for _ in 0..config.num_tables {
            let mut table_functions = Vec::new();
            for _ in 0..config.hash_bits {
                let mut projection = Vec::new();
                for _ in 0..dimension {
                    projection.push(rng.gen::<f32>() - 0.5);
                }
                // Normalize
                let norm: f32 = projection.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm > 0.0 {
                    for val in &mut projection {
                        *val /= norm;
                    }
                }
                table_functions.push(projection);
            }
            hash_functions.push(table_functions);
        }
        
        // Initialize hash tables
        let hash_tables = (0..config.num_tables)
            .map(|_| FnvHashMap::default())
            .collect();
        
        Self {
            hash_functions,
            hash_tables,
            config,
            dimension,
        }
    }
    
    /// Add vector to index
    pub fn add(&mut self, id: String, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimension {
            anyhow::bail!("Vector dimension mismatch: expected {}, got {}", 
                         self.dimension, vector.len());
        }
        
        // Adding vector to LSH index
        
        // Compute all hash values first to avoid borrowing issues
        let hash_values: Vec<u64> = (0..self.hash_tables.len())
            .map(|table_idx| self.compute_hash(vector, table_idx))
            .collect();
        
        // Hash values computed for all tables
        
        // Add to each hash table
        for (table_idx, hash_table) in self.hash_tables.iter_mut().enumerate() {
            let hash_value = hash_values[table_idx];
            let bucket = hash_table.entry(hash_value)
                .or_insert_with(Vec::new);
            bucket.push(id.clone());
            // Added to hash table bucket
        }
        
        Ok(())
    }
    
    /// Search for candidate vectors
    pub fn search_candidates(&self, query: &[f32]) -> Result<Vec<String>> {
        if query.len() != self.dimension {
            anyhow::bail!("Query dimension mismatch: expected {}, got {}", 
                         self.dimension, query.len());
        }
        
        // Searching LSH index for candidates
        
        let mut candidates = std::collections::HashSet::new();
        
        // Search in each hash table
        for (table_idx, hash_table) in self.hash_tables.iter().enumerate() {
            let hash_value = self.compute_hash(query, table_idx);
            println!("🔍 Table {}: computed hash = {}, table has {} buckets", 
                     table_idx, hash_value, hash_table.len());
            
            if let Some(ids) = hash_table.get(&hash_value) {
                println!("🔍 Table {}: found bucket with {} IDs", table_idx, ids.len());
                for id in ids {
                    candidates.insert(id.clone());
                }
            } else {
                println!("🔍 Table {}: no bucket found for hash {}", table_idx, hash_value);
                
                // Debug: Show what hashes actually exist in this table
                if hash_table.len() > 0 {
                    let existing_hashes: Vec<u64> = hash_table.keys().take(3).copied().collect();
                    println!("🔍 Table {}: existing hashes (sample): {:?}", table_idx, existing_hashes);
                }
            }
        }
        
        // Search completed
        Ok(candidates.into_iter().collect())
    }
    
    /// Remove vector from index
    pub fn remove(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        if vector.len() != self.dimension {
            anyhow::bail!("Vector dimension mismatch: expected {}, got {}", 
                         self.dimension, vector.len());
        }
        
        // Compute all hash values first to avoid borrowing issues
        let hash_values: Vec<u64> = (0..self.hash_tables.len())
            .map(|table_idx| self.compute_hash(vector, table_idx))
            .collect();
        
        // Remove from each hash table
        for (table_idx, hash_table) in self.hash_tables.iter_mut().enumerate() {
            let hash_value = hash_values[table_idx];
            
            if let Some(ids) = hash_table.get_mut(&hash_value) {
                ids.retain(|x| x != id);
                // Clean up empty buckets
                if ids.is_empty() {
                    hash_table.remove(&hash_value);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get index statistics
    pub fn stats(&self) -> LSHStats {
        let mut unique_vectors = std::collections::HashSet::new();
        let mut bucket_sizes = Vec::new();
        let mut non_empty_buckets = 0;
        
        for hash_table in &self.hash_tables {
            for bucket in hash_table.values() {
                if !bucket.is_empty() {
                    non_empty_buckets += 1;
                    bucket_sizes.push(bucket.len());
                    // Add unique vector IDs
                    for id in bucket {
                        unique_vectors.insert(id.clone());
                    }
                }
            }
        }
        
        let total_entries = unique_vectors.len();
        let average_bucket_size = if non_empty_buckets > 0 {
            bucket_sizes.iter().sum::<usize>() as f64 / non_empty_buckets as f64
        } else {
            0.0
        };
        
        bucket_sizes.sort_unstable();
        let median_bucket_size = if bucket_sizes.is_empty() {
            0
        } else {
            bucket_sizes[bucket_sizes.len() / 2]
        };
        
        LSHStats {
            total_vectors: total_entries,
            num_tables: self.config.num_tables,
            num_buckets: self.hash_tables.iter().map(|t| t.len()).sum(),
            non_empty_buckets,
            average_bucket_size,
            median_bucket_size,
            dimension: self.dimension,
            hash_bits: self.config.hash_bits,
        }
    }
    
    /// Clear the index
    pub fn clear(&mut self) {
        for hash_table in &mut self.hash_tables {
            hash_table.clear();
        }
    }
    
    /// Compute LSH hash for a vector in a specific table
    fn compute_hash(&self, vector: &[f32], table_idx: usize) -> u64 {
        let hash_functions = &self.hash_functions[table_idx];
        let mut hash_bits = BitVec::with_capacity(self.config.hash_bits);
        
        for projection in hash_functions {
            // Compute dot product
            let dot_product: f32 = vector.iter()
                .zip(projection.iter())
                .map(|(a, b)| a * b)
                .sum();
            
            // Hash bit is 1 if dot product >= 0, 0 otherwise
            hash_bits.push(dot_product >= 0.0);
        }
        
        // Convert bit vector to u64
        let mut hash_value = 0u64;
        for (i, bit) in hash_bits.iter().enumerate() {
            if bit && i < 64 {
                hash_value |= 1u64 << i;
            }
        }
        
        hash_value
    }
}

/// LSH index statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LSHStats {
    pub total_vectors: usize,
    pub num_tables: usize,
    pub num_buckets: usize,
    pub non_empty_buckets: usize,
    pub average_bucket_size: f64,
    pub median_bucket_size: usize,
    pub dimension: usize,
    pub hash_bits: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lsh_basic() {
        let config = LSHConfig::default();
        let mut index = LSHIndex::new(128, config);
        
        // Add some vectors
        let vec1 = vec![1.0; 128];
        let vec2 = vec![0.5; 128];
        let vec3 = vec![-1.0; 128];
        
        index.add("vec1".to_string(), &vec1).unwrap();
        index.add("vec2".to_string(), &vec2).unwrap();
        index.add("vec3".to_string(), &vec3).unwrap();
        
        // Search for similar vectors
        let candidates = index.search_candidates(&vec1).unwrap();
        assert!(!candidates.is_empty());
        
        let stats = index.stats();
        assert_eq!(stats.total_vectors, 3);
        assert!(stats.non_empty_buckets > 0);
    }
    
    #[test]
    fn test_lsh_similarity() {
        let config = LSHConfig::default();
        let mut index = LSHIndex::new(10, config);
        
        // Similar vectors should hash to similar buckets
        let vec1 = vec![1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let vec2 = vec![1.0, 0.9, 0.1, 0.0, 1.0, 0.0, 0.9, 0.1, 0.0, 1.0];
        let vec3 = vec![-1.0, -1.0, 0.0, 0.0, -1.0, 0.0, -1.0, 0.0, 0.0, -1.0];
        
        index.add("similar1".to_string(), &vec1).unwrap();
        index.add("similar2".to_string(), &vec2).unwrap();
        index.add("different".to_string(), &vec3).unwrap();
        
        let candidates = index.search_candidates(&vec1).unwrap();
        
        // Should find at least itself
        assert!(candidates.contains(&"similar1".to_string()));
    }
    
    #[test]
    fn test_lsh_remove() {
        let config = LSHConfig::default();
        let mut index = LSHIndex::new(64, config);
        
        let vector = vec![1.0; 64];
        index.add("test".to_string(), &vector).unwrap();
        
        let candidates_before = index.search_candidates(&vector).unwrap();
        assert!(candidates_before.contains(&"test".to_string()));
        
        index.remove("test", &vector).unwrap();
        
        let candidates_after = index.search_candidates(&vector).unwrap();
        assert!(!candidates_after.contains(&"test".to_string()));
    }
}
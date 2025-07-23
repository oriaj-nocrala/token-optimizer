//! ML response caching system for Layer 3 reliability

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use sha2::{Sha256, Digest};

/// ML response cache entry with prompt hash and response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLCacheEntry {
    /// SHA-256 hash of the prompt + model config
    pub prompt_hash: String,
    /// Model response (JSON string)
    pub response: String,
    /// Timestamp when cached
    pub cached_at: u64,
    /// Model used (deepseek, qwen_embedding, etc.)
    pub model_type: String,
    /// Hit count for cache eviction
    pub hit_count: u64,
}

/// ML response cache for preventing re-inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLResponseCache {
    /// Cache entries indexed by prompt hash
    pub entries: HashMap<String, MLCacheEntry>,
    /// Cache file path
    pub cache_file: PathBuf,
    /// Maximum cache size (entries)
    pub max_size: usize,
    /// Cache hit statistics
    pub stats: CacheStats,
}

/// Cache performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_size_bytes: usize,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            total_size_bytes: 0,
        }
    }
}

impl MLResponseCache {
    /// Create new ML response cache
    pub fn new(cache_dir: PathBuf, max_size: usize) -> Self {
        let cache_file = cache_dir.join("ml-response-cache.json");
        
        Self {
            entries: HashMap::new(),
            cache_file,
            max_size,
            stats: CacheStats::default(),
        }
    }

    /// Load cache from disk
    pub fn load(&mut self) -> Result<()> {
        if self.cache_file.exists() {
            let content = fs::read_to_string(&self.cache_file)?;
            let loaded: MLResponseCache = serde_json::from_str(&content)?;
            
            self.entries = loaded.entries;
            self.stats = loaded.stats;
            
            tracing::info!(
                "Loaded ML cache with {} entries, hit rate: {:.2}%",
                self.entries.len(),
                self.hit_rate() * 100.0
            );
        } else {
            tracing::info!("No existing ML cache found, starting fresh");
        }
        
        Ok(())
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.cache_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&self.cache_file, content)?;
        
        tracing::debug!("ML cache saved to: {}", self.cache_file.display());
        Ok(())
    }

    /// Generate prompt hash for caching
    pub fn generate_prompt_hash(prompt: &str, model_type: &str, config_hash: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(model_type.as_bytes());
        hasher.update(config_hash.as_bytes());
        
        format!("{:x}", hasher.finalize())
    }

    /// Get cached response if available
    pub fn get(&mut self, prompt_hash: &str) -> Option<String> {
        if let Some(entry) = self.entries.get_mut(prompt_hash) {
            entry.hit_count += 1;
            self.stats.hits += 1;
            
            tracing::debug!("ML cache HIT for hash: {}", &prompt_hash[..8]);
            Some(entry.response.clone())
        } else {
            self.stats.misses += 1;
            tracing::debug!("ML cache MISS for hash: {}", &prompt_hash[..8]);
            None
        }
    }

    /// Store response in cache
    pub fn put(&mut self, prompt_hash: String, response: String, model_type: String) -> Result<()> {
        // Check if cache is full and evict LRU entry
        if self.entries.len() >= self.max_size {
            self.evict_lru()?;
        }

        let entry = MLCacheEntry {
            prompt_hash: prompt_hash.clone(),
            response: response.clone(),
            cached_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            model_type,
            hit_count: 0,
        };

        self.entries.insert(prompt_hash.clone(), entry);
        self.stats.total_size_bytes += response.len();
        
        tracing::debug!("ML cache PUT for hash: {}", &prompt_hash[..8]);
        Ok(())
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) -> Result<()> {
        let mut oldest_hash = String::new();
        let mut oldest_time = u64::MAX;

        // Find entry with lowest hit count (LRU approximation)
        for (hash, entry) in &self.entries {
            if entry.hit_count < oldest_time {
                oldest_time = entry.hit_count;
                oldest_hash = hash.clone();
            }
        }

        if !oldest_hash.is_empty() {
            if let Some(removed) = self.entries.remove(&oldest_hash) {
                self.stats.evictions += 1;
                self.stats.total_size_bytes = self.stats.total_size_bytes.saturating_sub(removed.response.len());
                
                tracing::debug!("ML cache EVICTED hash: {}", &oldest_hash[..8]);
            }
        }

        Ok(())
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total > 0 {
            self.stats.hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.stats = CacheStats::default();
        self.save()?;
        
        tracing::info!("ML cache cleared");
        Ok(())
    }

    /// Get cache size in entries
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache contains hash
    pub fn contains(&self, prompt_hash: &str) -> bool {
        self.entries.contains_key(prompt_hash)
    }

    /// Remove specific entry
    pub fn remove(&mut self, prompt_hash: &str) -> Option<MLCacheEntry> {
        if let Some(entry) = self.entries.remove(prompt_hash) {
            self.stats.total_size_bytes = self.stats.total_size_bytes.saturating_sub(entry.response.len());
            Some(entry)
        } else {
            None
        }
    }

    /// Get cache entries for specific model
    pub fn get_entries_for_model(&self, model_type: &str) -> Vec<&MLCacheEntry> {
        self.entries
            .values()
            .filter(|entry| entry.model_type == model_type)
            .collect()
    }

    /// Pre-warm cache with common responses
    pub fn pre_warm(&mut self, responses: Vec<(String, String, String)>) -> Result<()> {
        for (prompt, response, model_type) in responses {
            let hash = Self::generate_prompt_hash(&prompt, &model_type, "default");
            self.put(hash, response, model_type)?;
        }
        
        tracing::info!("ML cache pre-warmed with {} responses", self.entries.len());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ml_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
    }

    #[test]
    fn test_prompt_hash_generation() {
        let hash1 = MLResponseCache::generate_prompt_hash("test prompt", "deepseek", "config1");
        let hash2 = MLResponseCache::generate_prompt_hash("test prompt", "deepseek", "config1");
        let hash3 = MLResponseCache::generate_prompt_hash("different prompt", "deepseek", "config1");
        
        assert_eq!(hash1, hash2); // Same inputs = same hash
        assert_ne!(hash1, hash3); // Different inputs = different hash
        assert_eq!(hash1.len(), 64); // SHA-256 hex string length
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        let hash = "test_hash".to_string();
        let response = r#"{"result": "test"}"#.to_string();
        
        // Put response in cache
        cache.put(hash.clone(), response.clone(), "deepseek".to_string()).unwrap();
        
        // Get response from cache
        let cached = cache.get(&hash);
        assert_eq!(cached, Some(response));
        assert_eq!(cache.stats.hits, 1);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        let result = cache.get("nonexistent_hash");
        assert_eq!(result, None);
        assert_eq!(cache.stats.hits, 0);
        assert_eq!(cache.stats.misses, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 2); // Small cache
        
        // Fill cache to capacity
        cache.put("hash1".to_string(), "response1".to_string(), "deepseek".to_string()).unwrap();
        cache.put("hash2".to_string(), "response2".to_string(), "deepseek".to_string()).unwrap();
        
        assert_eq!(cache.size(), 2);
        
        // Add one more entry to trigger eviction
        cache.put("hash3".to_string(), "response3".to_string(), "deepseek".to_string()).unwrap();
        
        assert_eq!(cache.size(), 2); // Still at max size
        assert_eq!(cache.stats.evictions, 1);
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        
        // Create cache and add entry
        {
            let mut cache = MLResponseCache::new(cache_dir.clone(), 100);
            cache.put("test_hash".to_string(), "test_response".to_string(), "deepseek".to_string()).unwrap();
            cache.save().unwrap();
        }
        
        // Load cache in new instance
        {
            let mut cache = MLResponseCache::new(cache_dir, 100);
            cache.load().unwrap();
            
            let result = cache.get("test_hash");
            assert_eq!(result, Some("test_response".to_string()));
        }
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        cache.put("hash1".to_string(), "response1".to_string(), "deepseek".to_string()).unwrap();
        cache.put("hash2".to_string(), "response2".to_string(), "qwen".to_string()).unwrap();
        
        assert_eq!(cache.size(), 2);
        
        cache.clear().unwrap();
        
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.stats.hits, 0);
        assert_eq!(cache.stats.misses, 0);
    }

    #[test]
    fn test_model_specific_entries() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        cache.put("hash1".to_string(), "response1".to_string(), "deepseek".to_string()).unwrap();
        cache.put("hash2".to_string(), "response2".to_string(), "qwen".to_string()).unwrap();
        cache.put("hash3".to_string(), "response3".to_string(), "deepseek".to_string()).unwrap();
        
        let deepseek_entries = cache.get_entries_for_model("deepseek");
        let qwen_entries = cache.get_entries_for_model("qwen");
        
        assert_eq!(deepseek_entries.len(), 2);
        assert_eq!(qwen_entries.len(), 1);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        cache.put("hash1".to_string(), "response1".to_string(), "deepseek".to_string()).unwrap();
        
        // 2 hits, 1 miss
        cache.get("hash1");
        cache.get("hash1");
        cache.get("nonexistent");
        
        assert_eq!(cache.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_pre_warm_cache() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = MLResponseCache::new(temp_dir.path().to_path_buf(), 100);
        
        let responses = vec![
            ("prompt1".to_string(), "response1".to_string(), "deepseek".to_string()),
            ("prompt2".to_string(), "response2".to_string(), "qwen".to_string()),
        ];
        
        cache.pre_warm(responses).unwrap();
        
        assert_eq!(cache.size(), 2);
        
        let hash1 = MLResponseCache::generate_prompt_hash("prompt1", "deepseek", "default");
        let cached = cache.get(&hash1);
        assert_eq!(cached, Some("response1".to_string()));
    }
}
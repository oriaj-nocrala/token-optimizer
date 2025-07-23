use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use crate::types::CacheEntry;
use crate::utils::hash_utils::calculate_file_hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCache {
    pub entries: HashMap<String, CacheEntry>,
    pub last_updated: DateTime<Utc>,
    pub cache_version: String,
}

impl SmartCache {
    pub fn new() -> Self {
        SmartCache {
            entries: HashMap::new(),
            last_updated: Utc::now(),
            cache_version: "1.0.0".to_string(),
        }
    }

    pub fn load_from_file(cache_path: &Path) -> Result<Self> {
        if cache_path.exists() {
            let content = fs::read_to_string(cache_path)?;
            let cache: SmartCache = serde_json::from_str(&content)?;
            Ok(cache)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save_to_file(&self, cache_path: &Path) -> Result<()> {
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(cache_path, content)?;
        Ok(())
    }

    pub fn get_entry(&self, file_path: &str) -> Option<&CacheEntry> {
        self.entries.get(file_path)
    }

    pub fn set_entry(&mut self, file_path: String, entry: CacheEntry) {
        self.entries.insert(file_path, entry);
        self.last_updated = Utc::now();
    }

    pub fn remove_entry(&mut self, file_path: &str) -> Option<CacheEntry> {
        self.last_updated = Utc::now();
        self.entries.remove(file_path)
    }

    pub fn is_file_cached(&self, file_path: &str) -> bool {
        self.entries.contains_key(file_path)
    }

    pub fn is_file_up_to_date(&self, file_path: &Path) -> Result<bool> {
        let file_path_str = file_path.to_string_lossy();
        
        if let Some(entry) = self.entries.get(file_path_str.as_ref()) {
            let current_hash = calculate_file_hash(file_path)?;
            Ok(entry.file_hash == current_hash)
        } else {
            Ok(false)
        }
    }

    pub fn get_outdated_files(&self, _root_path: &Path) -> Result<Vec<String>> {
        let mut outdated = Vec::new();
        
        for (file_path, _) in &self.entries {
            let path = PathBuf::from(file_path);
            if path.exists() {
                if !self.is_file_up_to_date(&path)? {
                    outdated.push(file_path.clone());
                }
            } else {
                outdated.push(file_path.clone());
            }
        }
        
        Ok(outdated)
    }

    pub fn clean_deleted_files(&mut self, _root_path: &Path) -> Result<usize> {
        let mut deleted_count = 0;
        let mut to_remove = Vec::new();
        
        for file_path in self.entries.keys() {
            let path = PathBuf::from(file_path);
            if !path.exists() {
                to_remove.push(file_path.clone());
            }
        }
        
        for file_path in to_remove {
            self.entries.remove(&file_path);
            deleted_count += 1;
        }
        
        if deleted_count > 0 {
            self.last_updated = Utc::now();
        }
        
        Ok(deleted_count)
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.entries.len();
        let mut total_size = 0u64;
        let mut oldest_entry = None;
        let mut newest_entry = None;
        
        for entry in self.entries.values() {
            total_size += entry.metadata.size;
            
            if oldest_entry.is_none() || entry.last_analyzed < oldest_entry.unwrap() {
                oldest_entry = Some(entry.last_analyzed);
            }
            
            if newest_entry.is_none() || entry.last_analyzed > newest_entry.unwrap() {
                newest_entry = Some(entry.last_analyzed);
            }
        }
        
        CacheStats {
            total_entries,
            total_size,
            oldest_entry,
            newest_entry,
            last_updated: self.last_updated,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_updated = Utc::now();
    }

    pub fn get_file_dependencies(&self, file_path: &str) -> Vec<String> {
        if let Some(entry) = self.entries.get(file_path) {
            entry.dependencies.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_file_dependents(&self, file_path: &str) -> Vec<String> {
        if let Some(entry) = self.entries.get(file_path) {
            entry.dependents.clone()
        } else {
            Vec::new()
        }
    }

    pub fn update_dependencies(&mut self, file_path: &str, dependencies: Vec<String>) {
        if let Some(entry) = self.entries.get_mut(file_path) {
            entry.dependencies = dependencies;
            self.last_updated = Utc::now();
        }
    }

    pub fn update_dependents(&mut self, file_path: &str, dependents: Vec<String>) {
        if let Some(entry) = self.entries.get_mut(file_path) {
            entry.dependents = dependents;
            self.last_updated = Utc::now();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: u64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl Default for SmartCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use crate::types::{FileType, Complexity, ChangeLogEntry, ChangeType, ImpactLevel};

    fn create_test_cache_entry(file_path: &str, file_hash: &str) -> CacheEntry {
        let metadata = FileMetadata {
            path: file_path.to_string(),
            size: 100,
            line_count: 10,
            last_modified: Utc::now(),
            file_type: FileType::Other,
            summary: "Test file".to_string(),
            relevant_sections: vec![],
            exports: vec![],
            imports: vec![],
            complexity: Complexity::Low,
            detailed_analysis: None,
        };

        let summary = CodeSummary {
            file_name: file_path.to_string(),
            file_type: "Other".to_string(),
            exports: vec![],
            imports: vec![],
            functions: vec![],
            classes: vec![],
            components: vec![],
            services: vec![],
            pipes: vec![],
            modules: vec![],
            key_patterns: vec![],
            dependencies: vec![],
            scss_variables: None,
            scss_mixins: None,
        };

        CacheEntry {
            file_hash: file_hash.to_string(),
            last_analyzed: Utc::now(),
            summary,
            metadata,
            change_log: vec![],
            dependencies: vec![],
            dependents: vec![],
        }
    }

    #[test]
    fn test_new_cache() {
        let cache = SmartCache::new();
        
        assert!(cache.entries.is_empty());
        assert_eq!(cache.cache_version, "1.0.0");
    }

    #[test]
    fn test_set_and_get_entry() {
        let mut cache = SmartCache::new();
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry.clone());
        
        let retrieved = cache.get_entry("test.rs");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().file_hash, "abc123");
    }

    #[test]
    fn test_remove_entry() {
        let mut cache = SmartCache::new();
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        
        assert!(cache.get_entry("test.rs").is_some());
        
        let removed = cache.remove_entry("test.rs");
        assert!(removed.is_some());
        assert!(cache.get_entry("test.rs").is_none());
    }

    #[test]
    fn test_is_file_cached() {
        let mut cache = SmartCache::new();
        
        assert!(!cache.is_file_cached("test.rs"));
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        
        assert!(cache.is_file_cached("test.rs"));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = SmartCache::new();
        
        let entry1 = create_test_cache_entry("test1.rs", "abc123");
        let entry2 = create_test_cache_entry("test2.rs", "def456");
        
        cache.set_entry("test1.rs".to_string(), entry1);
        cache.set_entry("test2.rs".to_string(), entry2);
        
        let stats = cache.get_cache_stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_size, 200); // Each entry has size 100
        assert!(stats.oldest_entry.is_some());
        assert!(stats.newest_entry.is_some());
    }

    #[test]
    fn test_clear_cache() {
        let mut cache = SmartCache::new();
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        
        assert_eq!(cache.entries.len(), 1);
        
        cache.clear();
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_save_and_load_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cache_file = temp_dir.path().join("test_cache.json");
        
        // Create and save cache
        let mut cache = SmartCache::new();
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        cache.save_to_file(&cache_file)?;
        
        // Load cache from file
        let loaded_cache = SmartCache::load_from_file(&cache_file)?;
        
        assert_eq!(loaded_cache.entries.len(), 1);
        assert!(loaded_cache.get_entry("test.rs").is_some());
        assert_eq!(loaded_cache.get_entry("test.rs").unwrap().file_hash, "abc123");
        
        Ok(())
    }

    #[test]
    fn test_load_nonexistent_cache() -> Result<()> {
        let cache_file = PathBuf::from("nonexistent_cache.json");
        let cache = SmartCache::load_from_file(&cache_file)?;
        
        assert!(cache.entries.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_is_file_up_to_date() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "test content")?;
        
        let mut cache = SmartCache::new();
        
        // File not in cache
        assert!(!cache.is_file_up_to_date(temp_file.path())?);
        
        // Add file to cache with correct hash
        let file_hash = calculate_file_hash(temp_file.path())?;
        let entry = create_test_cache_entry(
            &temp_file.path().to_string_lossy(),
            &file_hash
        );
        cache.set_entry(temp_file.path().to_string_lossy().to_string(), entry);
        
        // File should be up to date
        assert!(cache.is_file_up_to_date(temp_file.path())?);
        
        // Modify file content
        write!(temp_file, "modified content")?;
        
        // File should no longer be up to date (hash changed)
        assert!(!cache.is_file_up_to_date(temp_file.path())?);
        
        Ok(())
    }

    #[test]
    fn test_get_outdated_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache = SmartCache::new();
        
        // Create a temporary file
        let temp_file = temp_dir.path().join("test.rs");
        fs::write(&temp_file, "test content")?;
        
        // Add file to cache
        let file_hash = calculate_file_hash(&temp_file)?;
        let entry = create_test_cache_entry(&temp_file.to_string_lossy(), &file_hash);
        cache.set_entry(temp_file.to_string_lossy().to_string(), entry);
        
        // File should not be outdated
        let outdated = cache.get_outdated_files(temp_dir.path())?;
        assert!(outdated.is_empty());
        
        // Modify file content
        fs::write(&temp_file, "modified content")?;
        
        // File should now be outdated
        let outdated = cache.get_outdated_files(temp_dir.path())?;
        assert_eq!(outdated.len(), 1);
        assert!(outdated.contains(&temp_file.to_string_lossy().to_string()));
        
        Ok(())
    }

    #[test]
    fn test_clean_deleted_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache = SmartCache::new();
        
        // Create a temporary file
        let temp_file = temp_dir.path().join("test.rs");
        fs::write(&temp_file, "test content")?;
        
        // Add entries for existing and non-existing files
        let entry1 = create_test_cache_entry(&temp_file.to_string_lossy(), "abc123");
        let entry2 = create_test_cache_entry("/nonexistent/file.rs", "def456");
        
        cache.set_entry(temp_file.to_string_lossy().to_string(), entry1);
        cache.set_entry("/nonexistent/file.rs".to_string(), entry2);
        
        assert_eq!(cache.entries.len(), 2);
        
        // Clean deleted files
        let cleaned_count = cache.clean_deleted_files(temp_dir.path())?;
        
        assert_eq!(cleaned_count, 1);
        assert_eq!(cache.entries.len(), 1);
        assert!(cache.get_entry(&temp_file.to_string_lossy()).is_some());
        assert!(cache.get_entry("/nonexistent/file.rs").is_none());
        
        Ok(())
    }

    #[test]
    fn test_cache_serialization() -> Result<()> {
        let mut cache = SmartCache::new();
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        
        // Serialize to JSON
        let json = serde_json::to_string(&cache)?;
        
        // Deserialize from JSON
        let deserialized_cache: SmartCache = serde_json::from_str(&json)?;
        
        assert_eq!(deserialized_cache.entries.len(), 1);
        assert!(deserialized_cache.get_entry("test.rs").is_some());
        assert_eq!(deserialized_cache.cache_version, "1.0.0");
        
        Ok(())
    }

    #[test]
    fn test_dependencies_and_dependents() {
        let mut cache = SmartCache::new();
        
        let entry = create_test_cache_entry("test.rs", "abc123");
        cache.set_entry("test.rs".to_string(), entry);
        
        // Initially empty
        assert!(cache.get_file_dependencies("test.rs").is_empty());
        assert!(cache.get_file_dependents("test.rs").is_empty());
        
        // Update dependencies and dependents
        cache.update_dependencies("test.rs", vec!["dep1.rs".to_string(), "dep2.rs".to_string()]);
        cache.update_dependents("test.rs", vec!["user1.rs".to_string()]);
        
        let deps = cache.get_file_dependencies("test.rs");
        let dependents = cache.get_file_dependents("test.rs");
        
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"dep1.rs".to_string()));
        assert!(deps.contains(&"dep2.rs".to_string()));
        
        assert_eq!(dependents.len(), 1);
        assert!(dependents.contains(&"user1.rs".to_string()));
    }

    #[test]
    fn test_cache_with_change_log() {
        let mut cache = SmartCache::new();
        
        let mut entry = create_test_cache_entry("test.rs", "abc123");
        
        // Add change log entry
        let change_log_entry = ChangeLogEntry {
            timestamp: Utc::now(),
            change_type: ChangeType::Modified,
            description: "Updated function signature".to_string(),
            lines_changed: 5,
            impact_level: ImpactLevel::Medium,
        };
        entry.change_log.push(change_log_entry);
        
        cache.set_entry("test.rs".to_string(), entry);
        
        let retrieved = cache.get_entry("test.rs").unwrap();
        assert_eq!(retrieved.change_log.len(), 1);
        assert_eq!(retrieved.change_log[0].change_type, ChangeType::Modified);
        assert_eq!(retrieved.change_log[0].impact_level, ImpactLevel::Medium);
    }

    #[test]
    fn test_cache_stats_calculation() {
        let mut cache = SmartCache::new();
        
        // Create entries with different sizes and timestamps
        let mut entry1 = create_test_cache_entry("test1.rs", "abc123");
        entry1.metadata.size = 150;
        entry1.last_analyzed = Utc::now() - chrono::Duration::hours(2);
        
        let mut entry2 = create_test_cache_entry("test2.rs", "def456");
        entry2.metadata.size = 250;
        entry2.last_analyzed = Utc::now() - chrono::Duration::hours(1);
        
        let mut entry3 = create_test_cache_entry("test3.rs", "ghi789");
        entry3.metadata.size = 100;
        entry3.last_analyzed = Utc::now();
        
        cache.set_entry("test1.rs".to_string(), entry1.clone());
        cache.set_entry("test2.rs".to_string(), entry2.clone());
        cache.set_entry("test3.rs".to_string(), entry3.clone());
        
        let stats = cache.get_cache_stats();
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.total_size, 500); // 150 + 250 + 100
        assert_eq!(stats.oldest_entry.unwrap(), entry1.last_analyzed);
        assert_eq!(stats.newest_entry.unwrap(), entry3.last_analyzed);
    }

    #[test]
    fn test_default_implementation() {
        let cache = SmartCache::default();
        assert!(cache.entries.is_empty());
        assert_eq!(cache.cache_version, "1.0.0");
    }
}
//! Path normalization utilities for consistent file path handling

use std::path::{Path, PathBuf};

/// Normalize file paths to be consistent relative to project root
pub struct PathNormalizer {
    project_root: PathBuf,
}

impl PathNormalizer {
    /// Create a new path normalizer with the given project root
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Normalize an absolute path to be relative to the project root
    pub fn normalize_to_project_relative(&self, absolute_path: &Path) -> String {
        // First try to strip the project root prefix
        if let Ok(relative) = absolute_path.strip_prefix(&self.project_root) {
            return format!("./{}", relative.display());
        }

        // If that fails, convert to string and look for common patterns
        let path_str = absolute_path.to_string_lossy();
        
        // Look for patterns like "/path/to/project/src/app/..." and extract "src/app/..."
        if let Some(src_pos) = path_str.find("/src/") {
            return format!("./{}", &path_str[src_pos + 1..]);
        }
        
        // Look for patterns with project directory name
        if let Some(project_name) = self.project_root.file_name() {
            let project_name_str = project_name.to_string_lossy();
            if let Some(project_pos) = path_str.find(&format!("/{}/", project_name_str)) {
                let after_project = &path_str[project_pos + project_name_str.len() + 2..];
                return format!("./{}", after_project);
            }
        }
        
        // Fallback: return the path as-is but with ./ prefix if it doesn't have it
        if path_str.starts_with("./") {
            path_str.to_string()
        } else {
            format!("./{}", path_str)
        }
    }

    /// Normalize various path representations to a consistent format
    pub fn normalize_path_variations(&self, input_path: &str) -> String {
        let path = Path::new(input_path);
        
        // If it's already an absolute path, normalize it
        if path.is_absolute() {
            return self.normalize_to_project_relative(path);
        }
        
        // Handle relative paths
        if input_path.starts_with("./") {
            input_path.to_string()
        } else {
            format!("./{}", input_path)
        }
    }

    /// Create a lookup key for cache storage that is consistent
    pub fn create_cache_key(&self, file_path: &Path) -> String {
        if file_path.is_absolute() {
            self.normalize_to_project_relative(file_path)
        } else {
            // For relative paths, ensure they start with ./
            let path_str = file_path.to_string_lossy();
            if path_str.starts_with("./") {
                path_str.to_string()
            } else {
                format!("./{}", path_str)
            }
        }
    }

    /// Match various path representations against a cache key
    pub fn path_matches_cache_key(&self, cache_key: &str, lookup_path: &str) -> bool {
        // Normalize both paths to the same format
        let normalized_cache_key = cache_key;
        let normalized_lookup = self.normalize_path_variations(lookup_path);
        
        // Direct match
        if normalized_cache_key == normalized_lookup {
            return true;
        }
        
        // Try without ./ prefix
        let cache_without_prefix = cache_key.strip_prefix("./").unwrap_or(cache_key);
        let lookup_without_prefix = normalized_lookup.strip_prefix("./").unwrap_or(&normalized_lookup);
        
        if cache_without_prefix == lookup_without_prefix {
            return true;
        }
        
        // Try to match by extracting common suffixes
        // For example: "./calendario-psicologia/src/app/auth.service.ts" should match "src/app/auth.service.ts"
        if let Some(src_pos) = cache_key.find("/src/") {
            let cache_suffix = &cache_key[src_pos + 1..]; // "src/app/auth.service.ts"
            if lookup_without_prefix == cache_suffix {
                return true;
            }
        }
        
        // Try reverse: if lookup has project path and cache doesn't
        if let Some(src_pos) = lookup_without_prefix.find("/src/") {
            let lookup_suffix = &lookup_without_prefix[src_pos + 1..];
            if cache_without_prefix == lookup_suffix {
                return true;
            }
        }
        
        false
    }

    /// Get the project root directory
    pub fn get_project_root(&self) -> &Path {
        &self.project_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_normalize_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("my-project");
        fs::create_dir_all(&project_root).unwrap();
        
        let normalizer = PathNormalizer::new(&project_root);
        
        let absolute_path = project_root.join("src/app/auth.service.ts");
        let normalized = normalizer.normalize_to_project_relative(&absolute_path);
        
        assert_eq!(normalized, "./src/app/auth.service.ts");
    }
    
    #[test]
    fn test_normalize_path_variations() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("angular-project");
        
        let normalizer = PathNormalizer::new(&project_root);
        
        // Test various input formats
        assert_eq!(normalizer.normalize_path_variations("src/app/test.ts"), "./src/app/test.ts");
        assert_eq!(normalizer.normalize_path_variations("./src/app/test.ts"), "./src/app/test.ts");
        assert_eq!(normalizer.normalize_path_variations("app/test.ts"), "./app/test.ts");
    }

    #[test]
    fn test_cache_key_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("calendario-psicologia");
        fs::create_dir_all(&project_root).unwrap();
        
        let normalizer = PathNormalizer::new(&project_root);
        
        // Test absolute path
        let absolute_path = project_root.join("src/app/services/auth.service.ts");
        let cache_key = normalizer.create_cache_key(&absolute_path);
        assert_eq!(cache_key, "./src/app/services/auth.service.ts");
        
        // Test relative path
        let relative_path = Path::new("src/app/services/auth.service.ts");
        let cache_key2 = normalizer.create_cache_key(relative_path);
        assert_eq!(cache_key2, "./src/app/services/auth.service.ts");
    }

    #[test]
    fn test_path_matching() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("calendario-psicologia");
        
        let normalizer = PathNormalizer::new(&project_root);
        
        let cache_key = "./src/app/services/auth.service.ts";
        
        // Test various lookup patterns
        assert!(normalizer.path_matches_cache_key(cache_key, "src/app/services/auth.service.ts"));
        assert!(normalizer.path_matches_cache_key(cache_key, "./src/app/services/auth.service.ts"));
        assert!(normalizer.path_matches_cache_key(cache_key, "calendario-psicologia/src/app/services/auth.service.ts"));
        
        // Test project-prefixed cache key
        let cache_key_with_project = "./calendario-psicologia/src/app/services/auth.service.ts";
        assert!(normalizer.path_matches_cache_key(cache_key_with_project, "src/app/services/auth.service.ts"));
        assert!(normalizer.path_matches_cache_key(cache_key_with_project, "./src/app/services/auth.service.ts"));
        assert!(normalizer.path_matches_cache_key(cache_key_with_project, "calendario-psicologia/src/app/services/auth.service.ts"));
    }

    #[test]
    fn test_project_with_slash_in_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path().join("my-complex/project-name");
        
        let normalizer = PathNormalizer::new(&project_root);
        
        let test_path = "generic-project/src/app/test.component.ts";
        let normalized = normalizer.normalize_path_variations(test_path);
        
        // Should handle project names with special characters
        assert_eq!(normalized, "./generic-project/src/app/test.component.ts");
    }
}
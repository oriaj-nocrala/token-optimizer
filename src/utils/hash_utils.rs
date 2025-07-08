use sha2::{Sha256, Digest};
use std::path::Path;
use std::fs;
use anyhow::Result;

pub fn calculate_file_hash(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let hash = calculate_content_hash(&content);
    Ok(hash)
}

pub fn calculate_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

pub fn calculate_string_hash(content: &str) -> String {
    calculate_content_hash(content.as_bytes())
}

pub fn verify_file_hash(path: &Path, expected_hash: &str) -> Result<bool> {
    let actual_hash = calculate_file_hash(path)?;
    Ok(actual_hash == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_content_hash() {
        let content = b"Hello, world!";
        let hash = calculate_content_hash(content);
        
        // Known SHA-256 hash for "Hello, world!"
        assert_eq!(hash, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
    }

    #[test]
    fn test_calculate_string_hash() {
        let content = "Hello, world!";
        let hash = calculate_string_hash(content);
        
        // Should match the content hash
        assert_eq!(hash, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
    }

    #[test]
    fn test_empty_content_hash() {
        let content = b"";
        let hash = calculate_content_hash(content);
        
        // Known SHA-256 hash for empty content
        assert_eq!(hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    }

    #[test]
    fn test_different_content_different_hash() {
        let content1 = b"Hello, world!";
        let content2 = b"Hello, World!";
        
        let hash1 = calculate_content_hash(content1);
        let hash2 = calculate_content_hash(content2);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_content_same_hash() {
        let content = b"Test content";
        let hash1 = calculate_content_hash(content);
        let hash2 = calculate_content_hash(content);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_file_hash() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"File content for testing";
        fs::write(&temp_file, test_content)?;
        
        let hash = calculate_file_hash(temp_file.path())?;
        let expected_hash = calculate_content_hash(test_content);
        
        assert_eq!(hash, expected_hash);
        Ok(())
    }

    #[test]
    fn test_verify_file_hash_correct() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"File content for verification";
        fs::write(&temp_file, test_content)?;
        
        let expected_hash = calculate_content_hash(test_content);
        let is_valid = verify_file_hash(temp_file.path(), &expected_hash)?;
        
        assert!(is_valid);
        Ok(())
    }

    #[test]
    fn test_verify_file_hash_incorrect() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let test_content = b"File content for verification";
        fs::write(&temp_file, test_content)?;
        
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
        let is_valid = verify_file_hash(temp_file.path(), wrong_hash)?;
        
        assert!(!is_valid);
        Ok(())
    }

    #[test]
    fn test_hash_consistency_across_calls() {
        let content = "Consistent content";
        let hash1 = calculate_string_hash(content);
        let hash2 = calculate_string_hash(content);
        let hash3 = calculate_string_hash(content);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_unicode_content_hash() {
        let content = "Hello, 世界! 🌍";
        let hash = calculate_string_hash(content);
        
        // Should handle Unicode content correctly
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_large_content_hash() {
        let content = "x".repeat(10000);
        let hash = calculate_string_hash(&content);
        
        // Should handle large content
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_binary_content_hash() {
        let content: Vec<u8> = (0..255).collect();
        let hash = calculate_content_hash(&content);
        
        // Should handle binary content
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
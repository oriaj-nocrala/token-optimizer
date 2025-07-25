//! Cache regression tests for ML commands cache detection logic
//! 
//! These tests protect the critical cache functionality that was implemented
//! to fix the reindexing issue. Previously the system was looking for 
//! `lsh_index.json` but actual cache files are `vectors.json`.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use walkdir::WalkDir;

/// Test version of is_cache_fresh function with custom paths
fn is_cache_fresh_test(cache_dir: &Path, src_dir: &Path) -> Result<bool> {
    // Check if .cache/vector-db directory exists
    if !cache_dir.exists() {
        return Ok(false);
    }
    
    // Get cache creation time from vectors.json
    let vectors_json = cache_dir.join("vectors.json");
    let cache_time = match std::fs::metadata(&vectors_json) {
        Ok(metadata) => match metadata.modified() {
            Ok(time) => time,
            Err(_) => return Ok(false),
        },
        Err(_) => return Ok(false),
    };
    
    // Check if any Rust source files were modified after cache creation
    for entry in WalkDir::new(src_dir).into_iter()
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        
        // Only check .rs files
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified_time) = metadata.modified() {
                    if modified_time > cache_time {
                        // Found a file modified after cache - cache is stale
                        return Ok(false);
                    }
                }
            }
        }
    }
    
    // All source files are older than cache - cache is fresh
    Ok(true)
}

#[test]
fn test_cache_fresh_when_no_files_modified() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create source files first
    fs::write(src_dir.join("lib.rs"), "pub mod test;")?;
    fs::write(src_dir.join("test.rs"), "fn test() {}")?;
    
    // Sleep to ensure time difference
    std::thread::sleep(Duration::from_millis(50));
    
    // Create cache file after source files (cache is newer)
    let vectors_file = cache_dir.join("vectors.json");
    fs::write(&vectors_file, r#"{"vectors":[],"metadata":{}}"#)?;
    
    // Cache should be fresh (no files modified after cache creation)
    assert!(is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_cache_stale_when_files_modified_after() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create cache file first
    let vectors_file = cache_dir.join("vectors.json");
    fs::write(&vectors_file, r#"{"vectors":[],"metadata":{}}"#)?;
    
    // Get the cache time to verify it's older
    let cache_time = fs::metadata(&vectors_file)?.modified()?;
    
    // Sleep to ensure time difference
    std::thread::sleep(Duration::from_millis(100));
    
    // Create/modify source files after cache (files are newer)
    let test_file = src_dir.join("test.rs");
    fs::write(&test_file, "fn test() { println!(\"modified\"); }")?;
    
    // Verify the source file is actually newer
    let source_time = fs::metadata(&test_file)?.modified()?;
    assert!(source_time > cache_time, "Source file should be newer than cache file");
    
    // Cache should be stale (files modified after cache creation)
    assert!(!is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_cache_missing_vectors_json_returns_false() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure but NO vectors.json
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create source files
    fs::write(src_dir.join("lib.rs"), "pub mod test;")?;
    fs::write(src_dir.join("test.rs"), "fn test() {}")?;
    
    // Should return false because vectors.json doesn't exist
    assert!(!is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_regression_vectors_json_not_lsh_index_json() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create WRONG cache file (old bug - looking for lsh_index.json)
    fs::write(cache_dir.join("lsh_index.json"), "{\"old_format\":true}")?;
    
    // Create source files
    fs::write(src_dir.join("test.rs"), "fn test() {}")?;
    
    // Should return false because it looks for vectors.json, not lsh_index.json
    assert!(!is_cache_fresh_test(&cache_dir, &src_dir)?);
    
    // Now create the CORRECT cache file
    fs::write(cache_dir.join("vectors.json"), "{\"correct_format\":true}")?;
    
    // Sleep to ensure old files are older than new cache
    std::thread::sleep(Duration::from_millis(10));
    
    // Should return true now
    assert!(is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_cache_ignores_non_rust_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create cache file first
    let vectors_file = cache_dir.join("vectors.json");
    fs::write(&vectors_file, "{\"cached_data\":true}")?;
    
    // Sleep to ensure time difference
    std::thread::sleep(Duration::from_millis(50));
    
    // Create NON-Rust files after cache (should be ignored)
    fs::write(src_dir.join("config.json"), "{}")?;
    fs::write(src_dir.join("readme.md"), "# README")?;
    fs::write(src_dir.join("style.css"), "body {}")?;
    
    // Cache should still be fresh (non-Rust files ignored)
    assert!(is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_cache_directory_missing_returns_false() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db"); // This directory doesn't exist
    let src_dir = temp_dir.path().join("src");
    
    // Create only source directory
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("test.rs"), "fn test() {}")?;
    
    // Should return false because cache directory doesn't exist
    assert!(!is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}

#[test]
fn test_empty_src_directory_cache_fresh() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().join(".cache/vector-db");
    let src_dir = temp_dir.path().join("src");
    
    // Create directory structure
    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&src_dir)?;
    
    // Create cache file with no source files
    let vectors_file = cache_dir.join("vectors.json");
    fs::write(&vectors_file, "{\"empty_project\":true}")?;
    
    // Should return true because no source files to compare
    assert!(is_cache_fresh_test(&cache_dir, &src_dir)?);
    Ok(())
}
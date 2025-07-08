use anyhow::Result;
use std::path::Path;
use crate::cache::CacheManager;

pub fn run_summary(path: &Path, file: Option<&Path>, format: &str) -> Result<()> {
    let cache_manager = CacheManager::new(path)?;
    
    if let Some(file_path) = file {
        // Summary for specific file
        let file_path_str = file_path.to_string_lossy();
        if let Some(entry) = cache_manager.get_file_summary(&file_path_str) {
            match format {
                "json" => {
                    let json = serde_json::to_string_pretty(&entry.summary)?;
                    println!("{}", json);
                }
                _ => {
                    println!("File Summary: {}", entry.summary.file_name);
                    println!("Type: {}", entry.summary.file_type);
                    println!("Exports: {}", entry.summary.exports.join(", "));
                    println!("Imports: {}", entry.summary.imports.join(", "));
                    println!("Functions: {}", entry.summary.functions.len());
                    println!("Classes: {}", entry.summary.classes.len());
                    println!("Components: {}", entry.summary.components.len());
                    println!("Services: {}", entry.summary.services.len());
                    
                    if let Some(scss_vars) = &entry.summary.scss_variables {
                        println!("SCSS Variables: {}", scss_vars.join(", "));
                    }
                    
                    if let Some(scss_mixins) = &entry.summary.scss_mixins {
                        println!("SCSS Mixins: {}", scss_mixins.join(", "));
                    }
                }
            }
        } else {
            println!("File not found in cache: {}", file_path.display());
        }
    } else {
        // Summary for entire project
        let stats = cache_manager.get_cache_stats();
        
        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&stats)?;
                println!("{}", json);
            }
            _ => {
                println!("Project Summary");
                println!("===============");
                println!("Total files: {}", stats.total_entries);
                println!("Total size: {:.2} MB", stats.total_size as f64 / 1024.0 / 1024.0);
                println!("Last updated: {}", stats.last_updated.format("%Y-%m-%d %H:%M:%S"));
                
                if let Some(oldest) = stats.oldest_entry {
                    println!("Oldest entry: {}", oldest.format("%Y-%m-%d %H:%M:%S"));
                }
                
                if let Some(newest) = stats.newest_entry {
                    println!("Newest entry: {}", newest.format("%Y-%m-%d %H:%M:%S"));
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::cache::CacheManager;
    use std::path::PathBuf;

    fn create_test_project_structure(temp_dir: &TempDir) -> Result<()> {
        // Create TypeScript files with realistic content
        fs::create_dir_all(temp_dir.path().join("src/app/services"))?;
        
        fs::write(
            temp_dir.path().join("src/app/services/auth.service.ts"),
            r#"
import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
    providedIn: 'root'
})
export class AuthService {
    constructor(private http: HttpClient) {}
    
    login(credentials: any): Observable<any> {
        return this.http.post('/api/auth/login', credentials);
    }
    
    logout(): void {
        console.log('Logging out');
    }
}
            "#
        )?;
        
        fs::write(
            temp_dir.path().join("src/app/services/user.service.ts"),
            r#"
import { Injectable } from '@angular/core';

interface User {
    id: number;
    name: string;
}

@Injectable()
export class UserService {
    getUser(id: number): User {
        return { id, name: 'Test' };
    }
}
            "#
        )?;
        
        Ok(())
    }

    #[test]
    fn test_summary_command_with_path_variations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_project_structure(&temp_dir)?;
        
        // Analyze project to populate cache
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        cache_manager.analyze_project(temp_dir.path(), false)?;
        
        let auth_service_absolute = temp_dir.path().join("src/app/services/auth.service.ts");
        let auth_service_relative = PathBuf::from("src/app/services/auth.service.ts");
        let auth_service_dot_relative = PathBuf::from("./src/app/services/auth.service.ts");
        
        println!("=== CLI SUMMARY COMMAND PATH TEST ===");
        
        // Test different path formats that users might provide
        let test_paths = vec![
            ("absolute", &auth_service_absolute),
            ("relative", &auth_service_relative),
            ("dot-relative", &auth_service_dot_relative),
        ];
        
        for (path_type, test_path) in test_paths {
            println!("Testing {} path: {}", path_type, test_path.display());
            
            // Simulate the CLI command call
            // This will likely fail for relative paths, documenting the bug
            let result = run_summary(temp_dir.path(), Some(test_path), "json");
            
            match result {
                Ok(_) => println!("  ✅ SUCCESS: Path found in cache"),
                Err(e) => println!("  ❌ FAILED: {}", e),
            }
        }
        
        println!("==============================");
        
        // Document what's actually in the cache
        let cache_keys: Vec<String> = cache_manager.get_cache().entries.keys().cloned().collect();
        println!("Actual cache keys:");
        for key in cache_keys {
            println!("  '{}'", key);
        }
        
        Ok(())
    }

    #[test]
    fn test_summary_command_detailed_analysis_retrieval() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_project_structure(&temp_dir)?;
        
        // Analyze project
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        cache_manager.analyze_project(temp_dir.path(), false)?;
        
        // Get the actual cache key for auth service
        let cache_keys: Vec<String> = cache_manager.get_cache().entries.keys()
            .filter(|k| k.contains("auth.service.ts"))
            .cloned()
            .collect();
        
        if cache_keys.is_empty() {
            panic!("Auth service not found in cache");
        }
        
        let auth_service_key = &cache_keys[0];
        let auth_service_path = Path::new(auth_service_key);
        
        println!("=== CLI SUMMARY DETAILED ANALYSIS TEST ===");
        println!("Using cache key: {}", auth_service_key);
        
        // Test summary retrieval
        let result = run_summary(temp_dir.path(), Some(auth_service_path), "json");
        
        match result {
            Ok(_) => {
                // If successful, check what data is actually returned
                let entry = cache_manager.get_file_summary(auth_service_key);
                if let Some(entry) = entry {
                    println!("Summary data retrieved:");
                    println!("  File type: {}", entry.summary.file_type);
                    println!("  Functions count: {}", entry.summary.functions.len());
                    println!("  Classes count: {}", entry.summary.classes.len());
                    println!("  Detailed analysis present: {}", entry.metadata.detailed_analysis.is_some());
                    
                    if let Some(analysis) = &entry.metadata.detailed_analysis {
                        println!("  Detailed functions: {}", analysis.functions.len());
                        println!("  Detailed classes: {}", analysis.classes.len());
                    }
                    
                    // This documents whether CLI returns old CodeSummary or new detailed analysis
                    println!("Current CLI returns: CodeSummary (old format)");
                    println!("Should return: FileMetadata with detailed_analysis (new format)");
                }
            }
            Err(e) => println!("❌ FAILED: {}", e),
        }
        
        println!("==============================");
        
        Ok(())
    }

    #[test]
    fn test_summary_json_format_consistency() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_project_structure(&temp_dir)?;
        
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        cache_manager.analyze_project(temp_dir.path(), false)?;
        
        // Get a valid cache key
        let cache_keys: Vec<String> = cache_manager.get_cache().entries.keys()
            .filter(|k| k.contains(".ts"))
            .cloned()
            .collect();
        
        if let Some(valid_key) = cache_keys.first() {
            let path = Path::new(valid_key);
            
            println!("=== CLI JSON FORMAT TEST ===");
            println!("Testing with path: {}", valid_key);
            
            // Capture stdout to analyze JSON output
            let result = run_summary(temp_dir.path(), Some(path), "json");
            
            match result {
                Ok(_) => {
                    println!("✅ JSON format test completed");
                    
                    // Compare what CLI outputs vs what cache contains
                    let entry = cache_manager.get_file_summary(valid_key).unwrap();
                    
                    println!("Cache entry contains:");
                    println!("  summary.functions: {}", entry.summary.functions.len());
                    println!("  metadata.detailed_analysis: {}", entry.metadata.detailed_analysis.is_some());
                    
                    // The bug is that CLI outputs summary.functions (empty) instead of metadata.detailed_analysis.functions
                }
                Err(e) => println!("❌ FAILED: {}", e),
            }
            
            println!("==============================");
        }
        
        Ok(())
    }

    #[test]
    fn test_summary_file_not_found_error_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        cache_manager.analyze_project(temp_dir.path(), false)?;
        
        // Test with non-existent file
        let nonexistent_path = Path::new("nonexistent/file.ts");
        
        println!("=== CLI ERROR HANDLING TEST ===");
        
        let result = run_summary(temp_dir.path(), Some(nonexistent_path), "json");
        
        // This should fail gracefully
        match result {
            Ok(_) => println!("❌ UNEXPECTED: Command succeeded for non-existent file"),
            Err(_) => println!("✅ EXPECTED: Command failed for non-existent file"),
        }
        
        // Test with malformed path
        let malformed_path = Path::new("../../etc/passwd");
        let result = run_summary(temp_dir.path(), Some(malformed_path), "json");
        
        match result {
            Ok(_) => println!("❌ UNEXPECTED: Command succeeded for malformed path"),
            Err(_) => println!("✅ EXPECTED: Command failed for malformed path"),
        }
        
        println!("==============================");
        
        Ok(())
    }

    #[test]
    fn test_real_world_path_bug_reproduction() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create a structure that mimics the real calendario-psicologia project
        fs::create_dir_all(temp_dir.path().join("src/app"))?;
        fs::write(
            temp_dir.path().join("src/app/app.component.ts"),
            r#"
import { Component } from '@angular/core';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html'
})
export class AppComponent {
  title = 'test-app';
}
            "#
        )?;
        
        // Analyze from project root
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        cache_manager.analyze_project(temp_dir.path(), false)?;
        
        println!("=== REAL WORLD PATH BUG REPRODUCTION ===");
        println!("Project root: {}", temp_dir.path().display());
        
        // Show what's actually in cache
        let cache_keys: Vec<String> = cache_manager.get_cache().entries.keys().cloned().collect();
        println!("Cache keys:");
        for key in &cache_keys {
            println!("  '{}'", key);
        }
        
        // Test paths that should work but don't in real CLI usage
        let abs_path = format!("{}/src/app/app.component.ts", temp_dir.path().display());
        let test_cases = vec![
            ("calendario-psicologia style path", "calendario-psicologia/src/app/app.component.ts"),
            ("absolute project-prefixed path", abs_path.as_str()),
            ("relative from project root", "src/app/app.component.ts"),
            ("dot-relative", "./src/app/app.component.ts"),
        ];
        
        for (description, test_path) in test_cases {
            println!("Testing {}: '{}'", description, test_path);
            
            // Test the cache manager directly
            let cache_result = cache_manager.get_file_summary(test_path);
            println!("  Cache manager result: {}", if cache_result.is_some() { "✅ FOUND" } else { "❌ NOT FOUND" });
            
            // Test the CLI command
            let cli_result = run_summary(temp_dir.path(), Some(Path::new(test_path)), "text");
            println!("  CLI result: {}", if cli_result.is_ok() { "✅ SUCCESS" } else { "❌ FAILED" });
            
            // Test with the normalize_lookup_key function directly
            let normalized_key = cache_manager.normalize_lookup_key(test_path);
            println!("  Normalized key: '{}'", normalized_key);
            let direct_lookup = cache_manager.get_cache().get_entry(&normalized_key);
            println!("  Direct lookup: {}", if direct_lookup.is_some() { "✅ FOUND" } else { "❌ NOT FOUND" });
            
            println!();
        }
        
        println!("==============================");
        
        Ok(())
    }
}
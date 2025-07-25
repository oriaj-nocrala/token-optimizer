//! Context optimization engine for token-efficient code context

use anyhow::Result; 
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::ml::vector_db::EnhancedSearchResult;

/// Optimized context result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedContext {
    pub context: String,
    pub files: Vec<String>,
    pub total_tokens: usize,
    pub summary: String,
}

/// Context optimization engine
pub struct ContextOptimizer {
    /// Average tokens per character (empirically derived)
    tokens_per_char: f64,
}

impl ContextOptimizer {
    pub fn new() -> Self {
        Self {
            // Based on typical code: ~4.5 chars per token
            tokens_per_char: 1.0 / 4.5,
        }
    }
    
    /// Optimize context for token budget while maximizing relevance
    pub async fn optimize_context(
        &self,
        search_results: &[EnhancedSearchResult],
        max_tokens: usize,
        include_tests: bool,
        include_dependencies: bool,
    ) -> Result<OptimizedContext> {
        println!("🎯 Optimizing context for {} tokens budget", max_tokens);
        
        // Filter results based on preferences
        let mut filtered_results: Vec<&EnhancedSearchResult> = search_results.iter()
            .filter(|result| {
                // Skip test files if not requested
                if !include_tests && self.is_test_file(&result.entry.metadata.file_path) {
                    return false;
                }
                
                // Skip very low relevance scores
                if result.combined_score < 0.1 {
                    return false;
                }
                
                true
            })
            .collect();
        
        // Sort by relevance score (descending)
        filtered_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        
        println!("   Filtered to {} relevant results", filtered_results.len());
        
        // Build context incrementally within token budget
        let mut context_parts = Vec::new();
        let mut included_files = HashSet::new();
        let mut current_tokens = 0;
        let mut dependencies_added = HashMap::new();
        
        // Reserve tokens for structure and formatting
        let structure_tokens = max_tokens / 20; // 5% for structure
        let available_tokens = max_tokens - structure_tokens;
        
        for result in filtered_results {
            let file_path = &result.entry.metadata.file_path;
            
            // Skip if already included
            if included_files.contains(file_path) {
                continue;
            }
            
            // Estimate tokens for this result
            let content = self.extract_relevant_content(result);
            let estimated_tokens = self.estimate_tokens(&content);
            
            // Check if we have budget
            if current_tokens + estimated_tokens > available_tokens {
                // Try to include a smaller snippet
                let snippet = self.create_snippet(&content, available_tokens - current_tokens);
                if !snippet.is_empty() {
                    context_parts.push(self.format_file_section(file_path, &snippet, result.combined_score));
                    included_files.insert(file_path.clone());
                    current_tokens += self.estimate_tokens(&snippet);
                }
                break;
            }
            
            // Add the content
            context_parts.push(self.format_file_section(file_path, &content, result.combined_score));
            included_files.insert(file_path.clone());
            current_tokens += estimated_tokens;
            
            // Add dependencies if requested and budget allows
            if include_dependencies && current_tokens < available_tokens * 3 / 4 {
                self.add_dependencies(
                    result,
                    &mut context_parts,
                    &mut included_files,
                    &mut dependencies_added,
                    &mut current_tokens,
                    available_tokens,
                ).await?;
            }
        }
        
        // Build final context with structure
        let context = self.build_structured_context(&context_parts, max_tokens);
        let final_tokens = self.estimate_tokens(&context);
        
        // Calculate score range from the original search results  
        let min_score = search_results.iter()
            .map(|r| r.combined_score)
            .fold(f32::INFINITY, f32::min);
        let max_score = search_results.iter()
            .map(|r| r.combined_score)
            .fold(f32::NEG_INFINITY, f32::max);
            
        let summary = format!(
            "Optimized context with {} files, {:.1}% token efficiency, relevance scores: {:.2}-{:.2}",
            included_files.len(),
            (final_tokens as f64 / max_tokens as f64) * 100.0,
            min_score,
            max_score
        );
        
        println!("✅ Context optimization complete: {} tokens", final_tokens);
        
        Ok(OptimizedContext {
            context,
            files: included_files.into_iter().collect(),
            total_tokens: final_tokens,
            summary,
        })
    }
    
    /// Check if file is a test file
    fn is_test_file(&self, file_path: &str) -> bool {
        file_path.contains("test") || 
        file_path.contains("spec") || 
        file_path.ends_with(".test.ts") ||
        file_path.ends_with(".spec.ts") ||
        file_path.ends_with("_test.rs") ||
        file_path.contains("/tests/")
    }
    
    /// Extract relevant content from search result
    fn extract_relevant_content(&self, result: &EnhancedSearchResult) -> String {
        // For now, use tokens as a proxy for content
        // In a full implementation, this would extract the actual file content
        // around the matched sections
        
        let tokens = &result.entry.metadata.tokens;
        if tokens.len() > 50 {
            format!("// {} ({:?})\n{}", 
                result.entry.metadata.function_name.as_deref().unwrap_or("Content"),
                result.entry.metadata.code_type,
                tokens[..50].join(" ") + "\n// ... (truncated for brevity)"
            )
        } else {
            format!("// {} ({:?})\n{}", 
                result.entry.metadata.function_name.as_deref().unwrap_or("Content"),
                result.entry.metadata.code_type,
                tokens.join(" ")
            )
        }
    }
    
    /// Estimate tokens for text
    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 * self.tokens_per_char).ceil() as usize
    }
    
    /// Create a snippet within token budget
    fn create_snippet(&self, content: &str, max_tokens: usize) -> String {
        let max_chars = (max_tokens as f64 / self.tokens_per_char) as usize;
        if content.len() <= max_chars {
            content.to_string()
        } else {
            content[..max_chars].to_string() + "\n// ... (truncated)"
        }
    }
    
    /// Format file section with metadata
    fn format_file_section(&self, file_path: &str, content: &str, relevance: f32) -> String {
        format!(
            "// File: {} (relevance: {:.2})\n{}\n\n",
            file_path,
            relevance, 
            content
        )
    }
    
    /// Add dependencies if budget allows
    async fn add_dependencies(
        &self,
        _result: &EnhancedSearchResult,
        _context_parts: &mut Vec<String>,
        _included_files: &mut HashSet<String>,
        _dependencies_added: &mut HashMap<String, bool>,
        _current_tokens: &mut usize,
        _available_tokens: usize,
    ) -> Result<()> {
        // TODO: Implement dependency analysis and inclusion
        // This would analyze imports/exports and include relevant dependencies
        Ok(())
    }
    
    /// Build structured context with header/footer
    fn build_structured_context(&self, context_parts: &[String], max_tokens: usize) -> String {
        let content = context_parts.join("");
        
        let header = format!(
            "// Optimized code context (target: {} tokens)\n// Generated by token-optimizer MCP server\n\n",
            max_tokens
        );
        
        let footer = format!(
            "\n\n// End of optimized context - {} files included\n",
            context_parts.len()
        );
        
        format!("{}{}{}", header, content, footer)
    }
}
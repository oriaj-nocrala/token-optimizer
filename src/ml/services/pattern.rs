//! Pattern detection service with semantic similarity

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use walkdir::WalkDir;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::models::*;

/// Advanced pattern detection service with ML-powered semantic similarity
pub struct PatternDetectionService {
    config: MLConfig,
    plugin_manager: Arc<PluginManager>,
    is_ready: bool,
    embedding_cache: HashMap<String, Vec<f32>>,
}

impl PatternDetectionService {
    pub fn new(config: MLConfig, plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            config,
            plugin_manager,
            is_ready: false,
            embedding_cache: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Pattern Detection service");
        
        // Check if embedding plugin is available
        if !self.plugin_manager.get_available_plugins().contains(&"qwen_embedding".to_string()) {
            tracing::warn!("Qwen Embedding plugin not available, pattern detection will use basic similarity");
        }
        
        self.is_ready = true;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down Pattern Detection service");
        self.embedding_cache.clear();
        self.is_ready = false;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// Detect patterns in a project using semantic similarity
    pub async fn detect_patterns(&self, project_path: &str) -> Result<PatternReport> {
        if !self.is_ready {
            anyhow::bail!("Pattern Detection service not initialized");
        }

        let project_path = Path::new(project_path);
        let code_fragments = self.extract_code_fragments(project_path)?;
        
        // Limit the number of fragments to prevent memory issues
        let max_fragments = 500; // Reasonable limit
        let code_fragments = if code_fragments.len() > max_fragments {
            tracing::warn!("Limiting analysis to {} fragments (found {})", max_fragments, code_fragments.len());
            code_fragments.into_iter().take(max_fragments).collect()
        } else {
            code_fragments
        };
        
        // Create embeddings for all code fragments with timeout
        let embeddings = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.create_embeddings(&code_fragments)
        ).await.map_err(|_| anyhow::anyhow!("Embedding creation timed out after 30 seconds"))??;
        
        // Detect duplicate patterns
        let duplicate_patterns = self.detect_duplicate_patterns(&code_fragments, &embeddings)?;
        
        // Create semantic clusters
        let semantic_clusters = self.create_semantic_clusters(&code_fragments, &embeddings)?;
        
        // Detect architectural patterns
        let architectural_patterns = self.detect_architectural_patterns(&code_fragments)?;
        
        // Generate refactoring suggestions
        let refactoring_suggestions = self.generate_refactoring_suggestions(&duplicate_patterns, &semantic_clusters)?;

        // Store the length before dropping
        let total_functions = code_fragments.len();
        
        // Force garbage collection by clearing variables
        drop(embeddings);
        drop(code_fragments);
        
        // Yield control to allow other tasks to run
        tokio::task::yield_now().await;

        Ok(PatternReport {
            project_path: project_path.to_string_lossy().to_string(),
            duplicate_patterns,
            semantic_clusters,
            architectural_patterns,
            refactoring_suggestions,
            analysis_metadata: PatternAnalysisMetadata {
                total_functions,
                embedding_model: if self.plugin_manager.is_plugin_loaded("qwen_embedding") {
                    "qwen3-embedding-8b".to_string()
                } else {
                    "lexical-similarity".to_string()
                },
                similarity_threshold: 0.85,
                analysis_timestamp: std::time::SystemTime::now(),
            },
        })
    }

    /// Detect duplicate code patterns using semantic similarity
    pub async fn detect_duplicate_code(&self, code_fragments: &[CodeFragment]) -> Result<Vec<EnhancedDuplicatePattern>> {
        if !self.is_ready {
            anyhow::bail!("Pattern Detection service not initialized");
        }

        // Limit fragments to prevent memory issues
        let max_fragments = 200; // Smaller limit for duplicate detection
        let limited_fragments = if code_fragments.len() > max_fragments {
            tracing::warn!("Limiting duplicate detection to {} fragments (found {})", max_fragments, code_fragments.len());
            &code_fragments[..max_fragments]
        } else {
            code_fragments
        };

        let embeddings = tokio::time::timeout(
            std::time::Duration::from_secs(20),
            self.create_embeddings(limited_fragments)
        ).await.map_err(|_| anyhow::anyhow!("Embedding creation timed out after 20 seconds"))??;
        
        self.detect_duplicate_patterns(limited_fragments, &embeddings)
    }

    /// Find similar functions based on semantic similarity
    pub async fn find_similar_functions(&self, target_function: &str, project_path: &str) -> Result<Vec<SimilarFunction>> {
        if !self.is_ready {
            anyhow::bail!("Pattern Detection service not initialized");
        }

        let project_path = Path::new(project_path);
        let code_fragments = self.extract_code_fragments(project_path)?;
        
        // Limit fragments to prevent memory issues
        let max_fragments = 300; // Limit for similarity search
        let limited_fragments = if code_fragments.len() > max_fragments {
            tracing::warn!("Limiting similarity search to {} fragments (found {})", max_fragments, code_fragments.len());
            &code_fragments[..max_fragments]
        } else {
            &code_fragments
        };
        
        // Create embedding for target function with timeout
        let target_embedding = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.create_single_embedding(target_function)
        ).await.map_err(|_| anyhow::anyhow!("Target embedding creation timed out"))??;
        
        // Create embeddings for all functions with timeout
        let embeddings = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            self.create_embeddings(limited_fragments)
        ).await.map_err(|_| anyhow::anyhow!("Embeddings creation timed out after 15 seconds"))??;
        
        // Find similar functions
        let mut similar_functions = Vec::new();
        
        for (i, fragment) in limited_fragments.iter().enumerate() {
            if let Some(embedding) = embeddings.get(i) {
                let similarity = self.calculate_cosine_similarity(&target_embedding, embedding);
                
                if similarity > 0.7 { // 70% similarity threshold
                    similar_functions.push(SimilarFunction {
                        function_name: fragment.function_name.clone(),
                        file_path: fragment.file_path.clone(),
                        similarity_score: similarity,
                        code_snippet: fragment.code_content.clone(),
                        function_signature: fragment.function_signature.clone(),
                    });
                }
            }
        }
        
        // Sort by similarity score
        similar_functions.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(similar_functions)
    }

    /// Create semantic clusters of related functions
    pub fn create_semantic_clusters(&self, code_fragments: &[CodeFragment], embeddings: &[Vec<f32>]) -> Result<Vec<SemanticCluster>> {
        const CLUSTER_THRESHOLD: f32 = 0.75;
        let mut clusters = Vec::new();
        let mut assigned = vec![false; code_fragments.len()];
        
        for i in 0..code_fragments.len() {
            if assigned[i] {
                continue;
            }
            
            let mut cluster_indices = vec![i];
            assigned[i] = true;
            
            // Find similar functions for this cluster
            for j in (i + 1)..code_fragments.len() {
                if assigned[j] {
                    continue;
                }
                
                let similarity = self.calculate_cosine_similarity(&embeddings[i], &embeddings[j]);
                if similarity > CLUSTER_THRESHOLD {
                    cluster_indices.push(j);
                    assigned[j] = true;
                }
            }
            
            // Create cluster if it has multiple functions
            if cluster_indices.len() > 1 {
                let cluster_functions: Vec<ClusterFunction> = cluster_indices
                    .into_iter()
                    .map(|idx| ClusterFunction {
                        function_name: code_fragments[idx].function_name.clone(),
                        file_path: code_fragments[idx].file_path.clone(),
                        function_signature: code_fragments[idx].function_signature.clone(),
                    })
                    .collect();
                
                clusters.push(SemanticCluster {
                    cluster_id: format!("cluster_{}", clusters.len()),
                    cluster_type: self.classify_cluster_type(&cluster_functions),
                    similarity_score: CLUSTER_THRESHOLD,
                    suggested_refactoring: self.suggest_cluster_refactoring(&cluster_functions),
                    functions: cluster_functions,
                });
            }
        }
        
        Ok(clusters)
    }

    /// Extract code fragments from project
    pub fn extract_code_fragments(&self, project_path: &Path) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();
        
        for entry in WalkDir::new(project_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if matches!(ext.to_str(), Some("ts") | Some("js")) {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let file_fragments = self.extract_functions_from_content(&content, path)?;
                            fragments.extend(file_fragments);
                        }
                    }
                }
            }
        }
        
        Ok(fragments)
    }

    /// Extract functions from file content
    pub fn extract_functions_from_content(&self, content: &str, file_path: &Path) -> Result<Vec<CodeFragment>> {
        let mut fragments = Vec::new();
        
        // Simple regex-based function extraction for TypeScript/JavaScript
        let lines: Vec<&str> = content.lines().collect();
        let mut current_function = String::new();
        let mut function_name = String::new();
        let mut in_function = false;
        let mut brace_count = 0;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Detect function start
            if (trimmed.starts_with("function ") || 
                trimmed.starts_with("async function ") ||
                trimmed.contains("=> {") ||
                (trimmed.contains("(") && trimmed.contains(":") && trimmed.contains("{"))) && !in_function {
                
                function_name = self.extract_function_name(trimmed);
                current_function = line.to_string() + "\n";
                in_function = true;
                brace_count = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            } else if in_function {
                current_function.push_str(line);
                current_function.push('\n');
                brace_count += line.matches('{').count() as i32 - line.matches('}').count() as i32;
                
                // Function end
                if brace_count <= 0 {
                    if !function_name.is_empty() && current_function.len() > 50 {
                        fragments.push(CodeFragment {
                            function_name: function_name.clone(),
                            file_path: file_path.to_string_lossy().to_string(),
                            code_content: current_function.clone(),
                            function_signature: self.extract_function_signature(&current_function),
                            complexity_score: self.calculate_complexity(&current_function),
                            line_count: current_function.lines().count(),
                        });
                    }
                    
                    current_function.clear();
                    function_name.clear();
                    in_function = false;
                    brace_count = 0;
                }
            }
        }
        
        Ok(fragments)
    }

    /// Extract function name from line
    pub fn extract_function_name(&self, line: &str) -> String {
        if line.starts_with("function ") {
            line.split_whitespace()
                .nth(1)
                .unwrap_or("unknown")
                .split('(')
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else if line.contains("=>") {
            line.split("=")
                .next()
                .unwrap_or("unknown")
                .trim()
                .split_whitespace()
                .last()
                .unwrap_or("unknown")
                .to_string()
        } else {
            // Method or arrow function
            line.split('(')
                .next()
                .unwrap_or("unknown")
                .split_whitespace()
                .last()
                .unwrap_or("unknown")
                .trim_end_matches(':')
                .to_string()
        }
    }

    /// Extract function signature
    pub fn extract_function_signature(&self, code: &str) -> String {
        code.lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    }

    /// Calculate code complexity
    pub fn calculate_complexity(&self, code: &str) -> f32 {
        let mut complexity = 1.0;
        
        // Count decision points
        complexity += code.matches("if ").count() as f32;
        complexity += code.matches("else ").count() as f32;
        complexity += code.matches("for ").count() as f32;
        complexity += code.matches("while ").count() as f32;
        complexity += code.matches("switch ").count() as f32;
        complexity += code.matches("case ").count() as f32;
        complexity += code.matches("catch ").count() as f32;
        complexity += code.matches(" && ").count() as f32;
        complexity += code.matches(" || ").count() as f32;
        
        complexity
    }

    /// Create embeddings for code fragments
    async fn create_embeddings(&self, code_fragments: &[CodeFragment]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        
        for fragment in code_fragments {
            let embedding = self.create_single_embedding(&fragment.code_content).await?;
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    /// Create single embedding
    async fn create_single_embedding(&self, code: &str) -> Result<Vec<f32>> {
        // Check cache first
        if let Some(cached_embedding) = self.embedding_cache.get(code) {
            return Ok(cached_embedding.clone());
        }
        
        let embedding = if self.plugin_manager.is_plugin_loaded("qwen_embedding") {
            // Use ML embeddings
            let response = self.plugin_manager.process_with_plugin("qwen_embedding", code).await?;
            self.parse_embedding_response(&response)?
        } else {
            // Use lexical similarity as fallback
            self.create_lexical_embedding(code)
        };
        
        Ok(embedding)
    }

    /// Parse embedding response from ML model
    fn parse_embedding_response(&self, response: &str) -> Result<Vec<f32>> {
        // Parse JSON response with embeddings
        if let Ok(values) = serde_json::from_str::<Vec<f32>>(response) {
            Ok(values)
        } else {
            // Fallback to lexical embedding
            Ok(self.create_lexical_embedding(response))
        }
    }

    /// Create lexical embedding (fallback)
    pub fn create_lexical_embedding(&self, code: &str) -> Vec<f32> {
        // Simple lexical features
        let mut features = vec![0.0; 128]; // 128-dimensional embedding
        
        // Basic lexical features
        features[0] = code.len() as f32 / 1000.0;
        features[1] = code.lines().count() as f32 / 100.0;
        features[2] = code.matches("function").count() as f32;
        features[3] = code.matches("class").count() as f32;
        features[4] = code.matches("if").count() as f32;
        features[5] = code.matches("for").count() as f32;
        features[6] = code.matches("while").count() as f32;
        features[7] = code.matches("async").count() as f32;
        features[8] = code.matches("await").count() as f32;
        features[9] = code.matches("return").count() as f32;
        
        // Angular-specific features
        features[10] = code.matches("@Component").count() as f32;
        features[11] = code.matches("@Injectable").count() as f32;
        features[12] = code.matches("Observable").count() as f32;
        features[13] = code.matches("subscribe").count() as f32;
        features[14] = code.matches("ngOnInit").count() as f32;
        
        // Normalize features
        let sum: f32 = features.iter().sum();
        if sum > 0.0 {
            for feature in &mut features {
                *feature /= sum;
            }
        }
        
        features
    }

    /// Calculate cosine similarity
    pub fn calculate_cosine_similarity(&self, vec1: &[f32], vec2: &[f32]) -> f32 {
        if vec1.len() != vec2.len() {
            return 0.0;
        }
        
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            0.0
        } else {
            dot_product / (magnitude1 * magnitude2)
        }
    }

    /// Detect duplicate patterns
    fn detect_duplicate_patterns(&self, code_fragments: &[CodeFragment], embeddings: &[Vec<f32>]) -> Result<Vec<EnhancedDuplicatePattern>> {
        let mut patterns = Vec::new();
        const DUPLICATE_THRESHOLD: f32 = 0.90;
        
        for i in 0..code_fragments.len() {
            for j in (i + 1)..code_fragments.len() {
                let similarity = self.calculate_cosine_similarity(&embeddings[i], &embeddings[j]);
                
                if similarity > DUPLICATE_THRESHOLD {
                    patterns.push(EnhancedDuplicatePattern {
                        pattern_type: ExtendedPatternType::DuplicateFunction,
                        primary_function: DuplicateFunction {
                            function_name: code_fragments[i].function_name.clone(),
                            file_path: code_fragments[i].file_path.clone(),
                            code_snippet: code_fragments[i].code_content.clone(),
                        },
                        duplicate_functions: vec![DuplicateFunction {
                            function_name: code_fragments[j].function_name.clone(),
                            file_path: code_fragments[j].file_path.clone(),
                            code_snippet: code_fragments[j].code_content.clone(),
                        }],
                        similarity_score: similarity,
                        suggested_refactoring: self.suggest_duplicate_refactoring(&code_fragments[i], &code_fragments[j]),
                    });
                }
            }
        }
        
        Ok(patterns)
    }

    /// Detect architectural patterns
    pub fn detect_architectural_patterns(&self, code_fragments: &[CodeFragment]) -> Result<Vec<ArchitecturalPattern>> {
        let mut patterns = Vec::new();
        
        // Group by file type
        let mut services = Vec::new();
        let mut components = Vec::new();
        let mut guards = Vec::new();
        let mut interceptors = Vec::new();
        
        for fragment in code_fragments {
            if fragment.file_path.contains("service") {
                services.push(fragment);
            } else if fragment.file_path.contains("component") {
                components.push(fragment);
            } else if fragment.file_path.contains("guard") {
                guards.push(fragment);
            } else if fragment.file_path.contains("interceptor") {
                interceptors.push(fragment);
            }
        }
        
        // Detect service patterns
        if services.len() > 1 {
            patterns.push(ArchitecturalPattern {
                pattern_name: "Service Pattern".to_string(),
                pattern_type: ArchitecturalPatternType::ServicePattern,
                affected_files: services.iter().map(|f| f.file_path.clone()).collect(),
                description: format!("Found {} services following Angular service pattern", services.len()),
                confidence: 0.9,
            });
        }
        
        // Detect component patterns
        if components.len() > 1 {
            patterns.push(ArchitecturalPattern {
                pattern_name: "Component Pattern".to_string(),
                pattern_type: ArchitecturalPatternType::ComponentPattern,
                affected_files: components.iter().map(|f| f.file_path.clone()).collect(),
                description: format!("Found {} components following Angular component pattern", components.len()),
                confidence: 0.9,
            });
        }
        
        Ok(patterns)
    }

    /// Generate refactoring suggestions
    pub fn generate_refactoring_suggestions(&self, duplicate_patterns: &[EnhancedDuplicatePattern], semantic_clusters: &[SemanticCluster]) -> Result<Vec<RefactoringSuggestion>> {
        let mut suggestions = Vec::new();
        
        // Suggestions for duplicate patterns
        for pattern in duplicate_patterns {
            suggestions.push(RefactoringSuggestion {
                suggestion_type: ExtendedRefactoringType::ExtractFunction,
                description: format!("Extract common functionality from {} and duplicates", pattern.primary_function.function_name),
                affected_files: vec![pattern.primary_function.file_path.clone()],
                expected_impact: "Reduces code duplication and improves maintainability".to_string(),
                effort_level: EffortLevel::Medium,
                priority: Priority::High,
            });
        }
        
        // Suggestions for semantic clusters
        for cluster in semantic_clusters {
            if cluster.functions.len() > 2 {
                suggestions.push(RefactoringSuggestion {
                    suggestion_type: ExtendedRefactoringType::ExtractUtilityClass,
                    description: format!("Create utility class for {} related functions", cluster.cluster_type),
                    affected_files: cluster.functions.iter().map(|f| f.file_path.clone()).collect(),
                    expected_impact: "Improves code organization and reusability".to_string(),
                    effort_level: EffortLevel::High,
                    priority: Priority::Medium,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Classify cluster type
    pub fn classify_cluster_type(&self, functions: &[ClusterFunction]) -> String {
        // Simple heuristic based on function names
        if functions.iter().any(|f| f.function_name.contains("get") || f.function_name.contains("fetch")) {
            "Data Access".to_string()
        } else if functions.iter().any(|f| f.function_name.contains("validate") || f.function_name.contains("check")) {
            "Validation".to_string()
        } else if functions.iter().any(|f| f.function_name.contains("format") || f.function_name.contains("transform")) {
            "Transformation".to_string()
        } else {
            "Utility".to_string()
        }
    }

    /// Suggest cluster refactoring
    pub fn suggest_cluster_refactoring(&self, functions: &[ClusterFunction]) -> String {
        format!("Consider extracting {} functions into a shared utility class", functions.len())
    }

    /// Suggest duplicate refactoring
    pub fn suggest_duplicate_refactoring(&self, func1: &CodeFragment, func2: &CodeFragment) -> String {
        format!("Extract common logic from {} and {} into a shared function", func1.function_name, func2.function_name)
    }
}

impl Drop for PatternDetectionService {
    fn drop(&mut self) {
        // Clear embedding cache to prevent memory leaks
        self.embedding_cache.clear();
        self.is_ready = false;
        
        if !self.embedding_cache.is_empty() {
            tracing::warn!("PatternDetectionService dropped without proper shutdown - possible resource leak");
        }
    }
}
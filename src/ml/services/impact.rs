//! Impact analysis service

use anyhow::Result;
use std::sync::Arc;
use std::path::Path;
use walkdir::WalkDir;

use crate::ml::config::MLConfig;
use crate::ml::plugins::PluginManager;
use crate::ml::models::*;

/// Advanced impact analysis service with ML enhancement
pub struct ImpactAnalysisService {
    config: MLConfig,
    plugin_manager: Arc<PluginManager>,
    is_ready: bool,
}

impl ImpactAnalysisService {
    pub fn new(config: MLConfig, plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            config,
            plugin_manager,
            is_ready: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Impact Analysis service");
        
        // Check if required plugins are available
        if !self.plugin_manager.get_available_plugins().contains(&"qwen_reranker".to_string()) {
            tracing::warn!("Qwen Reranker plugin not available, using fallback analysis");
        }
        
        self.is_ready = true;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down Impact Analysis service");
        self.is_ready = false;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// Comprehensive impact analysis with ML enhancement
    pub async fn analyze_impact(&self, changed_file: &str, changed_functions: &[String]) -> Result<ImpactReport> {
        if !self.is_ready {
            anyhow::bail!("Impact Analysis service not initialized");
        }

        // Create base impact analysis
        let base_impact = self.create_base_impact_analysis(changed_file, changed_functions)?;

        // Enhance with ML analysis if available
        if self.plugin_manager.get_available_plugins().contains(&"qwen_reranker".to_string()) {
            let semantic_impact = self.generate_semantic_impact_analysis(changed_file, changed_functions).await?;
            let risk_assessment = self.assess_change_risk(changed_file, changed_functions).await?;
            let recommendations = self.generate_recommendations(changed_file, changed_functions).await?;

            Ok(ImpactReport::Enhanced {
                base_impact,
                semantic_impact,
                risk_assessment,
                recommendations,
                confidence: 0.85,
            })
        } else {
            // Fallback to basic analysis
            Ok(ImpactReport::Basic {
                base_impact,
                confidence: 0.6,
            })
        }
    }

    /// Analyze impact on specific project (for calendario-psicologia testing)
    pub async fn analyze_project_impact(&self, project_path: &Path, changed_file: &str, changed_functions: &[String]) -> Result<ProjectImpactReport> {
        if !self.is_ready {
            anyhow::bail!("Impact Analysis service not initialized");
        }

        let project_files = self.discover_project_files(project_path)?;
        let filtered_files = self.filter_relevant_files(&project_files, changed_file, changed_functions)?;
        
        let mut impacted_files = Vec::new();
        
        for file in filtered_files {
            let impact = self.analyze_file_impact(&file, changed_file, changed_functions).await?;
            if impact.impact_score > 0.1 {
                impacted_files.push(impact);
            }
        }

        // Sort by impact score
        impacted_files.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ProjectImpactReport {
            project_path: project_path.to_string_lossy().to_string(),
            changed_file: changed_file.to_string(),
            changed_functions: changed_functions.to_vec(),
            impacted_files,
            analysis_timestamp: std::time::SystemTime::now(),
        })
    }

    /// Predict cascade effects of changes
    pub async fn predict_cascade_effects(&self, changed_file: &str, changed_functions: &[String]) -> Result<Vec<CascadeEffect>> {
        if !self.is_ready {
            anyhow::bail!("Impact Analysis service not initialized");
        }

        let mut effects = Vec::new();
        
        // Direct effects
        for function in changed_functions {
            let direct_effect = self.analyze_direct_effect(changed_file, function).await?;
            effects.push(direct_effect);
        }

        // Indirect effects through ML analysis
        if self.plugin_manager.is_plugin_loaded("qwen_reranker") {
            let indirect_effects = self.analyze_indirect_effects(changed_file, changed_functions).await?;
            effects.extend(indirect_effects);
        }

        Ok(effects)
    }

    /// Create base impact analysis without ML
    fn create_base_impact_analysis(&self, changed_file: &str, changed_functions: &[String]) -> Result<BaseImpactAnalysis> {
        let mut analysis = BaseImpactAnalysis {
            changed_file: changed_file.to_string(),
            changed_functions: changed_functions.to_vec(),
            direct_dependencies: Vec::new(),
            estimated_affected_files: Vec::new(),
            change_type: self.classify_change_type(changed_file, changed_functions),
            severity: self.assess_base_severity(changed_file, changed_functions),
        };

        // Basic dependency analysis
        analysis.direct_dependencies = self.extract_direct_dependencies(changed_file)?;
        analysis.estimated_affected_files = self.estimate_affected_files(changed_file, changed_functions)?;

        Ok(analysis)
    }

    /// Generate semantic impact analysis with ML
    async fn generate_semantic_impact_analysis(&self, changed_file: &str, changed_functions: &[String]) -> Result<SemanticImpactAnalysis> {
        let query = format!(
            "Analyze the semantic impact of changing functions {:?} in file {}",
            changed_functions, changed_file
        );
        
        let response = self.plugin_manager.process_with_plugin("qwen_reranker", &query).await?;
        
        Ok(SemanticImpactAnalysis {
            semantic_relationships: self.parse_semantic_relationships(&response)?,
            conceptual_changes: self.parse_conceptual_changes(&response)?,
            domain_impact: self.parse_domain_impact(&response)?,
            architectural_implications: self.parse_architectural_implications(&response)?,
        })
    }

    /// Assess risk of changes
    async fn assess_change_risk(&self, changed_file: &str, changed_functions: &[String]) -> Result<ChangeRiskAssessment> {
        let query = format!(
            "Assess the risk of modifying functions {:?} in file {}",
            changed_functions, changed_file
        );
        
        if self.plugin_manager.is_plugin_loaded("deepseek") {
            let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
            self.parse_risk_assessment(&response)
        } else {
            Ok(self.create_basic_risk_assessment(changed_file, changed_functions))
        }
    }

    /// Generate actionable recommendations
    async fn generate_recommendations(&self, changed_file: &str, changed_functions: &[String]) -> Result<Vec<ActionableRecommendation>> {
        let query = format!(
            "Generate recommendations for safely modifying functions {:?} in file {}",
            changed_functions, changed_file
        );
        
        if self.plugin_manager.is_plugin_loaded("deepseek") {
            let response = self.plugin_manager.process_with_plugin("deepseek", &query).await?;
            self.parse_recommendations(&response)
        } else {
            Ok(self.create_basic_recommendations(changed_file, changed_functions))
        }
    }

    /// Analyze impact on specific file
    async fn analyze_file_impact(&self, target_file: &str, changed_file: &str, changed_functions: &[String]) -> Result<FileImpactAnalysis> {
        let mut analysis = FileImpactAnalysis {
            file_path: target_file.to_string(),
            impact_score: 0.0,
            impact_type: ImpactType::None,
            affected_functions: Vec::new(),
            reasoning: String::new(),
        };

        // Basic impact calculation
        if target_file == changed_file {
            analysis.impact_score = 1.0;
            analysis.impact_type = ImpactType::Direct;
            analysis.affected_functions = changed_functions.to_vec();
            analysis.reasoning = "Same file as changed file".to_string();
        } else {
            // Check for imports/dependencies
            let import_score = self.calculate_import_impact(target_file, changed_file)?;
            let semantic_score = if self.plugin_manager.is_plugin_loaded("qwen_reranker") {
                self.calculate_semantic_impact(target_file, changed_file, changed_functions).await?
            } else {
                0.0
            };
            
            analysis.impact_score = (import_score + semantic_score) / 2.0;
            analysis.impact_type = if analysis.impact_score > 0.7 {
                ImpactType::Direct
            } else if analysis.impact_score > 0.3 {
                ImpactType::Indirect
            } else {
                ImpactType::Minimal
            };
            
            analysis.reasoning = format!(
                "Import impact: {:.2}, Semantic impact: {:.2}",
                import_score, semantic_score
            );
        }

        Ok(analysis)
    }

    /// Helper methods
    pub fn discover_project_files(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut files = Vec::new();
        
        // Use walkdir for recursive file discovery
        for entry in WalkDir::new(project_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if matches!(ext.to_str(), Some("ts") | Some("js") | Some("html") | Some("scss")) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(files)
    }

    fn filter_relevant_files(&self, project_files: &[String], changed_file: &str, _changed_functions: &[String]) -> Result<Vec<String>> {
        let mut relevant_files = Vec::new();
        
        for file in project_files {
            // Always include the changed file
            if file == changed_file {
                relevant_files.push(file.clone());
                continue;
            }
            
            // Include files that might import or depend on the changed file
            if self.might_be_related(file, changed_file)? {
                relevant_files.push(file.clone());
            }
        }
        
        Ok(relevant_files)
    }

    fn might_be_related(&self, file1: &str, file2: &str) -> Result<bool> {
        // Basic relatedness check
        if file1.contains("service") && file2.contains("service") {
            return Ok(true);
        }
        
        if file1.contains("component") && file2.contains("service") {
            return Ok(true);
        }
        
        // Check if they're in similar directories
        let path1 = Path::new(file1);
        let path2 = Path::new(file2);
        
        if let (Some(dir1), Some(dir2)) = (path1.parent(), path2.parent()) {
            return Ok(dir1 == dir2);
        }
        
        Ok(false)
    }

    fn classify_change_type(&self, changed_file: &str, changed_functions: &[String]) -> ChangeType {
        // Check for test files first (priority)
        if changed_file.contains(".spec.") || changed_file.contains(".test.") || changed_file.contains("/test/") {
            ChangeType::TestModification
        } else if changed_functions.iter().any(|f| f.contains("test")) {
            ChangeType::TestModification
        } else if changed_file.contains("service") {
            ChangeType::ServiceModification
        } else if changed_file.contains("component") {
            ChangeType::ComponentModification
        } else {
            ChangeType::CodeModification
        }
    }

    fn assess_base_severity(&self, changed_file: &str, changed_functions: &[String]) -> Severity {
        let mut score = 0;
        
        // File type impact
        if changed_file.contains("service") {
            score += 2;
        } else if changed_file.contains("component") {
            score += 1;
        }
        
        // Function count impact
        score += changed_functions.len();
        
        // Public/private impact
        if changed_functions.iter().any(|f| f.contains("public") || f.contains("export")) {
            score += 2;
        }
        
        match score {
            0..=2 => Severity::Low,
            3..=5 => Severity::Medium,
            6..=8 => Severity::High,
            _ => Severity::Critical,
        }
    }

    fn extract_direct_dependencies(&self, _changed_file: &str) -> Result<Vec<String>> {
        // TODO: Implement actual dependency extraction
        Ok(Vec::new())
    }

    fn estimate_affected_files(&self, _changed_file: &str, _changed_functions: &[String]) -> Result<Vec<String>> {
        // TODO: Implement file impact estimation
        Ok(Vec::new())
    }

    fn calculate_import_impact(&self, _target_file: &str, _changed_file: &str) -> Result<f32> {
        // TODO: Implement import impact calculation
        Ok(0.0)
    }

    async fn calculate_semantic_impact(&self, _target_file: &str, _changed_file: &str, _changed_functions: &[String]) -> Result<f32> {
        // TODO: Implement semantic impact calculation with ML
        Ok(0.0)
    }

    async fn analyze_direct_effect(&self, changed_file: &str, function: &str) -> Result<CascadeEffect> {
        Ok(CascadeEffect {
            effect_type: EffectType::Direct,
            affected_component: changed_file.to_string(),
            affected_function: function.to_string(),
            impact_level: ImpactLevel::High,
            description: format!("Direct modification of {} in {}", function, changed_file),
        })
    }

    async fn analyze_indirect_effects(&self, _changed_file: &str, _changed_functions: &[String]) -> Result<Vec<CascadeEffect>> {
        // TODO: Implement indirect effect analysis with ML
        Ok(Vec::new())
    }

    fn create_basic_risk_assessment(&self, changed_file: &str, changed_functions: &[String]) -> ChangeRiskAssessment {
        ChangeRiskAssessment {
            overall_risk: RiskLevel::Medium,
            breaking_change_probability: if changed_file.contains("service") { 0.7 } else { 0.3 },
            regression_risk: if changed_functions.len() > 3 { 0.6 } else { 0.3 },
            performance_impact: 0.2,
            security_implications: Vec::new(),
            mitigation_strategies: vec![
                "Run unit tests".to_string(),
                "Review integration tests".to_string(),
                "Monitor for breaking changes".to_string(),
            ],
        }
    }

    fn create_basic_recommendations(&self, changed_file: &str, changed_functions: &[String]) -> Vec<ActionableRecommendation> {
        vec![
            ActionableRecommendation {
                recommendation_type: RecommendationType::Testing,
                description: format!("Test all functions in {}", changed_file),
                priority: Priority::High,
                estimated_effort: EffortLevel::Medium,
                implementation_steps: vec![
                    "Run existing unit tests".to_string(),
                    "Add tests for modified functions".to_string(),
                    "Run integration tests".to_string(),
                ],
            },
            ActionableRecommendation {
                recommendation_type: RecommendationType::Review,
                description: format!("Review impact of changes to {:?}", changed_functions),
                priority: Priority::Medium,
                estimated_effort: EffortLevel::Low,
                implementation_steps: vec![
                    "Check for dependent components".to_string(),
                    "Review API contracts".to_string(),
                ],
            },
        ]
    }

    // Parsing methods for ML responses
    fn parse_semantic_relationships(&self, _response: &str) -> Result<Vec<SemanticRelationship>> {
        // TODO: Implement proper parsing
        Ok(Vec::new())
    }

    fn parse_conceptual_changes(&self, _response: &str) -> Result<Vec<ConceptualChange>> {
        // TODO: Implement proper parsing
        Ok(Vec::new())
    }

    fn parse_domain_impact(&self, _response: &str) -> Result<DomainImpact> {
        // TODO: Implement proper parsing
        Ok(DomainImpact {
            affected_domains: Vec::new(),
            cross_domain_effects: Vec::new(),
        })
    }

    fn parse_architectural_implications(&self, _response: &str) -> Result<Vec<ArchitecturalImplication>> {
        // TODO: Implement proper parsing
        Ok(Vec::new())
    }

    fn parse_risk_assessment(&self, _response: &str) -> Result<ChangeRiskAssessment> {
        // TODO: Implement proper parsing
        Ok(self.create_basic_risk_assessment("", &[]))
    }

    fn parse_recommendations(&self, _response: &str) -> Result<Vec<ActionableRecommendation>> {
        // TODO: Implement proper parsing
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::config::MLConfig;

    #[tokio::test]
    async fn test_impact_analysis_service_creation() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = ImpactAnalysisService::new(config, plugin_manager);
        
        assert!(!service.is_ready());
    }

    #[tokio::test]
    async fn test_impact_analysis_service_initialization() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let mut service = ImpactAnalysisService::new(config, plugin_manager);
        
        let result = service.initialize().await;
        assert!(result.is_ok());
        assert!(service.is_ready());
    }

    #[tokio::test]
    async fn test_change_type_classification() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = ImpactAnalysisService::new(config, plugin_manager);
        
        // Test service classification
        let service_type = service.classify_change_type("auth.service.ts", &vec!["login".to_string()]);
        assert_eq!(service_type, ChangeType::ServiceModification);
        
        // Test component classification
        let component_type = service.classify_change_type("calendar.component.ts", &vec!["ngOnInit".to_string()]);
        assert_eq!(component_type, ChangeType::ComponentModification);
        
        // Test test classification
        let test_type = service.classify_change_type("auth.service.spec.ts", &vec!["testLogin".to_string()]);
        assert_eq!(test_type, ChangeType::TestModification);
    }

    #[tokio::test]
    async fn test_severity_assessment() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = ImpactAnalysisService::new(config, plugin_manager);
        
        // Test low severity
        let low_severity = service.assess_base_severity("helper.ts", &vec!["format".to_string()]);
        assert_eq!(low_severity, Severity::Low);
        
        // Test medium severity
        let medium_severity = service.assess_base_severity("auth.service.ts", &vec!["login".to_string()]);
        assert_eq!(medium_severity, Severity::Medium);
        
        // Test high severity
        let high_severity = service.assess_base_severity("auth.service.ts", &vec!["public_login".to_string(), "public_logout".to_string(), "export_user".to_string()]);
        assert_eq!(high_severity, Severity::High);
    }

    #[tokio::test]
    async fn test_file_relatedness() {
        let config = MLConfig::for_testing();
        let plugin_manager = Arc::new(PluginManager::new());
        let service = ImpactAnalysisService::new(config, plugin_manager);
        
        // Test service-to-service relatedness
        let is_related = service.might_be_related("user.service.ts", "auth.service.ts").unwrap();
        assert!(is_related);
        
        // Test component-to-service relatedness
        let is_related = service.might_be_related("login.component.ts", "auth.service.ts").unwrap();
        assert!(is_related);
        
        // Test same directory relatedness
        let is_related = service.might_be_related("src/app/services/user.service.ts", "src/app/services/auth.service.ts").unwrap();
        assert!(is_related);
    }
}

#[cfg(test)]
mod impact_test;
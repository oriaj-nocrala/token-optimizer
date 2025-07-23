//! ML model data structures and types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod downloader;
pub use downloader::{ModelDownloader, ModelInfo};

/// Smart context analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContext {
    pub function_name: String,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub dependencies: Vec<DependencyInfo>,
    pub usage_patterns: Vec<UsagePattern>,
    pub complexity_score: f32,
    pub impact_scope: ImpactScope,
    pub recommendations: Vec<String>,
}

/// Enhanced smart context with ML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSmartContext {
    pub base_context: SmartContext,
    pub semantic_analysis: SemanticAnalysis,
    pub risk_assessment: RiskAssessment,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

/// Smart context result from service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContextResult {
    pub ast_context: String,
    pub semantic_context: Option<SemanticContext>,
    pub confidence: f32,
    pub processing_time: std::time::Duration,
}

/// Semantic context from ML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticContext {
    pub related_functions: Vec<String>,
    pub conceptual_context: String,
    pub usage_patterns: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    pub dependency_type: DependencyType,
    pub source_file: String,
    pub target_file: String,
    pub functions: Vec<String>,
    pub strength: f32, // 0.0 to 1.0
}

/// Types of dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    Import,
    Export,
    FunctionCall,
    Inheritance,
    Composition,
    DataFlow,
}

/// Usage pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub pattern_type: PatternType,
    pub frequency: usize,
    pub confidence: f32,
    pub examples: Vec<String>,
}

/// Types of usage patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternType {
    CreationalPattern,
    StructuralPattern,
    BehavioralPattern,
    ArchitecturalPattern,
    AntiPattern,
}

/// Impact scope levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactScope {
    Local,     // Single function/method
    Component, // Single component/class
    Service,   // Single service/module
    Global,    // Cross-cutting concerns
}

/// Semantic analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub purpose: String,
    pub behavior_description: String,
    pub key_concepts: Vec<String>,
    pub semantic_relationships: Vec<SemanticRelationship>,
    pub context_relevance: f32,
}

/// Semantic relationship between code elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRelationship {
    pub relationship_type: RelationshipType,
    pub source: String,
    pub target: String,
    pub strength: f32,
    pub description: String,
}

/// Types of semantic relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    Implements,
    Uses,
    Depends,
    Extends,
    Aggregates,
    Composes,
    Invokes,
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub breaking_change_risk: f32,
    pub performance_impact: f32,
    pub security_implications: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: OptimizationType,
    pub description: String,
    pub expected_benefit: String,
    pub implementation_effort: EffortLevel,
    pub priority: Priority,
}

/// Types of optimizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationType {
    Performance,
    Memory,
    Maintainability,
    Security,
    Architecture,
    Testing,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Legacy impact entry (kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyImpactReport {
    pub changed_files: Vec<String>,
    pub changed_functions: Vec<String>,
    pub direct_impact: Vec<ImpactEntry>,
    pub indirect_impact: Vec<ImpactEntry>,
    pub risk_analysis: RiskAssessment,
    pub suggested_actions: Vec<String>,
    pub tests_to_run: Vec<String>,
}

/// Individual impact entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEntry {
    pub file_path: String,
    pub affected_functions: Vec<String>,
    pub impact_type: ImpactType,
    pub confidence: f32,
    pub reasoning: String,
}

/// Types of impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactType {
    Direct,
    Indirect,
    Transitive,
    Potential,
    Minimal,
    None,
}

/// Enhanced impact report with ML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactReport {
    Basic {
        base_impact: BaseImpactAnalysis,
        confidence: f32,
    },
    Enhanced {
        base_impact: BaseImpactAnalysis,
        semantic_impact: SemanticImpactAnalysis,
        risk_assessment: ChangeRiskAssessment,
        recommendations: Vec<ActionableRecommendation>,
        confidence: f32,
    },
}

/// Base impact analysis without ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseImpactAnalysis {
    pub changed_file: String,
    pub changed_functions: Vec<String>,
    pub direct_dependencies: Vec<String>,
    pub estimated_affected_files: Vec<String>,
    pub change_type: ChangeType,
    pub severity: Severity,
}

/// Semantic impact analysis with ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticImpactAnalysis {
    pub semantic_relationships: Vec<SemanticRelationship>,
    pub conceptual_changes: Vec<ConceptualChange>,
    pub domain_impact: DomainImpact,
    pub architectural_implications: Vec<ArchitecturalImplication>,
}

/// Change risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRiskAssessment {
    pub overall_risk: RiskLevel,
    pub breaking_change_probability: f32,
    pub regression_risk: f32,
    pub performance_impact: f32,
    pub security_implications: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}

/// Actionable recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionableRecommendation {
    pub recommendation_type: RecommendationType,
    pub description: String,
    pub priority: Priority,
    pub estimated_effort: EffortLevel,
    pub implementation_steps: Vec<String>,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationType {
    Testing,
    Review,
    Refactoring,
    Documentation,
    Monitoring,
    SecurityReview,
}

/// Project impact report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectImpactReport {
    pub project_path: String,
    pub changed_file: String,
    pub changed_functions: Vec<String>,
    pub impacted_files: Vec<FileImpactAnalysis>,
    pub analysis_timestamp: std::time::SystemTime,
}

/// File impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileImpactAnalysis {
    pub file_path: String,
    pub impact_score: f32,
    pub impact_type: ImpactType,
    pub affected_functions: Vec<String>,
    pub reasoning: String,
}

/// Cascade effect prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeEffect {
    pub effect_type: EffectType,
    pub affected_component: String,
    pub affected_function: String,
    pub impact_level: ImpactLevel,
    pub description: String,
}

/// Effect types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffectType {
    Direct,
    Indirect,
    Cascading,
    Ripple,
}

/// Impact levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Change types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    ServiceModification,
    ComponentModification,
    TestModification,
    CodeModification,
    ConfigurationChange,
    DatabaseChange,
    ArchitecturalChange,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Conceptual change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptualChange {
    pub concept: String,
    pub change_type: String,
    pub impact_description: String,
}

/// Domain impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainImpact {
    pub affected_domains: Vec<String>,
    pub cross_domain_effects: Vec<String>,
}

/// Architectural implication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalImplication {
    pub component: String,
    pub implication: String,
    pub severity: Severity,
}


/// Legacy pattern detection result (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyPatternReport {
    pub duplicates: Vec<DuplicatePattern>,
    pub design_patterns: Vec<DesignPattern>,
    pub anti_patterns: Vec<AntiPattern>,
    pub refactoring_opportunities: Vec<RefactoringOpportunity>,
}

/// Duplicate code pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicatePattern {
    pub similarity: f32,
    pub locations: Vec<CodeLocation>,
    pub pattern_type: String,
    pub refactoring_suggestion: String,
}

/// Design pattern identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPattern {
    pub pattern_name: String,
    pub confidence: f32,
    pub locations: Vec<CodeLocation>,
    pub description: String,
}

/// Anti-pattern identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub pattern_name: String,
    pub severity: Severity,
    pub locations: Vec<CodeLocation>,
    pub description: String,
    pub fix_suggestion: String,
}

/// Refactoring opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringOpportunity {
    pub opportunity_type: RefactoringType,
    pub description: String,
    pub locations: Vec<CodeLocation>,
    pub expected_benefit: String,
    pub effort_estimate: EffortLevel,
}

/// Types of refactoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefactoringType {
    ExtractMethod,
    ExtractClass,
    ExtractInterface,
    MoveMethod,
    RenameMethod,
    InlineMethod,
    RemoveDuplication,
    SimplifyConditional,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub function_name: Option<String>,
    pub class_name: Option<String>,
}

/// Severity levels (already defined above)
pub type SeverityLevel = Severity;

/// Semantic search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub results: Vec<SearchMatch>,
    pub total_matches: usize,
    pub search_time_ms: u64,
}

/// Individual search match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: String,
    pub relevance_score: f32,
    pub context: String,
    pub key_functions: Vec<String>,
    pub snippet: String,
    pub location: CodeLocation,
}

/// Token optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOptimization {
    pub task: String,
    pub token_budget: usize,
    pub recommended_files: Vec<FileRecommendation>,
    pub excluded_files: Vec<String>,
    pub optimization_strategy: OptimizationStrategy,
    pub estimated_tokens: usize,
}

/// File recommendation for token optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecommendation {
    pub file_path: String,
    pub priority: Priority,
    pub sections: Vec<String>,
    pub estimated_tokens: usize,
    pub relevance_score: f32,
}

/// Optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub focus_areas: Vec<String>,
    pub skip_areas: Vec<String>,
    pub context_reduction: f32,
    pub summarization_level: SummarizationLevel,
}

/// Summarization levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SummarizationLevel {
    None,
    Light,
    Medium,
    Aggressive,
}

/// Development session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentSession {
    pub session_id: Uuid,
    pub task_description: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub progress: SessionProgress,
    pub context: SessionContext,
    pub recommendations: Vec<String>,
}

/// Session progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProgress {
    pub completed_tasks: Vec<String>,
    pub in_progress_tasks: Vec<String>,
    pub pending_tasks: Vec<String>,
    pub blocked_tasks: Vec<String>,
}

/// Session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub files_modified: Vec<String>,
    pub patterns_established: Vec<String>,
    pub decisions_made: Vec<String>,
    pub next_actions: Vec<String>,
}

impl SmartContext {
    pub fn new(function_name: String, file_path: String, line_range: (usize, usize)) -> Self {
        Self {
            function_name,
            file_path,
            line_range,
            dependencies: Vec::new(),
            usage_patterns: Vec::new(),
            complexity_score: 0.0,
            impact_scope: ImpactScope::Local,
            recommendations: Vec::new(),
        }
    }
}

impl LegacyImpactReport {
    pub fn new() -> Self {
        Self {
            changed_files: Vec::new(),
            changed_functions: Vec::new(),
            direct_impact: Vec::new(),
            indirect_impact: Vec::new(),
            risk_analysis: RiskAssessment {
                overall_risk: RiskLevel::Low,
                breaking_change_risk: 0.0,
                performance_impact: 0.0,
                security_implications: Vec::new(),
                mitigation_strategies: Vec::new(),
            },
            suggested_actions: Vec::new(),
            tests_to_run: Vec::new(),
        }
    }
}

impl LegacyPatternReport {
    pub fn new() -> Self {
        Self {
            duplicates: Vec::new(),
            design_patterns: Vec::new(),
            anti_patterns: Vec::new(),
            refactoring_opportunities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_context_creation() {
        let context = SmartContext::new(
            "testFunction".to_string(),
            "src/test.ts".to_string(),
            (10, 20),
        );

        assert_eq!(context.function_name, "testFunction");
        assert_eq!(context.file_path, "src/test.ts");
        assert_eq!(context.line_range, (10, 20));
        assert_eq!(context.impact_scope, ImpactScope::Local);
    }

    #[test]
    fn test_impact_report_creation() {
        let report = LegacyImpactReport::new();

        assert!(report.changed_files.is_empty());
        assert!(report.direct_impact.is_empty());
        assert_eq!(report.risk_analysis.overall_risk, RiskLevel::Low);
    }

    #[test]
    fn test_pattern_report_creation() {
        let report = LegacyPatternReport::new();

        assert!(report.duplicates.is_empty());
        assert!(report.design_patterns.is_empty());
        assert!(report.anti_patterns.is_empty());
        assert!(report.refactoring_opportunities.is_empty());
    }

    #[test]
    fn test_dependency_type_serialization() {
        let dep_type = DependencyType::Import;
        let serialized = serde_json::to_string(&dep_type).unwrap();
        let deserialized: DependencyType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(dep_type, deserialized);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_code_location_serialization() {
        let location = CodeLocation {
            file_path: "src/test.ts".to_string(),
            line_start: 10,
            line_end: 20,
            function_name: Some("testFunction".to_string()),
            class_name: None,
        };

        let serialized = serde_json::to_string(&location).unwrap();
        let deserialized: CodeLocation = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(location.file_path, deserialized.file_path);
        assert_eq!(location.line_start, deserialized.line_start);
        assert_eq!(location.function_name, deserialized.function_name);
    }
}

// ============================================================================
// Pattern Detection Models
// ============================================================================

/// Code fragment for pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFragment {
    pub function_name: String,
    pub file_path: String,
    pub code_content: String,
    pub function_signature: String,
    pub complexity_score: f32,
    pub line_count: usize,
}

/// Pattern report with semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternReport {
    pub project_path: String,
    pub duplicate_patterns: Vec<EnhancedDuplicatePattern>,
    pub semantic_clusters: Vec<SemanticCluster>,
    pub architectural_patterns: Vec<ArchitecturalPattern>,
    pub refactoring_suggestions: Vec<RefactoringSuggestion>,
    pub analysis_metadata: PatternAnalysisMetadata,
}

/// Extended pattern types for pattern detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtendedPatternType {
    DuplicateFunction,
    SimilarLogic,
    CodeClone,
    ArchitecturalPattern,
}

/// Duplicate function information for pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateFunction {
    pub function_name: String,
    pub file_path: String,
    pub code_snippet: String,
}

/// Enhanced duplicate pattern detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedDuplicatePattern {
    pub pattern_type: ExtendedPatternType,
    pub primary_function: DuplicateFunction,
    pub duplicate_functions: Vec<DuplicateFunction>,
    pub similarity_score: f32,
    pub suggested_refactoring: String,
}

/// Semantic cluster of similar functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCluster {
    pub cluster_id: String,
    pub cluster_type: String,
    pub functions: Vec<ClusterFunction>,
    pub similarity_score: f32,
    pub suggested_refactoring: String,
}

/// Function in a semantic cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterFunction {
    pub function_name: String,
    pub file_path: String,
    pub function_signature: String,
}

/// Similar function result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFunction {
    pub function_name: String,
    pub file_path: String,
    pub similarity_score: f32,
    pub code_snippet: String,
    pub function_signature: String,
}

/// Architectural pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalPattern {
    pub pattern_name: String,
    pub pattern_type: ArchitecturalPatternType,
    pub affected_files: Vec<String>,
    pub description: String,
    pub confidence: f32,
}

/// Architectural pattern types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArchitecturalPatternType {
    ServicePattern,
    ComponentPattern,
    SingletonPattern,
    FactoryPattern,
    ObserverPattern,
    StrategyPattern,
    AdapterPattern,
}

/// Refactoring suggestion with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub suggestion_type: ExtendedRefactoringType,
    pub description: String,
    pub affected_files: Vec<String>,
    pub expected_impact: String,
    pub effort_level: EffortLevel,
    pub priority: Priority,
}

/// Extended refactoring types (additional to main RefactoringType)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtendedRefactoringType {
    ExtractFunction,
    ExtractClass,
    ExtractUtilityClass,
    MergeClasses,
    SplitClass,
    InlineFunction,
    MoveMethod,
    RenameMethod,
}

/// Pattern analysis metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysisMetadata {
    pub total_functions: usize,
    pub embedding_model: String,
    pub similarity_threshold: f32,
    pub analysis_timestamp: std::time::SystemTime,
}
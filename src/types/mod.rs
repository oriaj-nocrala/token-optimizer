use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub line_count: usize,
    pub last_modified: DateTime<Utc>,
    pub file_type: FileType,
    pub summary: String,
    pub relevant_sections: Vec<String>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub complexity: Complexity,
    pub detailed_analysis: Option<DetailedAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailedAnalysis {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub interfaces: Vec<InterfaceInfo>,
    pub enums: Vec<EnumInfo>,
    pub types: Vec<TypeAliasInfo>,
    pub variables: Vec<VariableInfo>,
    pub component_info: Option<ComponentInfo>,
    pub service_info: Option<ServiceInfo>,
    pub pipe_info: Option<PipeInfo>,
    pub module_info: Option<ModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Component,
    Service,
    Style,
    Config,
    Test,
    Pipe,
    Module,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Complexity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeSummary {
    pub file_name: String,
    pub file_type: String,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub components: Vec<ComponentInfo>,
    pub services: Vec<ServiceInfo>,
    pub pipes: Vec<PipeInfo>,
    pub modules: Vec<ModuleInfo>,
    pub key_patterns: Vec<String>,
    pub dependencies: Vec<String>,
    pub scss_variables: Option<Vec<String>>,
    pub scss_mixins: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: String,
    pub is_async: bool,
    pub modifiers: Vec<String>,
    pub location: LocationInfo,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationInfo {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<FunctionInfo>,
    pub properties: Vec<PropertyInfo>,
    pub extends: Option<String>,
    pub implements: Vec<String>,
    pub modifiers: Vec<String>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertyInfo {
    pub name: String,
    pub prop_type: String,
    pub modifiers: Vec<String>,
    pub location: LocationInfo,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterfaceInfo {
    pub name: String,
    pub properties: Vec<PropertyInfo>,
    pub methods: Vec<FunctionInfo>,
    pub extends: Vec<String>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TypeAliasInfo {
    pub name: String,
    pub type_definition: String,
    pub generics: Vec<String>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: String,
    pub is_const: bool,
    pub is_exported: bool,
    pub location: LocationInfo,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentInfo {
    pub name: String,
    pub selector: String,
    pub inputs: Vec<PropertyInfo>,
    pub outputs: Vec<PropertyInfo>,
    pub lifecycle: Vec<String>,
    pub template_summary: String,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceInfo {
    pub name: String,
    pub injectable: bool,
    pub provided_in: Option<String>,
    pub scope: ServiceScope,
    pub dependencies: Vec<ParameterInfo>,
    pub methods: Vec<FunctionInfo>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceScope {
    Root,        // providedIn: 'root'
    Platform,    // providedIn: 'platform'
    Module,      // provided in specific module
    Component,   // provided in component
    Singleton,   // module-level singleton
    Transient,   // new instance each time
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipeInfo {
    pub name: String,
    pub transform_method: FunctionInfo,
    pub is_pure: bool,
    pub is_standalone: bool,
    pub dependencies: Vec<ParameterInfo>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleInfo {
    pub name: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub declarations: Vec<String>,
    pub providers: Vec<String>,
    pub bootstrap: Vec<String>,
    pub schemas: Vec<String>,
    pub is_root_module: bool,
    pub is_feature_module: bool,
    pub is_shared_module: bool,
    pub lazy_routes: Vec<LazyRouteInfo>,
    pub location: LocationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LazyRouteInfo {
    pub path: String,
    pub module_path: String,
    pub component: Option<String>,
    pub preload_strategy: Option<String>,
    pub can_load_guards: Vec<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSummary {
    pub name: String,
    pub path: String,
    pub module_type: ModuleType,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub declarations: Vec<String>,
    pub providers: Vec<String>,
    pub lazy_routes: Vec<LazyRouteInfo>,
    pub feature_areas: Vec<String>,
    pub shared_resources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    Root,        // AppModule
    Feature,     // Feature modules (UserModule, ProductModule, etc.)
    Shared,      // Shared modules (SharedModule, CoreModule)
    Lazy,        // Lazy-loaded modules
    Routing,     // Routing modules (AppRoutingModule, FeatureRoutingModule)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheEntry {
    pub file_hash: String,
    pub last_analyzed: DateTime<Utc>,
    pub summary: CodeSummary,
    pub metadata: FileMetadata,
    pub change_log: Vec<ChangeLogEntry>,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeLogEntry {
    pub timestamp: DateTime<Utc>,
    pub change_type: ChangeType,
    pub description: String,
    pub lines_changed: usize,
    pub impact_level: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangeAnalysis {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub modified_files: Vec<ModifiedFile>,
    pub added_files: Vec<String>,
    pub deleted_files: Vec<String>,
    pub renamed_files: Vec<RenamedFile>,
    pub impact_scope: ImpactScope,
    pub relevant_context: Vec<String>,
    pub suggested_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifiedFile {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub sections_changed: Vec<String>,
    pub impacted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RenamedFile {
    pub old_path: String,
    pub new_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactScope {
    Local,
    Component,
    Service,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectOverview {
    pub project_name: String,
    pub last_updated: DateTime<Utc>,
    pub structure: ProjectStructure,
    pub recent_changes: ChangeAnalysis,
    pub active_features: Vec<String>,
    pub technical_stack: TechStack,
    pub health_metrics: HealthMetrics,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectStructure {
    pub components: Vec<ComponentSummary>,
    pub services: Vec<ServiceSummary>,
    pub pipes: Vec<PipeSummary>,
    pub modules: Vec<ModuleSummary>,
    pub styles: StyleSummary,
    pub routes: Vec<RouteSummary>,
    pub routing_analysis: RoutingAnalysis,
    pub interceptor_analysis: InterceptorAnalysis,
    pub state_management: StateManagementAnalysis,
    pub module_analysis: ModuleAnalysis,
    pub assets: AssetSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentSummary {
    pub name: String,
    pub path: String,
    pub complexity: Complexity,
    pub dependencies: Vec<String>,
    pub functions: Vec<String>,        // Function names from AST
    pub inputs: Vec<String>,           // @Input properties
    pub outputs: Vec<String>,          // @Output properties
    pub lifecycle_hooks: Vec<String>,  // OnInit, OnDestroy, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceSummary {
    pub name: String,
    pub path: String,
    pub injectable: bool,
    pub provided_in: Option<String>,
    pub scope: ServiceScope,
    pub dependencies: Vec<String>,
    pub functions: Vec<String>,       // Function names from AST
    pub observables: Vec<String>,     // Observable properties
    pub methods: Vec<String>,         // Public method names
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipeSummary {
    pub name: String,
    pub path: String,
    pub pipe_name: String,            // Name used in templates
    pub is_pure: bool,
    pub is_standalone: bool,
    pub dependencies: Vec<String>,
    pub transform_signature: String,  // transform(value: any, ...args: any[])
    pub use_cases: Vec<String>,       // Common usage patterns
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleSummary {
    pub variables: Vec<String>,
    pub mixins: Vec<String>,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteSummary {
    pub path: String,
    pub component: String,
    pub guards: Vec<String>,
    pub redirect_to: Option<String>,
    pub is_protected: bool,
    pub lazy_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GuardSummary {
    pub name: String,
    pub path: String,
    pub guard_type: GuardType,
    pub dependencies: Vec<String>,
    pub protected_routes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GuardType {
    CanActivate,
    CanDeactivate,
    CanLoad,
    Resolve,
    CanMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoutingAnalysis {
    pub routes: Vec<RouteSummary>,
    pub guards: Vec<GuardSummary>,
    pub protected_routes: Vec<RouteSummary>,
    pub redirects: Vec<RouteSummary>,
    pub lazy_routes: Vec<RouteSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterceptorSummary {
    pub name: String,
    pub path: String,
    pub interceptor_type: InterceptorType,
    pub dependencies: Vec<String>,
    pub handles_errors: bool,
    pub modifies_requests: bool,
    pub modifies_responses: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterceptorType {
    HttpInterceptorFn,  // Functional interceptor (Angular 14+)
    HttpInterceptor,    // Class-based interceptor (older)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterceptorAnalysis {
    pub interceptors: Vec<InterceptorSummary>,
    pub error_handlers: Vec<InterceptorSummary>,
    pub auth_interceptors: Vec<InterceptorSummary>,
    pub logging_interceptors: Vec<InterceptorSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateSummary {
    pub service_name: String,
    pub service_path: String,
    pub state_properties: Vec<StateProperty>,
    pub observables: Vec<ObservableProperty>,
    pub state_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateProperty {
    pub name: String,
    pub property_type: StateType,
    pub is_private: bool,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservableProperty {
    pub name: String,
    pub source_property: Option<String>,
    pub observable_type: ObservableType,
    pub is_readonly: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateType {
    BehaviorSubject,
    Subject,
    ReplaySubject,
    AsyncSubject,
    Signal,      // Angular 16+ signals
    WritableSignal,
    ComputedSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ObservableType {
    Observable,
    BehaviorSubject,
    Subject,
    ReplaySubject,
    AsyncSubject,
    Computed,
    Signal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateManagementAnalysis {
    pub services_with_state: Vec<StateSummary>,
    pub total_state_properties: usize,
    pub total_observables: usize,
    pub patterns_detected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleAnalysis {
    pub modules: Vec<ModuleSummary>,
    pub root_module: Option<ModuleSummary>,
    pub feature_modules: Vec<ModuleSummary>,
    pub shared_modules: Vec<ModuleSummary>,
    pub lazy_modules: Vec<ModuleSummary>,
    pub routing_modules: Vec<ModuleSummary>,
    pub lazy_loading_analysis: LazyLoadingAnalysis,
    pub dependency_graph: Vec<ModuleDependency>,
    pub service_scope_analysis: ServiceScopeAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceScopeAnalysis {
    pub root_services: Vec<ServiceSummary>,
    pub platform_services: Vec<ServiceSummary>,
    pub module_services: Vec<ServiceSummary>,
    pub component_services: Vec<ServiceSummary>,
    pub singleton_services: Vec<ServiceSummary>,
    pub transient_services: Vec<ServiceSummary>,
    pub scope_violations: Vec<ScopeViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopeViolation {
    pub service_name: String,
    pub violation_type: ScopeViolationType,
    pub description: String,
    pub recommended_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScopeViolationType {
    CircularDependency,
    InvalidScope,
    ScopeLeakage,
    DuplicateProvider,
    MissingProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LazyLoadingAnalysis {
    pub lazy_routes: Vec<LazyRouteInfo>,
    pub preload_strategies: Vec<String>,
    pub chunk_analysis: Vec<ChunkInfo>,
    pub loading_performance: LoadingPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChunkInfo {
    pub name: String,
    pub estimated_size: Option<u64>,
    pub dependencies: Vec<String>,
    pub preloaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoadingPerformance {
    pub total_lazy_routes: usize,
    pub preloaded_routes: usize,
    pub estimated_chunk_sizes: Vec<(String, u64)>,
    pub loading_bottlenecks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleDependency {
    pub module_name: String,
    pub depends_on: Vec<String>,
    pub imported_by: Vec<String>,
    pub circular_dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssetSummary {
    pub images: Vec<String>,
    pub fonts: Vec<String>,
    pub icons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TechStack {
    pub framework: String,
    pub language: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthMetrics {
    pub code_complexity: Complexity,
    pub test_coverage: f64,
    pub build_health: BuildHealth,
    pub bundle_size: u64,
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BuildHealth {
    Passing,
    Warnings,
    Failing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMetrics {
    pub load_time: f64,
    pub bundle_size: u64,
    pub memory_usage: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_file_metadata_creation() {
        let metadata = FileMetadata {
            path: "src/main.rs".to_string(),
            size: 1024,
            line_count: 50,
            last_modified: Utc::now(),
            file_type: FileType::Component,
            summary: "Main application entry point".to_string(),
            relevant_sections: vec!["main function".to_string()],
            exports: vec!["main".to_string()],
            imports: vec!["std::io".to_string()],
            complexity: Complexity::Low,
            detailed_analysis: None,
        };

        assert_eq!(metadata.path, "src/main.rs");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.line_count, 50);
        assert_eq!(metadata.file_type, FileType::Component);
        assert_eq!(metadata.complexity, Complexity::Low);
    }

    #[test]
    fn test_file_type_serialization() {
        let file_type = FileType::Service;
        let serialized = serde_json::to_string(&file_type).unwrap();
        let deserialized: FileType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(file_type, deserialized);
    }

    #[test]
    fn test_complexity_enum() {
        assert_eq!(Complexity::Low, Complexity::Low);
        assert_ne!(Complexity::Low, Complexity::High);
        
        let complexity = Complexity::Medium;
        let json = serde_json::to_string(&complexity).unwrap();
        let parsed: Complexity = serde_json::from_str(&json).unwrap();
        assert_eq!(complexity, parsed);
    }

    #[test]
    fn test_function_info_with_async() {
        let func = FunctionInfo {
            name: "fetch_data".to_string(),
            parameters: vec![ParameterInfo {
                name: "url".to_string(),
                param_type: "String".to_string(),
                is_optional: false,
                default_value: None,
            }],
            return_type: "Result<String>".to_string(),
            is_async: true,
            modifiers: Vec::new(),
            location: LocationInfo { line: 1, column: 1 },
            description: Some("Fetches data from URL".to_string()),
        };

        assert!(func.is_async);
        assert_eq!(func.name, "fetch_data");
        assert_eq!(func.parameters.len(), 1);
        assert!(func.description.is_some());
    }

    #[test]
    fn test_component_info_structure() {
        let component = ComponentInfo {
            name: "UserComponent".to_string(),
            selector: "app-user".to_string(),
            inputs: vec![PropertyInfo {
                name: "userId".to_string(),
                prop_type: "string".to_string(),
                modifiers: vec!["@Input()".to_string()],
                location: LocationInfo { line: 1, column: 1 },
                initial_value: None,
            }],
            outputs: vec![PropertyInfo {
                name: "userChanged".to_string(),
                prop_type: "EventEmitter".to_string(),
                modifiers: vec!["@Output()".to_string()],
                location: LocationInfo { line: 1, column: 1 },
                initial_value: None,
            }],
            lifecycle: vec!["ngOnInit".to_string(), "ngOnDestroy".to_string()],
            template_summary: "User profile display".to_string(),
            location: LocationInfo { line: 1, column: 1 },
        };

        assert_eq!(component.name, "UserComponent");
        assert_eq!(component.selector, "app-user");
        assert_eq!(component.inputs.len(), 1);
        assert_eq!(component.outputs.len(), 1);
        assert_eq!(component.lifecycle.len(), 2);
    }

    #[test]
    fn test_cache_entry_full_cycle() {
        let now = Utc::now();
        let metadata = FileMetadata {
            path: "test.rs".to_string(),
            size: 100,
            line_count: 10,
            last_modified: now,
            file_type: FileType::Test,
            summary: "Test file".to_string(),
            relevant_sections: vec![],
            exports: vec![],
            imports: vec![],
            complexity: Complexity::Low,
            detailed_analysis: None,
        };

        let summary = CodeSummary {
            file_name: "test.rs".to_string(),
            file_type: "Test".to_string(),
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

        let cache_entry = CacheEntry {
            file_hash: "abc123".to_string(),
            last_analyzed: now,
            summary,
            metadata,
            change_log: vec![],
            dependencies: vec![],
            dependents: vec![],
        };

        let json = serde_json::to_string(&cache_entry).unwrap();
        let parsed: CacheEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(cache_entry, parsed);
    }

    #[test]
    fn test_change_analysis_impact_scope() {
        let change = ChangeAnalysis {
            session_id: "session_123".to_string(),
            timestamp: Utc::now(),
            modified_files: vec![],
            added_files: vec!["new_file.rs".to_string()],
            deleted_files: vec![],
            renamed_files: vec![],
            impact_scope: ImpactScope::Global,
            relevant_context: vec!["Added new module".to_string()],
            suggested_actions: vec!["Run tests".to_string()],
        };

        assert_eq!(change.impact_scope, ImpactScope::Global);
        assert_eq!(change.added_files.len(), 1);
        assert_eq!(change.suggested_actions.len(), 1);
    }

    #[test]
    fn test_project_overview_structure() {
        let mut dependencies = HashMap::new();
        dependencies.insert("serde".to_string(), "1.0".to_string());
        
        let tech_stack = TechStack {
            framework: "Rust".to_string(),
            language: "Rust".to_string(),
            dependencies,
            dev_dependencies: HashMap::new(),
        };

        let health_metrics = HealthMetrics {
            code_complexity: Complexity::Medium,
            test_coverage: 85.5,
            build_health: BuildHealth::Passing,
            bundle_size: 2048,
            performance: PerformanceMetrics {
                load_time: 1.5,
                bundle_size: 2048,
                memory_usage: 1024,
            },
        };

        let project_structure = ProjectStructure {
            components: vec![],
            services: vec![],
            pipes: vec![],
            modules: vec![],
            styles: StyleSummary {
                variables: vec![],
                mixins: vec![],
                components: vec![],
            },
            routes: vec![],
            routing_analysis: RoutingAnalysis {
                routes: vec![],
                guards: vec![],
                protected_routes: vec![],
                redirects: vec![],
                lazy_routes: vec![],
            },
            interceptor_analysis: InterceptorAnalysis {
                interceptors: vec![],
                error_handlers: vec![],
                auth_interceptors: vec![],
                logging_interceptors: vec![],
            },
            state_management: StateManagementAnalysis {
                services_with_state: vec![],
                total_state_properties: 0,
                total_observables: 0,
                patterns_detected: vec![],
            },
            module_analysis: ModuleAnalysis {
                modules: vec![],
                root_module: None,
                feature_modules: vec![],
                shared_modules: vec![],
                lazy_modules: vec![],
                routing_modules: vec![],
                lazy_loading_analysis: LazyLoadingAnalysis {
                    lazy_routes: vec![],
                    preload_strategies: vec![],
                    chunk_analysis: vec![],
                    loading_performance: LoadingPerformance {
                        total_lazy_routes: 0,
                        preloaded_routes: 0,
                        estimated_chunk_sizes: vec![],
                        loading_bottlenecks: vec![],
                    },
                },
                dependency_graph: vec![],
                service_scope_analysis: ServiceScopeAnalysis {
                    root_services: vec![],
                    platform_services: vec![],
                    module_services: vec![],
                    component_services: vec![],
                    singleton_services: vec![],
                    transient_services: vec![],
                    scope_violations: vec![],
                },
            },
            assets: AssetSummary {
                images: vec![],
                fonts: vec![],
                icons: vec![],
            },
        };

        let overview = ProjectOverview {
            project_name: "token-optimizer".to_string(),
            last_updated: Utc::now(),
            structure: project_structure,
            recent_changes: ChangeAnalysis {
                session_id: "test".to_string(),
                timestamp: Utc::now(),
                modified_files: vec![],
                added_files: vec![],
                deleted_files: vec![],
                renamed_files: vec![],
                impact_scope: ImpactScope::Local,
                relevant_context: vec![],
                suggested_actions: vec![],
            },
            active_features: vec!["caching".to_string()],
            technical_stack: tech_stack,
            health_metrics,
            recommendations: vec!["Add more tests".to_string()],
        };

        assert_eq!(overview.project_name, "token-optimizer");
        assert_eq!(overview.technical_stack.framework, "Rust");
        assert_eq!(overview.health_metrics.test_coverage, 85.5);
        assert_eq!(overview.health_metrics.build_health, BuildHealth::Passing);
    }

    #[test]
    fn test_enum_equality() {
        assert_eq!(FileType::Component, FileType::Component);
        assert_ne!(FileType::Component, FileType::Service);
        
        assert_eq!(ChangeType::Created, ChangeType::Created);
        assert_ne!(ChangeType::Created, ChangeType::Modified);
        
        assert_eq!(ImpactLevel::High, ImpactLevel::High);
        assert_ne!(ImpactLevel::High, ImpactLevel::Low);
    }

    #[test]
    fn test_module_info_creation() {
        let module_info = ModuleInfo {
            name: "TestModule".to_string(),
            imports: vec!["CommonModule".to_string()],
            exports: vec!["TestComponent".to_string()],
            declarations: vec!["TestComponent".to_string()],
            providers: vec!["TestService".to_string()],
            bootstrap: vec![],
            schemas: vec![],
            is_root_module: false,
            is_feature_module: true,
            is_shared_module: false,
            lazy_routes: vec![],
            location: LocationInfo { line: 1, column: 1 },
        };

        assert_eq!(module_info.name, "TestModule");
        assert_eq!(module_info.imports.len(), 1);
        assert_eq!(module_info.exports.len(), 1);
        assert_eq!(module_info.declarations.len(), 1);
        assert_eq!(module_info.providers.len(), 1);
        assert!(module_info.is_feature_module);
        assert!(!module_info.is_root_module);
        assert!(!module_info.is_shared_module);
    }

    #[test]
    fn test_service_scope_enum() {
        let service_info = ServiceInfo {
            name: "TestService".to_string(),
            injectable: true,
            provided_in: Some("root".to_string()),
            scope: ServiceScope::Root,
            dependencies: vec![],
            methods: vec![],
            location: LocationInfo { line: 1, column: 1 },
        };

        assert_eq!(service_info.scope, ServiceScope::Root);
        assert_eq!(service_info.provided_in, Some("root".to_string()));
        assert!(service_info.injectable);
    }

    #[test]
    fn test_module_type_enum() {
        let module_summary = ModuleSummary {
            name: "AppModule".to_string(),
            path: "src/app/app.module.ts".to_string(),
            module_type: ModuleType::Root,
            imports: vec!["BrowserModule".to_string()],
            exports: vec![],
            declarations: vec!["AppComponent".to_string()],
            providers: vec![],
            lazy_routes: vec![],
            feature_areas: vec![],
            shared_resources: vec![],
        };

        assert_eq!(module_summary.module_type, ModuleType::Root);
        assert_eq!(module_summary.name, "AppModule");
        assert_eq!(module_summary.imports.len(), 1);
        assert_eq!(module_summary.declarations.len(), 1);
    }

    #[test]
    fn test_lazy_route_info() {
        let lazy_route = LazyRouteInfo {
            path: "/dashboard".to_string(),
            module_path: "./dashboard/dashboard.module".to_string(),
            component: Some("DashboardComponent".to_string()),
            preload_strategy: Some("PreloadAllModules".to_string()),
            can_load_guards: vec!["AuthGuard".to_string()],
            data: Some("{ title: 'Dashboard' }".to_string()),
        };

        assert_eq!(lazy_route.path, "/dashboard");
        assert_eq!(lazy_route.module_path, "./dashboard/dashboard.module");
        assert_eq!(lazy_route.component, Some("DashboardComponent".to_string()));
        assert_eq!(lazy_route.preload_strategy, Some("PreloadAllModules".to_string()));
        assert_eq!(lazy_route.can_load_guards.len(), 1);
        assert!(lazy_route.data.is_some());
    }

    #[test]
    fn test_service_scope_analysis() {
        let root_service = ServiceSummary {
            name: "AuthService".to_string(),
            path: "src/app/services/auth.service.ts".to_string(),
            injectable: true,
            provided_in: Some("root".to_string()),
            scope: ServiceScope::Root,
            dependencies: vec![],
            functions: vec![],
            observables: vec![],
            methods: vec![],
        };

        let scope_analysis = ServiceScopeAnalysis {
            root_services: vec![root_service],
            platform_services: vec![],
            module_services: vec![],
            component_services: vec![],
            singleton_services: vec![],
            transient_services: vec![],
            scope_violations: vec![],
        };

        assert_eq!(scope_analysis.root_services.len(), 1);
        assert_eq!(scope_analysis.root_services[0].scope, ServiceScope::Root);
        assert_eq!(scope_analysis.scope_violations.len(), 0);
    }

    #[test]
    fn test_scope_violation() {
        let violation = ScopeViolation {
            service_name: "TestService".to_string(),
            violation_type: ScopeViolationType::CircularDependency,
            description: "Circular dependency detected".to_string(),
            recommended_fix: "Refactor to remove circular dependency".to_string(),
        };

        assert_eq!(violation.service_name, "TestService");
        assert_eq!(violation.violation_type, ScopeViolationType::CircularDependency);
        assert!(!violation.description.is_empty());
        assert!(!violation.recommended_fix.is_empty());
    }

    #[test]
    fn test_service_scope_serialization() {
        let scopes = vec![
            ServiceScope::Root,
            ServiceScope::Platform,
            ServiceScope::Module,
            ServiceScope::Component,
            ServiceScope::Singleton,
            ServiceScope::Transient,
        ];

        for scope in scopes {
            let serialized = serde_json::to_string(&scope).unwrap();
            let deserialized: ServiceScope = serde_json::from_str(&serialized).unwrap();
            assert_eq!(scope, deserialized);
        }
    }

    #[test]
    fn test_module_type_serialization() {
        let module_types = vec![
            ModuleType::Root,
            ModuleType::Feature,
            ModuleType::Shared,
            ModuleType::Lazy,
            ModuleType::Routing,
        ];

        for module_type in module_types {
            let serialized = serde_json::to_string(&module_type).unwrap();
            let deserialized: ModuleType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(module_type, deserialized);
        }
    }
}
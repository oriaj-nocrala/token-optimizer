use anyhow::Result;
use std::path::Path;
use crate::types::{StateManagementAnalysis, StateSummary, StateProperty, ObservableProperty, StateType, ObservableType};
use crate::utils::file_utils;
use crate::cache::CacheManager;

pub struct StateAnalyzer;

impl StateAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_project_state(&self, cache_manager: &CacheManager) -> Result<StateManagementAnalysis> {
        let mut analysis = StateManagementAnalysis {
            services_with_state: Vec::new(),
            total_state_properties: 0,
            total_observables: 0,
            patterns_detected: Vec::new(),
        };

        // Analyze all services in the cache for state management patterns
        for (cached_path, entry) in &cache_manager.get_cache().entries {
            if matches!(entry.metadata.file_type, crate::types::FileType::Service) {
                // Use the actual file path from the cache entry metadata
                let actual_path = &entry.metadata.path;
                if let Ok(state_summary) = self.analyze_service_file(actual_path) {
                    if !state_summary.state_properties.is_empty() || !state_summary.observables.is_empty() {
                        analysis.total_state_properties += state_summary.state_properties.len();
                        analysis.total_observables += state_summary.observables.len();
                        analysis.services_with_state.push(state_summary);
                    }
                }
            }
        }

        // Detect common patterns
        analysis.patterns_detected = self.detect_patterns(&analysis.services_with_state);

        Ok(analysis)
    }

    fn analyze_service_file(&self, file_path: &str) -> Result<StateSummary> {
        let content = file_utils::read_file_content(Path::new(file_path))?;
        self.analyze_service_content(file_path, &content)
    }

    fn analyze_service_content(&self, file_path: &str, content: &str) -> Result<StateSummary> {
        let service_name = self.extract_service_name(file_path);
        
        let state_properties = self.extract_state_properties(content);
        let observables = self.extract_observables(content);
        let state_methods = self.extract_state_methods(content);

        Ok(StateSummary {
            service_name,
            service_path: file_path.to_string(),
            state_properties,
            observables,
            state_methods,
        })
    }

    fn extract_service_name(&self, file_path: &str) -> String {
        let base_name = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .replace(".service", "")
            .split('.')
            .next()
            .unwrap_or("Unknown")
            .to_string();
        
        // Capitalize first letter and add "Service"
        let mut chars: Vec<char> = base_name.chars().collect();
        if !chars.is_empty() {
            chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
        }
        let capitalized: String = chars.into_iter().collect();
        capitalized + "Service"
    }

    fn extract_state_properties(&self, content: &str) -> Vec<StateProperty> {
        let mut properties = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Look for BehaviorSubject, Subject, etc.
            if let Some(property) = self.parse_state_property_line(trimmed) {
                properties.push(property);
            }
        }

        properties
    }

    fn parse_state_property_line(&self, line: &str) -> Option<StateProperty> {
        // Match patterns like:
        // private authState = new BehaviorSubject<AuthState>({...});
        // private userSubject = new Subject<User>();
        // readonly signal = signal(initialValue);
        
        if line.contains("BehaviorSubject") {
            return self.extract_property_info(line, StateType::BehaviorSubject);
        }
        
        if line.contains("Subject") && !line.contains("BehaviorSubject") {
            return self.extract_property_info(line, StateType::Subject);
        }
        
        if line.contains("ReplaySubject") {
            return self.extract_property_info(line, StateType::ReplaySubject);
        }
        
        if line.contains("AsyncSubject") {
            return self.extract_property_info(line, StateType::AsyncSubject);
        }
        
        if line.contains("signal(") {
            return self.extract_property_info(line, StateType::Signal);
        }
        
        if line.contains("writableSignal(") {
            return self.extract_property_info(line, StateType::WritableSignal);
        }
        
        if line.contains("computed(") {
            return self.extract_property_info(line, StateType::ComputedSignal);
        }

        None
    }

    fn extract_property_info(&self, line: &str, state_type: StateType) -> Option<StateProperty> {
        // Extract property name and metadata
        let is_private = line.contains("private ");
        let is_readonly = line.contains("readonly ");
        
        // Find property name before the = sign
        if let Some(equals_pos) = line.find('=') {
            let left_side = &line[..equals_pos].trim();
            
            // Extract the property name (last word before =)
            let words: Vec<&str> = left_side.split_whitespace().collect();
            if let Some(property_name) = words.last() {
                let name = property_name.to_string();
                
                // Try to extract initial value
                let initial_value = self.extract_initial_value(line);
                
                return Some(StateProperty {
                    name,
                    property_type: state_type,
                    is_private: is_private || !is_readonly, // Assume private if not explicitly readonly
                    initial_value,
                });
            }
        }

        None
    }

    fn extract_initial_value(&self, line: &str) -> Option<String> {
        // Try to extract the initial value from constructor calls
        if let Some(paren_start) = line.find('(') {
            if let Some(paren_end) = line.rfind(')') {
                if paren_end > paren_start {
                    let inner = &line[paren_start + 1..paren_end];
                    if !inner.trim().is_empty() {
                        return Some(inner.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_observables(&self, content: &str) -> Vec<ObservableProperty> {
        let mut observables = Vec::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            
            // Look for observable declarations
            if let Some(observable) = self.parse_observable_line(trimmed) {
                observables.push(observable);
            }
        }

        observables
    }

    fn parse_observable_line(&self, line: &str) -> Option<ObservableProperty> {
        // Match patterns like:
        // public readonly authState$ = this.authState.asObservable();
        // public readonly user$ = this.authState$.pipe(map(state => state.user));
        // readonly isAuthenticated$ = computed(() => this.authState().isAuthenticated);
        // private userStream: Observable<User> = this.http.get('/api/user');
        // public dataFlow: Observable<Data> = this.service.getData();
        
        // 1. Check for $ naming convention (existing logic)
        if line.contains("$") && (line.contains("=") || line.contains("asObservable") || line.contains("pipe")) {
            let is_readonly = line.contains("readonly ");
            
            // Extract observable name
            if let Some(dollar_pos) = line.find('$') {
                let before_dollar = &line[..dollar_pos + 1];
                let words: Vec<&str> = before_dollar.split_whitespace().collect();
                
                if let Some(observable_name) = words.last() {
                    let name = observable_name.to_string();
                    
                    // Determine observable type and source
                    let (observable_type, source_property) = self.analyze_observable_source(line);
                    
                    return Some(ObservableProperty {
                        name,
                        source_property,
                        observable_type,
                        is_readonly,
                    });
                }
            }
        }
        
        // 2. Check for TypeScript type annotations (new logic)
        if (line.contains(": Observable<") || 
            line.contains(": BehaviorSubject<") || 
            line.contains(": Subject<") || 
            line.contains(": ReplaySubject<") || 
            line.contains(": AsyncSubject<")) && line.contains("=") {
            
            let is_readonly = line.contains("readonly ");
            
            // Extract property name before the colon
            if let Some(colon_pos) = line.find(':') {
                let before_colon = &line[..colon_pos];
                let words: Vec<&str> = before_colon.split_whitespace().collect();
                
                if let Some(property_name) = words.last() {
                    let name = property_name.to_string();
                    
                    // Determine observable type from type annotation
                    let observable_type = if line.contains(": BehaviorSubject<") {
                        ObservableType::BehaviorSubject
                    } else if line.contains(": Subject<") {
                        ObservableType::Subject
                    } else if line.contains(": ReplaySubject<") {
                        ObservableType::ReplaySubject
                    } else if line.contains(": AsyncSubject<") {
                        ObservableType::AsyncSubject
                    } else {
                        ObservableType::Observable
                    };
                    
                    return Some(ObservableProperty {
                        name,
                        source_property: None,
                        observable_type,
                        is_readonly,
                    });
                }
            }
        }
        
        // 3. Check for common observable creation patterns without $ suffix
        if (line.contains("asObservable()") || 
            line.contains(".pipe(") || 
            line.contains("new Observable(") ||
            line.contains("new BehaviorSubject(") ||
            line.contains("new Subject(") ||
            line.contains("this.http.get") ||
            line.contains("this.http.post") ||
            line.contains("this.http.put") ||
            line.contains("this.http.delete")) && line.contains("=") {
            
            let is_readonly = line.contains("readonly ");
            
            // Extract property name before the =
            if let Some(equals_pos) = line.find('=') {
                let left_side = &line[..equals_pos].trim();
                let words: Vec<&str> = left_side.split_whitespace().collect();
                
                if let Some(property_name) = words.last() {
                    let name = property_name.to_string();
                    
                    // Determine observable type from creation pattern
                    let (observable_type, source_property) = self.analyze_observable_source(line);
                    
                    return Some(ObservableProperty {
                        name,
                        source_property,
                        observable_type,
                        is_readonly,
                    });
                }
            }
        }
        
        // 4. Also check for computed signals
        if line.contains("computed(") {
            if let Some(equals_pos) = line.find('=') {
                let left_side = &line[..equals_pos].trim();
                let words: Vec<&str> = left_side.split_whitespace().collect();
                
                if let Some(property_name) = words.last() {
                    return Some(ObservableProperty {
                        name: property_name.to_string(),
                        source_property: None,
                        observable_type: ObservableType::Computed,
                        is_readonly: line.contains("readonly "),
                    });
                }
            }
        }

        None
    }

    fn analyze_observable_source(&self, line: &str) -> (ObservableType, Option<String>) {
        // Check what kind of observable this is based on the source
        
        // Direct creation patterns
        if line.contains("new BehaviorSubject(") {
            return (ObservableType::BehaviorSubject, None);
        }
        
        if line.contains("new Subject(") && !line.contains("BehaviorSubject") {
            return (ObservableType::Subject, None);
        }
        
        if line.contains("new ReplaySubject(") {
            return (ObservableType::ReplaySubject, None);
        }
        
        if line.contains("new AsyncSubject(") {
            return (ObservableType::AsyncSubject, None);
        }
        
        if line.contains("new Observable(") {
            return (ObservableType::Observable, None);
        }
        
        // HTTP Client observables
        if line.contains("this.http.get") || line.contains("this.http.post") || 
           line.contains("this.http.put") || line.contains("this.http.delete") {
            return (ObservableType::Observable, None);
        }
        
        // AsObservable pattern
        if line.contains("asObservable()") {
            // Extract source property name
            if let Some(this_pos) = line.find("this.") {
                let after_this = &line[this_pos + 5..];
                if let Some(dot_pos) = after_this.find('.') {
                    let source_name = after_this[..dot_pos].to_string();
                    return (ObservableType::Observable, Some(source_name));
                }
            }
        }
        
        // Pipe operations
        if line.contains("pipe(") {
            return (ObservableType::Observable, None);
        }
        
        // Type annotation patterns (fallback from type detection)
        if line.contains("BehaviorSubject") {
            return (ObservableType::BehaviorSubject, None);
        }
        
        if line.contains("Subject") && !line.contains("BehaviorSubject") {
            return (ObservableType::Subject, None);
        }
        
        if line.contains("ReplaySubject") {
            return (ObservableType::ReplaySubject, None);
        }
        
        if line.contains("AsyncSubject") {
            return (ObservableType::AsyncSubject, None);
        }
        
        if line.contains("computed(") {
            return (ObservableType::Computed, None);
        }

        (ObservableType::Observable, None)
    }

    fn extract_state_methods(&self, content: &str) -> Vec<String> {
        let mut methods = Vec::new();
        
        // Look for methods that likely manage state
        for line in content.lines() {
            let trimmed = line.trim();
            
            if trimmed.contains("fn ") || trimmed.contains("function ") {
                continue; // Skip function declarations (not TypeScript methods)
            }
            
            // Look for method signatures
            if let Some(method_name) = self.extract_method_name(trimmed) {
                if self.is_state_management_method(&method_name, content) {
                    methods.push(method_name);
                }
            }
        }

        methods
    }

    fn extract_method_name(&self, line: &str) -> Option<String> {
        // Match patterns like:
        // updateAuthState(isAuthenticated: boolean, user: User | null, loading: boolean): void {
        // private setLoadingState(loading: boolean): void {
        
        if line.contains("(") && (line.contains("):") || line.contains(") {")) {
            let paren_pos = line.find('(')?;
            let before_paren = &line[..paren_pos];
            
            // Get the last word before the parenthesis
            let words: Vec<&str> = before_paren.split_whitespace().collect();
            if let Some(method_name) = words.last() {
                return Some(method_name.to_string());
            }
        }

        None
    }

    fn is_state_management_method(&self, method_name: &str, content: &str) -> bool {
        let lower_name = method_name.to_lowercase();
        
        // Check if method name suggests state management
        let state_keywords = [
            "update", "set", "clear", "reset", "init", "store", "load", "refresh", 
            "state", "auth", "user", "loading", "next", "emit", "trigger"
        ];
        
        for keyword in state_keywords {
            if lower_name.contains(keyword) {
                return true;
            }
        }
        
        // Check if method body contains state-related operations
        let method_start = format!("{}(", method_name);
        if let Some(method_pos) = content.find(&method_start) {
            // Get approximate method body (next 10 lines)
            let lines_after: Vec<&str> = content[method_pos..].lines().take(10).collect();
            let method_body = lines_after.join(" ");
            
            if method_body.contains(".next(") || 
               method_body.contains(".emit(") || 
               method_body.contains(".update(") ||
               method_body.contains("BehaviorSubject") ||
               method_body.contains("Subject") {
                return true;
            }
        }

        false
    }

    fn detect_patterns(&self, services: &[StateSummary]) -> Vec<String> {
        let mut patterns = Vec::new();
        
        let total_services = services.len();
        let services_with_behavior_subjects = services.iter()
            .filter(|s| s.state_properties.iter().any(|p| matches!(p.property_type, StateType::BehaviorSubject)))
            .count();
        
        let services_with_observables = services.iter()
            .filter(|s| !s.observables.is_empty())
            .count();
        
        let services_with_readonly_observables = services.iter()
            .filter(|s| s.observables.iter().any(|o| o.is_readonly))
            .count();

        if services_with_behavior_subjects > 0 {
            patterns.push(format!("BehaviorSubject pattern used in {} services", services_with_behavior_subjects));
        }
        
        if services_with_observables > 0 {
            patterns.push(format!("Observable streams in {} services", services_with_observables));
        }
        
        if services_with_readonly_observables > 0 {
            patterns.push(format!("Readonly observable pattern in {} services", services_with_readonly_observables));
        }
        
        // Check for common naming patterns
        let dollar_suffix_count = services.iter()
            .map(|s| s.observables.iter().filter(|o| o.name.ends_with('$')).count())
            .sum::<usize>();
        
        if dollar_suffix_count > 0 {
            patterns.push(format!("Observable naming convention ($ suffix) used {} times", dollar_suffix_count));
        }
        
        // Check for state encapsulation pattern
        let encapsulated_state_count = services.iter()
            .filter(|s| {
                s.state_properties.iter().any(|p| p.is_private) &&
                s.observables.iter().any(|o| o.is_readonly)
            })
            .count();
        
        if encapsulated_state_count > 0 {
            patterns.push(format!("State encapsulation pattern (private state + readonly observables) in {} services", encapsulated_state_count));
        }

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use crate::cache::CacheManager;

    #[test]
    fn test_state_analyzer_creation() {
        let analyzer = StateAnalyzer::new();
        assert_eq!(std::mem::size_of_val(&analyzer), 0);
    }

    #[test]
    fn test_parse_behavior_subject() {
        let analyzer = StateAnalyzer::new();
        
        let line = "  private authState = new BehaviorSubject<AuthState>({ isAuthenticated: false });";
        let property = analyzer.parse_state_property_line(line);
        
        assert!(property.is_some());
        let prop = property.unwrap();
        assert_eq!(prop.name, "authState");
        assert!(matches!(prop.property_type, StateType::BehaviorSubject));
        assert!(prop.is_private);
        assert!(prop.initial_value.is_some());
    }

    #[test]
    fn test_parse_observable_property() {
        let analyzer = StateAnalyzer::new();
        
        let line = "  public readonly authState$ = this.authState.asObservable();";
        let observable = analyzer.parse_observable_line(line);
        
        assert!(observable.is_some());
        let obs = observable.unwrap();
        assert_eq!(obs.name, "authState$");
        assert!(obs.is_readonly);
        assert!(matches!(obs.observable_type, ObservableType::Observable));
        assert_eq!(obs.source_property, Some("authState".to_string()));
    }

    #[test]
    fn test_analyze_auth_service() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache_manager = CacheManager::new(temp_dir.path())?;
        
        let auth_service_content = r#"
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable, map } from 'rxjs';

interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  loading: boolean;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private authState = new BehaviorSubject<AuthState>({
    isAuthenticated: false,
    user: null,
    loading: true
  });

  public readonly authState$ = this.authState.asObservable();
  public readonly isAuthenticated$ = this.authState$.pipe(map(state => state.isAuthenticated));
  public readonly user$ = this.authState$.pipe(map(state => state.user));
  public readonly loading$ = this.authState$.pipe(map(state => state.loading));

  updateAuthState(isAuthenticated: boolean, user: User | null, loading: boolean): void {
    this.authState.next({
      isAuthenticated,
      user,
      loading
    });
  }

  private setLoadingState(loading: boolean): void {
    const currentState = this.authState.value;
    this.updateAuthState(currentState.isAuthenticated, currentState.user, loading);
  }
}
"#;
        
        let service_file = temp_dir.path().join("auth.service.ts");
        fs::write(&service_file, auth_service_content)?;
        
        // Analyze the file
        cache_manager.analyze_file(&service_file)?;
        
        let analyzer = StateAnalyzer::new();
        let analysis = analyzer.analyze_project_state(&cache_manager)?;
        
        
        assert_eq!(analysis.services_with_state.len(), 1);
        
        let auth_service = &analysis.services_with_state[0];
        assert_eq!(auth_service.service_name, "AuthService");
        
        // Check state properties
        assert_eq!(auth_service.state_properties.len(), 1);
        let auth_state_prop = &auth_service.state_properties[0];
        assert_eq!(auth_state_prop.name, "authState");
        assert!(matches!(auth_state_prop.property_type, StateType::BehaviorSubject));
        assert!(auth_state_prop.is_private);
        
        // Check observables
        assert_eq!(auth_service.observables.len(), 4);
        let observables: Vec<&str> = auth_service.observables.iter().map(|o| o.name.as_str()).collect();
        assert!(observables.contains(&"authState$"));
        assert!(observables.contains(&"isAuthenticated$"));
        assert!(observables.contains(&"user$"));
        assert!(observables.contains(&"loading$"));
        
        // Check state methods
        assert!(!auth_service.state_methods.is_empty());
        assert!(auth_service.state_methods.contains(&"updateAuthState".to_string()));
        
        // Check patterns
        assert!(!analysis.patterns_detected.is_empty());
        assert!(analysis.patterns_detected.iter().any(|p| p.contains("BehaviorSubject pattern")));
        assert!(analysis.patterns_detected.iter().any(|p| p.contains("Observable naming convention")));
        
        Ok(())
    }

    #[test]
    fn test_extract_state_methods() {
        let analyzer = StateAnalyzer::new();
        
        let content = r#"
        updateAuthState(isAuthenticated: boolean): void {
          this.authState.next({ isAuthenticated });
        }
        
        private setLoadingState(loading: boolean): void {
          this.authState.next({ loading });
        }
        
        regularMethod(): string {
          return "hello";
        }
        "#;
        
        let methods = analyzer.extract_state_methods(content);
        
        assert!(methods.contains(&"updateAuthState".to_string()));
        assert!(methods.contains(&"setLoadingState".to_string()));
        // regularMethod should not be included as it doesn't manage state
    }

    #[test]
    fn test_detect_patterns() {
        let analyzer = StateAnalyzer::new();
        
        let services = vec![
            StateSummary {
                service_name: "AuthService".to_string(),
                service_path: "auth.service.ts".to_string(),
                state_properties: vec![
                    StateProperty {
                        name: "authState".to_string(),
                        property_type: StateType::BehaviorSubject,
                        is_private: true,
                        initial_value: None,
                    }
                ],
                observables: vec![
                    ObservableProperty {
                        name: "authState$".to_string(),
                        source_property: Some("authState".to_string()),
                        observable_type: ObservableType::Observable,
                        is_readonly: true,
                    }
                ],
                state_methods: vec!["updateAuthState".to_string()],
            }
        ];
        
        let patterns = analyzer.detect_patterns(&services);
        
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.contains("BehaviorSubject pattern")));
        assert!(patterns.iter().any(|p| p.contains("Observable naming convention")));
        assert!(patterns.iter().any(|p| p.contains("State encapsulation pattern")));
    }

    #[test]
    fn test_observables_without_dollar_suffix() {
        let analyzer = StateAnalyzer::new();
        
        // Test TypeScript type annotations
        let line1 = "  private userStream: Observable<User> = this.http.get('/api/user');";
        let observable1 = analyzer.parse_observable_line(line1);
        assert!(observable1.is_some());
        let obs1 = observable1.unwrap();
        assert_eq!(obs1.name, "userStream");
        assert!(matches!(obs1.observable_type, ObservableType::Observable));
        
        // Test BehaviorSubject type annotation
        let line2 = "  public dataSource: BehaviorSubject<Data> = new BehaviorSubject(null);";
        let observable2 = analyzer.parse_observable_line(line2);
        assert!(observable2.is_some());
        let obs2 = observable2.unwrap();
        assert_eq!(obs2.name, "dataSource");
        assert!(matches!(obs2.observable_type, ObservableType::BehaviorSubject));
        
        // Test new Observable() creation
        let line3 = "  readonly events = new Observable(observer => { /* ... */ });";
        let observable3 = analyzer.parse_observable_line(line3);
        assert!(observable3.is_some());
        let obs3 = observable3.unwrap();
        assert_eq!(obs3.name, "events");
        assert!(matches!(obs3.observable_type, ObservableType::Observable));
        
        // Test HTTP client methods
        let line4 = "  private apiData = this.http.get<ApiResponse>('/api/data');";
        let observable4 = analyzer.parse_observable_line(line4);
        assert!(observable4.is_some());
        let obs4 = observable4.unwrap();
        assert_eq!(obs4.name, "apiData");
        assert!(matches!(obs4.observable_type, ObservableType::Observable));
    }
}
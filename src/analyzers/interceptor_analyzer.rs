use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;
use crate::types::{InterceptorAnalysis, InterceptorSummary, InterceptorType};
use crate::utils::file_utils;

pub struct InterceptorAnalyzer;

impl InterceptorAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_project_interceptors(&self, project_path: &Path) -> Result<InterceptorAnalysis> {
        let mut interceptor_analysis = InterceptorAnalysis {
            interceptors: Vec::new(),
            error_handlers: Vec::new(),
            auth_interceptors: Vec::new(),
            logging_interceptors: Vec::new(),
        };

        // Find and analyze interceptor files
        let interceptor_files = self.find_interceptor_files(project_path)?;
        for interceptor_file in interceptor_files {
            let interceptor = self.analyze_interceptor_file(&interceptor_file)?;
            if let Some(i) = interceptor {
                interceptor_analysis.interceptors.push(i);
            }
        }

        // Categorize interceptors by functionality
        self.categorize_interceptors(&mut interceptor_analysis);

        Ok(interceptor_analysis)
    }

    fn find_interceptor_files(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut interceptor_files = Vec::new();
        
        for entry in WalkDir::new(project_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let path_str = path.to_string_lossy();
                if path_str.contains("interceptor") && path_str.ends_with(".ts") && !path_str.contains(".spec.") {
                    interceptor_files.push(path_str.to_string());
                }
            }
        }

        Ok(interceptor_files)
    }

    fn analyze_interceptor_file(&self, file_path: &str) -> Result<Option<InterceptorSummary>> {
        let content = file_utils::read_file_content(Path::new(file_path))?;
        
        // Extract interceptor name from file path
        let interceptor_name = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .replace(".interceptor", "")
            .to_string();

        // Determine interceptor type
        let interceptor_type = if content.contains("HttpInterceptorFn") {
            InterceptorType::HttpInterceptorFn
        } else if content.contains("HttpInterceptor") {
            InterceptorType::HttpInterceptor
        } else {
            return Ok(None); // Not an interceptor file
        };

        // Extract dependencies (injected services)
        let dependencies = self.extract_interceptor_dependencies(&content);

        // Check functionality patterns
        let handles_errors = self.detects_error_handling(&content);
        let modifies_requests = self.detects_request_modification(&content);
        let modifies_responses = self.detects_response_modification(&content);

        Ok(Some(InterceptorSummary {
            name: interceptor_name,
            path: file_path.to_string(),
            interceptor_type,
            dependencies,
            handles_errors,
            modifies_requests,
            modifies_responses,
        }))
    }

    fn extract_interceptor_dependencies(&self, content: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Look for inject() calls
        for line in content.lines() {
            if line.contains("inject(") {
                if let Some(start) = line.find("inject(") {
                    let service_part = &line[start + 7..];
                    if let Some(end) = service_part.find(')') {
                        let service_name = service_part[..end].trim();
                        dependencies.push(service_name.to_string());
                    }
                }
            }
        }

        // Look for constructor injection (class-based interceptors)
        let mut in_constructor = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("constructor(") {
                in_constructor = true;
            }
            if in_constructor && trimmed.contains(")") && !trimmed.contains("constructor(") {
                in_constructor = false;
            }
            if in_constructor && trimmed.contains("private ") {
                // Extract service type from constructor parameter
                if let Some(colon_pos) = trimmed.find(':') {
                    let service_type = trimmed[colon_pos + 1..].trim();
                    if let Some(comma_pos) = service_type.find(',') {
                        dependencies.push(service_type[..comma_pos].trim().to_string());
                    } else if let Some(paren_pos) = service_type.find(')') {
                        dependencies.push(service_type[..paren_pos].trim().to_string());
                    } else {
                        dependencies.push(service_type.to_string());
                    }
                }
            }
        }

        dependencies
    }

    fn detects_error_handling(&self, content: &str) -> bool {
        content.contains("catchError") || 
        content.contains("HttpErrorResponse") ||
        content.contains("error.status") ||
        content.contains("throwError")
    }

    fn detects_request_modification(&self, content: &str) -> bool {
        content.contains("req.clone") ||
        content.contains("setHeaders") ||
        content.contains("setParams") ||
        content.contains("withCredentials") ||
        content.contains("req.url")
    }

    fn detects_response_modification(&self, content: &str) -> bool {
        content.contains("map(") ||
        content.contains("tap(") ||
        content.contains("response.") ||
        content.contains("res.body")
    }

    fn categorize_interceptors(&self, analysis: &mut InterceptorAnalysis) {
        for interceptor in &analysis.interceptors {
            // Categorize by functionality
            if interceptor.handles_errors {
                analysis.error_handlers.push(interceptor.clone());
            }

            // Auth interceptors typically handle authentication headers or 401 errors
            if interceptor.name.to_lowercase().contains("auth") ||
               interceptor.dependencies.iter().any(|dep| dep.to_lowercase().contains("auth")) ||
               interceptor.path.to_lowercase().contains("401") {
                analysis.auth_interceptors.push(interceptor.clone());
            }

            // Logging interceptors typically log requests/responses
            if interceptor.name.to_lowercase().contains("log") ||
               interceptor.dependencies.iter().any(|dep| dep.to_lowercase().contains("log")) {
                analysis.logging_interceptors.push(interceptor.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_interceptor_analyzer_creation() {
        let analyzer = InterceptorAnalyzer::new();
        assert_eq!(std::mem::size_of_val(&analyzer), 0);
    }

    #[test]
    fn test_detects_error_handling() {
        let analyzer = InterceptorAnalyzer::new();
        
        assert!(analyzer.detects_error_handling("catchError((error: HttpErrorResponse) => {"));
        assert!(analyzer.detects_error_handling("if (error.status === 401) {"));
        assert!(analyzer.detects_error_handling("return throwError(() => error);"));
        assert!(!analyzer.detects_error_handling("normal request handling"));
    }

    #[test]
    fn test_detects_request_modification() {
        let analyzer = InterceptorAnalyzer::new();
        
        assert!(analyzer.detects_request_modification("const authReq = req.clone({"));
        assert!(analyzer.detects_request_modification("withCredentials: true"));
        assert!(analyzer.detects_request_modification("setHeaders({'Authorization': token})"));
        assert!(!analyzer.detects_request_modification("normal request handling"));
    }

    #[test]
    fn test_analyze_functional_interceptor() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let interceptor_file = temp_dir.path().join("auth.interceptor.ts");
        
        let interceptor_content = r#"
import { inject } from '@angular/core';
import { HttpInterceptorFn, HttpErrorResponse } from '@angular/common/http';
import { Router } from '@angular/router';
import { catchError, throwError } from 'rxjs';

export const authInterceptor: HttpInterceptorFn = (req, next) => {
  const router = inject(Router);
  
  const authReq = req.clone({
    withCredentials: true
  });

  return next(authReq).pipe(
    catchError((error: HttpErrorResponse) => {
      if (error.status === 401) {
        router.navigate(['/home']);
      }
      return throwError(() => error);
    })
  );
};
"#;
        
        fs::write(&interceptor_file, interceptor_content)?;
        
        let analyzer = InterceptorAnalyzer::new();
        let interceptor = analyzer.analyze_interceptor_file(interceptor_file.to_str().unwrap())?;
        
        assert!(interceptor.is_some());
        let interceptor = interceptor.unwrap();
        
        assert_eq!(interceptor.name, "auth");
        assert!(matches!(interceptor.interceptor_type, InterceptorType::HttpInterceptorFn));
        assert!(interceptor.handles_errors);
        assert!(interceptor.modifies_requests);
        assert!(!interceptor.modifies_responses);
        assert!(interceptor.dependencies.contains(&"Router".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_analyze_class_based_interceptor() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let interceptor_file = temp_dir.path().join("logging.interceptor.ts");
        
        let interceptor_content = r#"
import { Injectable } from '@angular/core';
import { HttpInterceptor, HttpRequest, HttpHandler } from '@angular/common/http';
import { LoggingService } from '../services/logging.service';

@Injectable()
export class LoggingInterceptor implements HttpInterceptor {
  constructor(private loggingService: LoggingService) {}

  intercept(req: HttpRequest<any>, next: HttpHandler) {
    this.loggingService.log('Request started');
    return next.handle(req).pipe(
      tap(response => this.loggingService.log('Response received'))
    );
  }
}
"#;
        
        fs::write(&interceptor_file, interceptor_content)?;
        
        let analyzer = InterceptorAnalyzer::new();
        let interceptor = analyzer.analyze_interceptor_file(interceptor_file.to_str().unwrap())?;
        
        assert!(interceptor.is_some());
        let interceptor = interceptor.unwrap();
        
        assert_eq!(interceptor.name, "logging");
        assert!(matches!(interceptor.interceptor_type, InterceptorType::HttpInterceptor));
        assert!(!interceptor.handles_errors);
        assert!(!interceptor.modifies_requests);
        assert!(interceptor.modifies_responses); // tap() modifies the response stream
        assert!(interceptor.dependencies.contains(&"LoggingService".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_categorize_interceptors() {
        let analyzer = InterceptorAnalyzer::new();
        
        let auth_interceptor = InterceptorSummary {
            name: "auth".to_string(),
            path: "src/interceptors/auth.interceptor.ts".to_string(),
            interceptor_type: InterceptorType::HttpInterceptorFn,
            dependencies: vec!["Router".to_string()],
            handles_errors: true,
            modifies_requests: true,
            modifies_responses: false,
        };
        
        let logging_interceptor = InterceptorSummary {
            name: "logging".to_string(),
            path: "src/interceptors/logging.interceptor.ts".to_string(),
            interceptor_type: InterceptorType::HttpInterceptor,
            dependencies: vec!["LoggingService".to_string()],
            handles_errors: false,
            modifies_requests: false,
            modifies_responses: true,
        };
        
        let mut analysis = InterceptorAnalysis {
            interceptors: vec![auth_interceptor.clone(), logging_interceptor.clone()],
            error_handlers: Vec::new(),
            auth_interceptors: Vec::new(),
            logging_interceptors: Vec::new(),
        };
        
        analyzer.categorize_interceptors(&mut analysis);
        
        assert_eq!(analysis.error_handlers.len(), 1);
        assert_eq!(analysis.error_handlers[0].name, "auth");
        
        assert_eq!(analysis.auth_interceptors.len(), 1);
        assert_eq!(analysis.auth_interceptors[0].name, "auth");
        
        assert_eq!(analysis.logging_interceptors.len(), 1);
        assert_eq!(analysis.logging_interceptors[0].name, "logging");
    }
}
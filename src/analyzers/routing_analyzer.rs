use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;
use crate::types::{RoutingAnalysis, RouteSummary, GuardSummary, GuardType};
use crate::utils::file_utils;

pub struct RoutingAnalyzer;

impl RoutingAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze_project_routing(&self, project_path: &Path) -> Result<RoutingAnalysis> {
        let mut routing_analysis = RoutingAnalysis {
            routes: Vec::new(),
            guards: Vec::new(),
            protected_routes: Vec::new(),
            redirects: Vec::new(),
            lazy_routes: Vec::new(),
        };

        // Find and analyze route files
        let route_files = self.find_route_files(project_path)?;
        for route_file in route_files {
            let routes = self.analyze_route_file(&route_file)?;
            routing_analysis.routes.extend(routes);
        }

        // Find and analyze guard files
        let guard_files = self.find_guard_files(project_path)?;
        for guard_file in guard_files {
            let guard = self.analyze_guard_file(&guard_file)?;
            if let Some(g) = guard {
                routing_analysis.guards.push(g);
            }
        }

        // Categorize routes
        self.categorize_routes(&mut routing_analysis);

        Ok(routing_analysis)
    }

    fn find_route_files(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut route_files = Vec::new();
        
        for entry in WalkDir::new(project_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let path_str = path.to_string_lossy();
                
                // Exclude node_modules and test files
                if path_str.contains("node_modules") || path_str.contains(".spec.") {
                    continue;
                }
                
                // Only look in src directory for routes
                if path_str.contains("routes") && path_str.ends_with(".ts") && path_str.contains("/src/") {
                    route_files.push(path_str.to_string());
                }
            }
        }

        Ok(route_files)
    }

    fn find_guard_files(&self, project_path: &Path) -> Result<Vec<String>> {
        let mut guard_files = Vec::new();
        
        for entry in WalkDir::new(project_path) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                let path_str = path.to_string_lossy();
                
                // Exclude node_modules and test files
                if path_str.contains("node_modules") || path_str.contains(".spec.") {
                    continue;
                }
                
                // Only look in src directory for guards
                if path_str.contains("guard") && path_str.ends_with(".ts") && path_str.contains("/src/") {
                    guard_files.push(path_str.to_string());
                }
            }
        }

        Ok(guard_files)
    }

    fn analyze_route_file(&self, file_path: &str) -> Result<Vec<RouteSummary>> {
        let content = file_utils::read_file_content(Path::new(file_path))?;
        let mut routes = Vec::new();

        // Parse route definitions from TypeScript content
        // This is a simplified parser - in a real implementation, you'd use a proper TS parser
        let lines: Vec<&str> = content.lines().collect();
        let mut in_routes_array = false;
        let mut current_route: Option<RouteSummary> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.contains("Routes = [") || trimmed.contains("routes: Routes = [") {
                in_routes_array = true;
                continue;
            }

            if in_routes_array && trimmed == "];" {
                if let Some(route) = current_route.take() {
                    routes.push(route);
                }
                break;
            }

            if in_routes_array && trimmed == "{" {
                current_route = Some(RouteSummary {
                    path: String::new(),
                    component: String::new(),
                    guards: Vec::new(),
                    redirect_to: None,
                    is_protected: false,
                    lazy_loaded: false,
                });
            }

            if in_routes_array && trimmed == "}," {
                if let Some(route) = current_route.take() {
                    routes.push(route);
                }
            }

            if let Some(ref mut route) = current_route {
                if let Some(path_match) = self.extract_route_path(trimmed) {
                    route.path = path_match;
                }

                if let Some(component_match) = self.extract_route_component(trimmed) {
                    route.component = component_match;
                }

                if let Some(redirect) = self.extract_redirect_to(trimmed) {
                    route.redirect_to = Some(redirect);
                }

                if let Some(guards) = self.extract_guards(trimmed) {
                    route.guards = guards;
                    route.is_protected = !route.guards.is_empty();
                }

                if trimmed.contains("loadChildren") {
                    route.lazy_loaded = true;
                }
            }
        }

        Ok(routes)
    }

    fn analyze_guard_file(&self, file_path: &str) -> Result<Option<GuardSummary>> {
        let content = file_utils::read_file_content(Path::new(file_path))?;
        
        // Extract guard name from file path
        let guard_name = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .replace(".guard", "")
            .to_string();

        // Determine guard type
        let guard_type = if content.contains("CanActivateFn") {
            GuardType::CanActivate
        } else if content.contains("CanDeactivateFn") {
            GuardType::CanDeactivate
        } else if content.contains("CanLoadFn") {
            GuardType::CanLoad
        } else if content.contains("ResolveFn") {
            GuardType::Resolve
        } else if content.contains("CanMatchFn") {
            GuardType::CanMatch
        } else {
            GuardType::CanActivate // Default
        };

        // Extract dependencies (injected services)
        let dependencies = self.extract_guard_dependencies(&content);

        Ok(Some(GuardSummary {
            name: guard_name,
            path: file_path.to_string(),
            guard_type,
            dependencies,
            protected_routes: Vec::new(), // Will be populated later
        }))
    }

    fn extract_route_path(&self, line: &str) -> Option<String> {
        if line.contains("path:") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
        None
    }

    fn extract_route_component(&self, line: &str) -> Option<String> {
        if line.contains("component:") {
            if let Some(start) = line.find("component:") {
                let component_part = &line[start + 10..].trim();
                if let Some(comma_pos) = component_part.find(',') {
                    return Some(component_part[..comma_pos].trim().to_string());
                } else {
                    return Some(component_part.to_string());
                }
            }
        }
        None
    }

    fn extract_redirect_to(&self, line: &str) -> Option<String> {
        if line.contains("redirectTo:") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
        None
    }

    fn extract_guards(&self, line: &str) -> Option<Vec<String>> {
        if line.contains("canActivate:") || line.contains("canDeactivate:") || line.contains("canLoad:") {
            // Extract guard names from array
            if let Some(start) = line.find('[') {
                if let Some(end) = line[start..].find(']') {
                    let guards_str = &line[start + 1..start + end];
                    let guards: Vec<String> = guards_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    return Some(guards);
                }
            }
        }
        None
    }

    fn extract_guard_dependencies(&self, content: &str) -> Vec<String> {
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

        dependencies
    }

    fn categorize_routes(&self, analysis: &mut RoutingAnalysis) {
        for route in &analysis.routes {
            if route.is_protected {
                analysis.protected_routes.push(route.clone());
            }

            if route.redirect_to.is_some() {
                analysis.redirects.push(route.clone());
            }

            if route.lazy_loaded {
                analysis.lazy_routes.push(route.clone());
            }
        }

        // Update guards with their protected routes
        for guard in &mut analysis.guards {
            guard.protected_routes = analysis.protected_routes
                .iter()
                .filter(|route| route.guards.contains(&guard.name))
                .map(|route| route.path.clone())
                .collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_routing_analyzer_creation() {
        let analyzer = RoutingAnalyzer::new();
        // Just test that it can be created
        assert_eq!(std::mem::size_of_val(&analyzer), 0);
    }

    #[test]
    fn test_extract_route_path() {
        let analyzer = RoutingAnalyzer::new();
        
        assert_eq!(
            analyzer.extract_route_path("path: 'home',"),
            Some("home".to_string())
        );
        
        assert_eq!(
            analyzer.extract_route_path("path: \"/dashboard\","),
            Some("/dashboard".to_string())
        );
        
        assert_eq!(
            analyzer.extract_route_path("component: HomeComponent"),
            None
        );
    }

    #[test]
    fn test_extract_route_component() {
        let analyzer = RoutingAnalyzer::new();
        
        assert_eq!(
            analyzer.extract_route_component("component: HomeComponent,"),
            Some("HomeComponent".to_string())
        );
        
        assert_eq!(
            analyzer.extract_route_component("component: DashboardComponent"),
            Some("DashboardComponent".to_string())
        );
    }

    #[test]
    fn test_extract_guards() {
        let analyzer = RoutingAnalyzer::new();
        
        assert_eq!(
            analyzer.extract_guards("canActivate: [authGuard],"),
            Some(vec!["authGuard".to_string()])
        );
        
        assert_eq!(
            analyzer.extract_guards("canActivate: [authGuard, adminGuard],"),
            Some(vec!["authGuard".to_string(), "adminGuard".to_string()])
        );
    }

    #[test]
    fn test_analyze_route_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let route_file = temp_dir.path().join("app.routes.ts");
        
        let route_content = r#"
import { Routes } from '@angular/router';
import { HomeComponent } from './home/home.component';
import { DashboardComponent } from './dashboard/dashboard.component';
import { authGuard } from './guards/auth.guard';

export const routes: Routes = [
    {
        path: '',
        redirectTo: '/home',
        pathMatch: 'full'
    },
    {
        path: 'home',
        component: HomeComponent
    },
    {
        path: 'dashboard',
        component: DashboardComponent,
        canActivate: [authGuard]
    },
];
"#;
        
        fs::write(&route_file, route_content)?;
        
        let analyzer = RoutingAnalyzer::new();
        let routes = analyzer.analyze_route_file(route_file.to_str().unwrap())?;
        
        assert_eq!(routes.len(), 3);
        
        // Check redirect route
        let redirect_route = &routes[0];
        assert_eq!(redirect_route.path, "");
        assert_eq!(redirect_route.redirect_to, Some("/home".to_string()));
        
        // Check home route
        let home_route = &routes[1];
        assert_eq!(home_route.path, "home");
        assert_eq!(home_route.component, "HomeComponent");
        assert!(!home_route.is_protected);
        
        // Check protected route
        let dashboard_route = &routes[2];
        assert_eq!(dashboard_route.path, "dashboard");
        assert_eq!(dashboard_route.component, "DashboardComponent");
        assert!(dashboard_route.is_protected);
        assert_eq!(dashboard_route.guards, vec!["authGuard"]);
        
        Ok(())
    }

    #[test]
    fn test_analyze_guard_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let guard_file = temp_dir.path().join("auth.guard.ts");
        
        let guard_content = r#"
import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { AuthService } from '../services/auth.service';

export const authGuard: CanActivateFn = (route, state) => {
  const authService = inject(AuthService);
  const router = inject(Router);
  
  return authService.isAuthenticated();
};
"#;
        
        fs::write(&guard_file, guard_content)?;
        
        let analyzer = RoutingAnalyzer::new();
        let guard = analyzer.analyze_guard_file(guard_file.to_str().unwrap())?;
        
        assert!(guard.is_some());
        let guard = guard.unwrap();
        
        assert_eq!(guard.name, "auth");
        assert!(matches!(guard.guard_type, GuardType::CanActivate));
        assert_eq!(guard.dependencies.len(), 2);
        assert!(guard.dependencies.contains(&"AuthService".to_string()));
        assert!(guard.dependencies.contains(&"Router".to_string()));
        
        Ok(())
    }
}
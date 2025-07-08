use anyhow::Result;
use crate::types::*;

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        ReportGenerator
    }

    pub fn generate_text_report(&self, overview: &ProjectOverview) -> Result<String> {
        let mut report = String::new();
        
        report.push_str(&format!("# Project Overview: {}\n\n", overview.project_name));
        report.push_str(&format!("Last Updated: {}\n\n", overview.last_updated.format("%Y-%m-%d %H:%M:%S")));
        
        // Structure section
        report.push_str("## Project Structure\n\n");
        report.push_str(&format!("- Components: {}\n", overview.structure.components.len()));
        report.push_str(&format!("- Services: {}\n", overview.structure.services.len()));
        report.push_str(&format!("- Routes: {}\n", overview.structure.routes.len()));
        report.push_str(&format!("- SCSS Variables: {}\n", overview.structure.styles.variables.len()));
        report.push_str(&format!("- SCSS Mixins: {}\n\n", overview.structure.styles.mixins.len()));
        
        // Tech stack section
        report.push_str("## Technical Stack\n\n");
        report.push_str(&format!("- Framework: {}\n", overview.technical_stack.framework));
        report.push_str(&format!("- Language: {}\n", overview.technical_stack.language));
        report.push_str(&format!("- Dependencies: {}\n", overview.technical_stack.dependencies.len()));
        report.push_str(&format!("- Dev Dependencies: {}\n\n", overview.technical_stack.dev_dependencies.len()));
        
        // Health metrics section
        report.push_str("## Health Metrics\n\n");
        report.push_str(&format!("- Code Complexity: {:?}\n", overview.health_metrics.code_complexity));
        report.push_str(&format!("- Test Coverage: {:.1}%\n", overview.health_metrics.test_coverage));
        report.push_str(&format!("- Build Health: {:?}\n", overview.health_metrics.build_health));
        report.push_str(&format!("- Bundle Size: {:.2} MB\n\n", overview.health_metrics.bundle_size as f64 / 1024.0 / 1024.0));
        
        // Active features section
        report.push_str("## Active Features\n\n");
        for feature in &overview.active_features {
            report.push_str(&format!("- {}\n", feature));
        }
        report.push_str("\n");
        
        // Recommendations section
        report.push_str("## Recommendations\n\n");
        for recommendation in &overview.recommendations {
            report.push_str(&format!("- {}\n", recommendation));
        }
        
        Ok(report)
    }

    pub fn generate_json_report(&self, overview: &ProjectOverview) -> Result<String> {
        let json = serde_json::to_string_pretty(overview)?;
        Ok(json)
    }

    pub fn generate_markdown_report(&self, overview: &ProjectOverview) -> Result<String> {
        let mut report = String::new();
        
        report.push_str(&format!("# 📊 Project Overview: {}\n\n", overview.project_name));
        report.push_str(&format!("**Last Updated:** {}\n\n", overview.last_updated.format("%Y-%m-%d %H:%M:%S")));
        
        // Structure section
        report.push_str("## 🏗️ Project Structure\n\n");
        report.push_str("| Component | Count |\n");
        report.push_str("|-----------|-------|\n");
        report.push_str(&format!("| Components | {} |\n", overview.structure.components.len()));
        report.push_str(&format!("| Services | {} |\n", overview.structure.services.len()));
        report.push_str(&format!("| Routes | {} |\n", overview.structure.routes.len()));
        report.push_str(&format!("| SCSS Variables | {} |\n", overview.structure.styles.variables.len()));
        report.push_str(&format!("| SCSS Mixins | {} |\n\n", overview.structure.styles.mixins.len()));
        
        // Tech stack section
        report.push_str("## 🛠️ Technical Stack\n\n");
        report.push_str(&format!("- **Framework:** {}\n", overview.technical_stack.framework));
        report.push_str(&format!("- **Language:** {}\n", overview.technical_stack.language));
        report.push_str(&format!("- **Dependencies:** {}\n", overview.technical_stack.dependencies.len()));
        report.push_str(&format!("- **Dev Dependencies:** {}\n\n", overview.technical_stack.dev_dependencies.len()));
        
        // Health metrics section
        report.push_str("## 📈 Health Metrics\n\n");
        report.push_str(&format!("- **Code Complexity:** {:?}\n", overview.health_metrics.code_complexity));
        report.push_str(&format!("- **Test Coverage:** {:.1}%\n", overview.health_metrics.test_coverage));
        report.push_str(&format!("- **Build Health:** {:?}\n", overview.health_metrics.build_health));
        report.push_str(&format!("- **Bundle Size:** {:.2} MB\n\n", overview.health_metrics.bundle_size as f64 / 1024.0 / 1024.0));
        
        // Active features section
        report.push_str("## ⚡ Active Features\n\n");
        for feature in &overview.active_features {
            report.push_str(&format!("- ✅ {}\n", feature));
        }
        report.push_str("\n");
        
        // Recommendations section
        report.push_str("## 💡 Recommendations\n\n");
        for recommendation in &overview.recommendations {
            report.push_str(&format!("- 🔧 {}\n", recommendation));
        }
        
        Ok(report)
    }

    pub fn generate_summary_report(&self, overview: &ProjectOverview) -> Result<String> {
        let report = format!(
            "Project: {} | Components: {} | Services: {} | Coverage: {:.1}% | Health: {:?} | Size: {:.1}MB",
            overview.project_name,
            overview.structure.components.len(),
            overview.structure.services.len(),
            overview.health_metrics.test_coverage,
            overview.health_metrics.build_health,
            overview.health_metrics.bundle_size as f64 / 1024.0 / 1024.0
        );
        
        Ok(report)
    }
}
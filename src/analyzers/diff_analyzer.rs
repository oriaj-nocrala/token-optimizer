use anyhow::Result;
use std::path::Path;
use chrono::Utc;
use crate::types::*;
use crate::utils::GitUtils;

pub struct DiffAnalyzer {
    git_utils: Option<GitUtils>,
}

impl DiffAnalyzer {
    pub fn new(project_path: &Path) -> Result<Self> {
        let git_utils = GitUtils::new(project_path).ok();
        Ok(DiffAnalyzer { git_utils })
    }

    pub fn analyze_changes(&self, _project_path: &Path) -> Result<ChangeAnalysis> {
        let session_id = format!("session-{}", Utc::now().timestamp());
        let timestamp = Utc::now();
        
        let (modified_files, added_files, deleted_files) = if let Some(git) = &self.git_utils {
            let modified = git.get_modified_files()?;
            let untracked = git.get_untracked_files()?;
            
            let modified_files = self.analyze_modified_files(&modified)?;
            
            (modified_files, untracked, Vec::new()) // Simplified - deleted files detection would need more work
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        let impact_scope = self.determine_impact_scope(&modified_files);
        let relevant_context = self.extract_relevant_context(&modified_files)?;
        let suggested_actions = self.generate_suggested_actions(&modified_files)?;

        Ok(ChangeAnalysis {
            session_id,
            timestamp,
            modified_files,
            added_files,
            deleted_files,
            renamed_files: Vec::new(), // Simplified
            impact_scope,
            relevant_context,
            suggested_actions,
        })
    }

    fn analyze_modified_files(&self, file_paths: &[String]) -> Result<Vec<ModifiedFile>> {
        let mut modified_files = Vec::new();
        
        for file_path in file_paths {
            let change_type = self.determine_change_type(file_path)?;
            let (lines_added, lines_removed) = self.get_line_changes(file_path)?;
            let sections_changed = self.identify_changed_sections(file_path)?;
            let impacted_files = self.find_impacted_files(file_path)?;
            
            modified_files.push(ModifiedFile {
                path: file_path.clone(),
                change_type,
                lines_added,
                lines_removed,
                sections_changed,
                impacted_files,
            });
        }
        
        Ok(modified_files)
    }

    fn determine_change_type(&self, file_path: &str) -> Result<ChangeType> {
        // Simplified implementation
        if let Some(git) = &self.git_utils {
            let status = git.get_file_status(file_path)?;
            match status.as_str() {
                "new" => Ok(ChangeType::Created),
                "modified" => Ok(ChangeType::Modified),
                "deleted" => Ok(ChangeType::Deleted),
                "renamed" => Ok(ChangeType::Renamed),
                _ => Ok(ChangeType::Modified),
            }
        } else {
            Ok(ChangeType::Modified)
        }
    }

    fn get_line_changes(&self, file_path: &str) -> Result<(usize, usize)> {
        // Simplified implementation
        if let Some(git) = &self.git_utils {
            git.get_file_changes(file_path)
        } else {
            Ok((0, 0))
        }
    }

    fn identify_changed_sections(&self, _file_path: &str) -> Result<Vec<String>> {
        // Simplified implementation - would need actual diff parsing
        Ok(vec![
            "imports".to_string(),
            "main function".to_string(),
        ])
    }

    fn find_impacted_files(&self, _file_path: &str) -> Result<Vec<String>> {
        // Simplified implementation - would need dependency graph analysis
        Ok(vec![])
    }

    fn determine_impact_scope(&self, modified_files: &[ModifiedFile]) -> ImpactScope {
        if modified_files.is_empty() {
            return ImpactScope::Local;
        }

        let has_service_changes = modified_files.iter().any(|f| f.path.contains("service"));
        let has_component_changes = modified_files.iter().any(|f| f.path.contains("component"));
        let has_config_changes = modified_files.iter().any(|f| f.path.contains("config") || f.path.contains("json"));

        if has_config_changes || modified_files.len() > 10 {
            ImpactScope::Global
        } else if has_service_changes {
            ImpactScope::Service
        } else if has_component_changes {
            ImpactScope::Component
        } else {
            ImpactScope::Local
        }
    }

    fn extract_relevant_context(&self, modified_files: &[ModifiedFile]) -> Result<Vec<String>> {
        let mut context = Vec::new();
        
        for file in modified_files {
            context.push(format!("Modified: {} ({} lines added, {} lines removed)", 
                file.path, file.lines_added, file.lines_removed));
            
            for section in &file.sections_changed {
                context.push(format!("  - Changed section: {}", section));
            }
        }
        
        Ok(context)
    }

    fn generate_suggested_actions(&self, modified_files: &[ModifiedFile]) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        
        if modified_files.is_empty() {
            actions.push("No changes detected".to_string());
            return Ok(actions);
        }

        actions.push("Review the modified files for potential issues".to_string());
        
        if modified_files.iter().any(|f| f.path.contains("service")) {
            actions.push("Test service functionality after changes".to_string());
        }
        
        if modified_files.iter().any(|f| f.path.contains("component")) {
            actions.push("Verify component rendering and behavior".to_string());
        }
        
        if modified_files.len() > 5 {
            actions.push("Consider running full test suite due to extensive changes".to_string());
        }
        
        Ok(actions)
    }
}
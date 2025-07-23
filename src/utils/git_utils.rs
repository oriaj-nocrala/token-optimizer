use git2::{Repository, Status};
use anyhow::Result;
use std::path::Path;

pub struct GitUtils {
    repo: Repository,
}

impl GitUtils {
    pub fn new(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(GitUtils { repo })
    }

    pub fn get_modified_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let statuses = self.repo.statuses(None)?;
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("");
            let status = entry.status();
            
            // Only include files that have actual content changes (exclude added/deleted empty files)
            if status.contains(Status::WT_MODIFIED) {
                // Check if the file actually has content changes
                if let Ok((added, removed)) = self.get_file_changes(path) {
                    if added > 0 || removed > 0 {
                        files.push(path.to_string());
                    }
                }
            } else if status.contains(Status::INDEX_MODIFIED) {
                files.push(path.to_string());
            }
        }
        
        Ok(files)
    }

    pub fn get_file_changes(&self, file_path: &str) -> Result<(usize, usize)> {
        use git2::DiffOptions;
        
        let head = self.repo.head()?.peel_to_tree()?;
        let mut opts = DiffOptions::new();
        opts.pathspec(file_path);
        
        // Compare working directory to HEAD
        let diff = self.repo.diff_tree_to_workdir(Some(&head), Some(&mut opts))?;
        
        let mut lines_added = 0;
        let mut lines_removed = 0;
        
        diff.foreach(
            &mut |_delta, _progress| true,
            None,
            Some(&mut |_delta, _hunk| true),
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' => lines_added += 1,
                    '-' => lines_removed += 1,
                    _ => {}
                }
                true
            }),
        )?;
        
        Ok((lines_added, lines_removed))
    }

    pub fn get_branch_name(&self) -> Result<String> {
        let head = self.repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("detached".to_string())
        }
    }

    pub fn get_last_commit_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    pub fn is_clean(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }

    pub fn get_untracked_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let statuses = self.repo.statuses(None)?;
        
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("");
            let status = entry.status();
            
            if status.contains(Status::WT_NEW) {
                files.push(path.to_string());
            }
        }
        
        Ok(files)
    }

    pub fn get_file_status(&self, file_path: &str) -> Result<String> {
        let statuses = self.repo.statuses(None)?;
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                if path == file_path {
                    let status = entry.status();
                    return Ok(self.status_to_string(status));
                }
            }
        }
        
        Ok("unmodified".to_string())
    }

    fn status_to_string(&self, status: Status) -> String {
        if status.contains(Status::WT_NEW) {
            "new".to_string()
        } else if status.contains(Status::WT_MODIFIED) {
            "modified".to_string()
        } else if status.contains(Status::WT_DELETED) {
            "deleted".to_string()
        } else if status.contains(Status::WT_RENAMED) {
            "renamed".to_string()
        } else {
            "unknown".to_string()
        }
    }
}
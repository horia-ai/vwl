use crate::error::{Result, VwlError};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepoInfo {
    pub root: PathBuf,
    pub head_sha: String,
    pub tree_sha: String,
    pub canonical_base: Option<String>,
}

pub struct GitInspector;

impl GitInspector {
    pub fn find_repo_root(start_dir: &Path) -> Result<PathBuf> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(start_dir)
            .output()
            .map_err(|e| VwlError::GitCommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(VwlError::NotAGitRepository(
                start_dir.to_string_lossy().to_string(),
            ));
        }

        let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(root_str))
    }

    pub fn inspect(start_dir: &Path) -> Result<GitRepoInfo> {
        let root = Self::find_repo_root(start_dir)?;

        // HEAD commit SHA
        let head_output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&root)
            .output()
            .map_err(|e| VwlError::GitCommandFailed(e.to_string()))?;

        let head_sha = if head_output.status.success() {
            String::from_utf8_lossy(&head_output.stdout)
                .trim()
                .to_string()
        } else {
            "unborn".to_string()
        };

        // Tree SHA of HEAD
        let tree_sha = if head_sha != "unborn" {
            let tree_output = Command::new("git")
                .arg("rev-parse")
                .arg("HEAD^{tree}")
                .current_dir(&root)
                .output()
                .map_err(|e| VwlError::GitCommandFailed(e.to_string()))?;
            if tree_output.status.success() {
                String::from_utf8_lossy(&tree_output.stdout)
                    .trim()
                    .to_string()
            } else {
                "empty".to_string()
            }
        } else {
            "empty".to_string()
        };

        // Read canonical base if stored in .vwl/canonical_base
        let canonical_file = root.join(".vwl").join("canonical_base");
        let canonical_base = if canonical_file.exists() {
            std::fs::read_to_string(canonical_file)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        };

        Ok(GitRepoInfo {
            root,
            head_sha,
            tree_sha,
            canonical_base,
        })
    }

    pub fn set_canonical_base(repo_root: &Path, commit_sha: &str) -> Result<()> {
        let vwl_dir = repo_root.join(".vwl");
        if !vwl_dir.exists() {
            std::fs::create_dir_all(&vwl_dir)?;
        }
        std::fs::write(vwl_dir.join("canonical_base"), commit_sha)?;
        Ok(())
    }
}

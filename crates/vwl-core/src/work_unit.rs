use crate::error::{Result, VwlError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitEntry {
    pub sha: String,
    pub role: String,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkUnit {
    pub task: String,
    pub commit: String,
    pub base: String,
    #[serde(default)]
    pub commits: Vec<CommitEntry>,
    #[serde(default)]
    pub candidate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_snapshot: Option<String>,
}

impl WorkUnit {
    pub fn new(
        task: impl Into<String>,
        commit: impl Into<String>,
        base: impl Into<String>,
    ) -> Result<Self> {
        let task = task.into();
        let commit = commit.into();
        let base = base.into();

        if task.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit task cannot be empty".to_string(),
            ));
        }
        if commit.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit commit cannot be empty".to_string(),
            ));
        }
        if base.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit base cannot be empty".to_string(),
            ));
        }

        let commits = vec![CommitEntry {
            sha: commit.clone(),
            role: "implementation".to_string(),
            actor: "worker".to_string(),
        }];
        let candidate = commit.clone();

        Ok(Self {
            task,
            commit,
            base,
            commits,
            candidate,
            spec_digest: None,
            dependency_snapshot: None,
        })
    }

    pub fn new_sequence(
        task: impl Into<String>,
        base: impl Into<String>,
        commits: Vec<CommitEntry>,
        candidate: impl Into<String>,
        spec_digest: Option<String>,
        dependency_snapshot: Option<String>,
    ) -> Result<Self> {
        let task = task.into();
        let base = base.into();
        let candidate = candidate.into();

        if task.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit task cannot be empty".to_string(),
            ));
        }
        if base.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit base cannot be empty".to_string(),
            ));
        }
        if commits.is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit commits sequence must contain at least 1 commit".to_string(),
            ));
        }
        if candidate.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit candidate cannot be empty".to_string(),
            ));
        }

        if !commits.iter().any(|c| c.sha == candidate) {
            return Err(VwlError::InvalidWorkUnit(format!(
                "Candidate commit '{}' must exist in the commits sequence",
                candidate
            )));
        }

        let commit = candidate.clone();

        Ok(Self {
            task,
            commit,
            base,
            commits,
            candidate,
            spec_digest,
            dependency_snapshot,
        })
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        let mut wu: WorkUnit = serde_yaml::from_str(content)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        if wu.candidate.is_empty() && !wu.commit.is_empty() {
            wu.candidate = wu.commit.clone();
        }
        if wu.commit.is_empty() && !wu.candidate.is_empty() {
            wu.commit = wu.candidate.clone();
        }
        if wu.commits.is_empty() && !wu.commit.is_empty() {
            wu.commits = vec![CommitEntry {
                sha: wu.commit.clone(),
                role: "implementation".to_string(),
                actor: "worker".to_string(),
            }];
        }
        wu.validate()?;
        Ok(wu)
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.task.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit task cannot be empty".to_string(),
            ));
        }
        if self.base.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit base cannot be empty".to_string(),
            ));
        }
        if self.candidate.trim().is_empty() && self.commit.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit candidate/commit cannot be empty".to_string(),
            ));
        }
        if !self.commits.is_empty()
            && !self
                .commits
                .iter()
                .any(|c| c.sha == self.candidate || c.sha == self.commit)
        {
            return Err(VwlError::InvalidWorkUnit(
                "Declared candidate must exist in commits sequence".to_string(),
            ));
        }
        Ok(())
    }
}

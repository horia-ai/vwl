use crate::error::{Result, VwlError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkUnit {
    pub task: String,
    pub commit: String,
    pub base: String,
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

        Ok(Self { task, commit, base })
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        let wu: WorkUnit = serde_yaml::from_str(content)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
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
        if self.commit.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit commit cannot be empty".to_string(),
            ));
        }
        if self.base.trim().is_empty() {
            return Err(VwlError::InvalidWorkUnit(
                "WorkUnit base cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

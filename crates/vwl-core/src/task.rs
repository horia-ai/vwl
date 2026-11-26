use crate::error::{Result, VwlError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: String,
    pub objective: String,
    pub acceptance: Vec<String>,
    pub base: String,
}

impl Task {
    pub fn new(
        id: impl Into<String>,
        objective: impl Into<String>,
        acceptance: Vec<String>,
        base: impl Into<String>,
    ) -> Result<Self> {
        let id = id.into();
        let objective = objective.into();
        let base = base.into();

        if id.trim().is_empty() {
            return Err(VwlError::InvalidTask("Task id cannot be empty".to_string()));
        }
        if objective.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Task objective cannot be empty".to_string(),
            ));
        }
        if base.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Task base commit cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            id,
            objective,
            acceptance,
            base,
        })
    }

    pub fn compute_digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(b"\n");
        hasher.update(self.objective.as_bytes());
        hasher.update(b"\n");
        for acc in &self.acceptance {
            hasher.update(acc.as_bytes());
            hasher.update(b"\n");
        }
        hasher.update(self.base.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        let task: Task = serde_yaml::from_str(content)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        task.validate()?;
        Ok(task)
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(VwlError::InvalidTask("Task id cannot be empty".to_string()));
        }
        if self.objective.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Task objective cannot be empty".to_string(),
            ));
        }
        if self.base.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Task base commit cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

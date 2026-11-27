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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkepticReviewVerdict {
    pub passed: bool,
    pub task_id: String,
    pub objections: Vec<String>,
    pub suggestions: Vec<String>,
}

pub struct TaskSkeptic;

impl TaskSkeptic {
    pub fn review(task: &Task) -> SkepticReviewVerdict {
        let mut objections = Vec::new();
        let mut suggestions = Vec::new();

        // 1. Acceptance criteria presence
        if task.acceptance.is_empty() {
            objections.push("No acceptance criteria defined: task is unverifiable".to_string());
            suggestions.push(
                "Define at least one falsifiable, command-grounded acceptance criterion"
                    .to_string(),
            );
        }

        // 2. Subjective / unfalsifiable phrases
        let subjective_phrases = [
            "clean code",
            "looks good",
            "feels good",
            "works well",
            "fast enough",
            "easy to read",
            "well tested",
            "confidence",
            "properly done",
            "high quality",
        ];

        for acc in &task.acceptance {
            let lower = acc.to_lowercase();
            for phrase in &subjective_phrases {
                if lower.contains(phrase) {
                    objections.push(format!(
                        "Subjective/unfalsifiable criterion '{}' contains '{}'",
                        acc, phrase
                    ));
                    suggestions.push(format!(
                        "Replace '{}' with deterministic command assertion (e.g. 'cargo test', 'npm test')",
                        phrase
                    ));
                }
            }
        }

        // 3. Conversational / session dependency checks
        let conversational_cues = [
            "as discussed",
            "previous chat",
            "see conversation",
            "like we agreed",
            "as mentioned before",
            "in our chat",
            "earlier discussion",
        ];
        for cue in &conversational_cues {
            if task.objective.to_lowercase().contains(cue) {
                objections.push(format!(
                    "Conversational dependency detected in objective: '{}'",
                    cue
                ));
                suggestions.push(
                    "Task specification must be fully self-contained without conversational history"
                        .to_string(),
                );
            }
        }

        // 4. Base commit validity check
        if task.base.trim().is_empty() || task.base.eq_ignore_ascii_case("null") {
            objections.push("Invalid or missing canonical base commit".to_string());
            suggestions.push("Specify exact canonical git base SHA or 'HEAD'".to_string());
        }

        let passed = objections.is_empty();
        SkepticReviewVerdict {
            passed,
            task_id: task.id.clone(),
            objections,
            suggestions,
        }
    }
}

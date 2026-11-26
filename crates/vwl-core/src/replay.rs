use crate::decision::DecisionOutcome;
use crate::error::{Result, VwlError};
use crate::git::GitInspector;
use crate::ledger::LocalLedger;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayState {
    pub total_tasks: usize,
    pub confirmed_work_units: Vec<String>,
    pub canonical_head: Option<String>,
}

pub struct ReplayEngine;

impl ReplayEngine {
    /// Reconstructs authoritative state from Git + Ledger (WU-G0-008 & WU-G0-019).
    /// Pure functional replay: no reliance on in-memory state or background services.
    pub fn replay(repo_root: &Path) -> Result<ReplayState> {
        let repo_info = GitInspector::inspect(repo_root)?;
        let ledger = LocalLedger::new(repo_root)?;

        let tasks = ledger.get_tasks()?;
        let mut confirmed = Vec::new();

        let decisions_dir = repo_root.join(".vwl").join("ledger").join("decisions");
        if decisions_dir.exists() {
            for entry in std::fs::read_dir(decisions_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "yaml") {
                    let content = std::fs::read_to_string(path)?;
                    let dec: crate::decision::Decision = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    if dec.outcome == DecisionOutcome::Confirmed {
                        confirmed.push(dec.subject);
                    }
                }
            }
        }

        confirmed.sort();

        Ok(ReplayState {
            total_tasks: tasks.len(),
            confirmed_work_units: confirmed,
            canonical_head: repo_info.canonical_base.or(Some(repo_info.head_sha)),
        })
    }
}

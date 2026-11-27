use crate::decision::DecisionOutcome;
use crate::error::Result;
use crate::git::GitInspector;
use crate::ledger::LocalLedger;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayState {
    pub total_tasks: usize,
    pub confirmed_work_units: Vec<String>,
    pub canonical_head: Option<String>,
    pub frontier_tasks: Vec<String>,
    pub active_claims: Vec<String>,
    pub total_outcomes: usize,
    pub active_regressions: Vec<String>,
    #[serde(default)]
    pub total_incidents: usize,
    #[serde(default)]
    pub active_barriers: Vec<String>,
}

pub struct ReplayEngine;

impl ReplayEngine {
    /// Reconstructs authoritative state from Git + Ledger across all four VWL graphs:
    /// Intent, Provenance, Evidence, and Impact (WU-G0-008, WU-G05-012, WU-G07-009).
    /// Pure functional replay: no reliance on in-memory state or background services.
    /// Fails deterministically if any cycle or unresolvable authority divergence occurs.
    pub fn replay(repo_root: &Path) -> Result<ReplayState> {
        let repo_info = GitInspector::inspect(repo_root)?;
        let ledger = LocalLedger::new(repo_root)?;

        let graph = ledger.get_task_graph()?;
        graph.validate_no_cycles()?;

        let mut confirmed_wus = Vec::new();
        let mut confirmed_tasks = HashSet::new();

        let decisions = ledger.get_decisions()?;
        let work_units = ledger.get_work_units()?;

        for dec in decisions {
            if dec.outcome == DecisionOutcome::Confirmed {
                confirmed_wus.push(dec.subject.clone());
                for wu in &work_units {
                    if wu.candidate == dec.subject || wu.commit == dec.subject {
                        confirmed_tasks.insert(wu.task.clone());
                    }
                }
            }
        }

        confirmed_wus.sort();

        // Derive the Work Frontier deterministically
        let frontier = graph.get_frontier(&confirmed_tasks);
        let frontier_task_ids: Vec<String> = frontier.into_iter().map(|t| t.id).collect();

        // Active claims
        let mut active_claims: Vec<String> = graph.claims.keys().cloned().collect();
        active_claims.sort();

        // Impact Graph: Outcomes & Regressions
        let outcomes = ledger.get_outcomes().unwrap_or_default();
        let mut active_regressions = Vec::new();
        for o in &outcomes {
            if o.outcome_type == crate::repometer::OutcomeType::Regression {
                active_regressions.push(o.id.clone());
            }
        }
        active_regressions.sort();

        // Incidents & Barriers (v2.3)
        let incidents = ledger.list_incidents().unwrap_or_default();
        let mut active_barriers = Vec::new();
        for inc in &incidents {
            for b in &inc.barriers {
                if !b.resolved {
                    active_barriers.push(b.barrier_node.clone());
                }
            }
        }
        active_barriers.sort();
        active_barriers.dedup();

        Ok(ReplayState {
            total_tasks: graph.tasks.len(),
            confirmed_work_units: confirmed_wus,
            canonical_head: repo_info.canonical_base.or(Some(repo_info.head_sha)),
            frontier_tasks: frontier_task_ids,
            active_claims,
            total_outcomes: outcomes.len(),
            active_regressions,
            total_incidents: incidents.len(),
            active_barriers,
        })
    }
}

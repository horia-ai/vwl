use crate::graph::{DependencyKind, TaskGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorRole {
    WavePlanner,
    Dispatcher,
    Triage,
    Validator,
    Ingest,
    SelfHealer,
}

impl SupervisorRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::WavePlanner => "wave_planner",
            Self::Dispatcher => "dispatcher",
            Self::Triage => "triage",
            Self::Validator => "validator",
            Self::Ingest => "ingest",
            Self::SelfHealer => "self_healer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::WavePlanner => "Computes topological dependency waves and work readiness",
            Self::Dispatcher => "Monitors ready tasks and launches worker worktrees",
            Self::Triage => "Audits expired claims, deadlocks, and stalled tasks",
            Self::Validator => "Evaluates candidate diffs and verification suites",
            Self::Ingest => "Ingests specifications and generates task definitions",
            Self::SelfHealer => "Analyzes incidents and issues containment and repair tasks",
        }
    }

    pub fn all() -> &'static [SupervisorRole] {
        &[
            Self::WavePlanner,
            Self::Dispatcher,
            Self::Triage,
            Self::Validator,
            Self::Ingest,
            Self::SelfHealer,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WavePlan {
    pub waves: Vec<Vec<String>>,
    pub blocked_tasks: HashMap<String, Vec<String>>,
}

impl WavePlan {
    pub fn compute(graph: &TaskGraph, confirmed: &HashSet<String>) -> Self {
        let mut waves: Vec<Vec<String>> = Vec::new();
        let mut scheduled: HashSet<String> = confirmed.clone();
        let mut remaining: HashSet<String> = graph
            .tasks
            .keys()
            .filter(|id| !graph.is_cancelled(id) && !confirmed.contains(*id))
            .cloned()
            .collect();

        loop {
            let mut current_wave: Vec<String> = Vec::new();
            for task_id in &remaining {
                let mut deps_satisfied = true;
                for dep in &graph.dependencies {
                    if dep.to_task == *task_id {
                        match dep.kind {
                            DependencyKind::Requires | DependencyKind::After => {
                                if !scheduled.contains(&dep.from_task) {
                                    deps_satisfied = false;
                                    break;
                                }
                            }
                            DependencyKind::Conflicts => {
                                if scheduled.contains(&dep.from_task) {
                                    deps_satisfied = false;
                                    break;
                                }
                            }
                            DependencyKind::Evidence => {}
                        }
                    }
                }
                if deps_satisfied {
                    current_wave.push(task_id.clone());
                }
            }

            if current_wave.is_empty() {
                break;
            }

            current_wave.sort();
            for id in &current_wave {
                remaining.remove(id);
                scheduled.insert(id.clone());
            }
            waves.push(current_wave);
        }

        let mut blocked_tasks: HashMap<String, Vec<String>> = HashMap::new();
        for task_id in &remaining {
            let reasons = graph.why_blocked(task_id, confirmed);
            blocked_tasks.insert(task_id.clone(), reasons);
        }

        Self {
            waves,
            blocked_tasks,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DispatchAction {
    pub task_id: String,
    pub title: String,
    pub assigned_actor: Option<String>,
    pub recommended_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DispatchReport {
    pub ready_tasks_count: usize,
    pub active_claims_count: usize,
    pub dispatches: Vec<DispatchAction>,
}

pub struct Dispatcher;

impl Dispatcher {
    pub fn evaluate(
        graph: &TaskGraph,
        confirmed: &HashSet<String>,
        current_time: u64,
    ) -> DispatchReport {
        let frontier = graph.get_frontier(confirmed);
        let ready_tasks_count = frontier.len();
        let mut active_claims_count = 0;
        let mut dispatches = Vec::new();

        for task in frontier {
            let is_claimed = if let Some(claim) = graph.claims.get(&task.id) {
                if claim.is_active(current_time) {
                    active_claims_count += 1;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !is_claimed {
                dispatches.push(DispatchAction {
                    task_id: task.id.clone(),
                    title: task.objective.clone(),
                    assigned_actor: None,
                    recommended_command: format!("vwl work {}", task.id),
                });
            }
        }

        dispatches.sort_by(|a, b| a.task_id.cmp(&b.task_id));

        DispatchReport {
            ready_tasks_count,
            active_claims_count,
            dispatches,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriageKind {
    StaleClaim,
    UnmetDependency,
    CycleOrBlocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriageFinding {
    pub task_id: String,
    pub kind: TriageKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriageReport {
    pub findings: Vec<TriageFinding>,
    pub expired_claims: Vec<String>,
}

pub struct Triage;

impl Triage {
    pub fn evaluate(
        graph: &TaskGraph,
        confirmed: &HashSet<String>,
        current_time: u64,
    ) -> TriageReport {
        let mut findings = Vec::new();
        let mut expired_claims = Vec::new();

        for (task_id, claim) in &graph.claims {
            if !claim.is_active(current_time) {
                expired_claims.push(task_id.clone());
                findings.push(TriageFinding {
                    task_id: task_id.clone(),
                    kind: TriageKind::StaleClaim,
                    detail: format!("Claim by '{}' expired", claim.actor),
                });
            }
        }
        expired_claims.sort();

        for task_id in graph.tasks.keys() {
            if !confirmed.contains(task_id) && !graph.is_cancelled(task_id) {
                let reasons = graph.why_blocked(task_id, confirmed);
                if !reasons.is_empty() && !graph.is_ready(task_id, confirmed) {
                    findings.push(TriageFinding {
                        task_id: task_id.clone(),
                        kind: TriageKind::UnmetDependency,
                        detail: reasons.join("; "),
                    });
                }
            }
        }

        findings.sort_by(|a, b| a.task_id.cmp(&b.task_id));

        TriageReport {
            findings,
            expired_claims,
        }
    }
}

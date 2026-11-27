use crate::error::{Result, VwlError};
use crate::repometer::AttributionConfidence;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IncidentState {
    Clean,
    Suspect,
    RevalidateRequired,
    RepairRequired,
    WaitingOnRepair,
    Invalidated,
    Revalidated,
}

impl IncidentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "CLEAN",
            Self::Suspect => "SUSPECT",
            Self::RevalidateRequired => "REVALIDATE_REQUIRED",
            Self::RepairRequired => "REPAIR_REQUIRED",
            Self::WaitingOnRepair => "WAITING_ON_REPAIR",
            Self::Invalidated => "INVALIDATED",
            Self::Revalidated => "REVALIDATED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ContainmentStrategy {
    Rollback,
    RollForward,
    Isolate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainmentEvent {
    pub outcome_id: String,
    pub strategy: ContainmentStrategy,
    pub target: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestorationClocks {
    pub detected_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_restored_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_restored_at: Option<u64>,
}

impl RestorationClocks {
    pub fn new(detected_at: u64) -> Self {
        Self {
            detected_at,
            service_restored_at: None,
            evidence_restored_at: None,
        }
    }

    pub fn restore_service(&mut self, timestamp: u64) {
        self.service_restored_at = Some(timestamp);
    }

    pub fn restore_evidence(&mut self, timestamp: u64) {
        self.evidence_restored_at = Some(timestamp);
    }

    pub fn srt_secs(&self) -> Option<u64> {
        self.service_restored_at
            .map(|t| t.saturating_sub(self.detected_at))
    }

    pub fn ert_secs(&self) -> Option<u64> {
        self.evidence_restored_at
            .map(|t| t.saturating_sub(self.detected_at))
    }

    pub fn is_service_restored(&self) -> bool {
        self.service_restored_at.is_some()
    }

    pub fn is_evidence_restored(&self) -> bool {
        self.evidence_restored_at.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepairBarrier {
    pub barrier_node: String,
    pub waiting_descendants: Vec<String>,
    pub created_at: u64,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ObligationAssessment {
    Miss,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidatorObligation {
    pub gate_id: String,
    pub validator_id: String,
    pub detectable_classes: Vec<String>,
}

impl ValidatorObligation {
    pub fn assess(&self, failure_class: &str) -> ObligationAssessment {
        if self
            .detectable_classes
            .iter()
            .any(|c| c.eq_ignore_ascii_case(failure_class))
        {
            ObligationAssessment::Miss
        } else {
            ObligationAssessment::NotApplicable
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SkillContribution {
    Contributed,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlaMetric {
    pub service: String,
    pub window_secs: u64,
    pub total_eligible_minutes: u64,
    pub good_service_minutes: u64,
    pub bad_service_minutes: u64,
}

impl SlaMetric {
    pub fn new(
        service: impl Into<String>,
        window_secs: u64,
        total_eligible_minutes: u64,
        bad_service_minutes: u64,
    ) -> Self {
        let good_service_minutes = total_eligible_minutes.saturating_sub(bad_service_minutes);
        Self {
            service: service.into(),
            window_secs,
            total_eligible_minutes,
            good_service_minutes,
            bad_service_minutes,
        }
    }

    pub fn availability_pct(&self) -> f64 {
        if self.total_eligible_minutes == 0 {
            100.0
        } else {
            (self.good_service_minutes as f64 / self.total_eligible_minutes as f64) * 100.0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IncidentRecord {
    pub outcome_id: String,
    pub service: String,
    pub root_work_unit: String,
    pub state: IncidentState,
    pub clocks: RestorationClocks,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub containment: Option<ContainmentEvent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub barriers: Vec<RepairBarrier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_frontier: Vec<String>,
    #[serde(default)]
    pub node_states: HashMap<String, IncidentState>,
}

impl IncidentRecord {
    pub fn new(
        outcome_id: impl Into<String>,
        service: impl Into<String>,
        root_work_unit: impl Into<String>,
        detected_at: u64,
    ) -> Self {
        let outcome_id = outcome_id.into();
        let service = service.into();
        let root_work_unit = root_work_unit.into();
        let mut node_states = HashMap::new();
        node_states.insert(root_work_unit.clone(), IncidentState::RepairRequired);

        Self {
            outcome_id,
            service,
            root_work_unit,
            state: IncidentState::Suspect,
            clocks: RestorationClocks::new(detected_at),
            containment: None,
            barriers: Vec::new(),
            affected_frontier: Vec::new(),
            node_states,
        }
    }

    pub fn contain(
        &mut self,
        strategy: ContainmentStrategy,
        target: impl Into<String>,
        timestamp: u64,
    ) {
        self.containment = Some(ContainmentEvent {
            outcome_id: self.outcome_id.clone(),
            strategy,
            target: target.into(),
            timestamp,
        });
        self.clocks.restore_service(timestamp);
    }

    pub fn erect_barrier(
        &mut self,
        barrier_node: impl Into<String>,
        waiting: Vec<String>,
        timestamp: u64,
    ) {
        let node = barrier_node.into();
        self.node_states
            .insert(node.clone(), IncidentState::RepairRequired);
        for w in &waiting {
            self.node_states
                .insert(w.clone(), IncidentState::WaitingOnRepair);
        }
        self.barriers.push(RepairBarrier {
            barrier_node: node,
            waiting_descendants: waiting,
            created_at: timestamp,
            resolved: false,
        });
    }

    pub fn resolve_barrier(&mut self, barrier_node: &str) -> Vec<String> {
        let mut released = Vec::new();
        for b in &mut self.barriers {
            if b.barrier_node == barrier_node && !b.resolved {
                b.resolved = true;
                released.extend(b.waiting_descendants.clone());
            }
        }
        for r in &released {
            self.node_states
                .insert(r.clone(), IncidentState::RevalidateRequired);
        }
        released
    }

    pub fn set_node_state(&mut self, node: impl Into<String>, state: IncidentState) {
        self.node_states.insert(node.into(), state);
    }

    pub fn check_evidence_restored(&mut self, timestamp: u64) -> bool {
        let all_restored = self
            .node_states
            .values()
            .all(|s| *s == IncidentState::Revalidated || *s == IncidentState::Clean);
        if all_restored && !self.clocks.is_evidence_restored() {
            self.clocks.restore_evidence(timestamp);
            self.state = IncidentState::Revalidated;
            true
        } else {
            false
        }
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).map_err(|e| VwlError::SerializationError(e.to_string()))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))
    }
}

pub struct IncidentFrontierEngine;

impl IncidentFrontierEngine {
    /// Computes immediate affected frontier F1 for a root work unit,
    /// avoiding full descendant rebuild.
    pub fn compute_immediate_frontier(root_wu: &str, direct_children: &[String]) -> Vec<String> {
        let mut frontier = Vec::new();
        for child in direct_children {
            if child != root_wu && !frontier.contains(child) {
                frontier.push(child.clone());
            }
        }
        frontier.sort();
        frontier
    }

    /// Evaluates a revalidation wave with branch stopping:
    /// - PASS / UNAFFECTED stops branch traversal
    /// - REPAIR_REQUIRED places barrier and sets descendants to WAITING_ON_REPAIR
    pub fn process_wave(
        wave_nodes: &[String],
        results: &HashMap<String, IncidentState>,
        children_map: &HashMap<String, Vec<String>>,
    ) -> WaveProcessingOutcome {
        let mut revalidated = Vec::new();
        let mut repair_required = Vec::new();
        let mut unaffected = Vec::new();
        let mut barriers = Vec::new();
        let mut next_wave = Vec::new();

        for node in wave_nodes {
            let state = results
                .get(node)
                .copied()
                .unwrap_or(IncidentState::RevalidateRequired);
            match state {
                IncidentState::Revalidated => {
                    revalidated.push(node.clone());
                    // Branch stop! Does not propagate to children unless children have other stale dependencies
                }
                IncidentState::Clean => {
                    unaffected.push(node.clone());
                    // Branch stop!
                }
                IncidentState::RepairRequired => {
                    repair_required.push(node.clone());
                    // Collect all deep descendants for barrier
                    let mut descendants = Vec::new();
                    Self::collect_descendants(node, children_map, &mut descendants);
                    barriers.push(RepairBarrier {
                        barrier_node: node.clone(),
                        waiting_descendants: descendants,
                        created_at: 0,
                        resolved: false,
                    });
                }
                IncidentState::RevalidateRequired | IncidentState::Suspect => {
                    // Propagate to immediate children if any
                    if let Some(children) = children_map.get(node) {
                        for c in children {
                            if !next_wave.contains(c) {
                                next_wave.push(c.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        WaveProcessingOutcome {
            revalidated,
            repair_required,
            unaffected,
            barriers,
            next_wave,
        }
    }

    fn collect_descendants(
        node: &str,
        children_map: &HashMap<String, Vec<String>>,
        out: &mut Vec<String>,
    ) {
        if let Some(children) = children_map.get(node) {
            for c in children {
                if !out.contains(c) {
                    out.push(c.clone());
                    Self::collect_descendants(c, children_map, out);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaveProcessingOutcome {
    pub revalidated: Vec<String>,
    pub repair_required: Vec<String>,
    pub unaffected: Vec<String>,
    pub barriers: Vec<RepairBarrier>,
    pub next_wave: Vec<String>,
}

/// Evaluates whether a Node should receive negative REP evidence as a result of an incident.
///
/// Invariant: Ripple membership alone (Suspect, WaitingOnRepair, RevalidateRequired)
/// strictly MUST NOT create negative REP evidence.
/// Only causal roots with High or Medium attribution confidence, or Nodes whose own
/// Work Unit requires code repair upon revalidation, receive negative evidence.
pub fn evaluate_node_rep_impact(
    node_id: &str,
    incident_root_node: &str,
    attribution_confidence: AttributionConfidence,
    node_state: IncidentState,
) -> bool {
    if node_id == incident_root_node {
        matches!(
            attribution_confidence,
            AttributionConfidence::High | AttributionConfidence::Medium
        )
    } else {
        // Descendant only receives negative REP if its OWN revalidation failed and required repair
        matches!(
            node_state,
            IncidentState::RepairRequired | IncidentState::Invalidated
        )
    }
}

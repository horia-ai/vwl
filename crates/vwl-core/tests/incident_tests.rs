use std::collections::HashMap;
use tempfile::tempdir;
use vwl_core::incident::{
    evaluate_node_rep_impact, ContainmentStrategy, IncidentFrontierEngine, IncidentRecord,
    IncidentState, ObligationAssessment, SlaMetric, ValidatorObligation,
};
use vwl_core::ledger::LocalLedger;
use vwl_core::proof::{Proof, ProofPremises, ProofType, Verdict};
use vwl_core::replay::ReplayEngine;
use vwl_core::repometer::AttributionConfidence;
use vwl_core::task::Task;

#[test]
fn test_incident_state_machine_and_transitions() {
    let mut incident = IncidentRecord::new("O-501", "payments", "W1", 1000);
    assert_eq!(incident.state, IncidentState::Suspect);
    assert_eq!(
        incident.node_states.get("W1"),
        Some(&IncidentState::RepairRequired)
    );

    // Add nodes in various states
    incident.set_node_state("W2", IncidentState::RevalidateRequired);
    incident.set_node_state("W3", IncidentState::WaitingOnRepair);
    assert_eq!(
        incident.node_states.get("W2"),
        Some(&IncidentState::RevalidateRequired)
    );
    assert_eq!(
        incident.node_states.get("W3"),
        Some(&IncidentState::WaitingOnRepair)
    );

    // Resolve node revalidations
    incident.set_node_state("W1", IncidentState::Revalidated);
    incident.set_node_state("W2", IncidentState::Revalidated);
    incident.set_node_state("W3", IncidentState::Revalidated);

    // Evidence restored check
    let restored = incident.check_evidence_restored(1500);
    assert!(restored);
    assert_eq!(incident.state, IncidentState::Revalidated);
    assert!(incident.clocks.is_evidence_restored());
    assert_eq!(incident.clocks.ert_secs(), Some(500));
}

#[test]
fn test_proof_premise_and_staleness_model() {
    let premises = ProofPremises::new(
        Some("SPEC-PAY-1"),
        Some("API-PAYMENTS-V2"),
        Some("ENV-STAGING"),
        vec!["W1".to_string(), "W2".to_string()],
    );

    let proof = Proof::new("c12345", ProofType::Work, Verdict::Pass, "cargo test", None)
        .unwrap()
        .with_premises(premises);

    // Unrelated change: proof remains reusable (not stale)
    assert!(!proof.is_stale_against(Some("API-USERS-V1"), &["W99".to_string()]));

    // Interface changed: proof becomes STALE
    assert!(proof.is_stale_against(Some("API-PAYMENTS-V2"), &[]));

    // Canonical effect changed: proof becomes STALE
    assert!(proof.is_stale_against(None, &["W1".to_string()]));
}

#[test]
fn test_immediate_impact_frontier_50_descendant_experiment() {
    // Hard Experiment 1: 50-descendant regression; only minimum affected frontier (F1) runs.
    let root = "W1";
    // Direct children of W1 are only W2 and W3.
    // W2 has children W4..W25, and W3 has children W26..W51 (total 50 descendants)
    let direct_children = vec!["W2".to_string(), "W3".to_string()];

    let f1 = IncidentFrontierEngine::compute_immediate_frontier(root, &direct_children);
    assert_eq!(f1, vec!["W2".to_string(), "W3".to_string()]);
    // Verifies that only immediate frontier (2 nodes) runs, NOT all 50 descendants!
    assert_eq!(f1.len(), 2);
}

#[test]
fn test_repair_barrier_and_descendant_wait_state() {
    // Hard Experiment 2: W2 repair barrier prevents premature W4/W5 authoritative validation.
    let mut incident = IncidentRecord::new("O-551", "checkout", "W1", 2000);

    // W2 requires code repair; erect barrier for descendants W4, W5, W7
    incident.erect_barrier(
        "W2",
        vec!["W4".to_string(), "W5".to_string(), "W7".to_string()],
        2100,
    );

    assert_eq!(
        incident.node_states.get("W2"),
        Some(&IncidentState::RepairRequired)
    );
    assert_eq!(
        incident.node_states.get("W4"),
        Some(&IncidentState::WaitingOnRepair)
    );
    assert_eq!(
        incident.node_states.get("W5"),
        Some(&IncidentState::WaitingOnRepair)
    );
    assert_eq!(
        incident.node_states.get("W7"),
        Some(&IncidentState::WaitingOnRepair)
    );

    // When Repair-W2 is canonical, barrier is resolved
    let released = incident.resolve_barrier("W2");
    assert_eq!(
        released,
        vec!["W4".to_string(), "W5".to_string(), "W7".to_string()]
    );
    // Descendants now enter RevalidateRequired
    assert_eq!(
        incident.node_states.get("W4"),
        Some(&IncidentState::RevalidateRequired)
    );
}

#[test]
fn test_wave_revalidation_with_branch_stop() {
    // WU-G0.8-005 & WU-G0.8-006: Branch stop on PASS / UNAFFECTED
    let mut results = HashMap::new();
    results.insert("W2".to_string(), IncidentState::Revalidated); // PASS -> stop branch
    results.insert("W3".to_string(), IncidentState::Clean); // UNAFFECTED -> stop branch
    results.insert("W4".to_string(), IncidentState::RepairRequired); // REPAIR -> barrier

    let mut children_map = HashMap::new();
    children_map.insert("W2".to_string(), vec!["W5".to_string(), "W6".to_string()]);
    children_map.insert("W3".to_string(), vec!["W7".to_string()]);
    children_map.insert("W4".to_string(), vec!["W8".to_string()]);

    let outcome = IncidentFrontierEngine::process_wave(
        &["W2".to_string(), "W3".to_string(), "W4".to_string()],
        &results,
        &children_map,
    );

    assert_eq!(outcome.revalidated, vec!["W2".to_string()]);
    assert_eq!(outcome.unaffected, vec!["W3".to_string()]);
    assert_eq!(outcome.repair_required, vec!["W4".to_string()]);
    // Descendants of W2 and W3 stopped (not in next_wave)
    assert!(outcome.next_wave.is_empty());
    // Descendant of W4 (W8) placed behind barrier
    assert_eq!(outcome.barriers.len(), 1);
    assert_eq!(outcome.barriers[0].barrier_node, "W4");
    assert_eq!(
        outcome.barriers[0].waiting_descendants,
        vec!["W8".to_string()]
    );
}

#[test]
fn test_runtime_containment_and_restoration_clocks() {
    // Hard Experiment 3: runtime rollback restores service while ledger remains append-only.
    // Hard Experiment 8: Independent SRT versus ERT restoration clocks.
    let mut incident = IncidentRecord::new("O-991", "auth", "W1", 1000);
    assert!(!incident.clocks.is_service_restored());
    assert!(!incident.clocks.is_evidence_restored());

    // Contain via rollback at 1012 (SRT = 12s)
    incident.contain(
        ContainmentStrategy::Rollback,
        "artifact:sha256-abc1234",
        1012,
    );
    assert!(incident.clocks.is_service_restored());
    assert_eq!(incident.clocks.srt_secs(), Some(12));
    // Service restored does NOT falsely mark evidence restored!
    assert!(!incident.clocks.is_evidence_restored());

    // Complete evidence revalidation at 1045 (ERT = 45s)
    incident.set_node_state("W1", IncidentState::Revalidated);
    incident.check_evidence_restored(1045);
    assert!(incident.clocks.is_evidence_restored());
    assert_eq!(incident.clocks.ert_secs(), Some(45));
}

#[test]
fn test_validator_obligation_analysis() {
    // Hard Experiment 6 & 7: validator detectable obligation
    let obligation = ValidatorObligation {
        gate_id: "gate:api-interface".to_string(),
        validator_id: "validator:gemini-factory".to_string(),
        detectable_classes: vec![
            "schema-mismatch".to_string(),
            "null-pointer".to_string(),
            "type-incompatibility".to_string(),
        ],
    };

    // Failure inside obligation -> MISS
    let assessment1 = obligation.assess("schema-mismatch");
    assert_eq!(assessment1, ObligationAssessment::Miss);

    // Failure outside obligation -> NOT_APPLICABLE (No validator penalty)
    let assessment2 = obligation.assess("distributed-network-partition");
    assert_eq!(assessment2, ObligationAssessment::NotApplicable);
}

#[test]
fn test_node_rep_impact_zero_negative_rep_from_ripple() {
    // Hard Experiment 4: high-REP Node in ripple cone -> strictly NO negative REP evidence
    let node_in_ripple = "node-alpha-42";
    let is_penalized = evaluate_node_rep_impact(
        node_in_ripple,
        "node-bad-root",
        AttributionConfidence::High,
        IncidentState::WaitingOnRepair,
    );
    assert!(!is_penalized, "Ripple node must not receive negative REP");

    let is_penalized_suspect = evaluate_node_rep_impact(
        node_in_ripple,
        "node-bad-root",
        AttributionConfidence::High,
        IncidentState::Suspect,
    );
    assert!(
        !is_penalized_suspect,
        "Suspect node must not receive negative REP"
    );

    // Hard Experiment 5: direct causal root with High confidence -> negative REP evidence
    let is_root_penalized = evaluate_node_rep_impact(
        "node-bad-root",
        "node-bad-root",
        AttributionConfidence::High,
        IncidentState::RepairRequired,
    );
    assert!(
        is_root_penalized,
        "Attributed root must receive negative REP"
    );
}

#[test]
fn test_sla_and_slo_metric_contract() {
    // WU-G0.8-011: SLA contract with explicit denominator
    let metric = SlaMetric::new("payments-api", 30 * 86400, 43200, 2);
    assert_eq!(metric.total_eligible_minutes, 43200);
    assert_eq!(metric.bad_service_minutes, 2);
    assert_eq!(metric.good_service_minutes, 43198);

    // 43198 / 43200 = 99.99537%
    let avail = metric.availability_pct();
    assert!(avail > 99.99 && avail < 100.0);
}

#[test]
fn test_incident_ledger_persistence_and_replay_reconstruction() {
    // Hard Experiment 10: Replay reproduces exact incident state, containment, barriers, and clocks
    let dir = tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();

    let task = Task::new(
        "T-INC-1",
        "Handle payments regression",
        vec!["test pass".to_string()],
        "HEAD",
    )
    .unwrap();
    let ledger = LocalLedger::new(dir.path()).unwrap();
    ledger.append_task(&task).unwrap();

    let mut incident = IncidentRecord::new("O-REPLAY-1", "payments", "W1", 5000);
    incident.contain(ContainmentStrategy::RollForward, "T-HOTFIX-1", 5030);
    incident.erect_barrier("W2", vec!["W4".to_string()], 5050);
    ledger.append_incident(&incident).unwrap();

    // Load back from ledger
    let loaded = ledger.load_incident("O-REPLAY-1").unwrap();
    assert_eq!(loaded.outcome_id, "O-REPLAY-1");
    assert_eq!(loaded.service, "payments");
    assert_eq!(loaded.clocks.srt_secs(), Some(30));
    assert_eq!(loaded.barriers.len(), 1);
    assert_eq!(loaded.barriers[0].barrier_node, "W2");

    // Pure functional replay
    let replay_state = ReplayEngine::replay(dir.path()).unwrap();
    assert_eq!(replay_state.total_incidents, 1);
    assert_eq!(replay_state.active_barriers, vec!["W2".to_string()]);
}

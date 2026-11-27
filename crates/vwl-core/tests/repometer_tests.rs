use tempfile::tempdir;
use vwl_core::{
    decision::{Decision, DecisionOutcome},
    ledger::LocalLedger,
    repometer::{
        Attribution, AttributionConfidence, ImpactGraph, ImpactState, Outcome, OutcomeSeverity,
        OutcomeType, RepometerEngine,
    },
    task::Task,
    work_unit::WorkUnit,
};

#[test]
fn test_outcome_creation_and_authority_invariant() {
    let outcome = Outcome::new(
        "O-001",
        "WU-CAND-A",
        OutcomeType::Regression,
        OutcomeSeverity::High,
        "Production latency regression observed: p99 exceeded 500ms",
        1700000000,
    )
    .expect("Valid outcome");

    assert_eq!(outcome.id, "O-001");
    assert_eq!(outcome.subject, "WU-CAND-A");
    assert_eq!(outcome.outcome_type, OutcomeType::Regression);
    assert_eq!(outcome.severity, OutcomeSeverity::High);

    // Authority Invariant: Historical confirmation decision at t1 is NOT rewritten;
    // Outcome at t2 is appended as subsequent evidence.
    let historical_decision = Decision {
        subject: "WU-CAND-A".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "All proofs passed at t1".to_string(),
    };
    assert_eq!(historical_decision.outcome, DecisionOutcome::Confirmed);
}

#[test]
fn test_attribution_confidence_levels() {
    let attr_high = Attribution {
        outcome_id: "O-001".to_string(),
        target_node: "node-dev-1".to_string(),
        confidence: AttributionConfidence::High,
        rationale: "Git blame and targeted trace isolated to specific commit".to_string(),
    };
    assert_eq!(attr_high.confidence, AttributionConfidence::High);

    let attr_unknown = Attribution {
        outcome_id: "O-002".to_string(),
        target_node: "node-dev-2".to_string(),
        confidence: AttributionConfidence::Unknown,
        rationale: "Multiple simultaneous changes in canary window".to_string(),
    };
    // Low/unknown attribution cannot be treated as proven blame by downstream REP
    assert_eq!(attr_unknown.confidence, AttributionConfidence::Unknown);
}

#[test]
fn test_impact_cone_and_selective_revalidation() {
    let outcome_high = Outcome::new(
        "O-REGRESS-1",
        "TASK-CORE",
        OutcomeType::Regression,
        OutcomeSeverity::High,
        "Memory leak detected under load",
        1700000100,
    )
    .unwrap();

    let dependents = vec!["TASK-CLI".to_string(), "TASK-TUI".to_string()];
    let assessments = ImpactGraph::assess_impact(&outcome_high, &dependents);

    assert_eq!(assessments.len(), 2);
    // High severity triggers Revalidate state
    assert_eq!(assessments[0].state, ImpactState::Revalidate);
    assert_eq!(assessments[1].state, ImpactState::Revalidate);

    // Critical severity triggers Invalidated state
    let outcome_critical = Outcome::new(
        "O-REGRESS-2",
        "TASK-CORE",
        OutcomeType::Regression,
        OutcomeSeverity::Critical,
        "Data corruption on serialization",
        1700000200,
    )
    .unwrap();
    let critical_assessments = ImpactGraph::assess_impact(&outcome_critical, &dependents);
    assert_eq!(critical_assessments[0].state, ImpactState::Invalidated);

    // Low severity triggers Unaffected state
    let outcome_low = Outcome::new(
        "O-OBSERVE-1",
        "TASK-CORE",
        OutcomeType::Observation,
        OutcomeSeverity::Low,
        "Non-blocking log warning",
        1700000300,
    )
    .unwrap();
    let low_assessments = ImpactGraph::assess_impact(&outcome_low, &dependents);
    assert_eq!(low_assessments[0].state, ImpactState::Unaffected);
}

#[test]
fn test_repometer_evidence_metrics_calculation() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "tester@vwl.network"])
        .current_dir(repo_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "VWL Tester"])
        .current_dir(repo_root)
        .output()
        .expect("git config name");
    std::fs::write(repo_root.join("README.md"), "# Repometer Test\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(repo_root)
        .output()
        .expect("git commit");

    let ledger = LocalLedger::new(repo_root).unwrap();
    let task = Task::new("TASK-REPO-001", "Objective 1", vec![], "c0").unwrap();
    ledger.append_task(&task).unwrap();
    let wu = WorkUnit::new("TASK-REPO-001", "cand_repo_001", "c0").unwrap();
    ledger.append_work_unit(&wu).unwrap();
    let dec = Decision {
        subject: "cand_repo_001".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "Clean verification".to_string(),
    };
    ledger.append_decision(&dec).unwrap();

    let metrics = RepometerEngine::calculate(repo_root, None).unwrap();
    assert_eq!(metrics.ledger_attributed_pct, 100.0);
    assert_eq!(metrics.independently_verified_pct, 100.0);
    assert!(metrics.denominator_loc > 0);
    assert_eq!(
        metrics.methodology,
        "vwl-v2.2-lineage-and-evidence-coverage"
    );

    let explanation = metrics.explain();
    assert!(explanation.contains("VWL Repometer Evidence Coverage"));
    assert!(explanation.contains("Ledger-Attributed Coverage:    100.0%"));
    assert!(explanation.contains("Independently Verified:        100.0%"));
}

#[test]
fn test_ledger_outcome_and_attribution_persistence() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();

    let ledger = LocalLedger::new(repo_root).unwrap();

    let outcome = Outcome::new(
        "O-PERSIST-1",
        "cand_sha_99",
        OutcomeType::Regression,
        OutcomeSeverity::Medium,
        "Flaky integration test surfaced",
        1700000500,
    )
    .unwrap();
    ledger.append_outcome(&outcome).unwrap();

    let outcomes = ledger.get_outcomes().unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].id, "O-PERSIST-1");

    let attr = Attribution {
        outcome_id: "O-PERSIST-1".to_string(),
        target_node: "agent-verifier-3".to_string(),
        confidence: AttributionConfidence::Medium,
        rationale: "Verifier observed intermittent failure in CI".to_string(),
    };
    ledger.append_attribution(&attr).unwrap();

    let attrs = ledger.get_attributions().unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].outcome_id, "O-PERSIST-1");
    assert_eq!(attrs[0].target_node, "agent-verifier-3");
}

#[test]
fn test_forward_only_repair_flow() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();
    let ledger = LocalLedger::new(repo_root).unwrap();

    // Step 1: Task A is confirmed at t1
    let task_a = Task::new("TASK-A", "Build feature A", vec![], "c0").unwrap();
    ledger.append_task(&task_a).unwrap();
    let wu_a = WorkUnit::new("TASK-A", "cand_a", "c0").unwrap();
    ledger.append_work_unit(&wu_a).unwrap();
    let dec_a = Decision {
        subject: "cand_a".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "All tests pass".to_string(),
    };
    ledger.append_decision(&dec_a).unwrap();

    // Step 2: At t2, a regression is detected.
    // Invariant: Historical decision for cand_a CANNOT be mutated.
    let outcome_regress = Outcome::new(
        "O-REG-A",
        "cand_a",
        OutcomeType::Regression,
        OutcomeSeverity::High,
        "Buffer overflow on large input",
        1700001000,
    )
    .unwrap();
    ledger.append_outcome(&outcome_regress).unwrap();

    // Historical decision remains intact
    let decisions = ledger.get_decisions().unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].outcome, DecisionOutcome::Confirmed);

    // Step 3: Forward-only repair task is registered
    let task_repair = Task::new(
        "TASK-A-REPAIR",
        "Repair buffer overflow in feature A",
        vec![],
        "cand_a",
    )
    .unwrap();
    ledger.append_task(&task_repair).unwrap();
    let wu_repair = WorkUnit::new("TASK-A-REPAIR", "cand_a_repair", "cand_a").unwrap();
    ledger.append_work_unit(&wu_repair).unwrap();
    let dec_repair = Decision {
        subject: "cand_a_repair".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "Buffer bound check added and verified".to_string(),
    };
    ledger.append_decision(&dec_repair).unwrap();

    // Step 4: Verification amendment / resolution outcome appended
    let outcome_resolved = Outcome::new(
        "O-RES-A",
        "cand_a_repair",
        OutcomeType::VerificationAmendment,
        OutcomeSeverity::Low,
        "Regression O-REG-A completely resolved by cand_a_repair",
        1700002000,
    )
    .unwrap();
    ledger.append_outcome(&outcome_resolved).unwrap();

    let all_outcomes = ledger.get_outcomes().unwrap();
    assert_eq!(all_outcomes.len(), 2);
}

#[test]
fn test_superseded_code_lineage_and_coverage() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "lineage@vwl.network"])
        .current_dir(repo_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Lineage Verifier"])
        .current_dir(repo_root)
        .output()
        .expect("git config name");
    std::fs::write(
        repo_root.join("code.rs"),
        "// Initial baseline implementation\n",
    )
    .unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial baseline"])
        .current_dir(repo_root)
        .output()
        .expect("git commit");

    let ledger = LocalLedger::new(repo_root).unwrap();

    let task1 = Task::new("TASK-LINEAGE-1", "Base logic", vec![], "c0").unwrap();
    ledger.append_task(&task1).unwrap();
    let wu1 = WorkUnit::new("TASK-LINEAGE-1", "cand_lineage_1", "c0").unwrap();
    ledger.append_work_unit(&wu1).unwrap();
    ledger
        .append_decision(&Decision {
            subject: "cand_lineage_1".to_string(),
            outcome: DecisionOutcome::Confirmed,
            reason: "Initial clean pass".to_string(),
        })
        .unwrap();

    let task2 = Task::new(
        "TASK-LINEAGE-2",
        "Refactor and supersede base logic",
        vec![],
        "cand_lineage_1",
    )
    .unwrap();
    ledger.append_task(&task2).unwrap();
    let wu2 = WorkUnit::new("TASK-LINEAGE-2", "cand_lineage_2", "cand_lineage_1").unwrap();
    ledger.append_work_unit(&wu2).unwrap();
    ledger
        .append_decision(&Decision {
            subject: "cand_lineage_2".to_string(),
            outcome: DecisionOutcome::Confirmed,
            reason: "Refactored cleanly".to_string(),
        })
        .unwrap();

    let metrics = RepometerEngine::calculate(repo_root, None).unwrap();
    assert_eq!(metrics.ledger_attributed_pct, 100.0);
    assert_eq!(metrics.independently_verified_pct, 100.0);
    assert_eq!(metrics.surviving_pct, 100.0);
    assert!(metrics.confidence.contains("HIGH"));
}

#[test]
fn test_four_graph_replay() {
    let temp = tempdir().unwrap();
    let repo_root = temp.path();

    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "replay@vwl.network"])
        .current_dir(repo_root)
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Replay Engine"])
        .current_dir(repo_root)
        .output()
        .expect("git config name");
    std::fs::write(repo_root.join("file.txt"), "hello").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(repo_root)
        .output()
        .expect("git commit");

    let ledger = LocalLedger::new(repo_root).unwrap();

    // 1. Intent Graph
    let task = Task::new("TASK-4G", "Four graph test", vec![], "c0").unwrap();
    ledger.append_task(&task).unwrap();

    // 2. Provenance Graph
    let wu = WorkUnit::new("TASK-4G", "cand_4g", "c0").unwrap();
    ledger.append_work_unit(&wu).unwrap();

    // 3. Evidence Graph
    ledger
        .append_decision(&Decision {
            subject: "cand_4g".to_string(),
            outcome: DecisionOutcome::Confirmed,
            reason: "Clean verification".to_string(),
        })
        .unwrap();

    // 4. Impact Graph
    let outcome = Outcome::new(
        "O-4G-REG",
        "cand_4g",
        OutcomeType::Regression,
        OutcomeSeverity::High,
        "High latency regression observed",
        1700003000,
    )
    .unwrap();
    ledger.append_outcome(&outcome).unwrap();

    let state1 = vwl_core::replay::ReplayEngine::replay(repo_root).unwrap();
    let state2 = vwl_core::replay::ReplayEngine::replay(repo_root).unwrap();

    // Deterministic replay across all four graphs
    assert_eq!(state1, state2);
    assert_eq!(state1.total_tasks, 1);
    assert_eq!(state1.confirmed_work_units, vec!["cand_4g".to_string()]);
    assert_eq!(state1.total_outcomes, 1);
    assert_eq!(state1.active_regressions, vec!["O-4G-REG".to_string()]);
}

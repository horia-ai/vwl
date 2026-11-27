use std::collections::HashSet;
use tempfile::tempdir;
use vwl_core::{
    decision::{Decision, DecisionOutcome},
    graph::{Claim, DependencyKind, TaskDependency, TaskGraph},
    ledger::LocalLedger,
    proof::{Proof, ProofType, Verdict, VerificationPlane, VisibilityProfile},
    replay::ReplayEngine,
    task::Task,
    work_unit::{CommitEntry, WorkUnit},
};

#[test]
fn test_v22_workunit_commit_sequence_and_tree_effect() {
    let commits = vec![
        CommitEntry {
            sha: "sha_001_test".to_string(),
            role: "unit-test".to_string(),
            actor: "node-test-agent".to_string(),
        },
        CommitEntry {
            sha: "sha_002_impl".to_string(),
            role: "implementation".to_string(),
            actor: "node-dev-agent".to_string(),
        },
        CommitEntry {
            sha: "sha_003_refactor".to_string(),
            role: "refactor".to_string(),
            actor: "node-dev-agent".to_string(),
        },
    ];

    let wu = WorkUnit::new_sequence(
        "T-381",
        "base_c100",
        commits.clone(),
        "sha_003_refactor",
        Some("sha256:spec_digest_test".to_string()),
        Some("sha256:dep_snapshot_test".to_string()),
    )
    .expect("Valid v2.2 WorkUnit");

    assert_eq!(wu.task, "T-381");
    assert_eq!(wu.base, "base_c100");
    assert_eq!(wu.commits.len(), 3);
    assert_eq!(wu.candidate, "sha_003_refactor");
    assert_eq!(wu.commit, "sha_003_refactor"); // 100% backward compatibility
    assert_eq!(wu.spec_digest.as_deref(), Some("sha256:spec_digest_test"));

    // Serialization round-trip
    let yaml = wu.to_yaml().expect("serialize to yaml");
    let recovered = WorkUnit::from_yaml(&yaml).expect("deserialize from yaml");
    assert_eq!(wu, recovered);

    // Validation rejects empty commit sequence
    assert!(WorkUnit::new_sequence("T-381", "base_c100", vec![], "cand", None, None).is_err());

    // Validation rejects candidate commit not in sequence
    assert!(WorkUnit::new_sequence(
        "T-381",
        "base_c100",
        commits,
        "non_existent_candidate",
        None,
        None
    )
    .is_err());

    // Legacy one-commit WorkUnit constructor maps losslessly to length-1 sequence
    let legacy = WorkUnit::new("T-100", "sha_single", "base_root").expect("Legacy WorkUnit");
    assert_eq!(legacy.commits.len(), 1);
    assert_eq!(legacy.commits[0].sha, "sha_single");
    assert_eq!(legacy.candidate, "sha_single");
}

#[test]
fn test_v22_proof_metadata_and_planes() {
    let proof = Proof::new_v22(
        "candidate_sha_xyz",
        ProofType::Work,
        Verdict::Pass,
        "cargo test -p vwl-core",
        Some("All 40 tests passed".to_string()),
        Some("G0-WIRING".to_string()),
        Some(VerificationPlane::Deterministic),
        Some(VisibilityProfile::InterfaceIntegration),
        Some("x86_64-macos-rust-1.100.0".to_string()),
    )
    .expect("Valid v2.2 Proof");

    assert_eq!(proof.subject, "candidate_sha_xyz");
    assert_eq!(proof.gate.as_deref(), Some("G0-WIRING"));
    assert_eq!(proof.plane, Some(VerificationPlane::Deterministic));
    assert_eq!(
        proof.visibility,
        Some(VisibilityProfile::InterfaceIntegration)
    );

    let yaml = proof.to_yaml().expect("serialize proof");
    let recovered = Proof::from_yaml(&yaml).expect("deserialize proof");
    assert_eq!(proof, recovered);
}

#[test]
fn test_task_dependency_and_cycle_detection() {
    let mut graph = TaskGraph::new();

    let t1 = Task::new("TASK-001", "Base Architecture", vec![], "c0").unwrap();
    let t2 = Task::new("TASK-002", "Core Engine", vec![], "c0").unwrap();
    let t3 = Task::new("TASK-003", "CLI Interface", vec![], "c0").unwrap();

    graph.add_task(t1);
    graph.add_task(t2);
    graph.add_task(t3);

    // TASK-002 requires TASK-001
    graph
        .add_dependency(TaskDependency::new(
            "TASK-001",
            "TASK-002",
            DependencyKind::Requires,
        ))
        .expect("Valid dep");

    // TASK-003 requires TASK-002
    graph
        .add_dependency(TaskDependency::new(
            "TASK-002",
            "TASK-003",
            DependencyKind::Requires,
        ))
        .expect("Valid dep");

    assert!(graph.validate_no_cycles().is_ok());

    // Introducing TASK-003 -> TASK-001 cycle must be rejected!
    let cycle_dep = TaskDependency::new("TASK-003", "TASK-001", DependencyKind::Requires);
    assert!(graph.add_dependency(cycle_dep).is_err());
}

#[test]
fn test_derived_ready_blocked_and_work_frontier() {
    let mut graph = TaskGraph::new();

    let t1 = Task::new("T1", "Spec", vec!["test1".to_string()], "c0").unwrap();
    let t2 = Task::new("T2", "Impl", vec!["test2".to_string()], "c0").unwrap();
    let t3 = Task::new("T3", "Verify", vec!["test3".to_string()], "c0").unwrap();

    graph.add_task(t1);
    graph.add_task(t2);
    graph.add_task(t3);

    graph
        .add_dependency(TaskDependency::new("T1", "T2", DependencyKind::Requires))
        .unwrap();

    graph
        .add_dependency(TaskDependency::new("T2", "T3", DependencyKind::Requires))
        .unwrap();

    let mut canonical_confirmed = HashSet::new();

    // Initially: T1 is Ready; T2 and T3 are Blocked
    assert!(graph.is_ready("T1", &canonical_confirmed));
    assert!(!graph.is_ready("T2", &canonical_confirmed));
    assert!(!graph.is_ready("T3", &canonical_confirmed));

    let frontier = graph.get_frontier(&canonical_confirmed);
    assert_eq!(frontier.len(), 1);
    assert_eq!(frontier[0].id, "T1");

    let why_t2 = graph.why_blocked("T2", &canonical_confirmed);
    assert_eq!(why_t2.len(), 1);
    assert!(why_t2[0].contains("Requires unconfirmed task 'T1'"));

    // T1 is confirmed: now T2 becomes Ready
    canonical_confirmed.insert("T1".to_string());
    assert!(!graph.is_ready("T1", &canonical_confirmed)); // Already completed/confirmed
    assert!(graph.is_ready("T2", &canonical_confirmed));
    assert!(!graph.is_ready("T3", &canonical_confirmed));

    let frontier = graph.get_frontier(&canonical_confirmed);
    assert_eq!(frontier.len(), 1);
    assert_eq!(frontier[0].id, "T2");

    // T2 is confirmed: now T3 becomes Ready
    canonical_confirmed.insert("T2".to_string());
    assert!(graph.is_ready("T3", &canonical_confirmed));
    let frontier = graph.get_frontier(&canonical_confirmed);
    assert_eq!(frontier.len(), 1);
    assert_eq!(frontier[0].id, "T3");
}

#[test]
fn test_claim_object_expiration_and_actor_contention() {
    let mut graph = TaskGraph::new();
    let t1 = Task::new("T-CLAIM", "Claim Objective", vec![], "c0").unwrap();
    graph.add_task(t1);

    // Initial claim by Agent-A with expiration at timestamp 1000
    let claim_a = Claim::new("T-CLAIM", "Agent-A", 100).with_expiration(1000);
    assert!(graph.claim_task(claim_a, 200).is_ok());

    // Contention: Agent-B tries to claim while Agent-A's claim is active
    let claim_b = Claim::new("T-CLAIM", "Agent-B", 300);
    let contention_res = graph.claim_task(claim_b.clone(), 500);
    assert!(contention_res.is_err());
    assert!(contention_res
        .unwrap_err()
        .to_string()
        .contains("already actively claimed by actor 'Agent-A'"));

    // After expiration (timestamp 1500), Agent-B's claim succeeds
    assert!(graph.claim_task(claim_b, 1500).is_ok());
    assert_eq!(graph.claims.get("T-CLAIM").unwrap().actor, "Agent-B");

    // Release claim by owner succeeds
    assert!(graph.release_claim("T-CLAIM", Some("Agent-B")).is_ok());
    assert!(!graph.claims.contains_key("T-CLAIM"));
}

#[test]
fn test_dependency_snapshot_and_stale_revalidation() {
    let mut graph = TaskGraph::new();
    let t1 = Task::new("T1", "Upstream", vec![], "c0").unwrap();
    let t2 = Task::new("T2", "Downstream", vec![], "c0").unwrap();
    graph.add_task(t1);
    graph.add_task(t2);

    graph
        .add_dependency(TaskDependency::new("T1", "T2", DependencyKind::Requires))
        .unwrap();

    let mut canonical_confirmed = HashSet::new();

    // Snapshot when T1 is unconfirmed
    let snapshot_before = graph.compute_dependency_snapshot("T2", &canonical_confirmed);
    assert!(snapshot_before.starts_with("sha256:"));

    // Candidate verified against unconfirmed T1 is not stale yet against same state
    assert!(!graph.is_candidate_stale("T2", &snapshot_before, &canonical_confirmed));

    // When canonical state advances (T1 confirmed)
    canonical_confirmed.insert("T1".to_string());
    assert!(graph.is_candidate_stale("T2", &snapshot_before, &canonical_confirmed));

    let snapshot_after = graph.compute_dependency_snapshot("T2", &canonical_confirmed);
    assert_ne!(snapshot_before, snapshot_after);
}

#[test]
fn test_affected_cone_spec_propagation() {
    let mut graph = TaskGraph::new();
    for id in ["T1", "T2", "T3", "T4", "T5"] {
        graph.add_task(Task::new(id, "Task", vec![], "c0").unwrap());
    }

    // T1 -> T2 -> T3 -> T4
    // T1 -> T5
    graph
        .add_dependency(TaskDependency::new("T1", "T2", DependencyKind::Requires))
        .unwrap();
    graph
        .add_dependency(TaskDependency::new("T2", "T3", DependencyKind::Requires))
        .unwrap();
    graph
        .add_dependency(TaskDependency::new("T3", "T4", DependencyKind::Requires))
        .unwrap();
    graph
        .add_dependency(TaskDependency::new("T1", "T5", DependencyKind::Requires))
        .unwrap();

    // Affected cone of T2: should be {T3, T4}
    let cone_t2 = graph.get_affected_cone("T2");
    assert_eq!(cone_t2.len(), 2);
    assert!(cone_t2.contains("T3"));
    assert!(cone_t2.contains("T4"));
    assert!(!cone_t2.contains("T5"));

    // Affected cone of T1: should be {T2, T3, T4, T5}
    let cone_t1 = graph.get_affected_cone("T1");
    assert_eq!(cone_t1.len(), 4);
    assert!(cone_t1.contains("T2"));
    assert!(cone_t1.contains("T3"));
    assert!(cone_t1.contains("T4"));
    assert!(cone_t1.contains("T5"));
}

#[test]
fn test_task_cancellation_and_unsatisfiable_detection() {
    let mut graph = TaskGraph::new();
    let t1 = Task::new("T1", "Task 1", vec![], "c0").unwrap();
    let t2 = Task::new("T2", "Task 2", vec![], "c0").unwrap();
    graph.add_task(t1);
    graph.add_task(t2);

    graph
        .add_dependency(TaskDependency::new("T1", "T2", DependencyKind::Requires))
        .unwrap();

    let confirmed = HashSet::new();

    // Cancel T1
    graph.cancel_task("T1");
    assert!(graph.is_cancelled("T1"));
    assert!(!graph.is_ready("T1", &confirmed));

    // T2 requires cancelled T1 -> T2 is UNSATISFIABLE
    let unsatisfiable = graph.check_unsatisfiable(&confirmed);
    assert_eq!(unsatisfiable.len(), 1);
    assert_eq!(unsatisfiable[0].0, "T2");
    assert!(unsatisfiable[0].1.contains("was cancelled"));
}

#[test]
fn test_ledger_graph_persistence_and_deterministic_replay() {
    let temp_dir = tempdir().unwrap();
    let repo_root = temp_dir.path();

    // Initialize mock git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_root)
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.name", "VWL Tester"])
        .current_dir(repo_root)
        .output()
        .expect("git config name");
    std::process::Command::new("git")
        .args(["config", "user.email", "tester@vwl.network"])
        .current_dir(repo_root)
        .output()
        .expect("git config email");

    std::fs::write(repo_root.join("README.md"), "# VWL Test Repo\n").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_root)
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_root)
        .output()
        .expect("git commit");

    let ledger = LocalLedger::new(repo_root).unwrap();

    // Add tasks
    let t1 = Task::new("TASK-A", "Build Ledger", vec![], "c0").unwrap();
    let t2 = Task::new("TASK-B", "Build Frontier", vec![], "c0").unwrap();
    ledger.append_task(&t1).unwrap();
    ledger.append_task(&t2).unwrap();

    // Add dependency TASK-B requires TASK-A
    let dep = TaskDependency::new("TASK-A", "TASK-B", DependencyKind::Requires);
    ledger.append_dependency(&dep).unwrap();

    // Add claim on TASK-A
    let claim = Claim::new("TASK-A", "agent-node-1", 100);
    ledger.append_claim(&claim).unwrap();

    // Replay initial state: TASK-A should be on the Work Frontier
    let state1 = ReplayEngine::replay(repo_root).unwrap();
    assert_eq!(state1.total_tasks, 2);
    assert_eq!(state1.frontier_tasks, vec!["TASK-A".to_string()]);
    assert_eq!(state1.active_claims, vec!["TASK-A".to_string()]);
    assert_eq!(state1.confirmed_work_units.len(), 0);

    // Confirm WorkUnit for TASK-A
    let wu_a = WorkUnit::new("TASK-A", "sha_wu_a", "c0").unwrap();
    ledger.append_work_unit(&wu_a).unwrap();
    let dec_a = Decision {
        subject: "sha_wu_a".to_string(),
        outcome: DecisionOutcome::Confirmed,
        reason: "All proofs passed".to_string(),
    };
    ledger.append_decision(&dec_a).unwrap();

    // Replay after TASK-A confirmed: TASK-B should now deterministically emerge on the Work Frontier!
    let state2 = ReplayEngine::replay(repo_root).unwrap();
    assert_eq!(state2.confirmed_work_units, vec!["sha_wu_a".to_string()]);
    assert_eq!(state2.frontier_tasks, vec!["TASK-B".to_string()]);

    // Multiple replays must produce bit-for-bit identical state (Deterministic Replay Invariant)
    let state3 = ReplayEngine::replay(repo_root).unwrap();
    assert_eq!(state2, state3);
}

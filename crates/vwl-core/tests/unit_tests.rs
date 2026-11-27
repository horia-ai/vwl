use tempfile::tempdir;
use vwl_core::{
    decision::{DecisionEngine, DecisionOutcome},
    ledger::LocalLedger,
    proof::{Proof, ProofType, Verdict},
    task::{Task, TaskSkeptic},
    work_unit::WorkUnit,
};

#[test]
fn test_task_dto_roundtrip_and_digest() {
    let task = Task::new(
        "T-001",
        "Implement minimal CLI shell",
        vec![
            "cargo test passes".to_string(),
            "cargo clippy passes".to_string(),
        ],
        "c0_base_commit",
    )
    .expect("Task must be valid");

    // Deterministic digest
    let digest1 = task.compute_digest();
    let digest2 = task.compute_digest();
    assert_eq!(digest1, digest2);
    assert_eq!(digest1.len(), 64);

    // YAML round-trip
    let yaml = task.to_yaml().expect("YAML serialize");
    let recovered = Task::from_yaml(&yaml).expect("YAML deserialize");
    assert_eq!(task, recovered);
    assert_eq!(recovered.compute_digest(), digest1);
}

#[test]
fn test_task_validation_rejects_empty_fields() {
    assert!(Task::new("", "obj", vec![], "base").is_err());
    assert!(Task::new("T-01", "", vec![], "base").is_err());
    assert!(Task::new("T-01", "obj", vec![], "").is_err());
}

#[test]
fn test_work_unit_dto_validation() {
    let wu = WorkUnit::new("T-001", "commit_abc123", "base_c0").expect("Valid WorkUnit");
    assert_eq!(wu.task, "T-001");
    assert_eq!(wu.commit, "commit_abc123");
    assert_eq!(wu.base, "base_c0");

    assert!(WorkUnit::new("", "commit", "base").is_err());
    assert!(WorkUnit::new("T-01", "", "base").is_err());
    assert!(WorkUnit::new("T-01", "commit", "").is_err());
}

#[test]
fn test_proof_dto_and_digest() {
    let proof = Proof::new(
        "commit_abc123",
        ProofType::Work,
        Verdict::Pass,
        "cargo test",
        Some("All tests passed".to_string()),
    )
    .expect("Valid proof");

    let d1 = proof.compute_digest();
    let d2 = proof.compute_digest();
    assert_eq!(d1, d2);
    assert_eq!(d1.len(), 64);
}

#[test]
fn test_decision_engine_truth_table() {
    let commit = "candidate_sha";
    let required = vec![ProofType::Work, ProofType::Structural];

    // Case 1: No proofs -> Pending
    let d1 = DecisionEngine::evaluate(commit, &[], &required);
    assert_eq!(d1.outcome, DecisionOutcome::Pending);

    // Case 2: Only 1 of 2 required proofs passed -> Pending
    let p_work_pass =
        Proof::new(commit, ProofType::Work, Verdict::Pass, "cargo test", None).unwrap();
    let d2 = DecisionEngine::evaluate(commit, std::slice::from_ref(&p_work_pass), &required);
    assert_eq!(d2.outcome, DecisionOutcome::Pending);

    // Case 3: Both required proofs passed -> Confirmed
    let p_struct_pass = Proof::new(
        commit,
        ProofType::Structural,
        Verdict::Pass,
        "cargo clippy",
        None,
    )
    .unwrap();
    let d3 = DecisionEngine::evaluate(
        commit,
        &[p_work_pass.clone(), p_struct_pass.clone()],
        &required,
    );
    assert_eq!(d3.outcome, DecisionOutcome::Confirmed);

    // Case 4: Any required proof fails -> Rejected
    let p_struct_fail = Proof::new(
        commit,
        ProofType::Structural,
        Verdict::Fail,
        "cargo clippy",
        None,
    )
    .unwrap();
    let d4 = DecisionEngine::evaluate(commit, &[p_work_pass.clone(), p_struct_fail], &required);
    assert_eq!(d4.outcome, DecisionOutcome::Rejected);
}

#[test]
fn test_local_ledger_append_and_idempotency() {
    let dir = tempdir().expect("tempdir");
    let ledger = LocalLedger::new(dir.path()).expect("LocalLedger");

    let task = Task::new("T-001", "Objective", vec!["test".to_string()], "base_sha").unwrap();

    // First append succeeds
    ledger.append_task(&task).expect("append task");

    // Idempotent duplicate append succeeds
    ledger.append_task(&task).expect("idempotent append");

    // Task is retrieved
    let tasks = ledger.get_tasks().expect("get tasks");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0], task);

    // Conflicting task with same digest cannot happen because digest is content-addressed,
    // but duplicate WorkUnit with different content is rejected:
    let wu1 = WorkUnit::new("T-001", "commit_1", "base_1").unwrap();
    let wu2 = WorkUnit::new("T-001", "commit_1", "base_diff").unwrap();
    ledger.append_work_unit(&wu1).expect("append wu1");
    assert!(ledger.append_work_unit(&wu2).is_err());
}

#[test]
fn test_task_skeptic_review_pass() {
    let task = Task::new(
        "T-SPEC-001",
        "Implement deterministic JSON serialization for state ledger",
        vec![
            "cargo test --test ledger_tests".to_string(),
            "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
        ],
        "c0_canonical_base_sha",
    )
    .unwrap();

    let verdict = TaskSkeptic::review(&task);
    assert!(verdict.passed);
    assert_eq!(verdict.task_id, "T-SPEC-001");
    assert!(verdict.objections.is_empty());
}

#[test]
fn test_task_skeptic_review_subjective_criteria_fail() {
    let task = Task::new(
        "T-SPEC-002",
        "Refactor auth logic",
        vec![
            "clean code that feels good".to_string(),
            "well tested and fast enough".to_string(),
        ],
        "c0_canonical_base_sha",
    )
    .unwrap();

    let verdict = TaskSkeptic::review(&task);
    assert!(!verdict.passed);
    assert!(verdict.objections.len() >= 3);
    assert!(verdict.objections.iter().any(|o| o.contains("clean code")));
    assert!(verdict.objections.iter().any(|o| o.contains("well tested")));
    assert!(verdict.objections.iter().any(|o| o.contains("fast enough")));
}

#[test]
fn test_task_skeptic_review_conversational_cue_fail() {
    let task = Task::new(
        "T-SPEC-003",
        "Implement the retry policy as discussed in our previous chat",
        vec!["cargo test -p vwl-core".to_string()],
        "c0_canonical_base_sha",
    )
    .unwrap();

    let verdict = TaskSkeptic::review(&task);
    assert!(!verdict.passed);
    assert!(verdict
        .objections
        .iter()
        .any(|o| o.contains("as discussed") || o.contains("previous chat")));
}

#[test]
fn test_task_skeptic_review_missing_criteria_and_base_fail() {
    let task = Task {
        id: "T-SPEC-004".to_string(),
        objective: "Do some work".to_string(),
        acceptance: vec![],
        base: "".to_string(),
    };

    let verdict = TaskSkeptic::review(&task);
    assert!(!verdict.passed);
    assert!(verdict
        .objections
        .iter()
        .any(|o| o.contains("No acceptance criteria")));
    assert!(verdict
        .objections
        .iter()
        .any(|o| o.contains("Invalid or missing canonical base commit")));
}

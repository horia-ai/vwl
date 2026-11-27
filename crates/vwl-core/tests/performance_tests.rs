use std::collections::HashSet;
use std::time::Instant;
use tempfile::tempdir;
use vwl_core::decision::{Decision, DecisionOutcome};
use vwl_core::graph::{DependencyKind, TaskDependency, TaskGraph};
use vwl_core::ledger::LocalLedger;
use vwl_core::proof::{Proof, ProofType, Verdict};
use vwl_core::replay::ReplayEngine;
use vwl_core::repometer::{Outcome, OutcomeSeverity, OutcomeType, RepometerEngine};
use vwl_core::task::Task;
use vwl_core::work_unit::WorkUnit;

#[test]
fn test_dense_task_graph_frontier_derivation_latency() {
    let mut graph = TaskGraph::new();
    let num_tasks = 1000;

    // Build a wide and deep DAG
    for i in 0..num_tasks {
        let task = Task::new(
            format!("T-PERF-{:04}", i),
            format!("Objective for task {}", i),
            vec!["unit tests pass".to_string()],
            "HEAD",
        )
        .unwrap();
        graph.add_task(task);

        if i > 0 && i % 5 == 0 {
            // Add dependencies
            let dep = TaskDependency::new(
                format!("T-PERF-{:04}", i),
                format!("T-PERF-{:04}", i - 1),
                DependencyKind::Requires,
            );
            graph.add_dependency(dep).unwrap();
        }
    }

    let mut confirmed = HashSet::new();
    for i in 0..(num_tasks / 2) {
        confirmed.insert(format!("T-PERF-{:04}", i));
    }

    let start = Instant::now();
    let frontier = graph.get_frontier(&confirmed);
    let elapsed = start.elapsed();

    println!(
        "Dense DAG (1,000 tasks) frontier derivation completed in {:?}. Frontier size: {}",
        elapsed,
        frontier.len()
    );

    // Enforce performance gate: frontier calculation must take < 250ms
    assert!(
        elapsed.as_millis() < 250,
        "Performance gate violated: frontier derivation took {:?}",
        elapsed
    );
    assert!(!frontier.is_empty());
}

#[test]
fn test_replay_throughput_performance_gate() {
    let temp = tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let ledger = LocalLedger::new(temp.path()).unwrap();
    let num_records = 100;

    for i in 0..num_records {
        let task_id = format!("T-THROUGHPUT-{:03}", i);
        let commit_sha = format!("sha_cand_{:03}", i);
        let task = Task::new(
            &task_id,
            "Throughput test task",
            vec!["cargo check".to_string()],
            "HEAD",
        )
        .unwrap();
        ledger.append_task(&task).unwrap();

        let wu = WorkUnit::new(task_id, commit_sha.clone(), "HEAD").unwrap();
        ledger.append_work_unit(&wu).unwrap();

        let proof = Proof::new(
            &commit_sha,
            ProofType::Work,
            Verdict::Pass,
            "cargo test",
            None,
        )
        .unwrap();
        ledger.append_proof(&proof).unwrap();

        let dec = Decision {
            subject: commit_sha,
            outcome: DecisionOutcome::Confirmed,
            reason: "All proofs passed".to_string(),
        };
        ledger.append_decision(&dec).unwrap();
    }

    let start = Instant::now();
    let replay_state = ReplayEngine::replay(temp.path()).unwrap();
    let elapsed = start.elapsed();

    println!(
        "Replay of {} ledger records completed in {:?}. Total tasks: {}",
        num_records * 4,
        elapsed,
        replay_state.total_tasks
    );

    // Enforce performance gate: Replay of 400 objects (< 5000ms in debug, < 500ms in release)
    let threshold_ms = if cfg!(debug_assertions) { 5000 } else { 500 };
    assert!(
        elapsed.as_millis() < threshold_ms,
        "Performance gate violated: replay took {:?} (threshold: {}ms)",
        elapsed,
        threshold_ms
    );
    assert_eq!(replay_state.total_tasks, num_records);
    assert_eq!(replay_state.confirmed_work_units.len(), num_records);
}

#[test]
fn test_repometer_calculation_latency_and_benchmark_outcome() {
    let temp = tempdir().unwrap();
    std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let ledger = LocalLedger::new(temp.path()).unwrap();

    let start = Instant::now();
    let metrics = RepometerEngine::calculate(temp.path(), None).unwrap();
    let elapsed = start.elapsed();

    println!(
        "Repometer evidence calculation completed in {:?}. Methodology: {}",
        elapsed, metrics.methodology
    );

    // Enforce performance gate: Repometer calculation on clean repo (< 500ms in debug, < 150ms in release)
    let threshold_ms = if cfg!(debug_assertions) { 500 } else { 150 };
    assert!(
        elapsed.as_millis() < threshold_ms,
        "Performance gate violated: repometer took {:?} (threshold: {}ms)",
        elapsed,
        threshold_ms
    );

    // Record formal Benchmark Outcome
    let benchmark_outcome = Outcome::new(
        "O-BENCH-PERF-01",
        "vwl-core::RepometerEngine",
        OutcomeType::Benchmark,
        OutcomeSeverity::Low,
        format!(
            "Latency: {}ms, Methodology: {}",
            elapsed.as_millis(),
            metrics.methodology
        ),
        1000,
    )
    .unwrap();

    ledger.append_outcome(&benchmark_outcome).unwrap();
    let loaded = ledger.get_outcomes().unwrap();
    assert!(loaded.iter().any(|o| o.id == "O-BENCH-PERF-01"));
}

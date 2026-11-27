use std::collections::HashSet;
use vwl_core::graph::{Claim, DependencyKind, TaskDependency, TaskGraph};
use vwl_core::supervisor::{Dispatcher, SupervisorRole, Triage, WavePlan};
use vwl_core::task::Task;

fn create_sample_task(id: &str, title: &str) -> Task {
    Task::new(
        id,
        title,
        vec!["gate-precommit passes".to_string()],
        "0000000000000000000000000000000000000000",
    )
    .unwrap()
}

#[test]
fn test_supervisor_roles_metadata() {
    assert_eq!(SupervisorRole::WavePlanner.as_str(), "wave_planner");
    assert_eq!(SupervisorRole::Dispatcher.as_str(), "dispatcher");
    assert_eq!(SupervisorRole::Triage.as_str(), "triage");
    assert_eq!(SupervisorRole::Validator.as_str(), "validator");
    assert_eq!(SupervisorRole::Ingest.as_str(), "ingest");
    assert_eq!(SupervisorRole::SelfHealer.as_str(), "self_healer");

    assert!(!SupervisorRole::WavePlanner.description().is_empty());
    assert!(!SupervisorRole::Dispatcher.description().is_empty());
    assert_eq!(SupervisorRole::all().len(), 6);
}

#[test]
fn test_wave_planner_topological_waves() {
    let mut graph = TaskGraph::new();
    let t1 = create_sample_task("TASK-1", "Base Setup");
    let t2 = create_sample_task("TASK-2", "Core Engine");
    let t3 = create_sample_task("TASK-3", "CLI Layer");
    let t4 = create_sample_task("TASK-4", "Unreachable");

    graph.add_task(t1);
    graph.add_task(t2);
    graph.add_task(t3);
    graph.add_task(t4);

    // TASK-2 requires TASK-1
    graph
        .add_dependency(TaskDependency::new(
            "TASK-1",
            "TASK-2",
            DependencyKind::Requires,
        ))
        .unwrap();
    // TASK-3 requires TASK-2
    graph
        .add_dependency(TaskDependency::new(
            "TASK-2",
            "TASK-3",
            DependencyKind::Requires,
        ))
        .unwrap();
    // TASK-4 requires non-existent TASK-99
    graph
        .add_dependency(TaskDependency::new(
            "TASK-99",
            "TASK-4",
            DependencyKind::Requires,
        ))
        .unwrap();

    let confirmed = HashSet::new();
    let plan = WavePlan::compute(&graph, &confirmed);

    // Wave 0 should be TASK-1 (no unmet dependencies)
    assert_eq!(plan.waves.len(), 3);
    assert_eq!(plan.waves[0], vec!["TASK-1"]);
    // Wave 1 should be TASK-2 (depends on Wave 0)
    assert_eq!(plan.waves[1], vec!["TASK-2"]);
    // Wave 2 should be TASK-3 (depends on Wave 1)
    assert_eq!(plan.waves[2], vec!["TASK-3"]);

    // TASK-4 is blocked due to missing dependency TASK-99
    assert!(plan.blocked_tasks.contains_key("TASK-4"));
}

#[test]
fn test_dispatcher_frontier_dispatch() {
    let mut graph = TaskGraph::new();
    let t1 = create_sample_task("TASK-1", "Ready Task Unclaimed");
    let t2 = create_sample_task("TASK-2", "Ready Task Claimed");
    let t3 = create_sample_task("TASK-3", "Blocked Task");

    graph.add_task(t1);
    graph.add_task(t2);
    graph.add_task(t3);

    graph
        .add_dependency(TaskDependency::new(
            "TASK-1",
            "TASK-3",
            DependencyKind::Requires,
        ))
        .unwrap();

    let current_time = 1500;
    // t2 is claimed actively
    let claim = Claim::new("TASK-2", "worker-alice", 1000).with_expiration(2000);
    graph.claim_task(claim, current_time).unwrap();

    let confirmed = HashSet::new();
    let report = Dispatcher::evaluate(&graph, &confirmed, current_time);

    assert_eq!(report.ready_tasks_count, 2); // TASK-1 and TASK-2
    assert_eq!(report.active_claims_count, 1); // TASK-2 is claimed
    assert_eq!(report.dispatches.len(), 1); // Only TASK-1 needs dispatch
    assert_eq!(report.dispatches[0].task_id, "TASK-1");
    assert_eq!(report.dispatches[0].recommended_command, "vwl work TASK-1");
}

#[test]
fn test_triage_audit_expired_claims_and_blockers() {
    let mut graph = TaskGraph::new();
    let t1 = create_sample_task("TASK-1", "Expired Claim Task");
    let t2 = create_sample_task("TASK-2", "Cycle Blocked Task");
    graph.add_task(t1);
    graph.add_task(t2);

    // Expired claim on TASK-1
    let claim = Claim::new("TASK-1", "worker-bob", 500).with_expiration(1000);
    graph.claim_task(claim, 600).unwrap();

    // TASK-2 requires TASK-99 which is missing
    graph
        .add_dependency(TaskDependency::new(
            "TASK-99",
            "TASK-2",
            DependencyKind::Requires,
        ))
        .unwrap();

    let current_time = 1500; // Greater than expiration 1000
    let confirmed = HashSet::new();
    let triage = Triage::evaluate(&graph, &confirmed, current_time);

    assert!(triage.expired_claims.contains(&"TASK-1".to_string()));
    assert!(!triage.findings.is_empty());
}

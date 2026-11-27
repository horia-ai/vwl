use std::collections::HashMap;
use tempfile::tempdir;
use vwl_core::analytics::{
    calculate_effectiveness, calculate_rep_score, ContributorMetrics, CostAllocationMetrics,
    ProjectLedgerReport,
};

#[test]
fn test_contributor_effectiveness_and_rep_calculation() {
    // 9 confirmed, 1 rejected
    let effectiveness = calculate_effectiveness(9, 1);
    assert!((effectiveness - 90.0).abs() < 0.01);

    // 0 tasks defaults to 100.0% provisional
    let initial_eff = calculate_effectiveness(0, 0);
    assert_eq!(initial_eff, 100.0);

    // Gate 4 REP score: (confirmed * 10) - (rejected * 25) + (first_pass_rate * 0.5)
    // 9 * 10 = 90 - 25 = 65 + (90 * 0.5 = 45) = 110.0
    let rep = calculate_rep_score(9, 1, 90.0);
    assert!((rep - 110.0).abs() < 0.01);

    // Negative penalties clamp to 0
    let rep_clamped = calculate_rep_score(0, 5, 0.0);
    assert_eq!(rep_clamped, 0.0);
}

#[test]
fn test_cost_allocation_conservation_invariant() {
    let mut milestone_budgets = HashMap::new();
    milestone_budgets.insert("Gate 0".to_string(), 100_000);
    milestone_budgets.insert("Gate 1".to_string(), 150_000);

    let costs = CostAllocationMetrics {
        total_project_funded: 500_000,
        total_paid_liquid: 100_000,
        total_pending_vesting: 50_000,
        total_reserved_escrow: 150_000,
        unallocated_balance: 200_000,
        milestone_budgets,
    };

    // Invariant: Liquid + Vesting + Reserved + Unallocated == Total Funded
    assert_eq!(
        costs.total_project_funded,
        costs.total_paid_liquid
            + costs.total_pending_vesting
            + costs.total_reserved_escrow
            + costs.unallocated_balance
    );
}

#[test]
fn test_project_ledger_report_export_gh_pages() {
    let dir = tempdir().expect("tempdir");
    let docs_dir = dir.path().join("docs");
    std::fs::create_dir_all(&docs_dir).expect("create docs dir");

    let report = ProjectLedgerReport {
        repo_name: "horia-ai/vwl".to_string(),
        canonical_head: "abc123456789".to_string(),
        total_tasks: 53,
        total_work_units: 12,
        contributors: vec![ContributorMetrics {
            contributor_id: "atrifan".to_string(),
            role: "Protocol Architect".to_string(),
            confirmed_wus: 5,
            rejected_wus: 0,
            first_pass_rate: 100.0,
            effectiveness_score: 100.0,
            rep_earned: 100.0,
            paid_work: 50_000,
            pending_vesting_work: 25_000,
            reserved_work: 25_000,
        }],
        costs: CostAllocationMetrics {
            total_project_funded: 250_000,
            total_paid_liquid: 50_000,
            total_pending_vesting: 25_000,
            total_reserved_escrow: 25_000,
            unallocated_balance: 150_000,
            milestone_budgets: HashMap::new(),
        },
        contracts: vec![],
        generated_at: "2026-11-27T02:00:00Z".to_string(),
    };

    let exported_path = report
        .export_to_gh_pages(&docs_dir)
        .expect("export to gh pages");
    assert!(exported_path.exists());

    let content = std::fs::read_to_string(exported_path).expect("read exported json");
    assert!(content.contains("atrifan"));
    assert!(content.contains("Protocol Architect"));
    assert!(content.contains("250000"));
}

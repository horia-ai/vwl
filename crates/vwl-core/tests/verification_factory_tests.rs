use vwl_core::{
    factory::{
        Challenge, ChangeProfile, GateDefinition, GatesManifest, RiskLevel, RiskProfile,
        ShadowTrafficContract, TestIntegrityGate, VerificationPlan, VerifierArtifact, WiringGate,
    },
    proof::{Proof, ProofType, Verdict, VerificationPlane},
};

#[test]
fn test_change_and_risk_profile_derivation() {
    let change = ChangeProfile::new(
        vec!["rust", "solidity"],
        vec!["backend", "api"],
        vec!["persistence-change"],
    );
    let risk = RiskProfile::new(RiskLevel::High, RiskLevel::Critical, RiskLevel::Low);

    assert!(risk.is_high_risk());
    assert_eq!(risk.security, RiskLevel::Critical);

    let plan = VerificationPlan::derive("WU-CAND-001", &change, &risk);

    // Plan must include baseline
    assert!(plan.required_gates.contains(&"compile".to_string()));
    assert!(plan.required_gates.contains(&"lint".to_string()));
    assert!(plan.required_gates.contains(&"unit".to_string()));

    // Persistence change & high data integrity require wiring, test-integrity, mutation
    assert!(plan.required_gates.contains(&"wiring".to_string()));
    assert!(plan.required_gates.contains(&"test-integrity".to_string()));
    assert!(plan
        .required_gates
        .contains(&"mutation-targeted".to_string()));

    // API & high security require adversarial-risk & integration-blackbox
    assert!(plan
        .required_gates
        .contains(&"adversarial-risk".to_string()));
    assert!(plan
        .required_gates
        .contains(&"integration-blackbox".to_string()));

    // Critical security adds conditional deployment-shadow
    assert!(plan
        .conditional_gates
        .contains(&"deployment-shadow".to_string()));
}

#[test]
fn test_verifier_artifact_candidate_isolation_invariant() {
    // Valid verifier artifact: commit is distinct from subject candidate
    let valid_artifact = VerifierArtifact::new(
        "VA-001",
        "cand_sha_123",
        "verifier-agent",
        "verifier_commit_456",
        "reproducer",
        "fn test_edge_case_failure() { assert!(false); }",
    );
    assert!(valid_artifact.is_ok());

    // Invariant: Verifier artifact commit MUST NOT equal subject candidate!
    // "Judge cannot silently rewrite defendant"
    let invalid_mutation = VerifierArtifact::new(
        "VA-002",
        "cand_sha_123",
        "malicious-verifier",
        "cand_sha_123", // Attempting to mutate candidate directly
        "reproducer",
        "malicious edit",
    );
    assert!(invalid_mutation.is_err());
    assert!(invalid_mutation
        .unwrap_err()
        .to_string()
        .contains("Verifier cannot mutate candidate directly"));
}

#[test]
fn test_challenge_and_reproducer_flow() {
    let challenge = Challenge::new(
        "CH-001",
        "cand_sha_failed",
        "verifier-agent-1",
        "unit-test",
        Some("VA-001".to_string()),
        "Assertion failed on boundary input 0",
    );

    assert_eq!(challenge.subject, "cand_sha_failed");
    assert_eq!(challenge.failing_gate, "unit-test");
    assert_eq!(challenge.artifact_id.as_deref(), Some("VA-001"));
}

#[test]
fn test_wiring_proof_gate() {
    let source = "pub struct OrderRouter;\nimpl OrderRouter { pub fn route() {} }";
    let manifest_wired = "register_module(\"OrderRouter\");";
    let manifest_unwired = "register_module(\"OtherModule\");";

    // When component is defined and wired in manifest -> PASS
    let proof_pass = WiringGate::verify_wiring("cand_1", source, manifest_wired, "OrderRouter");
    assert_eq!(proof_pass.verdict, Verdict::Pass);
    assert_eq!(proof_pass.gate.as_deref(), Some("wiring"));

    // When component is defined but NOT wired in manifest -> FAIL (dead/unwired code detected)
    let proof_fail = WiringGate::verify_wiring("cand_1", source, manifest_unwired, "OrderRouter");
    assert_eq!(proof_fail.verdict, Verdict::Fail);
    assert!(proof_fail
        .evidence
        .output
        .unwrap()
        .contains("Wiring verification failed"));
}

#[test]
fn test_test_integrity_gate_detects_relaxed_assertions() {
    let clean_diff = "
+ #[test]
+ fn test_new_case() {
+     assert_eq!(add(2, 2), 4);
+ }
";
    let proof_clean = TestIntegrityGate::verify_test_integrity("cand_clean", clean_diff);
    assert_eq!(proof_clean.verdict, Verdict::Pass);

    let relaxed_diff = "
- assert_eq!(balance, 1000);
- fn test_security_bounds() {
+ #[ignore]
+ fn test_security_bounds() {
";
    let proof_relaxed = TestIntegrityGate::verify_test_integrity("cand_relaxed", relaxed_diff);
    assert_eq!(proof_relaxed.verdict, Verdict::Fail);
    let output = proof_relaxed.evidence.output.unwrap();
    assert!(output.contains("Removed or weakened assertion"));
    assert!(output.contains("Deleted test case"));
    assert!(output.contains("Added ignore annotation"));
}

#[test]
fn test_shadow_traffic_contract_enforces_read_only() {
    let valid_contract = ShadowTrafficContract::default();
    assert!(valid_contract.validate().is_ok());

    let violating_contract = ShadowTrafficContract {
        is_read_only: false,
        ..Default::default()
    };
    assert!(violating_contract.validate().is_err());

    let side_effect_contract = ShadowTrafficContract {
        suppress_side_effects: false,
        ..Default::default()
    };
    assert!(side_effect_contract.validate().is_err());
}

#[test]
fn test_verification_plan_enforces_required_gates() {
    let change = ChangeProfile::new(vec!["rust"], vec!["backend"], vec!["persistence-change"]);
    let risk = RiskProfile::new(RiskLevel::High, RiskLevel::Low, RiskLevel::Low);
    let plan = VerificationPlan::derive("cand_target", &change, &risk);

    let passing_proofs = vec![
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "compile",
            None,
            Some("compile".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "lint",
            None,
            Some("lint".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "unit",
            None,
            Some("unit".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "wiring",
            None,
            Some("wiring".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "test-integrity",
            None,
            Some("test-integrity".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
        Proof::new_v22(
            "cand_target",
            ProofType::Work,
            Verdict::Pass,
            "mutation-targeted",
            None,
            Some("mutation-targeted".to_string()),
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
        .unwrap(),
    ];

    assert!(plan.validate_proofs(&passing_proofs).is_ok());

    // Omitting a required gate (e.g. mutation-targeted) must fail validation!
    let incomplete_proofs = passing_proofs[0..5].to_vec();
    let err = plan.validate_proofs(&incomplete_proofs);
    assert!(err.is_err());
    assert!(err
        .unwrap_err()
        .to_string()
        .contains("Required gate 'mutation-targeted' has not passed"));
}

#[test]
fn test_gates_manifest_detect_rust_and_node() {
    let temp = tempfile::tempdir().unwrap();

    // Create Cargo.toml and package.json
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(temp.path().join("tsconfig.json"), "{}\n").unwrap();
    let pkg_json = r#"{
  "name": "demo-node",
  "scripts": {
    "test": "vitest",
    "lint": "eslint",
    "test:mutation": "stryker run"
  }
}"#;
    std::fs::write(temp.path().join("package.json"), pkg_json).unwrap();

    let manifest = GatesManifest::detect(temp.path());
    assert!(manifest.languages.contains(&"rust".to_string()));
    assert!(manifest.languages.contains(&"typescript".to_string()));

    // Verify gates
    let gate_names: Vec<String> = manifest.gates.iter().map(|g| g.name.clone()).collect();
    assert!(gate_names.contains(&"unit-tests".to_string()));
    assert!(gate_names.contains(&"clippy-lint".to_string()));
    assert!(gate_names.contains(&"format-check".to_string()));
    assert!(gate_names.contains(&"node-tests".to_string()));
    assert!(gate_names.contains(&"node-lint".to_string()));
    assert!(gate_names.contains(&"typecheck".to_string()));
    assert!(gate_names.contains(&"mutation-gate".to_string()));
    assert!(gate_names.contains(&"skeptic-review".to_string()));

    // Mutation gate should NOT be on precommit by default
    let mutation_gate = manifest
        .gates
        .iter()
        .find(|g| g.name == "mutation-gate")
        .unwrap();
    assert!(!mutation_gate.precommit);

    // Save and load round-trip
    let saved_path = manifest.save(temp.path()).unwrap();
    assert!(saved_path.exists());

    let loaded = GatesManifest::load_or_detect(temp.path()).unwrap();
    assert_eq!(manifest, loaded);
}

#[test]
fn test_gates_manifest_run_gates_execution() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = GatesManifest {
        version: "1.0".to_string(),
        project: "test-run".to_string(),
        languages: vec!["custom".to_string()],
        gates: vec![
            GateDefinition {
                name: "echo-pass".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "echo 'hello passed'".to_string(),
                precommit: true,
                timeout_secs: 10,
                description: "Passing command".to_string(),
            },
            GateDefinition {
                name: "fail-gate".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "sh -c 'exit 42'".to_string(),
                precommit: false,
                timeout_secs: 10,
                description: "Failing non-precommit command".to_string(),
            },
        ],
    };

    // Run only precommit -> only echo-pass runs
    let precommit_results = manifest.run_gates(temp.path(), true).unwrap();
    assert_eq!(precommit_results.len(), 1);
    assert!(precommit_results[0].passed);
    assert_eq!(precommit_results[0].name, "echo-pass");
    assert!(precommit_results[0].stdout.contains("hello passed"));

    // Run all gates -> echo-pass passes, fail-gate fails
    let all_results = manifest.run_gates(temp.path(), false).unwrap();
    assert_eq!(all_results.len(), 2);
    assert!(all_results[0].passed);
    assert!(!all_results[1].passed);
    assert_eq!(all_results[1].name, "fail-gate");
}

use crate::error::{Result, VwlError};
use crate::proof::{Proof, ProofType, Verdict, VerificationPlane, VisibilityProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeProfile {
    pub languages: Vec<String>,
    pub surfaces: Vec<String>,
    pub effects: Vec<String>,
}

impl ChangeProfile {
    pub fn new(languages: Vec<&str>, surfaces: Vec<&str>, effects: Vec<&str>) -> Self {
        Self {
            languages: languages.into_iter().map(String::from).collect(),
            surfaces: surfaces.into_iter().map(String::from).collect(),
            effects: effects.into_iter().map(String::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RiskProfile {
    pub data_integrity: RiskLevel,
    pub security: RiskLevel,
    pub user_facing: RiskLevel,
}

impl RiskProfile {
    pub fn new(data_integrity: RiskLevel, security: RiskLevel, user_facing: RiskLevel) -> Self {
        Self {
            data_integrity,
            security,
            user_facing,
        }
    }

    pub fn is_high_risk(&self) -> bool {
        self.data_integrity >= RiskLevel::High
            || self.security >= RiskLevel::High
            || self.user_facing >= RiskLevel::High
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationPlan {
    pub subject: String,
    pub required_gates: Vec<String>,
    pub conditional_gates: Vec<String>,
    pub visibility: VisibilityProfile,
}

impl VerificationPlan {
    pub fn derive(subject: impl Into<String>, change: &ChangeProfile, risk: &RiskProfile) -> Self {
        let subject = subject.into();
        let mut required = vec![
            "compile".to_string(),
            "lint".to_string(),
            "unit".to_string(),
        ];
        let mut conditional = Vec::new();

        // Baseline visibility
        let mut visibility = VisibilityProfile::InterfaceIntegration;

        // Backend / Data Integrity rules
        if change.effects.iter().any(|e| e == "persistence-change")
            || risk.data_integrity >= RiskLevel::High
        {
            required.push("test-integrity".to_string());
            required.push("wiring".to_string());
            required.push("mutation-targeted".to_string());
        }

        // Security / API rules
        if change.surfaces.iter().any(|s| s == "api") || risk.security >= RiskLevel::High {
            required.push("adversarial-risk".to_string());
            required.push("integration-blackbox".to_string());
            visibility = VisibilityProfile::BlackBoxE2e;
        }

        // UI / Frontend rules
        if change.surfaces.iter().any(|s| s == "frontend") || risk.user_facing >= RiskLevel::High {
            required.push("ui-visual".to_string());
            required.push("a11y".to_string());
        }

        // Critical risk requires deployment-shadow
        if risk.security == RiskLevel::Critical || risk.data_integrity == RiskLevel::Critical {
            conditional.push("deployment-shadow".to_string());
        }

        required.sort();
        required.dedup();
        conditional.sort();
        conditional.dedup();

        Self {
            subject,
            required_gates: required,
            conditional_gates: conditional,
            visibility,
        }
    }

    pub fn validate_proofs(&self, proofs: &[Proof]) -> Result<()> {
        for gate in &self.required_gates {
            let passed = proofs.iter().any(|p| {
                p.subject == self.subject
                    && p.gate.as_deref() == Some(gate.as_str())
                    && p.verdict == Verdict::Pass
            });
            if !passed {
                return Err(VwlError::VerificationFailed(format!(
                    "Required gate '{}' has not passed for subject '{}'",
                    gate, self.subject
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifierArtifact {
    pub id: String,
    pub subject: String,
    pub verifier: String,
    pub commit: String,
    pub purpose: String,
    pub content: String,
}

impl VerifierArtifact {
    pub fn new(
        id: impl Into<String>,
        subject: impl Into<String>,
        verifier: impl Into<String>,
        commit: impl Into<String>,
        purpose: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<Self> {
        let id = id.into();
        let subject = subject.into();
        let verifier = verifier.into();
        let commit = commit.into();
        let purpose = purpose.into();
        let content = content.into();

        if id.trim().is_empty() || subject.trim().is_empty() {
            return Err(VwlError::InvalidProof(
                "VerifierArtifact ID and subject cannot be empty".to_string(),
            ));
        }

        // Candidate isolation invariant: Verifier artifact commit MUST NOT equal subject
        if commit == subject {
            return Err(VwlError::InvalidProof(
                "Verifier cannot mutate candidate directly (Judge cannot rewrite defendant)"
                    .to_string(),
            ));
        }

        Ok(Self {
            id,
            subject,
            verifier,
            commit,
            purpose,
            content,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Challenge {
    pub id: String,
    pub subject: String,
    pub verifier: String,
    pub failing_gate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<String>,
    pub reason: String,
}

impl Challenge {
    pub fn new(
        id: impl Into<String>,
        subject: impl Into<String>,
        verifier: impl Into<String>,
        failing_gate: impl Into<String>,
        artifact_id: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            verifier: verifier.into(),
            failing_gate: failing_gate.into(),
            artifact_id,
            reason: reason.into(),
        }
    }
}

pub struct WiringGate;

impl WiringGate {
    pub fn verify_wiring(
        subject: &str,
        source_code: &str,
        registration_manifest: &str,
        component_name: &str,
    ) -> Proof {
        let is_defined = source_code.contains(component_name);
        let is_wired = registration_manifest.contains(component_name);

        if is_defined && is_wired {
            Proof::new_v22(
                subject,
                ProofType::Structural,
                Verdict::Pass,
                format!("check-wiring --component {}", component_name),
                Some(format!(
                    "Component '{}' is defined and active in registration manifest",
                    component_name
                )),
                Some("wiring".to_string()),
                Some(VerificationPlane::Deterministic),
                Some(VisibilityProfile::WhiteBox),
                Some("runtime-env".to_string()),
            )
            .expect("Valid Proof")
        } else {
            Proof::new_v22(
                subject,
                ProofType::Structural,
                Verdict::Fail,
                format!("check-wiring --component {}", component_name),
                Some(format!(
                    "Wiring verification failed: component '{}' (defined={}, wired={})",
                    component_name, is_defined, is_wired
                )),
                Some("wiring".to_string()),
                Some(VerificationPlane::Deterministic),
                Some(VisibilityProfile::WhiteBox),
                Some("runtime-env".to_string()),
            )
            .expect("Valid Proof")
        }
    }
}

pub struct TestIntegrityGate;

impl TestIntegrityGate {
    pub fn verify_test_integrity(subject: &str, test_diff: &str) -> Proof {
        let mut relaxations = Vec::new();

        for line in test_diff.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('-') && !trimmed.starts_with("---") {
                if trimmed.contains("assert") {
                    relaxations.push(format!("Removed or weakened assertion: {}", trimmed));
                } else if trimmed.contains("test") && trimmed.contains("fn ") {
                    relaxations.push(format!("Deleted test case: {}", trimmed));
                }
            }
            if trimmed.starts_with('+')
                && (trimmed.contains("#[ignore]") || trimmed.contains("// ignore"))
            {
                relaxations.push(format!("Added ignore annotation to test: {}", trimmed));
            }
        }

        if relaxations.is_empty() {
            Proof::new_v22(
                subject,
                ProofType::Work,
                Verdict::Pass,
                "verify-test-integrity --diff",
                Some("No test oracle relaxations or deletions detected".to_string()),
                Some("test-integrity".to_string()),
                Some(VerificationPlane::Deterministic),
                Some(VisibilityProfile::WhiteBox),
                None,
            )
            .expect("Valid Proof")
        } else {
            Proof::new_v22(
                subject,
                ProofType::Work,
                Verdict::Fail,
                "verify-test-integrity --diff",
                Some(format!(
                    "Test integrity violations detected:\n{}",
                    relaxations.join("\n")
                )),
                Some("test-integrity".to_string()),
                Some(VerificationPlane::Deterministic),
                Some(VisibilityProfile::WhiteBox),
                None,
            )
            .expect("Valid Proof")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShadowTrafficContract {
    pub is_read_only: bool,
    pub suppress_side_effects: bool,
    pub max_requests: u32,
    pub max_duration_secs: u64,
}

impl Default for ShadowTrafficContract {
    fn default() -> Self {
        Self {
            is_read_only: true,
            suppress_side_effects: true,
            max_requests: 1000,
            max_duration_secs: 60,
        }
    }
}

impl ShadowTrafficContract {
    pub fn validate(&self) -> Result<()> {
        if !self.is_read_only {
            return Err(VwlError::VerificationFailed(
                "Shadow traffic contract violation: shadow traffic MUST be read-only".to_string(),
            ));
        }
        if !self.suppress_side_effects {
            return Err(VwlError::VerificationFailed(
                "Shadow traffic contract violation: side effects must be suppressed".to_string(),
            ));
        }
        Ok(())
    }
}

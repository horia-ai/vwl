use crate::error::{Result, VwlError};
use crate::proof::{Proof, ProofType, Verdict, VerificationPlane, VisibilityProfile};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateDefinition {
    pub name: String,
    pub plane: VerificationPlane,
    pub command: String,
    pub precommit: bool,
    pub timeout_secs: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatesManifest {
    pub version: String,
    pub project: String,
    pub languages: Vec<String>,
    pub gates: Vec<GateDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateRunResult {
    pub name: String,
    pub plane: VerificationPlane,
    pub command: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

impl GatesManifest {
    pub fn detect(repo_root: &Path) -> Self {
        let mut languages = Vec::new();
        let mut gates = Vec::new();

        // 1. Rust detection
        if repo_root.join("Cargo.toml").exists() {
            languages.push("rust".to_string());
            gates.push(GateDefinition {
                name: "unit-tests".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "cargo test --workspace".to_string(),
                precommit: true,
                timeout_secs: 120,
                description: "Deterministic Rust test suite execution".to_string(),
            });
            gates.push(GateDefinition {
                name: "clippy-lint".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
                precommit: true,
                timeout_secs: 60,
                description: "Enforce zero compiler and clippy warnings".to_string(),
            });
            gates.push(GateDefinition {
                name: "format-check".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "cargo fmt --check".to_string(),
                precommit: true,
                timeout_secs: 30,
                description: "Enforce standard rustfmt style".to_string(),
            });
        }

        // 2. Node / TypeScript / JavaScript detection
        let pkg_json = repo_root.join("package.json");
        if pkg_json.exists() {
            let is_ts = repo_root.join("tsconfig.json").exists();
            languages.push(if is_ts {
                "typescript".to_string()
            } else {
                "javascript".to_string()
            });

            let mut has_test = true;
            let mut has_lint = false;
            let mut has_mutation = false;

            if let Ok(content) = std::fs::read_to_string(&pkg_json) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(scripts) = val.get("scripts").and_then(|s| s.as_object()) {
                        has_test = scripts.contains_key("test");
                        has_lint = scripts.contains_key("lint");
                        has_mutation = scripts.contains_key("test:mutation")
                            || scripts.contains_key("stryker");
                    }
                }
            }

            if has_test {
                gates.push(GateDefinition {
                    name: "node-tests".to_string(),
                    plane: VerificationPlane::Deterministic,
                    command: "npm test".to_string(),
                    precommit: true,
                    timeout_secs: 60,
                    description: "Execute Node.js test suite".to_string(),
                });
            }

            if has_lint {
                gates.push(GateDefinition {
                    name: "node-lint".to_string(),
                    plane: VerificationPlane::Deterministic,
                    command: "npm run lint".to_string(),
                    precommit: true,
                    timeout_secs: 45,
                    description: "Execute linter checks".to_string(),
                });
            }

            if is_ts {
                gates.push(GateDefinition {
                    name: "typecheck".to_string(),
                    plane: VerificationPlane::Deterministic,
                    command: "npx tsc --noEmit".to_string(),
                    precommit: true,
                    timeout_secs: 45,
                    description: "TypeScript strict typecheck".to_string(),
                });
            }

            if has_mutation {
                gates.push(GateDefinition {
                    name: "mutation-gate".to_string(),
                    plane: VerificationPlane::Deterministic,
                    command: "npm run test:mutation".to_string(),
                    precommit: false,
                    timeout_secs: 180,
                    description: "Stryker mutation testing gate".to_string(),
                });
            }
        }

        // 3. Python detection
        if repo_root.join("pyproject.toml").exists()
            || repo_root.join("requirements.txt").exists()
            || repo_root.join("setup.py").exists()
        {
            languages.push("python".to_string());
            gates.push(GateDefinition {
                name: "pytest".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "pytest".to_string(),
                precommit: true,
                timeout_secs: 60,
                description: "Execute Python pytest suite".to_string(),
            });
            gates.push(GateDefinition {
                name: "python-lint".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "ruff check .".to_string(),
                precommit: true,
                timeout_secs: 30,
                description: "Python linter check via ruff".to_string(),
            });
        }

        // 4. Go detection
        if repo_root.join("go.mod").exists() {
            languages.push("go".to_string());
            gates.push(GateDefinition {
                name: "go-test".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "go test ./...".to_string(),
                precommit: true,
                timeout_secs: 60,
                description: "Execute Go test suite".to_string(),
            });
        }

        // 5. Solidity detection
        if repo_root.join("foundry.toml").exists() {
            languages.push("solidity".to_string());
            gates.push(GateDefinition {
                name: "foundry-test".to_string(),
                plane: VerificationPlane::Deterministic,
                command: "forge test".to_string(),
                precommit: true,
                timeout_secs: 60,
                description: "Foundry smart contract tests".to_string(),
            });
        }

        // Always include Plane 1 Skeptic gate
        gates.push(GateDefinition {
            name: "skeptic-review".to_string(),
            plane: VerificationPlane::Judgment,
            command: "vwl task review --json .vwl/ledger/tasks/*.yaml 2>/dev/null || true"
                .to_string(),
            precommit: true,
            timeout_secs: 10,
            description: "Skeptic adversarial review gate for draft tasks".to_string(),
        });

        let project_name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed-project")
            .to_string();

        Self {
            version: "1.0".to_string(),
            project: project_name,
            languages,
            gates,
        }
    }

    pub fn load_or_detect(repo_root: &Path) -> Result<Self> {
        let gates_file = repo_root.join(".vwl").join("gates.yaml");
        if gates_file.exists() {
            let content = std::fs::read_to_string(gates_file)?;
            let manifest: Self = serde_yaml::from_str(&content)
                .map_err(|e| VwlError::SerializationError(e.to_string()))?;
            Ok(manifest)
        } else {
            Ok(Self::detect(repo_root))
        }
    }

    pub fn save(&self, repo_root: &Path) -> Result<PathBuf> {
        let vwl_dir = repo_root.join(".vwl");
        std::fs::create_dir_all(&vwl_dir)?;
        let gates_path = vwl_dir.join("gates.yaml");
        let yaml_str =
            serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))?;
        std::fs::write(&gates_path, yaml_str)?;
        Ok(gates_path)
    }

    pub fn run_gates(&self, repo_root: &Path, precommit_only: bool) -> Result<Vec<GateRunResult>> {
        let mut results = Vec::new();

        for gate in &self.gates {
            if precommit_only && !gate.precommit {
                continue;
            }

            let start = std::time::Instant::now();
            let output = std::process::Command::new("sh")
                .args(["-c", &gate.command])
                .current_dir(repo_root)
                .output();

            let duration_ms = start.elapsed().as_millis() as u64;

            match output {
                Ok(out) => {
                    let passed = out.status.success();
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();

                    results.push(GateRunResult {
                        name: gate.name.clone(),
                        plane: gate.plane,
                        command: gate.command.clone(),
                        passed,
                        duration_ms,
                        stdout,
                        stderr,
                    });
                }
                Err(e) => {
                    results.push(GateRunResult {
                        name: gate.name.clone(),
                        plane: gate.plane,
                        command: gate.command.clone(),
                        passed: false,
                        duration_ms,
                        stdout: String::new(),
                        stderr: e.to_string(),
                    });
                }
            }
        }

        Ok(results)
    }
}

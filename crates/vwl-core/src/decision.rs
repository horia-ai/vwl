use crate::proof::{Proof, ProofType, Verdict};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DecisionOutcome {
    Confirmed,
    Rejected,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Decision {
    pub subject: String,
    pub outcome: DecisionOutcome,
    pub reason: String,
}

pub struct DecisionEngine;

impl DecisionEngine {
    /// Deterministic evaluation of candidate commit against verification proofs (WU-G0-006).
    /// All required proofs must PASS -> Confirmed.
    /// Any required proof FAILS -> Rejected.
    /// Otherwise -> Pending.
    pub fn evaluate(subject: &str, proofs: &[Proof], required_types: &[ProofType]) -> Decision {
        if proofs.is_empty() {
            return Decision {
                subject: subject.to_string(),
                outcome: DecisionOutcome::Pending,
                reason: "No proofs submitted".to_string(),
            };
        }

        // Check for any failing proofs
        for proof in proofs {
            if proof.subject == subject && proof.verdict == Verdict::Fail {
                return Decision {
                    subject: subject.to_string(),
                    outcome: DecisionOutcome::Rejected,
                    reason: format!(
                        "Proof for {:?} failed with command: {}",
                        proof.r#type, proof.evidence.command
                    ),
                };
            }
        }

        // Verify all required proof types are present with PASS verdict
        for req in required_types {
            let has_passed = proofs
                .iter()
                .any(|p| p.subject == subject && p.r#type == *req && p.verdict == Verdict::Pass);

            if !has_passed {
                return Decision {
                    subject: subject.to_string(),
                    outcome: DecisionOutcome::Pending,
                    reason: format!("Required proof {:?} has not passed", req),
                };
            }
        }

        Decision {
            subject: subject.to_string(),
            outcome: DecisionOutcome::Confirmed,
            reason: "All required verification proofs passed cleanly".to_string(),
        }
    }
}

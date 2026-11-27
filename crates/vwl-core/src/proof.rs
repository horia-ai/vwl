use crate::error::{Result, VwlError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProofType {
    Work,
    Impact,
    Structural,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Fail,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerificationPlane {
    Deterministic,
    Judgment,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VisibilityProfile {
    WhiteBox,
    InterfaceIntegration,
    BlackBoxE2e,
    AdversarialBlackBox,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofEvidence {
    pub command: String,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proof {
    pub subject: String,
    pub r#type: ProofType,
    pub verdict: Verdict,
    pub evidence: ProofEvidence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane: Option<VerificationPlane>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<VisibilityProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl Proof {
    pub fn new(
        subject: impl Into<String>,
        r#type: ProofType,
        verdict: Verdict,
        command: impl Into<String>,
        output: Option<String>,
    ) -> Result<Self> {
        Self::new_v22(
            subject,
            r#type,
            verdict,
            command,
            output,
            None,
            Some(VerificationPlane::Deterministic),
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_v22(
        subject: impl Into<String>,
        r#type: ProofType,
        verdict: Verdict,
        command: impl Into<String>,
        output: Option<String>,
        gate: Option<String>,
        plane: Option<VerificationPlane>,
        visibility: Option<VisibilityProfile>,
        environment: Option<String>,
    ) -> Result<Self> {
        let subject = subject.into();
        let command = command.into();

        if subject.trim().is_empty() {
            return Err(VwlError::InvalidProof(
                "Subject commit cannot be empty".to_string(),
            ));
        }
        if command.trim().is_empty() {
            return Err(VwlError::InvalidProof(
                "Evidence command cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            subject,
            r#type,
            verdict,
            evidence: ProofEvidence { command, output },
            gate,
            plane,
            visibility,
            environment,
        })
    }

    pub fn compute_digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.subject.as_bytes());
        hasher.update(b"\n");
        hasher.update(format!("{:?}", self.r#type).as_bytes());
        hasher.update(b"\n");
        hasher.update(format!("{:?}", self.verdict).as_bytes());
        hasher.update(b"\n");
        hasher.update(self.evidence.command.as_bytes());
        if let Some(ref g) = self.gate {
            hasher.update(b"\n");
            hasher.update(g.as_bytes());
        }
        if let Some(ref p) = self.plane {
            hasher.update(b"\n");
            hasher.update(format!("{:?}", p).as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn from_yaml(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).map_err(|e| VwlError::SerializationError(e.to_string()))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))
    }
}

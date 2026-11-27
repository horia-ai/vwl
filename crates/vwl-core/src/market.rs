use crate::error::{Result, VwlError};
use crate::identity::{verify_signature, NodeKeypair};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Role {
    pub id: String,
    pub title: String,
    pub skills: Vec<String>,
    pub budget_work: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectManifest {
    pub name: String,
    pub repository: String,
    pub description: String,
    pub topics: Vec<String>,
    pub roles: Vec<Role>,
}

impl ProjectManifest {
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        let manifest: ProjectManifest = serde_yaml::from_str(yaml_str)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        if manifest.name.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Project name cannot be empty".to_string(),
            ));
        }
        if manifest.repository.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Repository cannot be empty".to_string(),
            ));
        }
        Ok(manifest)
    }

    pub fn load_from_repo(repo_root: &Path) -> Result<Self> {
        let manifest_path = repo_root.join(".vwl").join("project.yaml");
        if !manifest_path.exists() {
            return Err(VwlError::InvalidTask(format!(
                "Manifest not found at {}",
                manifest_path.display()
            )));
        }
        let content = fs::read_to_string(&manifest_path)?;
        Self::from_yaml(&content)
    }

    pub fn find_role(&self, role_id: &str) -> Option<&Role> {
        self.roles.iter().find(|r| r.id == role_id)
    }

    pub fn compute_digest(&self) -> String {
        let yaml = serde_yaml::to_string(self).unwrap_or_default();
        let hash = Sha256::digest(yaml.as_bytes());
        hex::encode(hash)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Offer {
    pub id: String,
    pub role_id: String,
    pub node_id: String,
    pub node_pubkey: String,
    pub proposed_budget_work: u64,
    pub signature: String,
    pub status: String,
    pub created_at: u64,
}

impl Offer {
    pub fn new(
        keypair: &NodeKeypair,
        node_id: &str,
        role_id: &str,
        proposed_budget_work: u64,
        created_at: u64,
    ) -> Self {
        let node_pubkey = keypair.public_key_hex();
        let offer_id = format!("OFFER-{}-{}", role_id, &node_pubkey[..8]);
        let unsigned = format!(
            "offer_id:{}|role:{}|node:{}|pubkey:{}|budget:{}|ts:{}",
            offer_id, role_id, node_id, node_pubkey, proposed_budget_work, created_at
        );
        let signature = keypair.sign_message(unsigned.as_bytes());

        Self {
            id: offer_id,
            role_id: role_id.to_string(),
            node_id: node_id.to_string(),
            node_pubkey,
            proposed_budget_work,
            signature,
            status: "Submitted".to_string(),
            created_at,
        }
    }

    pub fn verify(&self) -> Result<bool> {
        let unsigned = format!(
            "offer_id:{}|role:{}|node:{}|pubkey:{}|budget:{}|ts:{}",
            self.id,
            self.role_id,
            self.node_id,
            self.node_pubkey,
            self.proposed_budget_work,
            self.created_at
        );
        verify_signature(&self.node_pubkey, unsigned.as_bytes(), &self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_manifest_roundtrip() {
        let raw = r#"
name: "vwl"
repository: "https://github.com/horia-ai/vwl"
description: "VWL Decentralized Labor Protocol"
topics:
  - "vwl-project"
roles:
  - id: "ROLE-DEV"
    title: "Core Engineer"
    skills:
      - "tdd-mutation-testing"
    budget_work: 100000
    status: "Open"
"#;
        let manifest = ProjectManifest::from_yaml(raw).unwrap();
        assert_eq!(manifest.name, "vwl");
        assert_eq!(manifest.roles.len(), 1);
        let role = manifest.find_role("ROLE-DEV").unwrap();
        assert_eq!(role.budget_work, 100000);
        assert!(!manifest.compute_digest().is_empty());
    }

    #[test]
    fn test_offer_signature_and_verification() {
        let keypair = NodeKeypair::generate();
        let offer = Offer::new(&keypair, "node-123", "ROLE-DEV", 95000, 1700000000);

        assert_eq!(offer.status, "Submitted");
        assert!(offer.verify().unwrap());

        let mut tampered = offer.clone();
        tampered.proposed_budget_work = 500000;
        assert!(!tampered.verify().unwrap());
    }
}

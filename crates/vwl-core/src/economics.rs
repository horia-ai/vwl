use crate::error::{Result, VwlError};
use crate::identity::{verify_signature, NodeKeypair};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const WORK_PER_USD: u64 = 1_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeAccount {
    pub node_id: String,
    pub liquid_work: u64,
    pub earned_work: u64,
    pub vesting_work: u64,
    pub last_vest_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectAccount {
    pub project_id: String,
    pub total_deposited_work: u64,
    pub project_locked_work: u64,
    pub reserved_work: u64,
    pub disbursed_work: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Engagement {
    pub id: String,
    pub project_id: String,
    pub node_id: String,
    pub allocated_work: u64,
    pub vesting_duration_secs: u64,
    pub created_at: u64,
    pub status: String,
    pub project_signature: String,
    pub node_signature: String,
}

pub struct EngagementParams<'a> {
    pub project_id: &'a str,
    pub node_id: &'a str,
    pub work_amount: u64,
    pub vesting_duration_secs: u64,
    pub now: u64,
}

impl Engagement {
    pub fn new(
        project_key: &NodeKeypair,
        node_key: &NodeKeypair,
        params: &EngagementParams,
    ) -> Self {
        let id = format!(
            "ENGAGEMENT-{}-{}-{}",
            params.project_id, params.node_id, params.now
        );
        let unsigned = format!(
            "id:{}|proj:{}|node:{}|alloc:{}|vesting:{}|ts:{}",
            id,
            params.project_id,
            params.node_id,
            params.work_amount,
            params.vesting_duration_secs,
            params.now
        );
        let project_signature = project_key.sign_message(unsigned.as_bytes());
        let node_signature = node_key.sign_message(unsigned.as_bytes());

        Self {
            id,
            project_id: params.project_id.to_string(),
            node_id: params.node_id.to_string(),
            allocated_work: params.work_amount,
            vesting_duration_secs: params.vesting_duration_secs,
            created_at: params.now,
            status: "Reserved".to_string(),
            project_signature,
            node_signature,
        }
    }

    pub fn verify(&self, project_pubkey: &str, node_pubkey: &str) -> Result<bool> {
        let unsigned = format!(
            "id:{}|proj:{}|node:{}|alloc:{}|vesting:{}|ts:{}",
            self.id,
            self.project_id,
            self.node_id,
            self.allocated_work,
            self.vesting_duration_secs,
            self.created_at
        );
        let proj_valid =
            verify_signature(project_pubkey, unsigned.as_bytes(), &self.project_signature)?;
        let node_valid = verify_signature(node_pubkey, unsigned.as_bytes(), &self.node_signature)?;
        Ok(proj_valid && node_valid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EconomicLedger {
    pub projects: HashMap<String, ProjectAccount>,
    pub nodes: HashMap<String, NodeAccount>,
    pub engagements: HashMap<String, Engagement>,
}

impl EconomicLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn deposit_fiat_usd(&mut self, project_id: &str, usd_amount: u64) -> u64 {
        let work_minted = usd_amount * WORK_PER_USD;
        let account = self
            .projects
            .entry(project_id.to_string())
            .or_insert_with(|| ProjectAccount {
                project_id: project_id.to_string(),
                total_deposited_work: 0,
                project_locked_work: 0,
                reserved_work: 0,
                disbursed_work: 0,
            });

        account.total_deposited_work += work_minted;
        account.project_locked_work += work_minted;
        work_minted
    }

    pub fn create_engagement(
        &mut self,
        project_key: &NodeKeypair,
        node_key: &NodeKeypair,
        params: EngagementParams,
    ) -> Result<Engagement> {
        let project = self.projects.get_mut(params.project_id).ok_or_else(|| {
            VwlError::InvalidTask(format!("Project '{}' not found", params.project_id))
        })?;

        if project.project_locked_work < params.work_amount {
            return Err(VwlError::InvalidTask(format!(
                "Insufficient project locked funds: available {}, requested {}",
                project.project_locked_work, params.work_amount
            )));
        }

        project.project_locked_work -= params.work_amount;
        project.reserved_work += params.work_amount;

        self.nodes
            .entry(params.node_id.to_string())
            .or_insert_with(|| NodeAccount {
                node_id: params.node_id.to_string(),
                liquid_work: 0,
                earned_work: 0,
                vesting_work: 0,
                last_vest_ts: params.now,
            });

        let engagement = Engagement::new(project_key, node_key, &params);

        self.engagements
            .insert(engagement.id.clone(), engagement.clone());
        Ok(engagement)
    }

    pub fn settle_verified_work(&mut self, engagement_id: &str, now: u64) -> Result<()> {
        let engagement = self.engagements.get_mut(engagement_id).ok_or_else(|| {
            VwlError::InvalidTask(format!("Engagement '{}' not found", engagement_id))
        })?;

        if engagement.status != "Reserved" && engagement.status != "Active" {
            return Err(VwlError::InvalidTask(format!(
                "Engagement not in settleable state: {}",
                engagement.status
            )));
        }

        let amount = engagement.allocated_work;
        let project = self
            .projects
            .get_mut(&engagement.project_id)
            .ok_or_else(|| VwlError::InvalidTask("Project missing".to_string()))?;

        project.reserved_work -= amount;
        project.disbursed_work += amount;

        let node = self
            .nodes
            .get_mut(&engagement.node_id)
            .ok_or_else(|| VwlError::InvalidTask("Node missing".to_string()))?;

        if engagement.vesting_duration_secs == 0 {
            node.liquid_work += amount;
            node.earned_work += amount;
        } else {
            node.vesting_work += amount;
            node.earned_work += amount;
            node.last_vest_ts = now;
        }

        engagement.status = "Earned".to_string();
        Ok(())
    }

    pub fn release_vesting(&mut self, node_id: &str, now: u64) -> Result<u64> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| VwlError::InvalidTask(format!("Node '{}' not found", node_id)))?;

        if node.vesting_work == 0 {
            return Ok(0);
        }

        let released = node.vesting_work;
        node.vesting_work = 0;
        node.liquid_work += released;
        node.last_vest_ts = now;
        Ok(released)
    }

    pub fn check_conservation_invariant(&self) -> Result<()> {
        let total_deposited: u64 = self.projects.values().map(|p| p.total_deposited_work).sum();

        let total_project_locked: u64 = self.projects.values().map(|p| p.project_locked_work).sum();

        let total_reserved: u64 = self.projects.values().map(|p| p.reserved_work).sum();

        let total_node_vesting: u64 = self.nodes.values().map(|n| n.vesting_work).sum();

        let total_node_liquid: u64 = self.nodes.values().map(|n| n.liquid_work).sum();

        let total_allocated =
            total_project_locked + total_reserved + total_node_vesting + total_node_liquid;

        if total_deposited != total_allocated {
            return Err(VwlError::InvalidTask(format!(
                "Economic Conservation Invariant Violated! Total Deposited ({}) != Total Accounted ({})",
                total_deposited, total_allocated
            )));
        }

        Ok(())
    }

    pub fn save_to_ledger(&self, repo_root: &Path) -> Result<()> {
        let ledger_dir = repo_root.join(".vwl").join("ledger");
        fs::create_dir_all(&ledger_dir)?;
        let path = ledger_dir.join("economics.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_ledger(repo_root: &Path) -> Result<Self> {
        let path = repo_root.join(".vwl").join("ledger").join("economics.json");
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = fs::read_to_string(path)?;
        let ledger: Self =
            serde_json::from_str(&data).map_err(|e| VwlError::SerializationError(e.to_string()))?;
        Ok(ledger)
    }

    pub fn load_or_create(repo_root: &Path) -> Result<Self> {
        Self::load_from_ledger(repo_root)
    }

    pub fn verify_conservation(&self) -> Result<()> {
        self.check_conservation_invariant()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiat_deposit_and_conservation() {
        let mut ledger = EconomicLedger::new();
        let minted = ledger.deposit_fiat_usd("proj-1", 500); // $500 = 500,000 WORK
        assert_eq!(minted, 500_000);

        ledger.check_conservation_invariant().unwrap();
    }

    #[test]
    fn test_engagement_lifecycle_and_conservation_invariant() {
        let mut ledger = EconomicLedger::new();
        ledger.deposit_fiat_usd("proj-1", 1000); // 1,000,000 WORK

        let proj_key = NodeKeypair::generate();
        let node_key = NodeKeypair::generate();
        let node_id = "node-worker-1";

        // 1. Atomic reservation
        let eng = ledger
            .create_engagement(
                &proj_key,
                &node_key,
                EngagementParams {
                    project_id: "proj-1",
                    node_id,
                    work_amount: 200_000,
                    vesting_duration_secs: 3600,
                    now: 1000,
                },
            )
            .unwrap();

        assert_eq!(eng.status, "Reserved");
        assert!(eng
            .verify(&proj_key.public_key_hex(), &node_key.public_key_hex())
            .unwrap());
        ledger.check_conservation_invariant().unwrap();

        // 2. Settle verified work
        ledger.settle_verified_work(&eng.id, 1050).unwrap();
        ledger.check_conservation_invariant().unwrap();

        let node = ledger.nodes.get(node_id).unwrap();
        assert_eq!(node.vesting_work, 200_000);
        assert_eq!(node.liquid_work, 0);

        // 3. Release linear vesting
        let released = ledger.release_vesting(node_id, 2000).unwrap();
        assert_eq!(released, 200_000);

        let node_updated = ledger.nodes.get(node_id).unwrap();
        assert_eq!(node_updated.liquid_work, 200_000);
        assert_eq!(node_updated.vesting_work, 0);

        // Invariant holds strictly
        ledger.check_conservation_invariant().unwrap();
    }

    #[test]
    fn test_conservation_invariant_catches_unauthorized_inflation() {
        let mut ledger = EconomicLedger::new();
        ledger.deposit_fiat_usd("proj-1", 100);

        // Corrupt ledger balance
        if let Some(node) = ledger.nodes.get_mut("unauthorized") {
            node.liquid_work += 999;
        } else {
            ledger.nodes.insert(
                "unauthorized".to_string(),
                NodeAccount {
                    node_id: "unauthorized".to_string(),
                    liquid_work: 500,
                    earned_work: 500,
                    vesting_work: 0,
                    last_vest_ts: 0,
                },
            );
        }

        assert!(ledger.check_conservation_invariant().is_err());
    }
}

use crate::decision::Decision;
use crate::error::{Result, VwlError};
use crate::proof::Proof;
use crate::task::Task;
use crate::work_unit::WorkUnit;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LocalLedger {
    root_dir: PathBuf,
}

impl LocalLedger {
    pub fn new(repo_root: &Path) -> Result<Self> {
        let root_dir = repo_root.join(".vwl").join("ledger");
        fs::create_dir_all(root_dir.join("tasks"))?;
        fs::create_dir_all(root_dir.join("work_units"))?;
        fs::create_dir_all(root_dir.join("proofs"))?;
        fs::create_dir_all(root_dir.join("decisions"))?;

        Ok(Self { root_dir })
    }

    pub fn append_task(&self, task: &Task) -> Result<()> {
        let digest = task.compute_digest();
        let file_path = self.root_dir.join("tasks").join(format!("{}.yaml", digest));

        if file_path.exists() {
            let existing_content = fs::read_to_string(&file_path)?;
            let existing_task = Task::from_yaml(&existing_content)?;
            if existing_task != *task {
                return Err(VwlError::LedgerError(format!(
                    "Conflicting task content for digest {}",
                    digest
                )));
            }
            return Ok(()); // Idempotent
        }

        fs::write(file_path, task.to_yaml()?)?;
        Ok(())
    }

    pub fn append_work_unit(&self, wu: &WorkUnit) -> Result<()> {
        let file_path = self
            .root_dir
            .join("work_units")
            .join(format!("{}.yaml", wu.commit));

        if file_path.exists() {
            let existing_content = fs::read_to_string(&file_path)?;
            let existing_wu = WorkUnit::from_yaml(&existing_content)?;
            if existing_wu != *wu {
                return Err(VwlError::LedgerError(format!(
                    "Conflicting work unit for commit {}",
                    wu.commit
                )));
            }
            return Ok(()); // Idempotent
        }

        fs::write(file_path, wu.to_yaml()?)?;
        Ok(())
    }

    pub fn append_proof(&self, proof: &Proof) -> Result<()> {
        let digest = proof.compute_digest();
        let file_path = self
            .root_dir
            .join("proofs")
            .join(format!("{}.yaml", digest));

        if file_path.exists() {
            return Ok(()); // Idempotent
        }

        fs::write(file_path, proof.to_yaml()?)?;
        Ok(())
    }

    pub fn append_decision(&self, decision: &Decision) -> Result<()> {
        let file_path = self
            .root_dir
            .join("decisions")
            .join(format!("{}.yaml", decision.subject));

        let yaml = serde_yaml::to_string(decision)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let tasks_dir = self.root_dir.join("tasks");
        if tasks_dir.exists() {
            for entry in fs::read_dir(tasks_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    tasks.push(Task::from_yaml(&content)?);
                }
            }
        }
        Ok(tasks)
    }

    pub fn get_work_units(&self) -> Result<Vec<WorkUnit>> {
        let mut work_units = Vec::new();
        let wu_dir = self.root_dir.join("work_units");
        if wu_dir.exists() {
            for entry in fs::read_dir(wu_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    work_units.push(WorkUnit::from_yaml(&content)?);
                }
            }
        }
        Ok(work_units)
    }

    pub fn get_decisions(&self) -> Result<Vec<Decision>> {
        let mut decisions = Vec::new();
        let dec_dir = self.root_dir.join("decisions");
        if dec_dir.exists() {
            for entry in fs::read_dir(dec_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let dec: Decision = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    decisions.push(dec);
                }
            }
        }
        Ok(decisions)
    }

    pub fn get_decision_for_subject(&self, subject: &str) -> Result<Option<Decision>> {
        let file_path = self.root_dir.join("decisions").join(format!("{}.yaml", subject));
        if file_path.exists() {
            let content = fs::read_to_string(file_path)?;
            let dec: Decision = serde_yaml::from_str(&content)
                .map_err(|e| VwlError::SerializationError(e.to_string()))?;
            Ok(Some(dec))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_proofs(&self) -> Result<Vec<Proof>> {
        let mut proofs = Vec::new();
        let proofs_dir = self.root_dir.join("proofs");
        if proofs_dir.exists() {
            for entry in fs::read_dir(proofs_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    proofs.push(Proof::from_yaml(&content)?);
                }
            }
        }
        Ok(proofs)
    }

    pub fn get_proofs_for_subject(&self, subject: &str) -> Result<Vec<Proof>> {
        let mut proofs = Vec::new();
        let proofs_dir = self.root_dir.join("proofs");
        if proofs_dir.exists() {
            for entry in fs::read_dir(proofs_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let proof = Proof::from_yaml(&content)?;
                    if proof.subject == subject {
                        proofs.push(proof);
                    }
                }
            }
        }
        Ok(proofs)
    }
}

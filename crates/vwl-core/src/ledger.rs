use crate::decision::Decision;
use crate::error::{Result, VwlError};
use crate::factory::{Challenge, VerificationPlan, VerifierArtifact};
use crate::graph::{Claim, TaskDependency, TaskGraph};
use crate::proof::Proof;
use crate::repometer::{Attribution, Outcome};
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
        fs::create_dir_all(root_dir.join("dependencies"))?;
        fs::create_dir_all(root_dir.join("claims"))?;
        fs::create_dir_all(root_dir.join("plans"))?;
        fs::create_dir_all(root_dir.join("artifacts"))?;
        fs::create_dir_all(root_dir.join("challenges"))?;
        fs::create_dir_all(root_dir.join("outcomes"))?;
        fs::create_dir_all(root_dir.join("attributions"))?;

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

    pub fn append_dependency(&self, dep: &TaskDependency) -> Result<()> {
        // Validate cycle prevention
        let mut graph = self.get_task_graph()?;
        graph.add_dependency(dep.clone())?;

        let safe_from = dep.from_task.replace(['/', '\\', ':'], "_");
        let safe_to = dep.to_task.replace(['/', '\\', ':'], "_");
        let file_name = format!("{}_{}_{}.yaml", safe_from, safe_to, dep.kind.as_str());
        let file_path = self.root_dir.join("dependencies").join(file_name);

        if file_path.exists() {
            return Ok(()); // Idempotent
        }

        let yaml =
            serde_yaml::to_string(dep).map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_dependencies(&self) -> Result<Vec<TaskDependency>> {
        let mut deps = Vec::new();
        let deps_dir = self.root_dir.join("dependencies");
        if deps_dir.exists() {
            for entry in fs::read_dir(deps_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let dep: TaskDependency = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    deps.push(dep);
                }
            }
        }
        deps.sort();
        Ok(deps)
    }

    pub fn append_claim(&self, claim: &Claim) -> Result<()> {
        let safe_id = claim.task_id.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("claims")
            .join(format!("{}.yaml", safe_id));
        let yaml = serde_yaml::to_string(claim)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn release_claim(&self, task_id: &str) -> Result<()> {
        let safe_id = task_id.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("claims")
            .join(format!("{}.yaml", safe_id));
        if file_path.exists() {
            let _ = fs::remove_file(file_path);
        }
        Ok(())
    }

    pub fn get_claims(&self) -> Result<Vec<Claim>> {
        let mut claims = Vec::new();
        let claims_dir = self.root_dir.join("claims");
        if claims_dir.exists() {
            for entry in fs::read_dir(claims_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let claim: Claim = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    claims.push(claim);
                }
            }
        }
        claims.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        Ok(claims)
    }

    pub fn get_task_graph(&self) -> Result<TaskGraph> {
        let mut graph = TaskGraph::new();
        for task in self.get_tasks()? {
            graph.add_task(task);
        }
        for dep in self.get_dependencies()? {
            graph.dependencies.push(dep);
        }
        graph.validate_no_cycles()?;
        for claim in self.get_claims()? {
            graph.claims.insert(claim.task_id.clone(), claim);
        }
        Ok(graph)
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
        tasks.sort_by(|a, b| a.id.cmp(&b.id));
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
        work_units.sort_by(|a, b| a.commit.cmp(&b.commit));
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
        decisions.sort_by(|a, b| a.subject.cmp(&b.subject));
        Ok(decisions)
    }

    pub fn get_decision_for_subject(&self, subject: &str) -> Result<Option<Decision>> {
        let file_path = self
            .root_dir
            .join("decisions")
            .join(format!("{}.yaml", subject));
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

    pub fn append_verification_plan(&self, plan: &VerificationPlan) -> Result<()> {
        let safe_subject = plan.subject.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("plans")
            .join(format!("{}.yaml", safe_subject));
        let yaml =
            serde_yaml::to_string(plan).map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_verification_plan(&self, subject: &str) -> Result<Option<VerificationPlan>> {
        let safe_subject = subject.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("plans")
            .join(format!("{}.yaml", safe_subject));
        if file_path.exists() {
            let content = fs::read_to_string(file_path)?;
            let plan: VerificationPlan = serde_yaml::from_str(&content)
                .map_err(|e| VwlError::SerializationError(e.to_string()))?;
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    pub fn append_verifier_artifact(&self, artifact: &VerifierArtifact) -> Result<()> {
        let safe_id = artifact.id.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("artifacts")
            .join(format!("{}.yaml", safe_id));
        let yaml = serde_yaml::to_string(artifact)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_verifier_artifacts_for_subject(
        &self,
        subject: &str,
    ) -> Result<Vec<VerifierArtifact>> {
        let mut artifacts = Vec::new();
        let artifacts_dir = self.root_dir.join("artifacts");
        if artifacts_dir.exists() {
            for entry in fs::read_dir(artifacts_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let art: VerifierArtifact = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    if art.subject == subject {
                        artifacts.push(art);
                    }
                }
            }
        }
        artifacts.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(artifacts)
    }

    pub fn append_challenge(&self, challenge: &Challenge) -> Result<()> {
        let safe_id = challenge.id.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("challenges")
            .join(format!("{}.yaml", safe_id));
        let yaml = serde_yaml::to_string(challenge)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_challenges(&self) -> Result<Vec<Challenge>> {
        let mut challenges = Vec::new();
        let ch_dir = self.root_dir.join("challenges");
        if ch_dir.exists() {
            for entry in fs::read_dir(ch_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let ch: Challenge = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    challenges.push(ch);
                }
            }
        }
        challenges.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(challenges)
    }

    pub fn append_outcome(&self, outcome: &Outcome) -> Result<()> {
        let safe_id = outcome.id.replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("outcomes")
            .join(format!("{}.yaml", safe_id));
        let yaml = serde_yaml::to_string(outcome)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_outcomes(&self) -> Result<Vec<Outcome>> {
        let mut outcomes = Vec::new();
        let out_dir = self.root_dir.join("outcomes");
        if out_dir.exists() {
            for entry in fs::read_dir(out_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let out: Outcome = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    outcomes.push(out);
                }
            }
        }
        outcomes.sort_by_key(|a| a.timestamp);
        Ok(outcomes)
    }

    pub fn append_attribution(&self, attribution: &Attribution) -> Result<()> {
        let safe_id = format!("{}_{}", attribution.outcome_id, attribution.target_node)
            .replace(['/', '\\', ':'], "_");
        let file_path = self
            .root_dir
            .join("attributions")
            .join(format!("{}.yaml", safe_id));
        let yaml = serde_yaml::to_string(attribution)
            .map_err(|e| VwlError::SerializationError(e.to_string()))?;
        fs::write(file_path, yaml)?;
        Ok(())
    }

    pub fn get_attributions(&self) -> Result<Vec<Attribution>> {
        let mut attributions = Vec::new();
        let attr_dir = self.root_dir.join("attributions");
        if attr_dir.exists() {
            for entry in fs::read_dir(attr_dir)? {
                let entry = entry?;
                if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                    let content = fs::read_to_string(entry.path())?;
                    let attr: Attribution = serde_yaml::from_str(&content)
                        .map_err(|e| VwlError::SerializationError(e.to_string()))?;
                    attributions.push(attr);
                }
            }
        }
        attributions.sort_by(|a, b| a.outcome_id.cmp(&b.outcome_id));
        Ok(attributions)
    }
}

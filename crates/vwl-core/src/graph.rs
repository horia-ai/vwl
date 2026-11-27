use crate::error::{Result, VwlError};
use crate::task::Task;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum DependencyKind {
    Requires,
    After,
    Evidence,
    Conflicts,
}

impl DependencyKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requires => "requires",
            Self::After => "after",
            Self::Evidence => "evidence",
            Self::Conflicts => "conflicts",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskDependency {
    pub from_task: String,
    pub to_task: String,
    pub kind: DependencyKind,
}

impl TaskDependency {
    pub fn new(
        from_task: impl Into<String>,
        to_task: impl Into<String>,
        kind: DependencyKind,
    ) -> Self {
        Self {
            from_task: from_task.into(),
            to_task: to_task.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claim {
    pub task_id: String,
    pub actor: String,
    pub claimed_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

impl Claim {
    pub fn new(task_id: impl Into<String>, actor: impl Into<String>, claimed_at: u64) -> Self {
        Self {
            task_id: task_id.into(),
            actor: actor.into(),
            claimed_at,
            expires_at: None,
        }
    }

    pub fn with_expiration(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_active(&self, current_time: u64) -> bool {
        if let Some(exp) = self.expires_at {
            current_time < exp
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskGraph {
    pub tasks: HashMap<String, Task>,
    pub dependencies: Vec<TaskDependency>,
    #[serde(default)]
    pub claims: HashMap<String, Claim>,
    #[serde(default)]
    pub cancelled_tasks: HashSet<String>,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id.clone(), task);
    }

    pub fn cancel_task(&mut self, task_id: impl Into<String>) {
        let id = task_id.into();
        self.cancelled_tasks.insert(id.clone());
        self.claims.remove(&id);
    }

    pub fn is_cancelled(&self, task_id: &str) -> bool {
        self.cancelled_tasks.contains(task_id)
    }

    pub fn add_dependency(&mut self, dep: TaskDependency) -> Result<()> {
        if dep.from_task.trim().is_empty() || dep.to_task.trim().is_empty() {
            return Err(VwlError::InvalidTask(
                "Dependency task IDs cannot be empty".to_string(),
            ));
        }

        if dep.from_task == dep.to_task {
            return Err(VwlError::InvalidTask(format!(
                "Self-referential dependency cycle on task '{}'",
                dep.from_task
            )));
        }

        // Avoid exact duplicate dependencies
        if self.dependencies.contains(&dep) {
            return Ok(());
        }

        self.dependencies.push(dep);

        if let Err(e) = self.validate_no_cycles() {
            self.dependencies.pop();
            return Err(e);
        }

        Ok(())
    }

    pub fn validate_no_cycles(&self) -> Result<()> {
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for dep in &self.dependencies {
            if dep.kind != DependencyKind::Conflicts {
                adj.entry(&dep.from_task).or_default().push(&dep.to_task);
            }
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut recursion_stack: HashSet<&str> = HashSet::new();

        fn dfs<'a>(
            node: &'a str,
            adj: &HashMap<&'a str, Vec<&'a str>>,
            visited: &mut HashSet<&'a str>,
            rec: &mut HashSet<&'a str>,
        ) -> bool {
            visited.insert(node);
            rec.insert(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        if dfs(neighbor, adj, visited, rec) {
                            return true;
                        }
                    } else if rec.contains(neighbor) {
                        return true;
                    }
                }
            }

            rec.remove(node);
            false
        }

        for &node in adj.keys() {
            if !visited.contains(node) && dfs(node, &adj, &mut visited, &mut recursion_stack) {
                return Err(VwlError::InvalidTask(format!(
                    "Dependency cycle detected involving task '{}'",
                    node
                )));
            }
        }

        Ok(())
    }

    pub fn claim_task(&mut self, claim: Claim, current_time: u64) -> Result<()> {
        if !self.tasks.contains_key(&claim.task_id) {
            return Err(VwlError::InvalidTask(format!(
                "Task '{}' does not exist in graph",
                claim.task_id
            )));
        }

        if self.is_cancelled(&claim.task_id) {
            return Err(VwlError::InvalidTask(format!(
                "Cannot claim cancelled task '{}'",
                claim.task_id
            )));
        }

        if let Some(existing) = self.claims.get(&claim.task_id) {
            if existing.is_active(current_time) && existing.actor != claim.actor {
                return Err(VwlError::LedgerError(format!(
                    "Task '{}' is already actively claimed by actor '{}' (contention)",
                    claim.task_id, existing.actor
                )));
            }
        }

        self.claims.insert(claim.task_id.clone(), claim);
        Ok(())
    }

    pub fn release_claim(&mut self, task_id: &str, actor: Option<&str>) -> Result<()> {
        if let Some(existing) = self.claims.get(task_id) {
            if let Some(expected_actor) = actor {
                if existing.actor != expected_actor {
                    return Err(VwlError::LedgerError(format!(
                        "Cannot release claim on task '{}': owned by '{}', not '{}'",
                        task_id, existing.actor, expected_actor
                    )));
                }
            }
            self.claims.remove(task_id);
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn is_ready(&self, task_id: &str, canonical_confirmed_tasks: &HashSet<String>) -> bool {
        if !self.tasks.contains_key(task_id) {
            return false;
        }

        if self.is_cancelled(task_id) {
            return false;
        }

        if canonical_confirmed_tasks.contains(task_id) {
            return false;
        }

        for dep in &self.dependencies {
            if dep.to_task == task_id {
                match dep.kind {
                    DependencyKind::Requires => {
                        if !canonical_confirmed_tasks.contains(&dep.from_task) {
                            return false;
                        }
                    }
                    DependencyKind::Conflicts => {
                        if canonical_confirmed_tasks.contains(&dep.from_task) {
                            return false;
                        }
                    }
                    DependencyKind::After => {
                        if !canonical_confirmed_tasks.contains(&dep.from_task) {
                            return false;
                        }
                    }
                    DependencyKind::Evidence => {}
                }
            }
        }

        true
    }

    pub fn get_frontier(&self, canonical_confirmed_tasks: &HashSet<String>) -> Vec<Task> {
        let mut ready_tasks: Vec<Task> = self
            .tasks
            .values()
            .filter(|t| self.is_ready(&t.id, canonical_confirmed_tasks))
            .cloned()
            .collect();

        ready_tasks.sort_by(|a, b| a.id.cmp(&b.id));
        ready_tasks
    }

    pub fn why_blocked(
        &self,
        task_id: &str,
        canonical_confirmed_tasks: &HashSet<String>,
    ) -> Vec<String> {
        if !self.tasks.contains_key(task_id) {
            return vec![format!("Task '{}' does not exist in graph", task_id)];
        }

        if self.is_cancelled(task_id) {
            return vec![format!("Task '{}' has been cancelled", task_id)];
        }

        if canonical_confirmed_tasks.contains(task_id) {
            return vec![format!(
                "Task '{}' is already confirmed in canonical state",
                task_id
            )];
        }

        let mut reasons = Vec::new();

        for dep in &self.dependencies {
            if dep.to_task == task_id {
                match dep.kind {
                    DependencyKind::Requires => {
                        if !canonical_confirmed_tasks.contains(&dep.from_task) {
                            reasons.push(format!(
                                "Requires unconfirmed task '{}' in canonical state",
                                dep.from_task
                            ));
                        }
                    }
                    DependencyKind::Conflicts => {
                        if canonical_confirmed_tasks.contains(&dep.from_task) {
                            reasons.push(format!(
                                "Conflicts with confirmed canonical task '{}'",
                                dep.from_task
                            ));
                        }
                    }
                    DependencyKind::After => {
                        if !canonical_confirmed_tasks.contains(&dep.from_task) {
                            reasons.push(format!(
                                "Must sequence after unconfirmed task '{}'",
                                dep.from_task
                            ));
                        }
                    }
                    DependencyKind::Evidence => {}
                }
            }
        }

        if reasons.is_empty() {
            vec![format!("Task '{}' is READY on Work Frontier", task_id)]
        } else {
            reasons
        }
    }

    pub fn get_affected_cone(&self, task_id: &str) -> HashSet<String> {
        let mut cone = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(task_id.to_string());

        while let Some(current) = queue.pop_front() {
            for dep in &self.dependencies {
                if dep.from_task == current
                    && dep.kind != DependencyKind::Conflicts
                    && cone.insert(dep.to_task.clone())
                {
                    queue.push_back(dep.to_task.clone());
                }
            }
        }

        cone
    }

    pub fn compute_dependency_snapshot(
        &self,
        task_id: &str,
        canonical_confirmed_tasks: &HashSet<String>,
    ) -> String {
        let mut upstream: Vec<&TaskDependency> = self
            .dependencies
            .iter()
            .filter(|d| d.to_task == task_id)
            .collect();

        upstream.sort();

        let mut hasher = Sha256::new();
        hasher.update(task_id.as_bytes());

        for dep in upstream {
            let confirmed = canonical_confirmed_tasks.contains(&dep.from_task);
            let entry = format!("{}:{}:{};", dep.from_task, dep.kind.as_str(), confirmed);
            hasher.update(entry.as_bytes());
        }

        format!("sha256:{:x}", hasher.finalize())
    }

    pub fn is_candidate_stale(
        &self,
        task_id: &str,
        recorded_snapshot: &str,
        canonical_confirmed_tasks: &HashSet<String>,
    ) -> bool {
        let current_snapshot = self.compute_dependency_snapshot(task_id, canonical_confirmed_tasks);
        current_snapshot != recorded_snapshot
    }

    pub fn check_unsatisfiable(
        &self,
        canonical_confirmed_tasks: &HashSet<String>,
    ) -> Vec<(String, String)> {
        let mut unsatisfiable = Vec::new();

        for id in self.tasks.keys() {
            if self.is_cancelled(id) || canonical_confirmed_tasks.contains(id) {
                continue;
            }

            for dep in &self.dependencies {
                if dep.to_task == *id {
                    match dep.kind {
                        DependencyKind::Requires => {
                            if self.is_cancelled(&dep.from_task) {
                                unsatisfiable.push((
                                    id.clone(),
                                    format!(
                                        "Required upstream task '{}' was cancelled",
                                        dep.from_task
                                    ),
                                ));
                            }
                        }
                        DependencyKind::Conflicts
                            if canonical_confirmed_tasks.contains(&dep.from_task) =>
                        {
                            unsatisfiable.push((
                                id.clone(),
                                format!(
                                    "Conflicting task '{}' is already confirmed canonical",
                                    dep.from_task
                                ),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        unsatisfiable
    }

    pub fn format_intent_graph(&self) -> String {
        let mut out = String::new();
        out.push_str("=== VWL INTENT GRAPH (Task DAG) ===\n");
        let mut sorted_tasks: Vec<&Task> = self.tasks.values().collect();
        sorted_tasks.sort_by(|a, b| a.id.cmp(&b.id));

        for task in sorted_tasks {
            let status = if self.is_cancelled(&task.id) {
                "[CANCELLED]"
            } else if let Some(claim) = self.claims.get(&task.id) {
                &format!("[CLAIMED by {}]", claim.actor)
            } else {
                "[OPEN]"
            };
            out.push_str(&format!(
                "Task {}: {} {}\n",
                task.id, task.objective, status
            ));

            let upstream: Vec<&TaskDependency> = self
                .dependencies
                .iter()
                .filter(|d| d.to_task == task.id)
                .collect();

            for dep in upstream {
                out.push_str(&format!(
                    "  ├── {} upstream {}\n",
                    dep.kind.as_str().to_uppercase(),
                    dep.from_task
                ));
            }
        }
        out
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| VwlError::SerializationError(e.to_string()))
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| VwlError::SerializationError(e.to_string()))
    }
}

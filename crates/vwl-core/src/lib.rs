pub mod analytics;
pub mod decision;
pub mod economics;
pub mod error;
pub mod git;
pub mod github;
pub mod graph;
pub mod identity;
pub mod ledger;
pub mod market;
pub mod proof;
pub mod replay;
pub mod skills;
pub mod task;
pub mod work_unit;

pub use analytics::{
    calculate_effectiveness, calculate_rep_score, ContributorMetrics, CostAllocationMetrics,
    ProjectLedgerReport,
};
pub use decision::{Decision, DecisionEngine, DecisionOutcome};
pub use economics::{
    EconomicLedger, Engagement, EngagementParams, NodeAccount, ProjectAccount, WORK_PER_USD,
};
pub use error::{Result, VwlError};
pub use git::{GitInspector, GitRepoInfo};
pub use github::{
    resolve_github_token, GithubClient, GithubComment, GithubIssue, GithubSyncSummary, GithubUser,
};
pub use graph::{Claim, DependencyKind, TaskDependency, TaskGraph};
pub use identity::{
    verify_signature, NodeCredential, NodeKeypair, ProjectCredential, SignedObject,
};
pub use ledger::LocalLedger;
pub use market::{Offer, ProjectManifest, Role};
pub use proof::{Proof, ProofEvidence, ProofType, Verdict, VerificationPlane, VisibilityProfile};
pub use replay::{ReplayEngine, ReplayState};
pub use skills::{install_skill, list_skills, SkillDef, SKILLS};
pub use task::Task;
pub use work_unit::{CommitEntry, WorkUnit};

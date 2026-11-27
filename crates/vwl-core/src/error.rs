use thiserror::Error;

#[derive(Error, Debug)]
pub enum VwlError {
    #[error("Not a Git repository: {0}")]
    NotAGitRepository(String),

    #[error("Git execution failed: {0}")]
    GitCommandFailed(String),

    #[error("Invalid Task: {0}")]
    InvalidTask(String),

    #[error("Invalid WorkUnit: {0}")]
    InvalidWorkUnit(String),

    #[error("Invalid Proof: {0}")]
    InvalidProof(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Ledger error: {0}")]
    LedgerError(String),

    #[error("Replay divergence: {0}")]
    ReplayDivergence(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, VwlError>;

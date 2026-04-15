use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToriiError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Branch not found: {0}")]
    #[allow(dead_code)]
    BranchNotFound(String),

    #[error("Snapshot error: {0}")]
    Snapshot(String),

    #[error("Mirror error: {0}")]
    Mirror(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, ToriiError>;

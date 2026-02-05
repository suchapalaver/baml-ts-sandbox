use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProvenanceError {
    #[error("provenance storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, ProvenanceError>;

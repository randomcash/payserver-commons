//! Error types for data service operations.

use thiserror::Error;

/// Error types for repository operations.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// Database error.
    #[error("database error: {0}")]
    Database(String),

    /// Entity not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Invalid data (e.g., unsupported enum variant, data corruption).
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// The write is well-formed but the current state refuses it - a unique
    /// key already taken, a row still referenced by another.
    ///
    /// Separate from `Database` because it is the caller's to fix: an API
    /// mapping this to 500 tells a merchant to retry something that will never
    /// succeed, where 409 tells them what to change (RCS-234).
    #[error("conflict: {0}")]
    Conflict(String),
}

/// Result type for repository operations.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

//! Error types for WatsonX-RS

use thiserror::Error;

/// Result type alias for WatsonX operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using WatsonX-RS
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// API errors from WatsonX
    #[error("WatsonX API error: {0}")]
    Api(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid input errors
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Model not found errors
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Project not found errors
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(String),
}

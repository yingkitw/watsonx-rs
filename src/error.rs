//! Error types for WatsonX-RS
//!
//! This module provides comprehensive error handling for WatsonX operations.
//! All errors include descriptive messages with actionable guidance where possible.

use thiserror::Error;

/// Result type alias for WatsonX operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using WatsonX-RS
#[derive(Clone, Debug, Error)]
pub enum Error {
    /// Network-related errors (connection failures, DNS issues, etc.)
    ///
    /// **Possible causes:**
    /// - Network connectivity issues
    /// - Firewall blocking requests
    /// - DNS resolution failures
    /// - Invalid API endpoint URL
    ///
    /// **Suggested actions:**
    /// - Check internet connection
    /// - Verify API endpoint URL is correct
    /// - Check firewall/proxy settings
    /// - Retry the request
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication errors (invalid credentials, expired tokens, etc.)
    ///
    /// **Possible causes:**
    /// - Invalid API key
    /// - Expired access token
    /// - Missing authentication credentials
    /// - Incorrect IAM URL configuration
    ///
    /// **Suggested actions:**
    /// - Verify API key is correct in environment variables or config
    /// - Check that WATSONX_API_KEY is set correctly
    /// - Ensure IAM URL is correct for your region
    /// - Re-authenticate by calling `connect()` again
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// API errors from WatsonX (HTTP errors, API-specific issues)
    ///
    /// **Possible causes:**
    /// - Invalid request parameters
    /// - Model not available
    /// - Project access issues
    /// - API endpoint not found
    /// - Request payload too large
    ///
    /// **Suggested actions:**
    /// - Check the error message for specific details
    /// - Verify model ID is correct and available
    /// - Ensure project ID has access to the requested model
    /// - Check API version compatibility
    /// - Review request parameters
    #[error("WatsonX API error: {0}")]
    Api(String),

    /// Timeout errors (request took too long)
    ///
    /// **Possible causes:**
    /// - Network latency
    /// - Model processing time exceeded timeout
    /// - Server overload
    ///
    /// **Suggested actions:**
    /// - Increase timeout value in configuration
    /// - Retry the request
    /// - Use a faster model
    /// - Reduce max_tokens if generating long responses
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Serialization/deserialization errors (JSON parsing issues)
    ///
    /// **Possible causes:**
    /// - Invalid JSON response from API
    /// - Response format changed
    /// - Malformed request payload
    ///
    /// **Suggested actions:**
    /// - Check API response format
    /// - Update SDK to latest version
    /// - Verify request payload structure
    /// - Report issue if response format is unexpected
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration errors (invalid settings, missing required fields)
    ///
    /// **Possible causes:**
    /// - Missing required environment variables
    /// - Invalid configuration values
    /// - Missing API key or project ID
    ///
    /// **Suggested actions:**
    /// - Set required environment variables (WATSONX_API_KEY, WATSONX_PROJECT_ID)
    /// - Verify configuration values are valid
    /// - Check .env file exists and is properly formatted
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid input errors (bad user input)
    ///
    /// **Possible causes:**
    /// - Empty prompt
    /// - Invalid model ID format
    /// - Out-of-range parameter values
    ///
    /// **Suggested actions:**
    /// - Verify input parameters are valid
    /// - Check parameter ranges (e.g., temperature 0.0-2.0)
    /// - Ensure required fields are provided
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Rate limiting errors (too many requests)
    ///
    /// **Possible causes:**
    /// - API rate limit exceeded
    /// - Too many concurrent requests
    ///
    /// **Suggested actions:**
    /// - Wait before retrying
    /// - Reduce request frequency
    /// - Implement exponential backoff
    /// - Check your API quota/limits
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Model not found errors (invalid or unavailable model)
    ///
    /// **Possible causes:**
    /// - Model ID is incorrect
    /// - Model not available in your region
    /// - Model not accessible with your project
    ///
    /// **Suggested actions:**
    /// - Verify model ID spelling and format
    /// - Use `list_models()` to see available models
    /// - Check model availability in your region
    /// - Ensure project has access to the model
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Project not found errors (invalid project ID)
    ///
    /// **Possible causes:**
    /// - Project ID is incorrect
    /// - Project doesn't exist
    /// - Project access denied
    ///
    /// **Suggested actions:**
    /// - Verify WATSONX_PROJECT_ID is correct
    /// - Check project exists in IBM Cloud
    /// - Verify you have access to the project
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    /// I/O errors (file operations, etc.)
    ///
    /// **Possible causes:**
    /// - File not found
    /// - Permission denied
    /// - Disk full
    ///
    /// **Suggested actions:**
    /// - Check file path is correct
    /// - Verify file permissions
    /// - Check disk space
    #[error("I/O error: {0}")]
    Io(String),
}

impl Error {
    /// Check if this error is retryable
    ///
    /// Returns `true` for errors that might succeed on retry:
    /// - Network errors
    /// - Timeout errors
    /// - Rate limit errors (after waiting)
    /// - Some API errors (5xx server errors)
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::Timeout(_) | Error::RateLimit(_)
        )
    }

    /// Check if this error requires user action
    ///
    /// Returns `true` for errors that need user intervention:
    /// - Authentication errors
    /// - Configuration errors
    /// - Invalid input errors
    /// - Model not found errors
    /// - Project not found errors
    #[must_use]
    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            Error::Authentication(_)
                | Error::Configuration(_)
                | Error::InvalidInput(_)
                | Error::ModelNotFound(_)
                | Error::ProjectNotFound(_)
        )
    }

    /// Get a user-friendly error message with suggestions
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Error::Network(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Check your internet connection and verify the API endpoint URL is correct.",
                    msg
                )
            }
            Error::Authentication(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Verify your WATSONX_API_KEY is set correctly in your environment or .env file.",
                    msg
                )
            }
            Error::Api(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Check the error details above, verify your model ID and project ID are correct.",
                    msg
                )
            }
            Error::Timeout(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Try increasing the timeout value or reducing max_tokens in your configuration.",
                    msg
                )
            }
            Error::Configuration(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Ensure WATSONX_API_KEY and WATSONX_PROJECT_ID are set in your environment or .env file.",
                    msg
                )
            }
            Error::ModelNotFound(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Use list_models() to see available models, or verify the model ID is correct.",
                    msg
                )
            }
            Error::ProjectNotFound(msg) => {
                format!(
                    "{}\n\nTroubleshooting: Verify your WATSONX_PROJECT_ID is correct and you have access to the project.",
                    msg
                )
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(Error::Network("test".to_string()).is_retryable());
        assert!(Error::Timeout("test".to_string()).is_retryable());
        assert!(Error::RateLimit("test".to_string()).is_retryable());
        assert!(!Error::Authentication("test".to_string()).is_retryable());
        assert!(!Error::Configuration("test".to_string()).is_retryable());
        assert!(!Error::InvalidInput("test".to_string()).is_retryable());
        assert!(!Error::Api("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_requires_user_action() {
        assert!(Error::Authentication("test".to_string()).requires_user_action());
        assert!(Error::Configuration("test".to_string()).requires_user_action());
        assert!(Error::InvalidInput("test".to_string()).requires_user_action());
        assert!(Error::ModelNotFound("test".to_string()).requires_user_action());
        assert!(Error::ProjectNotFound("test".to_string()).requires_user_action());
        assert!(!Error::Network("test".to_string()).requires_user_action());
        assert!(!Error::Timeout("test".to_string()).requires_user_action());
        assert!(!Error::Api("test".to_string()).requires_user_action());
    }

    #[test]
    fn test_error_user_message() {
        let network_err = Error::Network("connection failed".to_string());
        let msg = network_err.user_message();
        assert!(msg.contains("connection failed"));
        assert!(msg.contains("Troubleshooting"));

        let auth_err = Error::Authentication("invalid key".to_string());
        let msg = auth_err.user_message();
        assert!(msg.contains("invalid key"));
        assert!(msg.contains("WATSONX_API_KEY"));

        let config_err = Error::Configuration("missing var".to_string());
        let msg = config_err.user_message();
        assert!(msg.contains("missing var"));
        assert!(msg.contains("WATSONX_PROJECT_ID"));

        let model_err = Error::ModelNotFound("bad-model".to_string());
        let msg = model_err.user_message();
        assert!(msg.contains("bad-model"));
        assert!(msg.contains("list_models"));

        let timeout_err = Error::Timeout("request timed out".to_string());
        let msg = timeout_err.user_message();
        assert!(msg.contains("timed out"));
        assert!(msg.contains("timeout"));
    }

    #[test]
    fn test_error_display() {
        let err = Error::Network("test error".to_string());
        assert!(err.to_string().contains("Network error: test error"));

        let err = Error::Authentication("auth failed".to_string());
        assert!(err.to_string().contains("Authentication error: auth failed"));

        let err = Error::Api("api error".to_string());
        assert!(err.to_string().contains("WatsonX API error: api error"));
    }
}

//! Simplified Watson Orchestrate connection helper
//!
//! This module provides a single-step connection initialization that handles
//! all the complexity of configuration, token generation, and client setup.

use crate::error::{Error, Result};
use super::client::OrchestrateClient;
use super::config::OrchestrateConfig;

/// Simplified connection builder for Watson Orchestrate
/// 
/// # Example
/// ```ignore
/// let client = OrchestrateConnection::new()
///     .from_env()
///     .await?;
/// ```
pub struct OrchestrateConnection;

impl OrchestrateConnection {
    /// Create a new connection builder
    pub fn new() -> Self {
        Self
    }

    /// Initialize from environment variables (simplest approach)
    /// 
    /// Required environment variables:
    /// - `WXO_INSTANCE_ID`: Watson Orchestrate instance ID
    /// - `WXO_KEY`: Watson Orchestrate API key
    /// 
    /// Optional environment variables:
    /// - `WXO_REGION`: Region (defaults to us-south)
    /// - `WXO_URL`: Custom base URL
    /// 
    /// # Example
    /// ```ignore
    /// let client = OrchestrateConnection::new()
    ///     .from_env()
    ///     .await?;
    /// ```
    pub async fn from_env(self) -> Result<OrchestrateClient> {
        // Load config from environment
        let config = OrchestrateConfig::from_env()
            .map_err(|e| Error::Configuration(e))?;

        // Get API key
        let api_key = std::env::var("WXO_KEY")
            .or_else(|_| std::env::var("WATSONX_API_KEY"))
            .or_else(|_| std::env::var("IAM_API_KEY"))
            .map_err(|_| Error::Configuration(
                "WXO_KEY environment variable not set. Please set it in your .env file".to_string()
            ))?;

        // Generate token
        let token = OrchestrateClient::generate_jwt_token(&api_key).await?;

        // Create and return authenticated client
        Ok(OrchestrateClient::new(config).with_token(token))
    }

    /// Initialize with explicit parameters (for programmatic use)
    /// 
    /// # Example
    /// ```ignore
    /// let client = OrchestrateConnection::new()
    ///     .with_credentials(
    ///         "instance-id-123",
    ///         "api-key-xyz",
    ///         "us-south"
    ///     )
    ///     .await?;
    /// ```
    pub async fn with_credentials(
        self,
        instance_id: &str,
        api_key: &str,
        region: &str,
    ) -> Result<OrchestrateClient> {
        // Create config
        let config = OrchestrateConfig {
            instance_id: instance_id.to_string(),
            region: region.to_string(),
            base_url: format!(
                "https://{}.watson-orchestrate.cloud.ibm.com/api/v1/",
                region
            ),
        };

        // Generate token
        let token = OrchestrateClient::generate_jwt_token(api_key).await?;

        // Create and return authenticated client
        Ok(OrchestrateClient::new(config).with_token(token))
    }

    /// Initialize with custom base URL (for non-standard deployments)
    /// 
    /// # Example
    /// ```ignore
    /// let client = OrchestrateConnection::new()
    ///     .with_custom_url(
    ///         "instance-id-123",
    ///         "api-key-xyz",
    ///         "https://custom.domain.com/api/v1/"
    ///     )
    ///     .await?;
    /// ```
    pub async fn with_custom_url(
        self,
        instance_id: &str,
        api_key: &str,
        base_url: &str,
    ) -> Result<OrchestrateClient> {
        // Create config with custom URL
        let config = OrchestrateConfig {
            instance_id: instance_id.to_string(),
            region: "custom".to_string(),
            base_url: base_url.to_string(),
        };

        // Generate token
        let token = OrchestrateClient::generate_jwt_token(api_key).await?;

        // Create and return authenticated client
        Ok(OrchestrateClient::new(config).with_token(token))
    }
}

impl Default for OrchestrateConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_builder_creation() {
        let _conn = OrchestrateConnection::new();
        let _conn2 = OrchestrateConnection::default();
    }
}

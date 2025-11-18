//! Simplified WatsonX AI connection helper
//!
//! This module provides a single-step connection initialization that handles
//! all the complexity of configuration, token generation, and client setup.

use crate::config::WatsonxConfig;
use crate::client::WatsonxClient;
use crate::error::Result;

/// Simplified connection builder for WatsonX AI
/// 
/// # Example
/// ```ignore
/// let client = WatsonxConnection::new()
///     .from_env()
///     .await?;
/// ```
pub struct WatsonxConnection;

impl WatsonxConnection {
    /// Create a new connection builder
    pub fn new() -> Self {
        Self
    }

    /// Initialize from environment variables (simplest approach)
    /// 
    /// Required environment variables:
    /// - `WATSONX_API_KEY`: IBM Cloud API key
    /// - `WATSONX_PROJECT_ID`: WatsonX project ID
    /// 
    /// Optional environment variables:
    /// - `WATSONX_API_URL`: API endpoint (defaults to IBM Cloud)
    /// - `IAM_IBM_CLOUD_URL`: IAM endpoint (defaults to IBM Cloud)
    /// - `WATSONX_API_VERSION`: API version (defaults to 2023-05-29)
    /// - `WATSONX_TIMEOUT_SECS`: Request timeout (defaults to 120)
    /// 
    /// # Example
    /// ```ignore
    /// let client = WatsonxConnection::new()
    ///     .from_env()
    ///     .await?;
    /// ```
    pub async fn from_env(self) -> Result<WatsonxClient> {
        // Load config from environment
        let config = WatsonxConfig::from_env()?;

        // Create and connect client
        let mut client = WatsonxClient::new(config)?;
        client.connect().await?;

        Ok(client)
    }

    /// Initialize with explicit parameters (for programmatic use)
    /// 
    /// # Example
    /// ```ignore
    /// let client = WatsonxConnection::new()
    ///     .with_credentials(
    ///         "api-key-xyz",
    ///         "project-id-123"
    ///     )
    ///     .await?;
    /// ```
    pub async fn with_credentials(
        self,
        api_key: &str,
        project_id: &str,
    ) -> Result<WatsonxClient> {
        // Create config
        let config = WatsonxConfig {
            api_key: api_key.to_string(),
            project_id: project_id.to_string(),
            iam_url: "https://iam.cloud.ibm.com".to_string(),
            api_url: "https://us-south.ml.cloud.ibm.com".to_string(),
            api_version: "2023-05-29".to_string(),
            timeout_secs: 120,
        };

        // Create and connect client
        let mut client = WatsonxClient::new(config)?;
        client.connect().await?;

        Ok(client)
    }

    /// Initialize with custom endpoints (for non-standard deployments)
    /// 
    /// # Example
    /// ```ignore
    /// let client = WatsonxConnection::new()
    ///     .with_custom_endpoints(
    ///         "api-key-xyz",
    ///         "project-id-123",
    ///         "https://custom-iam.com",
    ///         "https://custom-api.com"
    ///     )
    ///     .await?;
    /// ```
    pub async fn with_custom_endpoints(
        self,
        api_key: &str,
        project_id: &str,
        iam_url: &str,
        api_url: &str,
    ) -> Result<WatsonxClient> {
        // Create config with custom endpoints
        let config = WatsonxConfig {
            api_key: api_key.to_string(),
            project_id: project_id.to_string(),
            iam_url: iam_url.to_string(),
            api_url: api_url.to_string(),
            api_version: "2023-05-29".to_string(),
            timeout_secs: 120,
        };

        // Create and connect client
        let mut client = WatsonxClient::new(config)?;
        client.connect().await?;

        Ok(client)
    }

    /// Initialize with full configuration (for advanced use)
    /// 
    /// # Example
    /// ```ignore
    /// let client = WatsonxConnection::new()
    ///     .with_config(config)
    ///     .await?;
    /// ```
    pub async fn with_config(self, config: WatsonxConfig) -> Result<WatsonxClient> {
        // Create and connect client
        let mut client = WatsonxClient::new(config)?;
        client.connect().await?;

        Ok(client)
    }
}

impl Default for WatsonxConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_builder_creation() {
        let _conn = WatsonxConnection::new();
        let _conn2 = WatsonxConnection::default();
    }
}

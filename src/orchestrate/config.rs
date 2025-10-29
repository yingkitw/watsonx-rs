//! Watson Orchestrate Configuration
//!
//! Configuration management for Watson Orchestrate operations,
//! including environment variable handling and URL construction.

/// Configuration for WatsonX Orchestrate operations
#[derive(Clone, Debug)]
pub struct OrchestrateConfig {
    pub instance_id: String,
    /// Region (defaults to us-south, can be set via WXO_REGION env var)
    pub region: String,
    /// Base URL (from WXO_URL env var, with {} placeholder for instance_id)
    pub base_url: String,
}

impl OrchestrateConfig {
    /// Create configuration from environment variables
    /// Reads: WXO_INSTANCE_ID (required), WXO_REGION (optional), WXO_URL (optional)
    pub fn from_env() -> Result<Self, String> {
        use std::env;
        
        let instance_id = env::var("WXO_INSTANCE_ID")
            .map_err(|_| "WXO_INSTANCE_ID must be set in environment variables".to_string())?;
        
        let region = env::var("WXO_REGION")
            .unwrap_or_else(|_| "us-south".to_string());
        
        // Read base URL from WXO_URL env var, with fallback to default pattern
        let base_url = env::var("WXO_URL")
            .unwrap_or_else(|_| {
                format!(
                    "https://{}.watson-orchestrate.cloud.ibm.com/api/v1/",
                    region
                )
            });
        
        Ok(Self {
            instance_id,
            region,
            base_url,
        })
    }

    /// Create a new Orchestrate configuration with instance ID
    pub fn new(instance_id: String) -> Self {
        Self {
            instance_id,
            region: "us-south".to_string(),
            base_url: "https://us-south.watson-orchestrate.cloud.ibm.com/api/v1/".to_string(),
        }
    }

    /// Get the base URL with instance ID substituted
    pub fn get_base_url(&self) -> String {
        // Replace {} placeholder with instance_id if present
        self.base_url.replace("{}", &self.instance_id)
    }
}

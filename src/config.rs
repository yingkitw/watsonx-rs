//! WatsonX configuration

use crate::error::{Error, Result};
use crate::models::{DEFAULT_API_URL, DEFAULT_IAM_URL};
use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for WatsonX AI client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatsonxConfig {
    /// IBM Cloud API key for authentication
    pub api_key: String,
    /// WatsonX project ID
    pub project_id: String,
    /// IAM URL for authentication
    pub iam_url: String,
    /// WatsonX API URL
    pub api_url: String,
    /// API version
    pub api_version: String,
    /// Default timeout for requests
    pub timeout_secs: u64,
}

impl WatsonxConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self> {
        #[cfg(feature = "dotenv")]
        dotenvy::dotenv().ok();

        let api_key = env::var("WATSONX_API_KEY")
            .or_else(|_| env::var("API_KEY"))
            .map_err(|_| {
                Error::Configuration(
                    "WATSONX_API_KEY or API_KEY environment variable not found".to_string(),
                )
            })?;
        if api_key.trim().is_empty() {
            return Err(Error::Configuration(
                "WATSONX_API_KEY or API_KEY is set but empty".to_string(),
            ));
        }

        let project_id = env::var("WATSONX_PROJECT_ID")
            .or_else(|_| env::var("PROJECT_ID"))
            .map_err(|_| {
                Error::Configuration(
                    "WATSONX_PROJECT_ID or PROJECT_ID environment variable not found".to_string(),
                )
            })?;
        if project_id.trim().is_empty() {
            return Err(Error::Configuration(
                "WATSONX_PROJECT_ID or PROJECT_ID is set but empty".to_string(),
            ));
        }

        let iam_url = env::var("IAM_IBM_CLOUD_URL")
            .unwrap_or_else(|_| DEFAULT_IAM_URL.to_string());

        let api_url = env::var("WATSONX_API_URL")
            .unwrap_or_else(|_| DEFAULT_API_URL.to_string());

        let api_version = env::var("WATSONX_API_VERSION")
            .unwrap_or_else(|_| "2023-05-29".to_string());

        let timeout_secs = env::var("WATSONX_TIMEOUT_SECS")
            .unwrap_or_else(|_| "120".to_string())
            .parse()
            .unwrap_or(120);

        Ok(Self {
            api_key,
            project_id,
            iam_url,
            api_url,
            api_version,
            timeout_secs,
        })
    }

    /// Create configuration with explicit values
    pub fn new(api_key: String, project_id: String) -> Self {
        Self {
            api_key,
            project_id,
            iam_url: DEFAULT_IAM_URL.to_string(),
            api_url: DEFAULT_API_URL.to_string(),
            api_version: "2023-05-29".to_string(),
            timeout_secs: 120,
        }
    }

    /// Set the IAM URL
    pub fn with_iam_url(mut self, iam_url: String) -> Self {
        self.iam_url = iam_url;
        self
    }

    /// Set the API URL
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.api_url = api_url;
        self
    }

    /// Set the API version
    pub fn with_api_version(mut self, api_version: String) -> Self {
        self.api_version = api_version;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.trim().is_empty() {
            return Err(Error::Configuration("API key cannot be empty".to_string()));
        }

        if self.project_id.trim().is_empty() {
            return Err(Error::Configuration("Project ID cannot be empty".to_string()));
        }

        if self.iam_url.trim().is_empty() {
            return Err(Error::Configuration("IAM URL cannot be empty".to_string()));
        }

        if self.api_url.trim().is_empty() {
            return Err(Error::Configuration("API URL cannot be empty".to_string()));
        }

        Ok(())
    }
}

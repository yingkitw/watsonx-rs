//! WatsonX Orchestrate client implementation
//!
//! This module provides the main client for interacting with WatsonX Orchestrate services,
//! including custom assistants, document collections, and chat functionality.

use crate::error::{Error, Result};
use super::types::*;
use super::config::OrchestrateConfig;
use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// WatsonX Orchestrate client for managing custom assistants and document collections
pub struct OrchestrateClient {
    pub(crate) config: OrchestrateConfig,
    pub(crate) access_token: Option<String>,
    pub(crate) client: Client,
}

impl OrchestrateClient {
    /// Create a new Orchestrate client (matches wxo-client-main pattern)
    pub fn new(config: OrchestrateConfig) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(300)) // 5 minute timeout for streaming
            .tcp_keepalive(Duration::from_secs(60))
            .http1_title_case_headers()
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config,
            access_token: None,
            client,
        }
    }

    /// Set the access token for authentication
    pub fn with_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }

    /// Set the access token for authentication (mutable)
    pub fn set_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Get the current configuration
    pub fn config(&self) -> &OrchestrateConfig {
        &self.config
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }

    /// Generate IAM Access Token from Watson Orchestrate API key
    /// This is required for Watson Orchestrate SaaS authentication
    pub async fn generate_jwt_token(api_key: &str) -> Result<String> {
        let client = reqwest::Client::new();
        let body = format!(
            "grant_type=urn:ibm:params:oauth:grant-type:apikey&apikey={}",
            api_key
        );

        let response = client
            .post("https://iam.cloud.ibm.com/identity/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to generate IAM token: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to generate IAM token: {} - {}",
                status, error_text
            )));
        }

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(format!("Failed to parse IAM token response: {}", e)))?;

        Ok(token_response.access_token)
    }

    // ============================================================================
    // Custom Assistant Management
    // ============================================================================

    /// List all custom assistants
    pub async fn list_assistants(&self) -> Result<Vec<CustomAssistant>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants",
            self.config.get_base_url()
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list assistants: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try to parse as direct array first
        if let Ok(assistants) = serde_json::from_str::<Vec<CustomAssistant>>(&text) {
            return Ok(assistants);
        }

        // Try to extract from wrapper object
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(assistants_array) = obj.get("assistants").and_then(|a| a.as_array()) {
                let assistants: Result<Vec<CustomAssistant>> = assistants_array
                    .iter()
                    .map(|item| {
                        serde_json::from_value::<CustomAssistant>(item.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return assistants;
            }
        }

        Ok(Vec::new())
    }

    /// Send multiple messages in a batch
    pub async fn send_batch_messages(&self, request: BatchMessageRequest) -> Result<BatchMessageResponse> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/batch/messages", base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to send batch messages: {} - {}",
                status, error_text
            )));
        }

        let batch_response: BatchMessageResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(batch_response)
    }

    // ============================================================================
    // Skills Management
    // ============================================================================

    /// List all skills
    pub async fn list_skills(&self) -> Result<Vec<Skill>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/skills", base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list skills: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        if let Ok(skills) = serde_json::from_str::<Vec<Skill>>(&text) {
            return Ok(skills);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(skills_array) = obj.get("skills").and_then(|s| s.as_array()) {
                let skills: Result<Vec<Skill>> = skills_array
                    .iter()
                    .map(|skill| {
                        serde_json::from_value::<Skill>(skill.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return skills;
            }
        }

        Ok(Vec::new())
    }

    /// Get a specific skill by ID
    pub async fn get_skill(&self, skill_id: &str) -> Result<Skill> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/skills/{}", base_url, skill_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to get skill {}: {} - {}",
                skill_id, status, error_text
            )));
        }

        let skill: Skill = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(skill)
    }
}

// Helper structs for SSE parsing
#[derive(serde::Deserialize)]
struct ChatChunk {
    content: Option<String>,
    metadata: Option<HashMap<String, Value>>,
}

#[derive(serde::Deserialize)]
struct EventData {
    event: String,
    data: Value,
}

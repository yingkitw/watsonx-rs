//! Agent management operations

use crate::error::{Error, Result};
use super::types::Agent;
use super::OrchestrateClient;

impl OrchestrateClient {
    /// List all agents (Watson Orchestrate API)
    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        
        // Try different endpoint paths
        let endpoints = vec![
            format!("{}/agents", base_url),
            format!("{}/orchestrate/agents", base_url),
            format!("{}/assistants", base_url),
            format!("{}/orchestrate/assistants", base_url),
        ];

        for url in endpoints {
            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .header("X-Instance-ID", &self.config.instance_id)
                .send()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;

            if response.status().is_success() {
                // Parse the JSON array response directly
                let agents: Vec<Agent> = response
                    .json()
                    .await
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                return Ok(agents);
            }
        }

        // If all endpoints failed, return error with diagnostic info
        Err(Error::Api(format!(
            "Failed to list agents: All endpoint paths returned 404. Tried: {}/agents, {}/orchestrate/agents, {}/assistants, {}/orchestrate/assistants",
            base_url, base_url, base_url, base_url
        )))
    }

    /// Get a specific agent by ID
    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/agents/{}", base_url, agent_id);

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
                "Failed to get agent {}: {} - {}",
                agent_id, status, error_text
            )));
        }

        let agent: Agent = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(agent)
    }
}

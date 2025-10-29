//! Run management operations

use crate::error::{Error, Result};
use super::types::RunInfo;
use super::OrchestrateClient;

impl OrchestrateClient {
    /// Get information about a specific run
    pub async fn get_run(&self, run_id: &str) -> Result<RunInfo> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/runs/{}", base_url, run_id);

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
                "Failed to get run {}: {} - {}",
                run_id, status, error_text
            )));
        }

        let run: RunInfo = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(run)
    }

    /// List all runs for an agent
    pub async fn list_runs(&self, agent_id: Option<&str>) -> Result<Vec<RunInfo>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = if let Some(agent_id) = agent_id {
            format!("{}/runs?agent_id={}", base_url, agent_id)
        } else {
            format!("{}/runs", base_url)
        };

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
                "Failed to list runs: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        if let Ok(runs) = serde_json::from_str::<Vec<RunInfo>>(&text) {
            return Ok(runs);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(runs_array) = obj.get("runs").and_then(|r| r.as_array()) {
                let runs: Result<Vec<RunInfo>> = runs_array
                    .iter()
                    .map(|run| {
                        serde_json::from_value::<RunInfo>(run.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return runs;
            }
        }

        Ok(Vec::new())
    }

    /// Cancel a running execution
    pub async fn cancel_run(&self, run_id: &str) -> Result<()> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/runs/{}/cancel", base_url, run_id);

        let response = self
            .client
            .post(&url)
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
                "Failed to cancel run {}: {} - {}",
                run_id, status, error_text
            )));
        }

        Ok(())
    }
}

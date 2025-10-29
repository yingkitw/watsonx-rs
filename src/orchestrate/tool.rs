//! Tool management operations

use crate::error::{Error, Result};
use super::types::{Tool, ToolExecutionRequest, ToolExecutionResult, ToolUpdateRequest, ToolTestRequest, ToolTestResult, ToolExecutionHistory, ToolVersion};
use super::OrchestrateClient;

impl OrchestrateClient {
    /// List all tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/tools", base_url);

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
                "Failed to list tools: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        if let Ok(tools) = serde_json::from_str::<Vec<Tool>>(&text) {
            return Ok(tools);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(tools_array) = obj.get("tools").and_then(|t| t.as_array()) {
                let tools: Result<Vec<Tool>> = tools_array
                    .iter()
                    .map(|tool| {
                        serde_json::from_value::<Tool>(tool.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return tools;
            }
        }

        Ok(Vec::new())
    }

    /// Get a specific tool by ID
    pub async fn get_tool(&self, tool_id: &str) -> Result<Tool> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/tools/{}", base_url, tool_id);

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
                "Failed to get tool {}: {} - {}",
                tool_id, status, error_text
            )));
        }

        let tool: Tool = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(tool)
    }

    /// Execute a tool directly
    pub async fn execute_tool(&self, request: ToolExecutionRequest) -> Result<ToolExecutionResult> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/tools/{}/execute", base_url, request.tool_id);

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
                "Failed to execute tool: {} - {}",
                status, error_text
            )));
        }

        let result: ToolExecutionResult = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(result)
    }

    /// Update a tool
    pub async fn update_tool(&self, tool_id: &str, request: ToolUpdateRequest) -> Result<Tool> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/orchestrate/tools/{}", base_url, tool_id);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("X-Instance-ID", &self.config.instance_id)
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
                "Failed to update tool: {} - {}",
                status, error_text
            )));
        }

        let tool: Tool = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(tool)
    }

    /// Delete a tool
    pub async fn delete_tool(&self, tool_id: &str) -> Result<()> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/orchestrate/tools/{}", base_url, tool_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Instance-ID", &self.config.instance_id)
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
                "Failed to delete tool: {} - {}",
                status, error_text
            )));
        }

        Ok(())
    }

    /// Test a tool with sample input
    pub async fn test_tool(&self, request: ToolTestRequest) -> Result<ToolTestResult> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/orchestrate/tools/{}/test", base_url, request.tool_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("X-Instance-ID", &self.config.instance_id)
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
                "Failed to test tool: {} - {}",
                status, error_text
            )));
        }

        let result: ToolTestResult = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(result)
    }

    /// Get tool execution history
    pub async fn get_tool_execution_history(&self, tool_id: &str, limit: Option<u32>) -> Result<Vec<ToolExecutionHistory>> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let mut url = format!("{}/orchestrate/tools/{}/execution-history", base_url, tool_id);
        
        if let Some(l) = limit {
            url.push_str(&format!("?limit={}", l));
        }

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("X-Instance-ID", &self.config.instance_id)
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
                "Failed to get tool execution history: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if let Ok(history) = serde_json::from_str::<Vec<ToolExecutionHistory>>(&text) {
            return Ok(history);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(history_array) = obj.get("history").and_then(|h| h.as_array()) {
                let history: Result<Vec<ToolExecutionHistory>> = history_array
                    .iter()
                    .map(|item| {
                        serde_json::from_value::<ToolExecutionHistory>(item.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return history;
            }
        }

        Ok(Vec::new())
    }

    /// Get tool versions
    pub async fn get_tool_versions(&self, tool_id: &str) -> Result<Vec<ToolVersion>> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/orchestrate/tools/{}/versions", base_url, tool_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("X-Instance-ID", &self.config.instance_id)
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
                "Failed to get tool versions: {} - {}",
                status, error_text
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if let Ok(versions) = serde_json::from_str::<Vec<ToolVersion>>(&text) {
            return Ok(versions);
        }

        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(versions_array) = obj.get("versions").and_then(|v| v.as_array()) {
                let versions: Result<Vec<ToolVersion>> = versions_array
                    .iter()
                    .map(|item| {
                        serde_json::from_value::<ToolVersion>(item.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return versions;
            }
        }

        Ok(Vec::new())
    }
}

//! Thread management operations

use crate::error::{Error, Result};
use super::types::{ThreadInfo, Message};
use super::OrchestrateClient;
use serde_json::Value;

#[derive(serde::Deserialize)]
struct EventData {
    event: String,
    data: Value,
}

impl OrchestrateClient {
    /// List all threads for an agent
    pub async fn list_threads(&self, agent_id: Option<&str>) -> Result<Vec<ThreadInfo>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = if let Some(agent_id) = agent_id {
            format!("{}/threads?agent_id={}", base_url, agent_id)
        } else {
            format!("{}/threads", base_url)
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
                "Failed to list threads: {} - {}",
                status, error_text
            )));
        }

        let threads: Vec<ThreadInfo> = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(threads)
    }

    /// Create a new thread for conversation
    pub async fn create_thread(&self, agent_id: Option<&str>) -> Result<ThreadInfo> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/threads", base_url);

        let mut body = serde_json::json!({});
        if let Some(agent_id) = agent_id {
            body["agent_id"] = serde_json::json!(agent_id);
        }

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
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
                "Failed to create thread: {} - {}",
                status, error_text
            )));
        }

        let thread: ThreadInfo = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(thread)
    }

    /// Delete a thread
    pub async fn delete_thread(&self, thread_id: &str) -> Result<()> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/threads/{}", base_url, thread_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", api_key))
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
                "Failed to delete thread {}: {} - {}",
                thread_id, status, error_text
            )));
        }

        Ok(())
    }

    /// Get conversation history from a thread
    pub async fn get_thread_messages(&self, thread_id: &str) -> Result<Vec<Message>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/threads/{}/messages", base_url, thread_id);

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
                "Failed to get thread messages: {} - {}",
                status, error_text
            )));
        }

        // Try to parse as direct array first
        let text = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if let Ok(messages) = serde_json::from_str::<Vec<Message>>(&text) {
            return Ok(messages);
        }

        // Try to extract from wrapper object
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(messages_array) = obj.get("messages").and_then(|m| m.as_array()) {
                let messages: Result<Vec<Message>> = messages_array
                    .iter()
                    .map(|item| {
                        serde_json::from_value::<Message>(item.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return messages;
            }
        }

        // Fallback to empty vec
        Ok(Vec::new())
    }
}

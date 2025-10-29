//! WatsonX Orchestrate client implementation
//!
//! This module provides the main client for interacting with WatsonX Orchestrate services,
//! including custom assistants, document collections, and chat functionality.

use crate::error::{Error, Result};
use crate::orchestrate_types::*;
use futures::StreamExt;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// WatsonX Orchestrate client for managing custom assistants and document collections
pub struct OrchestrateClient {
    config: OrchestrateConfig,
    access_token: Option<String>,
    client: Client,
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

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
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

        let assistants_response: AssistantsResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(assistants_response.assistants)
    }

    /// List all agents (Watson Orchestrate API - matches wxo-client implementation)
    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/agents", base_url);

        let response = self
            .client
            .get(&url)
            .header("IAM-API_KEY", api_key)
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
                "Failed to list agents: {} - {}",
                status, error_text
            )));
        }

        // Parse the JSON array response directly
        let agents: Vec<Agent> = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(agents)
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
            .header("IAM-API_KEY", api_key)
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

    /// Get thread messages/history for a conversation
    pub async fn get_thread_messages(&self, thread_id: &str) -> Result<Vec<Message>> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/threads/{}/messages", base_url, thread_id);

        let response = self
            .client
            .get(&url)
            .header("IAM-API_KEY", api_key)
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
                "Failed to get thread messages {}: {} - {}",
                thread_id, status, error_text
            )));
        }

        // Try to parse as Vec<Message> first, then try to extract from wrapper
        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try direct parsing first
        if let Ok(messages) = serde_json::from_str::<Vec<Message>>(&text) {
            return Ok(messages);
        }

        // Try parsing as object with messages field
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(messages_array) = obj.get("messages").and_then(|m| m.as_array()) {
                let messages: Result<Vec<Message>> = messages_array
                    .iter()
                    .map(|msg| {
                        serde_json::from_value::<Message>(msg.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return messages;
            }
        }

        // If all else fails, return empty vec (API may not support this endpoint)
        Ok(Vec::new())
    }

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
            .header("IAM-API_KEY", api_key)
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

    /// Send a message to an agent and get response (matches wxo-client pattern)
    /// Uses /runs/stream endpoint and maintains thread_id for conversation continuity
    pub async fn send_message(&self, agent_id: &str, message: &str, thread_id: Option<String>) -> Result<(String, Option<String>)> {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/runs/stream", base_url);

        let payload = MessagePayload {
            message: Message {
                role: "user".to_string(),
                content: message.to_string(),
            },
            additional_properties: HashMap::new(),
            context: HashMap::new(),
            agent_id: agent_id.to_string(),
            thread_id: thread_id.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("IAM-API_KEY", api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
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
                "Failed to send message: {} - {}",
                status, error_text
            )));
        }

        // Parse streaming response to get the final answer and thread_id
        let text = response.text().await.map_err(|e| Error::Network(e.to_string()))?;
        let mut answer = String::new();
        let mut new_thread_id = thread_id;

        for line in text.lines() {
            if !line.is_empty() {
                if let Ok(event_data) = serde_json::from_str::<EventData>(&line) {
                    // Look for message.created event for the final answer
                    if event_data.event == "message.created" {
                        if let Some(data_obj) = event_data.data.as_object() {
                            if let Some(message_obj) = data_obj.get("message") {
                                if let Some(content_array) = message_obj.get("content").and_then(|c| c.as_array()) {
                                    if let Some(first_content) = content_array.first() {
                                        if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                            answer = text.to_string();
                                        }
                                    }
                                }
                            }
                            if let Some(tid) = data_obj.get("thread_id").and_then(|t| t.as_str()) {
                                new_thread_id = Some(tid.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }

        Ok((answer, new_thread_id))
    }

    /// Stream response from an agent (matches wxo-client pattern)
    pub async fn stream_message<F>(
        &self,
        agent_id: &str,
        message: &str,
        thread_id: Option<String>,
        mut callback: F,
    ) -> Result<Option<String>>
    where
        F: FnMut(String) -> Result<()>,
    {
        let api_key = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (API key) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/runs/stream", base_url);

        let payload = MessagePayload {
            message: Message {
                role: "user".to_string(),
                content: message.to_string(),
            },
            additional_properties: HashMap::new(),
            context: HashMap::new(),
            agent_id: agent_id.to_string(),
            thread_id: thread_id.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("IAM-API_KEY", api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("X-Accel-Buffering", "no") // Disable proxy buffering
            .json(&payload)
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
                "Failed to stream message: {} - {}",
                status, error_text
            )));
        }

        // Process streaming response incrementally - process chunks as they arrive
        // Use stream() to get real-time chunks with better control over buffering
        let mut stream = response.bytes_stream();
        let mut buffer = Vec::<u8>::new();
        let mut new_thread_id = thread_id;
        let mut chunk_count = 0;

        // Process stream chunks in real-time - as each chunk arrives from the server
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| Error::Network(e.to_string()))?;
            chunk_count += 1;

            // Small delay to simulate real-time processing and prevent buffering
            if chunk_count > 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            // Append chunk bytes to buffer
            buffer.extend_from_slice(&chunk);

            // Process all complete lines from buffer immediately - process ALL available lines
            // Orchestrate SSE format: one JSON object per line (newline delimited)
            loop {
                // Find newline position in byte buffer
                let newline_pos = buffer.iter().position(|&b| b == b'\n');

                if let Some(newline_pos) = newline_pos {
                    // Extract line bytes
                    let line_bytes = buffer[..newline_pos].to_vec();
                    // Remove processed line from buffer
                    buffer = buffer[newline_pos + 1..].to_vec();

                    // Convert to string (handle invalid UTF-8 gracefully)
                    if let Ok(line) = String::from_utf8(line_bytes) {
                        let trimmed = line.trim();

                        if !trimmed.is_empty() {
                            // Parse JSON event line - try to extract text delta immediately
                            if let Ok(event_data) = serde_json::from_str::<EventData>(trimmed) {
                                // Look for message.delta events for streaming
                                if event_data.event == "message.delta" {
                                    if let Some(data_obj) = event_data.data.as_object() {
                                        // Try multiple paths to find the text content
                                        // Path 1: data.delta.content[0].text (nested delta - primary)
                                        if let Some(delta_obj) = data_obj.get("delta").and_then(|d| d.as_object()) {
                                            if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                                if let Some(first_content) = content_array.first() {
                                                    if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                        callback(text.to_string())?;
                                                    }
                                                }
                                            }
                                        }
                                        // Path 2: data.content[0].text (direct content - fallback)
                                        else if let Some(content_array) = data_obj.get("content").and_then(|c| c.as_array()) {
                                            if let Some(first_content) = content_array.first() {
                                                if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                    callback(text.to_string())?;
                                                }
                                            }
                                        }
                                        // Update thread_id if provided
                                        if let Some(tid) = data_obj.get("thread_id").and_then(|t| t.as_str()) {
                                            new_thread_id = Some(tid.to_string());
                                        }
                                    }
                                }
                                // Also look for message.created event to capture final thread_id
                                else if event_data.event == "message.created" {
                                    if let Some(data_obj) = event_data.data.as_object() {
                                        if let Some(tid) = data_obj.get("thread_id").and_then(|t| t.as_str()) {
                                            new_thread_id = Some(tid.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // No complete line yet - wait for more chunks
                    break;
                }
            }
        }

        // Process any remaining buffer content (final partial line, if any)
        if !buffer.is_empty() {
            if let Ok(line) = String::from_utf8(buffer) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(event_data) = serde_json::from_str::<EventData>(trimmed) {
                        if event_data.event == "message.delta" {
                            if let Some(data_obj) = event_data.data.as_object() {
                                // Try multiple paths to find the text content
                                // Path 1: data.delta.content[0].text (nested delta - primary)
                                if let Some(delta_obj) = data_obj.get("delta").and_then(|d| d.as_object()) {
                                    if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                        if let Some(first_content) = content_array.first() {
                                            if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                callback(text.to_string())?;
                                            }
                                        }
                                    }
                                }
                                // Path 2: data.content[0].text (direct content - fallback)
                                else if let Some(content_array) = data_obj.get("content").and_then(|c| c.as_array()) {
                                    if let Some(first_content) = content_array.first() {
                                        if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                            callback(text.to_string())?;
                                        }
                                    }
                                }
                                if let Some(tid) = data_obj.get("thread_id").and_then(|t| t.as_str()) {
                                    new_thread_id = Some(tid.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(new_thread_id)
    }

    /// Get a specific custom assistant by ID
    pub async fn get_assistant(&self, assistant_id: &str) -> Result<CustomAssistant> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants/{}",
            self.config.get_base_url(), assistant_id
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
                "Failed to get assistant {}: {} - {}",
                assistant_id, status, error_text
            )));
        }

        let assistant: CustomAssistant = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(assistant)
    }

    /// Create a new custom assistant
    pub async fn create_assistant(&self, request: CreateAssistantRequest) -> Result<CustomAssistant> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants",
            self.config.get_base_url()
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to create assistant: {} - {}",
                status, error_text
            )));
        }

        let assistant: CustomAssistant = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(assistant)
    }

    /// Update an existing custom assistant
    pub async fn update_assistant(
        &self,
        assistant_id: &str,
        request: UpdateAssistantRequest,
    ) -> Result<CustomAssistant> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants/{}",
            self.config.get_base_url(), assistant_id
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to update assistant {}: {} - {}",
                assistant_id, status, error_text
            )));
        }

        let assistant: CustomAssistant = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(assistant)
    }

    /// Delete a custom assistant
    pub async fn delete_assistant(&self, assistant_id: &str) -> Result<()> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants/{}",
            self.config.get_base_url(), assistant_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to delete assistant {}: {} - {}",
                assistant_id, status, error_text
            )));
        }

        Ok(())
    }

    // ============================================================================
    // Chat Functionality
    // ============================================================================

    /// Send a chat message to an assistant
    pub async fn send_chat_message(
        &self,
        assistant_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants/{}/chat",
            self.config.get_base_url(), assistant_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to send chat message: {} - {}",
                status, error_text
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(chat_response)
    }

    /// Send a streaming chat message to an assistant
    pub async fn send_chat_message_stream<F>(
        &self,
        assistant_id: &str,
        request: ChatRequest,
        mut callback: F,
    ) -> Result<ChatResponse>
    where
        F: FnMut(String) -> Result<()>,
    {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/assistants/{}/chat?stream=true",
            self.config.get_base_url(), assistant_id
        );

        let session_id = request.session_id.clone();
        let mut request_body = request;
        request_body.stream = true;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request_body)
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
                "Failed to send streaming chat message: {} - {}",
                status, error_text
            )));
        }

        // Process streaming response
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_response = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::Network(e.to_string()))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            buffer.push_str(&chunk_str);

            // Process complete lines
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                if line.starts_with("data: ") {
                    let data = &line[6..]; // Remove "data: " prefix
                    if data == "[DONE]" {
                        break;
                    }

                    // Parse JSON data
                    if let Ok(chat_chunk) = serde_json::from_str::<ChatChunk>(data) {
                        if let Some(content) = chat_chunk.content {
                            full_response.push_str(&content);
                            callback(content)?;
                        }
                    }
                }
            }
        }

        // Create final response
        let final_response = ChatResponse {
            message: full_response,
            session_id: session_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            message_id: Uuid::new_v4().to_string(),
            metadata: HashMap::new(),
            tool_calls: None,
        };

        Ok(final_response)
    }

    // ============================================================================
    // Document Collection Management
    // ============================================================================

    /// List all document collections
    pub async fn list_collections(&self) -> Result<Vec<DocumentCollection>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        let url = format!("{}/collections", base_url);

        let response = self
            .client
            .get(&url)
            .header("IAM-API_KEY", access_token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            
            // Return empty collection for 404 (endpoint not available)
            if status == 404 {
                return Ok(Vec::new());
            }
            
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list collections: {} - {}",
                status, error_text
            )));
        }

        // Try flexible parsing
        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try direct parsing first
        if let Ok(collections) = serde_json::from_str::<Vec<DocumentCollection>>(&text) {
            return Ok(collections);
        }

        // Try parsing as CollectionsResponse
        if let Ok(collections_response) = serde_json::from_str::<CollectionsResponse>(&text) {
            return Ok(collections_response.collections);
        }

        // Try parsing as object with collections field
        if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(collections_array) = obj.get("collections").and_then(|c| c.as_array()) {
                let collections: Result<Vec<DocumentCollection>> = collections_array
                    .iter()
                    .map(|col| {
                        serde_json::from_value::<DocumentCollection>(col.clone())
                            .map_err(|e| Error::Serialization(e.to_string()))
                    })
                    .collect();
                return collections;
            }
        }

        // If all else fails, return empty vec
        Ok(Vec::new())
    }

    /// Create a new document collection
    pub async fn create_collection(&self, request: CreateCollectionRequest) -> Result<DocumentCollection> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections",
            self.config.get_base_url()
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to create collection: {} - {}",
                status, error_text
            )));
        }

        let collection: DocumentCollection = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(collection)
    }

    /// Add documents to a collection
    pub async fn add_documents(
        &self,
        collection_id: &str,
        request: AddDocumentsRequest,
    ) -> Result<Vec<Document>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}/documents",
            self.config.get_base_url(), collection_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to add documents: {} - {}",
                status, error_text
            )));
        }

        let documents_response: DocumentsResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(documents_response.documents)
    }

    /// Search documents in a collection
    pub async fn search_documents(
        &self,
        collection_id: &str,
        request: SearchRequest,
    ) -> Result<SearchResponse> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}/search",
            self.config.get_base_url(), collection_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to search documents: {} - {}",
                status, error_text
            )));
        }

        let search_response: SearchResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(search_response)
    }

    /// Delete a document collection
    pub async fn delete_collection(&self, collection_id: &str) -> Result<()> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}",
            self.config.get_base_url(), collection_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to delete collection {}: {} - {}",
                collection_id, status, error_text
            )));
        }

        Ok(())
    }

    /// Get a specific document collection by ID
    pub async fn get_collection(&self, collection_id: &str) -> Result<DocumentCollection> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}",
            self.config.get_base_url(), collection_id
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
                "Failed to get collection {}: {} - {}",
                collection_id, status, error_text
            )));
        }

        let collection: DocumentCollection = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(collection)
    }

    /// Get a specific document by ID from a collection
    pub async fn get_document(&self, collection_id: &str, document_id: &str) -> Result<Document> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}/documents/{}",
            self.config.get_base_url(), collection_id, document_id
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
                "Failed to get document {} from collection {}: {} - {}",
                document_id, collection_id, status, error_text
            )));
        }

        let document: Document = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(document)
    }

    /// Delete a document from a collection
    pub async fn delete_document(&self, collection_id: &str, document_id: &str) -> Result<()> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token first.".to_string())
        })?;

        let url = format!(
            "{}/v1/collections/{}/documents/{}",
            self.config.get_base_url(), collection_id, document_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", access_token))
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
                "Failed to delete document {} from collection {}: {} - {}",
                document_id, collection_id, status, error_text
            )));
        }

        Ok(())
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
            .header("IAM-API_KEY", api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            
            // Return empty skills for 404 (endpoint not available)
            if status == 404 {
                return Ok(Vec::new());
            }
            
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list skills: {} - {}",
                status, error_text
            )));
        }

        // Try to parse as Vec<Skill> first, then try to extract from wrapper
        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try direct parsing first
        if let Ok(skills) = serde_json::from_str::<Vec<Skill>>(&text) {
            return Ok(skills);
        }

        // Try parsing as object with skills field
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

        // If all else fails, return empty vec
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
            .header("IAM-API_KEY", api_key)
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

    // ============================================================================
    // Tools Management
    // ============================================================================

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
            .header("IAM-API_KEY", api_key)
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

        // Try to parse as Vec<Tool> first, then try to extract from wrapper
        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try direct parsing first
        if let Ok(tools) = serde_json::from_str::<Vec<Tool>>(&text) {
            return Ok(tools);
        }

        // Try parsing as object with tools field
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

        // If all else fails, return empty vec
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
            .header("IAM-API_KEY", api_key)
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

    // ============================================================================
    // Advanced Capabilities - Runs and Execution
    // ============================================================================

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
            .header("IAM-API_KEY", api_key)
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
            .header("IAM-API_KEY", api_key)
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

        // Try to parse as Vec<RunInfo> first, then try to extract from wrapper
        let text = response
            .text()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Try direct parsing first
        if let Ok(runs) = serde_json::from_str::<Vec<RunInfo>>(&text) {
            return Ok(runs);
        }

        // Try parsing as object with runs field
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

        // If all else fails, return empty vec
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
            .header("IAM-API_KEY", api_key)
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
            .header("IAM-API_KEY", api_key)
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
            .header("IAM-API_KEY", api_key)
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
            .header("IAM-API_KEY", api_key)
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
            .header("IAM-API_KEY", api_key)
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
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Deserialize)]
struct AssistantsResponse {
    assistants: Vec<CustomAssistant>,
}

#[derive(Deserialize)]
struct CollectionsResponse {
    collections: Vec<DocumentCollection>,
}

#[derive(Deserialize)]
struct DocumentsResponse {
    documents: Vec<Document>,
}

#[derive(Deserialize)]
struct ChatChunk {
    content: Option<String>,
    metadata: Option<HashMap<String, Value>>,
}

#[derive(Deserialize)]
struct EventData {
    event: String,
    data: Value,
}

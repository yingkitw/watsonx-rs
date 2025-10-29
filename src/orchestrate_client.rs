//! WatsonX Orchestrate client implementation
//!
//! This module provides the main client for interacting with WatsonX Orchestrate services,
//! including custom assistants, document collections, and chat functionality.

use crate::error::{Error, Result};
use crate::orchestrate_types::*;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
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
        let client = Client::new();

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

        // Process streaming response
        let text = response.text().await.map_err(|e| Error::Network(e.to_string()))?;
        let mut new_thread_id = thread_id;

        for line in text.lines() {
            if !line.is_empty() {
                if let Ok(event_data) = serde_json::from_str::<EventData>(&line) {
                    // Look for message.delta events for streaming
                    if event_data.event == "message.delta" {
                        if let Some(data_obj) = event_data.data.as_object() {
                            if let Some(delta_obj) = data_obj.get("delta") {
                                if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                    if let Some(first_content) = content_array.first() {
                                        if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                            callback(text.to_string())?;
                                        }
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

        let url = format!(
            "{}/v1/collections",
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
                "Failed to list collections: {} - {}",
                status, error_text
            )));
        }

        let collections_response: CollectionsResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        Ok(collections_response.collections)
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

// Re-export futures for streaming
use futures::StreamExt;

//! Chat and messaging operations

use crate::error::{Error, Result};
use super::types::{Message, MessagePayload, ChatWithDocsRequest, ChatWithDocsResponse, ChatWithDocsStatus};
use super::OrchestrateClient;
use std::collections::HashMap;
use serde_json::Value;
use futures::StreamExt;

#[derive(serde::Deserialize)]
struct EventData {
    event: String,
    data: Value,
}

impl OrchestrateClient {
    /// Send a message to an agent and get response (matches wxo-client pattern)
    /// Uses /runs/stream endpoint and maintains thread_id for conversation continuity
    pub async fn send_message(&self, agent_id: &str, message: &str, thread_id: Option<String>) -> Result<(String, Option<String>)> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
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
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("X-Instance-ID", &self.config.instance_id)
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

        let text = response.text().await.map_err(|e| Error::Network(e.to_string()))?;
        let mut answer = String::new();
        let mut new_thread_id = thread_id;

        for line in text.lines() {
            if !line.is_empty() {
                if let Ok(event_data) = serde_json::from_str::<EventData>(&line) {
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
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
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
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .header("X-Accel-Buffering", "no")
            .header("X-Instance-ID", &self.config.instance_id)
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

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::<u8>::new();
        let mut new_thread_id = thread_id;
        let mut chunk_count = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| Error::Network(e.to_string()))?;
            chunk_count += 1;

            if chunk_count > 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }

            buffer.extend_from_slice(&chunk);

            loop {
                let newline_pos = buffer.iter().position(|&b| b == b'\n');

                if let Some(newline_pos) = newline_pos {
                    let line_bytes = buffer[..newline_pos].to_vec();
                    buffer = buffer[newline_pos + 1..].to_vec();

                    if let Ok(line) = String::from_utf8(line_bytes) {
                        let trimmed = line.trim();

                        if !trimmed.is_empty() {
                            if let Ok(event_data) = serde_json::from_str::<EventData>(trimmed) {
                                if event_data.event == "message.delta" {
                                    if let Some(data_obj) = event_data.data.as_object() {
                                        if let Some(delta_obj) = data_obj.get("delta").and_then(|d| d.as_object()) {
                                            if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                                if let Some(first_content) = content_array.first() {
                                                    if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                        callback(text.to_string())?;
                                                    }
                                                }
                                            }
                                        } else if let Some(content_array) = data_obj.get("content").and_then(|c| c.as_array()) {
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
                                } else if event_data.event == "message.created" {
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
                    break;
                }
            }
        }

        if !buffer.is_empty() {
            if let Ok(line) = String::from_utf8(buffer) {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(event_data) = serde_json::from_str::<EventData>(trimmed) {
                        if event_data.event == "message.delta" {
                            if let Some(data_obj) = event_data.data.as_object() {
                                if let Some(delta_obj) = data_obj.get("delta").and_then(|d| d.as_object()) {
                                    if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                        if let Some(first_content) = content_array.first() {
                                            if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                callback(text.to_string())?;
                                            }
                                        }
                                    }
                                } else if let Some(content_array) = data_obj.get("content").and_then(|c| c.as_array()) {
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

    /// Get the status of chat with documents knowledge base for a thread
    pub async fn get_chat_with_docs_status(&self, agent_id: &str, thread_id: &str) -> Result<ChatWithDocsStatus> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        
        let endpoints = vec![
            format!("{}/orchestrate/agents/{}/threads/{}/chat_with_docs_status", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/chat_with_docs_status", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/chat_with_docs/status", base_url, agent_id, thread_id),
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
                let status: ChatWithDocsStatus = response
                    .json()
                    .await
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                return Ok(status);
            }
        }

        Err(Error::Api(format!(
            "Failed to get chat with docs status: All endpoint paths returned 404. Chat with documents may not be available in this instance."
        )))
    }

    /// Send a message with document context (chat with documents)
    pub async fn chat_with_docs(&self, agent_id: &str, thread_id: &str, request: ChatWithDocsRequest) -> Result<ChatWithDocsResponse> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        
        let endpoints = vec![
            format!("{}/orchestrate/agents/{}/threads/{}/chat_with_docs", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/chat_with_docs", base_url, agent_id, thread_id),
            format!("{}/orchestrate/agents/{}/threads/{}/runs/stream", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/runs/stream", base_url, agent_id, thread_id),
        ];

        for url in endpoints {
            let payload = if url.contains("chat_with_docs") {
                serde_json::json!({
                    "message": request.message,
                    "document_content": request.document_content,
                    "document_path": request.document_path,
                    "context": request.context,
                })
            } else {
                serde_json::json!({
                    "message": {
                        "role": "user",
                        "content": request.message,
                    },
                    "agent_id": agent_id,
                    "thread_id": thread_id,
                    "document_content": request.document_content,
                    "document_path": request.document_path,
                    "context": request.context,
                })
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .header("X-Instance-ID", &self.config.instance_id)
                .json(&payload)
                .send()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;

            if response.status().is_success() {
                let text = response
                    .text()
                    .await
                    .map_err(|e| Error::Network(e.to_string()))?;

                if let Ok(chat_response) = serde_json::from_str::<ChatWithDocsResponse>(&text) {
                    return Ok(chat_response);
                }

                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    let message = value
                        .get("message")
                        .and_then(|m| m.as_str())
                        .or_else(|| value.get("content").and_then(|c| c.as_str()))
                        .unwrap_or("No response")
                        .to_string();

                    return Ok(ChatWithDocsResponse {
                        message,
                        documents_used: None,
                        confidence: None,
                        metadata: None,
                    });
                }

                return Ok(ChatWithDocsResponse {
                    message: text,
                    documents_used: None,
                    confidence: None,
                    metadata: None,
                });
            }
        }

        Err(Error::Api(format!(
            "Failed to chat with docs: All endpoint paths returned 404. Chat with documents may not be available in this instance."
        )))
    }

    /// Stream chat with documents response
    pub async fn stream_chat_with_docs<F>(
        &self,
        agent_id: &str,
        thread_id: &str,
        request: ChatWithDocsRequest,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> Result<()>,
    {
        let token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Set access token (Bearer token) first.".to_string())
        })?;

        let base_url = self.config.get_base_url();
        
        let endpoints = vec![
            format!("{}/orchestrate/agents/{}/threads/{}/chat_with_docs", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/chat_with_docs", base_url, agent_id, thread_id),
            format!("{}/orchestrate/agents/{}/threads/{}/runs/stream", base_url, agent_id, thread_id),
            format!("{}/agents/{}/threads/{}/runs/stream", base_url, agent_id, thread_id),
        ];

        for url in endpoints {
            let payload = if url.contains("chat_with_docs") {
                serde_json::json!({
                    "message": request.message,
                    "document_content": request.document_content,
                    "document_path": request.document_path,
                    "context": request.context,
                })
            } else {
                serde_json::json!({
                    "message": {
                        "role": "user",
                        "content": request.message,
                    },
                    "agent_id": agent_id,
                    "thread_id": thread_id,
                    "document_content": request.document_content,
                    "document_path": request.document_path,
                    "context": request.context,
                })
            };

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .header("Accept", "text/event-stream")
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .header("X-Accel-Buffering", "no")
                .header("X-Instance-ID", &self.config.instance_id)
                .json(&payload)
                .send()
                .await
                .map_err(|e| Error::Network(e.to_string()))?;

            if !response.status().is_success() {
                continue;
            }

            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result.map_err(|e| Error::Network(e.to_string()))?;
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if !line.is_empty() && line.starts_with("data:") {
                        let data_str = &line[5..].trim();
                        if let Ok(event_data) = serde_json::from_str::<EventData>(data_str) {
                            if event_data.event == "message.delta" {
                                if let Some(data_obj) = event_data.data.as_object() {
                                    if let Some(delta_obj) = data_obj.get("delta").and_then(|d| d.as_object()) {
                                        if let Some(content_array) = delta_obj.get("content").and_then(|c| c.as_array()) {
                                            if let Some(first_content) = content_array.first() {
                                                if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                                    callback(text.to_string())?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            return Ok(());
        }

        Err(Error::Api(format!(
            "Failed to stream chat with docs: All endpoint paths returned 404. Chat with documents may not be available in this instance."
        )))
    }
}

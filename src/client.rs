//! WatsonX AI client implementation

use crate::config::WatsonxConfig;
use crate::error::{Error, Result};
use crate::models::*;
use crate::types::*;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use uuid::Uuid;

/// WatsonX AI client for interacting with IBM WatsonX services
pub struct WatsonxClient {
    config: WatsonxConfig,
    access_token: Option<String>,
    client: Client,
    current_model: String,
}

#[derive(Serialize)]
struct TokenRequest {
    grant_type: String,
    apikey: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Serialize)]
struct GenerationParams {
    decoding_method: String,
    max_new_tokens: u32,
    min_new_tokens: u32,
    top_k: u32,
    top_p: f32,
    repetition_penalty: f32,
    stop_sequences: Vec<String>,
}

#[derive(Serialize)]
struct GenerationRequest {
    input: String,
    parameters: GenerationParams,
    model_id: String,
    project_id: String,
}

#[derive(Deserialize)]
struct GenerationResults {
    generated_text: String,
}

#[derive(Deserialize)]
struct GenerationData {
    results: Vec<GenerationResults>,
}

#[derive(Deserialize)]
struct ModelSpec {
    model_id: String,
    label: Option<String>,
    provider: Option<String>,
    source: Option<String>,
    short_description: Option<String>,
    long_description: Option<String>,
    functions: Option<Vec<Function>>,
    lifecycle: Option<Vec<Lifecycle>>,
}

#[derive(Deserialize)]
struct Function {
    id: String,
}

#[derive(Deserialize)]
struct Lifecycle {
    id: String,
    start_date: Option<String>,
}

#[derive(Deserialize)]
struct ModelsResponse {
    resources: Vec<ModelSpec>,
}

impl WatsonxClient {
    /// Create a new WatsonX client from configuration
    pub fn new(config: WatsonxConfig) -> Result<Self> {
        config.validate()?;
        
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;

        Ok(Self {
            config,
            access_token: None,
            client,
            current_model: DEFAULT_MODEL.to_string(),
        })
    }

    /// Create a new WatsonX client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = WatsonxConfig::from_env()?;
        Self::new(config)
    }

    /// Set the model to use for generation
    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.current_model = model_id.into();
        self
    }

    /// Get the current model ID
    pub fn model_id(&self) -> &str {
        &self.current_model
    }

    /// Connect to WatsonX and authenticate
    pub async fn connect(&mut self) -> Result<()> {
        let token_request = TokenRequest {
            grant_type: "urn:ibm:params:oauth:grant-type:apikey".to_string(),
            apikey: self.config.api_key.clone(),
        };

        let url = format!("https://{}/identity/token", self.config.iam_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&token_request)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Authentication(format!(
                "Authentication failed: {}",
                response.status()
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        self.access_token = Some(token_response.access_token);
        Ok(())
    }

    /// Generate text using the current model
    pub async fn generate(&self, prompt: &str) -> Result<GenerationResult> {
        let config = GenerationConfig {
            model_id: self.current_model.clone(),
            ..Default::default()
        };
        self.generate_with_config(prompt, &config).await
    }

    /// Generate text with custom configuration
    pub async fn generate_with_config(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationResult> {
        let _start_time = Instant::now();
        let request_id = Uuid::new_v4().to_string();

        let generation_future = self.perform_text_stream_generation(prompt, config, &request_id);

        let text = match timeout(config.timeout, generation_future).await {
            Ok(result) => result?,
            Err(_) => return Err(Error::Timeout("Request timed out".to_string())),
        };

        Ok(GenerationResult::new(text, config.model_id.clone())
            .with_request_id(request_id))
    }

    /// Generate text using the standard generation endpoint (returns complete response)
    pub async fn generate_text(
        &self,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<GenerationResult> {
        let _start_time = Instant::now();
        let request_id = Uuid::new_v4().to_string();

        let generation_future = self.perform_text_generation(prompt, config, &request_id);

        let text = match timeout(config.timeout, generation_future).await {
            Ok(result) => result?,
            Err(_) => return Err(Error::Timeout("Request timed out".to_string())),
        };

        Ok(GenerationResult::new(text, config.model_id.clone())
            .with_request_id(request_id))
    }

    /// Generate text with streaming callback for real-time output
    pub async fn generate_text_stream<F>(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        callback: F,
    ) -> Result<GenerationResult>
    where
        F: Fn(&str) + Send + Sync,
    {
        let request_id = Uuid::new_v4().to_string();
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not connected. Call connect() first.".to_string())
        })?;

        let params = GenerationParams {
            decoding_method: "greedy".to_string(),
            max_new_tokens: config.max_tokens,
            min_new_tokens: 1,
            top_k: config.top_k.unwrap_or(50),
            top_p: config.top_p.unwrap_or(1.0),
            repetition_penalty: config.repetition_penalty.unwrap_or(1.1),
            stop_sequences: config.stop_sequences.clone(),
        };

        let request_body = GenerationRequest {
            input: prompt.to_string(),
            parameters: params,
            model_id: config.model_id.clone(),
            project_id: self.config.project_id.clone(),
        };

        let url = format!(
            "{}/ml/v1/text/generation_stream?version={}",
            self.config.api_url, self.config.api_version
        );

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
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
                "WatsonX API request failed with status {}: {}",
                status, error_text
            )));
        }

        let mut answer = String::new();
        
        // Use bytes_stream for true streaming - process chunks as they arrive
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        // Process stream chunks in real-time
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| Error::Network(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete lines from buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("id:") || trimmed.starts_with("event:") {
                    continue;
                }

                if trimmed.starts_with("data:") {
                    let json_data = if trimmed.starts_with("data: ") {
                        &trimmed[6..]
                    } else {
                        &trimmed[5..]
                    };

                    if json_data.trim().is_empty() || json_data.trim() == "[DONE]" {
                        continue;
                    }

                    match serde_json::from_str::<GenerationData>(json_data) {
                        Ok(data) => {
                            if let Some(result) = data.results.first() {
                                let generated_text = &result.generated_text;
                                answer.push_str(generated_text);
                                // Call the callback immediately with the new chunk
                                callback(generated_text);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse SSE data: {}", e);
                        }
                    }
                }
            }
        }

        // Process any remaining data in buffer
        if !buffer.is_empty() {
            let trimmed = buffer.trim();
            if trimmed.starts_with("data:") {
                let json_data = if trimmed.starts_with("data: ") {
                    &trimmed[6..]
                } else {
                    &trimmed[5..]
                };

                if !json_data.trim().is_empty() && json_data.trim() != "[DONE]" {
                    if let Ok(data) = serde_json::from_str::<GenerationData>(json_data) {
                        if let Some(result) = data.results.first() {
                            let generated_text = &result.generated_text;
                            answer.push_str(generated_text);
                            callback(generated_text);
                        }
                    }
                }
            }
        }

        if answer.trim().is_empty() {
            return Err(Error::Api(
                "Empty response from WatsonX API".to_string(),
            ));
        }

        Ok(GenerationResult::new(answer, config.model_id.clone())
            .with_request_id(request_id))
    }

    /// Perform text generation request using streaming endpoint
    async fn perform_text_stream_generation(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        _request_id: &str,
    ) -> Result<String> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Call connect() first.".to_string())
        })?;

        let params = GenerationParams {
            decoding_method: "greedy".to_string(),
            max_new_tokens: config.max_tokens,
            min_new_tokens: 5,
            top_k: config.top_k.unwrap_or(50),
            top_p: config.top_p.unwrap_or(1.0),
            repetition_penalty: config.repetition_penalty.unwrap_or(1.1),
            stop_sequences: config.stop_sequences.clone(),
        };

        let request_body = GenerationRequest {
            input: prompt.to_string(),
            parameters: params,
            model_id: config.model_id.clone(),
            project_id: self.config.project_id.clone(),
        };

        let url = format!(
            "{}/ml/v1/text/generation_stream?version={}",
            self.config.api_url, self.config.api_version
        );

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
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
                "WatsonX API request failed with status {}: {}",
                status, error_text
            )));
        }

        let mut answer = String::new();
        
        // Use bytes_stream for true streaming - process chunks as they arrive
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        // Process stream chunks in real-time
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| Error::Network(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete lines from buffer
            while let Some(newline_pos) = buffer.find('\n') {
                let line = buffer[..newline_pos].to_string();
                buffer = buffer[newline_pos + 1..].to_string();

                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("id:") || trimmed.starts_with("event:") {
                    continue;
                }

                if trimmed.starts_with("data:") {
                    let json_data = if trimmed.starts_with("data: ") {
                        &trimmed[6..]
                    } else {
                        &trimmed[5..]
                    };

                    if json_data.trim().is_empty() || json_data.trim() == "[DONE]" {
                        continue;
                    }

                    match serde_json::from_str::<GenerationData>(json_data) {
                        Ok(data) => {
                            if let Some(result) = data.results.first() {
                                let generated_text = &result.generated_text;
                                answer.push_str(generated_text);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse SSE data: {}", e);
                        }
                    }
                }
            }
        }

        // Process any remaining data in buffer
        if !buffer.is_empty() {
            let trimmed = buffer.trim();
            if trimmed.starts_with("data:") {
                let json_data = if trimmed.starts_with("data: ") {
                    &trimmed[6..]
                } else {
                    &trimmed[5..]
                };

                if !json_data.trim().is_empty() && json_data.trim() != "[DONE]" {
                    if let Ok(data) = serde_json::from_str::<GenerationData>(json_data) {
                        if let Some(result) = data.results.first() {
                            let generated_text = &result.generated_text;
                            answer.push_str(generated_text);
                        }
                    }
                }
            }
        }

        if answer.trim().is_empty() {
            return Err(Error::Api(
                "Empty response from WatsonX API".to_string(),
            ));
        }

        // Clean up the response
        let mut cleaned_answer = answer.trim().to_string();

        if cleaned_answer.starts_with("Answer:") {
            cleaned_answer = cleaned_answer
                .strip_prefix("Answer:")
                .unwrap_or(&cleaned_answer)
                .trim()
                .to_string();
        }

        if let Some(query_pos) = cleaned_answer.find("Query:") {
            cleaned_answer = cleaned_answer[..query_pos].trim().to_string();
        }

        let final_answer = cleaned_answer
            .lines()
            .next()
            .unwrap_or(&cleaned_answer)
            .trim()
            .to_string();

        Ok(final_answer)
    }

    /// Perform text generation request using standard endpoint
    async fn perform_text_generation(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        _request_id: &str,
    ) -> Result<String> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Call connect() first.".to_string())
        })?;

        let params = GenerationParams {
            decoding_method: "greedy".to_string(),
            max_new_tokens: config.max_tokens,
            min_new_tokens: 5,
            top_k: config.top_k.unwrap_or(50),
            top_p: config.top_p.unwrap_or(1.0),
            repetition_penalty: config.repetition_penalty.unwrap_or(1.1),
            stop_sequences: config.stop_sequences.clone(),
        };

        let request_body = GenerationRequest {
            input: prompt.to_string(),
            parameters: params,
            model_id: config.model_id.clone(),
            project_id: self.config.project_id.clone(),
        };

        // Use non-streaming endpoint
        let url = format!(
            "{}/ml/v1/text/generation?version={}",
            self.config.api_url, self.config.api_version
        );

        let response = self
            .client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
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
                "WatsonX API request failed with status {}: {}",
                status, error_text
            )));
        }

        // Parse the complete JSON response
        let generation_data: GenerationData = response
            .json()
            .await
            .map_err(|e| Error::Serialization(e.to_string()))?;

        if let Some(result) = generation_data.results.first() {
            Ok(result.generated_text.clone())
        } else {
            Err(Error::Api("No generation results returned".to_string()))
        }
    }

    /// List available foundation models
    pub async fn list_models(&self) -> Result<Vec<crate::types::ModelInfo>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not connected. Call connect() first.".to_string())
        })?;

        let url = format!(
            "{}/ml/v1/foundation_model_specs?version={}",
            self.config.api_url, self.config.api_version
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
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
                "Failed to list models with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let models_response: ModelsResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::Serialization(format!("Failed to parse models response: {}", e)))?;

        let model_infos = models_response
            .resources
            .into_iter()
            .map(|spec| {
                let supported_tasks = spec.functions
                    .map(|functions| functions.into_iter().map(|f| f.id).collect());
                
                let available = spec.lifecycle
                    .and_then(|lifecycle| {
                        lifecycle.iter()
                            .find(|l| l.id == "available")
                            .map(|_| true)
                    });

                crate::types::ModelInfo {
                    model_id: spec.model_id,
                    name: spec.label,
                    description: spec.long_description.or(spec.short_description),
                    provider: spec.provider,
                    version: None, // Not available in API response
                    supported_tasks,
                    max_context_length: None, // Not available in API response
                    available,
                }
            })
            .collect();

        Ok(model_infos)
    }

    /// Assess the quality of generated text
    pub fn assess_quality(&self, text: &str, _prompt: &str) -> f32 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Check if result is not empty and reasonable length
        max_score += 0.3;
        let trimmed = text.trim();
        if !trimmed.is_empty() && trimmed.len() > 8 && trimmed.len() < 200 {
            score += 0.3;
        }

        // Check for common patterns
        max_score += 0.2;
        let common_patterns = [
            "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        ];
        if common_patterns.iter().any(|pattern| text.to_lowercase().contains(pattern)) {
            score += 0.2;
        }

        // Check if it doesn't contain obvious errors
        max_score += 0.2;
        let error_indicators = ["error", "failed", "invalid", "unknown", "not found"];
        if !error_indicators
            .iter()
            .any(|indicator| text.to_lowercase().contains(indicator))
        {
            score += 0.2;
        }

        // Check for proper sentence structure
        max_score += 0.15;
        let sentence_count = text.split('.').filter(|s| !s.trim().is_empty()).count();
        if sentence_count > 0 {
            score += 0.15;
        }

        // Check for reasonable word count
        max_score += 0.15;
        let word_count = text.split_whitespace().count();
        if word_count > 3 && word_count < 100 {
            score += 0.15;
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_assessment() {
        let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());
        let client = WatsonxClient::new(config).unwrap();

        let good_text = "This is a well-formed sentence with proper grammar.";
        let score = client.assess_quality(good_text, "test prompt");
        assert!(score > 0.5);

        let bad_text = "error";
        let score = client.assess_quality(bad_text, "test prompt");
        assert!(score < 0.5);
    }

    #[test]
    fn test_config_validation() {
        let config = WatsonxConfig::new("".to_string(), "test_project".to_string());
        assert!(config.validate().is_err());

        let config = WatsonxConfig::new("test_key".to_string(), "".to_string());
        assert!(config.validate().is_err());

        let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());
        assert!(config.validate().is_ok());
    }
}

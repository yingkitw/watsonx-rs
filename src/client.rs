//! WatsonX AI client implementation

use crate::config::WatsonxConfig;
use crate::error::{Error, Result};
use crate::models::*;
use crate::types::*;
use futures::future::join_all;
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
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

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
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error details available".to_string());
            return Err(Error::Authentication(format!(
                "Failed to authenticate with IAM service (HTTP {}): {}. Verify your WATSONX_API_KEY is correct and the IAM URL is accessible.",
                status, error_text
            )));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::Serialization(format!(
                "Failed to parse JSON response: {}. The API response format may have changed. Please report this issue.",
                e
            )))?;

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
            Error::Authentication(
                "Not authenticated. Call connect() first to obtain an access token.".to_string(),
            )
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
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "WatsonX API request failed (HTTP {}): {}. Verify your model ID '{}' is correct and your project has access to it.",
                status, error_text, config.model_id
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
                "Received empty response from WatsonX API. The model may have generated no output, or the response format was unexpected. Try adjusting your prompt or parameters.".to_string(),
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
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "WatsonX API request failed (HTTP {}): {}. Verify your model ID '{}' is correct and your project has access to it.",
                status, error_text, config.model_id
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
                "Received empty response from WatsonX API. The model may have generated no output, or the response format was unexpected. Try adjusting your prompt or parameters.".to_string(),
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

        Self::perform_text_generation_internal(
            &self.client,
            access_token,
            &self.config.project_id,
            &self.config.api_url,
            &self.config.api_version,
            prompt,
            config,
        )
        .await
    }

    /// Internal method for text generation that can be called from spawned tasks
    /// This allows true parallelism by not requiring &self
    async fn perform_text_generation_internal(
        client: &Client,
        access_token: &str,
        project_id: &str,
        api_url: &str,
        api_version: &str,
        prompt: &str,
        config: &GenerationConfig,
    ) -> Result<String> {
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
            project_id: project_id.to_string(),
        };

        // Use non-streaming endpoint
        let url = format!(
            "{}/ml/v1/text/generation?version={}",
            api_url, api_version
        );

        let response = client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "WatsonX API request failed (HTTP {}): {}. Verify your model ID '{}' is correct and your project has access to it.",
                status, error_text, config.model_id
            )));
        }

        // Parse the complete JSON response
        let generation_data: GenerationData = response
            .json()
            .await
            .map_err(|e| Error::Serialization(format!(
                "Failed to parse JSON response: {}. The API response format may have changed. Please report this issue.",
                e
            )))?;

        if let Some(result) = generation_data.results.first() {
            Ok(result.generated_text.clone())
        } else {
            Err(Error::Api(
                "No generation results returned from API. The model may not have generated any output. Try adjusting your prompt or parameters.".to_string(),
            ))
        }
    }

    /// List available foundation models
    pub async fn list_models(&self) -> Result<Vec<crate::types::ModelInfo>> {
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication(
                "Not authenticated. Call connect() first to obtain an access token.".to_string(),
            )
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
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api(format!(
                "Failed to list available models (HTTP {}): {}. Verify your project ID is correct and you have access to the models API.",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| Error::Network(format!(
                "Network request failed: {}. Check your internet connection and verify the API endpoint URL is correct.",
                e
            )))?;

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

    /// Generate text for multiple prompts concurrently and collect all results
    /// 
    /// This method executes all generation requests in parallel by spawning each
    /// request as a separate async task, maximizing parallelism for I/O-bound operations.
    /// Results are collected once all requests complete (or fail). Each request can
    /// have its own configuration, or use a shared default configuration.
    /// 
    /// # Arguments
    /// 
    /// * `requests` - Vector of batch requests, each containing a prompt and optional config
    /// * `default_config` - Default configuration to use for requests without explicit config
    /// 
    /// # Returns
    /// 
    /// A `BatchGenerationResult` containing all results, with per-item error handling.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use watsonx_rs::{WatsonxClient, WatsonxConfig, BatchRequest, GenerationConfig, models::models};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WatsonxConfig::from_env()?;
    /// let mut client = WatsonxClient::new(config)?;
    /// client.connect().await?;
    /// 
    /// let default_config = GenerationConfig::default()
    ///     .with_model(models::GRANITE_4_H_SMALL);
    /// 
    /// let requests = vec![
    ///     BatchRequest::new("Write a haiku about Rust")
    ///         .with_id("haiku-1"),
    ///     BatchRequest::new("Explain async/await in one sentence")
    ///         .with_id("async-1"),
    ///     BatchRequest::new("What is ownership in Rust?")
    ///         .with_id("ownership-1"),
    /// ];
    /// 
    /// let batch_result = client.generate_batch(requests, &default_config).await?;
    /// 
    /// println!("Total: {}, Successful: {}, Failed: {}", 
    ///     batch_result.total, batch_result.successful, batch_result.failed);
    /// 
    /// for item in batch_result.results {
    ///     if let Some(result) = item.result {
    ///         println!("[{}] {}", item.id.unwrap_or_default(), result.text);
    ///     } else if let Some(error) = item.error {
    ///         println!("[{}] Error: {}", item.id.unwrap_or_default(), error);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_batch(
        &self,
        requests: Vec<BatchRequest>,
        default_config: &GenerationConfig,
    ) -> Result<BatchGenerationResult> {
        let start_time = Instant::now();

        // Check authentication before spawning tasks
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Call connect() first.".to_string())
        })?;

        // Clone necessary parts for spawning tasks
        // reqwest::Client is designed to be cloned (uses connection pooling internally)
        let http_client = self.client.clone();
        let access_token = access_token.clone();
        let project_id = self.config.project_id.clone();
        let api_url = self.config.api_url.clone();
        let api_version = self.config.api_version.clone();

        // Spawn each request as a separate async task for true parallelism
        let tasks: Vec<_> = requests
            .into_iter()
            .map(|req| {
                let prompt = req.prompt.clone();
                let config = req.config.clone().unwrap_or_else(|| default_config.clone());
                let id = req.id.clone();
                
                // Clone necessary data for the spawned task
                let http_client = http_client.clone();
                let access_token = access_token.clone();
                let project_id = project_id.clone();
                let api_url = api_url.clone();
                let api_version = api_version.clone();
                
                // Spawn as a separate task for true parallelism
                tokio::spawn(async move {
                    // Call the internal generation method directly
                    let result = Self::perform_text_generation_internal(
                        &http_client,
                        &access_token,
                        &project_id,
                        &api_url,
                        &api_version,
                        &prompt,
                        &config,
                    ).await;
                    
                    match result {
                        Ok(text) => {
                            let gen_result = GenerationResult::new(text, config.model_id.clone());
                            BatchItemResult::success(id, prompt, gen_result)
                        }
                        Err(error) => BatchItemResult::failure(id, prompt, error),
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete and collect results
        let results: Vec<BatchItemResult> = join_all(tasks)
            .await
            .into_iter()
            .map(|task_result| {
                // Handle task join errors (shouldn't happen in normal operation)
                task_result.unwrap_or_else(|e| {
                    BatchItemResult::failure(
                        None,
                        String::new(),
                        Error::Network(format!("Task join error: {}", e)),
                    )
                })
            })
            .collect();
        
        let duration = start_time.elapsed();
        
        Ok(BatchGenerationResult::new(results, duration))
    }

    /// Generate text for multiple prompts concurrently using a shared configuration
    /// 
    /// Convenience method that uses the same configuration for all prompts.
    /// 
    /// # Arguments
    /// 
    /// * `prompts` - Vector of prompts to generate text for
    /// * `config` - Configuration to use for all requests
    /// 
    /// # Returns
    /// 
    /// A `BatchGenerationResult` containing all results.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models::models};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WatsonxConfig::from_env()?;
    /// let mut client = WatsonxClient::new(config)?;
    /// client.connect().await?;
    /// 
    /// let gen_config = GenerationConfig::default()
    ///     .with_model(models::GRANITE_4_H_SMALL);
    /// 
    /// let prompts = vec![
    ///     "Write a haiku about Rust".to_string(),
    ///     "Explain async/await in one sentence".to_string(),
    ///     "What is ownership in Rust?".to_string(),
    /// ];
    /// 
    /// let batch_result = client.generate_batch_simple(prompts, &gen_config).await?;
    /// 
    /// for item in batch_result.results {
    ///     if let Some(result) = item.result {
    ///         println!("Generated: {}", result.text);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_batch_simple(
        &self,
        prompts: Vec<String>,
        config: &GenerationConfig,
    ) -> Result<BatchGenerationResult> {
        let requests: Vec<BatchRequest> = prompts
            .into_iter()
            .map(|prompt| BatchRequest::new(prompt))
            .collect();
        
        self.generate_batch(requests, config).await
    }

    /// Create a chat completion from a list of messages
    /// 
    /// This method uses the WatsonX AI chat completion API endpoint to generate
    /// responses based on a conversation history. It supports system, user, and
    /// assistant messages for multi-turn conversations.
    /// 
    /// # Arguments
    /// 
    /// * `messages` - Vector of chat messages representing the conversation
    /// * `config` - Configuration for the chat completion
    /// 
    /// # Returns
    /// 
    /// A `ChatCompletionResult` containing the generated message and metadata.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use watsonx_rs::{WatsonxClient, WatsonxConfig, ChatMessage, ChatCompletionConfig, models::models};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WatsonxConfig::from_env()?;
    /// let mut client = WatsonxClient::new(config)?;
    /// client.connect().await?;
    /// 
    /// let chat_config = ChatCompletionConfig::default()
    ///     .with_model(models::GRANITE_4_H_SMALL);
    /// 
    /// let messages = vec![
    ///     ChatMessage::system("You are a helpful assistant."),
    ///     ChatMessage::user("What is Rust?"),
    /// ];
    /// 
    /// let result = client.chat_completion(messages, &chat_config).await?;
    /// println!("Assistant: {}", result.content());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &ChatCompletionConfig,
    ) -> Result<ChatCompletionResult> {
        let request_id = Uuid::new_v4().to_string();
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Call connect() first.".to_string())
        })?;

        // Build request body
        let mut request_body = serde_json::json!({
            "model": config.model_id,
            "messages": messages,
            "max_tokens": config.max_tokens,
        });

        // Add optional parameters
        if let Some(temperature) = config.temperature {
            request_body["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temperature as f64).unwrap());
        }
        if let Some(top_p) = config.top_p {
            request_body["top_p"] = serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap());
        }
        if let Some(top_k) = config.top_k {
            request_body["top_k"] = serde_json::Value::Number(serde_json::Number::from(top_k));
        }
        if !config.stop_sequences.is_empty() {
            request_body["stop"] = serde_json::json!(config.stop_sequences);
        }
        if let Some(repetition_penalty) = config.repetition_penalty {
            request_body["repetition_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(repetition_penalty as f64).unwrap());
        }

        // Try both possible endpoints
        let endpoints = vec![
            format!("{}/ml/gateway/v1/chat/completions", self.config.api_url),
            format!("{}/ml/v1/chat/completions?version={}", self.config.api_url, self.config.api_version),
        ];

        let mut last_error = None;
        for url in endpoints {
            let response = self
                .client
                .post(&url)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", access_token))
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let completion_data: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|e| Error::Serialization(format!(
                "Failed to parse JSON response: {}. The API response format may have changed. Please report this issue.",
                e
            )))?;

                    // Parse response - handle different response formats
                    let choice = completion_data["choices"]
                        .as_array()
                        .and_then(|choices| choices.first())
                        .ok_or_else(|| Error::Api("No choices in response".to_string()))?;

                    let message_content = choice["message"]["content"]
                        .as_str()
                        .ok_or_else(|| Error::Api("No message content in response".to_string()))?;

                    let message = ChatMessage::assistant(message_content);
                    let mut result = ChatCompletionResult::new(message, config.model_id.clone())
                        .with_request_id(request_id.clone());

                    // Extract token usage if available
                    if let Some(usage) = completion_data.get("usage") {
                        if let Some(prompt_tokens) = usage["prompt_tokens"].as_u64() {
                            if let Some(completion_tokens) = usage["completion_tokens"].as_u64() {
                                if let Some(total_tokens) = usage["total_tokens"].as_u64() {
                                    result = result.with_tokens(
                                        prompt_tokens as u32,
                                        completion_tokens as u32,
                                        total_tokens as u32,
                                    );
                                }
                            }
                        }
                    }

                    // Extract finish reason if available
                    if let Some(reason) = choice["finish_reason"].as_str() {
                        result = result.with_finish_reason(reason);
                    }

                    return Ok(result);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let error_text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    last_error = Some(Error::Api(format!(
                        "Chat completion failed with status {}: {}",
                        status, error_text
                    )));
                    // Try next endpoint
                    continue;
                }
                Err(e) => {
                    last_error = Some(Error::Network(e.to_string()));
                    // Try next endpoint
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Api("All chat completion endpoints failed".to_string())
        }))
    }

    /// Create a chat completion with streaming callback for real-time output
    /// 
    /// This method uses the WatsonX AI chat completion streaming endpoint to generate
    /// responses in real-time. The callback is invoked for each chunk of text as it
    /// arrives from the API.
    /// 
    /// # Arguments
    /// 
    /// * `messages` - Vector of chat messages representing the conversation
    /// * `config` - Configuration for the chat completion
    /// * `callback` - Function called for each text chunk received
    /// 
    /// # Returns
    /// 
    /// A `ChatCompletionResult` containing the complete generated message and metadata.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use watsonx_rs::{WatsonxClient, WatsonxConfig, ChatMessage, ChatCompletionConfig, models::models};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WatsonxConfig::from_env()?;
    /// let mut client = WatsonxClient::new(config)?;
    /// client.connect().await?;
    /// 
    /// let chat_config = ChatCompletionConfig::default()
    ///     .with_model(models::GRANITE_4_H_SMALL);
    /// 
    /// let messages = vec![
    ///     ChatMessage::system("You are a helpful assistant."),
    ///     ChatMessage::user("Explain async/await in Rust."),
    /// ];
    /// 
    /// let result = client.chat_completion_stream(messages, &chat_config, |chunk| {
    ///     print!("{}", chunk);
    ///     std::io::Write::flush(&mut std::io::stdout()).unwrap();
    /// }).await?;
    /// 
    /// println!("\nTotal tokens: {:?}", result.total_tokens);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chat_completion_stream<F>(
        &self,
        messages: Vec<ChatMessage>,
        config: &ChatCompletionConfig,
        callback: F,
    ) -> Result<ChatCompletionResult>
    where
        F: Fn(&str) + Send + Sync,
    {
        let request_id = Uuid::new_v4().to_string();
        let access_token = self.access_token.as_ref().ok_or_else(|| {
            Error::Authentication("Not authenticated. Call connect() first.".to_string())
        })?;

        // Build request body
        let mut request_body = serde_json::json!({
            "model": config.model_id,
            "messages": messages,
            "max_tokens": config.max_tokens,
            "stream": true,
        });

        // Add optional parameters
        if let Some(temperature) = config.temperature {
            request_body["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temperature as f64).unwrap());
        }
        if let Some(top_p) = config.top_p {
            request_body["top_p"] = serde_json::Value::Number(serde_json::Number::from_f64(top_p as f64).unwrap());
        }
        if let Some(top_k) = config.top_k {
            request_body["top_k"] = serde_json::Value::Number(serde_json::Number::from(top_k));
        }
        if !config.stop_sequences.is_empty() {
            request_body["stop"] = serde_json::json!(config.stop_sequences);
        }
        if let Some(repetition_penalty) = config.repetition_penalty {
            request_body["repetition_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(repetition_penalty as f64).unwrap());
        }

        // Try both possible endpoints
        let endpoints = vec![
            format!("{}/ml/gateway/v1/chat/completions", self.config.api_url),
            format!("{}/ml/v1/chat/completions?version={}", self.config.api_url, self.config.api_version),
        ];

        let mut last_error = None;
        for url in endpoints {
            let response = self
                .client
                .post(&url)
                .header("Accept", "text/event-stream")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Cache-Control", "no-cache")
                .header("Connection", "keep-alive")
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let mut answer = String::new();
                    let mut stream = resp.bytes_stream();
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

                                match serde_json::from_str::<serde_json::Value>(json_data) {
                                    Ok(data) => {
                                        // Extract content from delta or message
                                        if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
                                            if let Some(choice) = choices.first() {
                                                if let Some(delta) = choice.get("delta") {
                                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                        answer.push_str(content);
                                                        callback(content);
                                                    }
                                                } else if let Some(message) = choice.get("message") {
                                                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                                        answer.push_str(content);
                                                        callback(content);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Ignore parse errors for individual chunks
                                        continue;
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
                                if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_data) {
                                    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
                                        if let Some(choice) = choices.first() {
                                            if let Some(delta) = choice.get("delta") {
                                                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                                    answer.push_str(content);
                                                    callback(content);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if answer.trim().is_empty() {
                        return Err(Error::Api("Empty response from chat completion API".to_string()));
                    }

                    let message = ChatMessage::assistant(&answer);
                    return Ok(ChatCompletionResult::new(message, config.model_id.clone())
                        .with_request_id(request_id));
                }
                Ok(resp) => {
                    let status = resp.status();
                    let error_text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    last_error = Some(Error::Api(format!(
                        "Chat completion stream failed with status {}: {}",
                        status, error_text
                    )));
                    // Try next endpoint
                    continue;
                }
                Err(e) => {
                    last_error = Some(Error::Network(e.to_string()));
                    // Try next endpoint
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Api("All chat completion streaming endpoints failed".to_string())
        }))
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

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn test_batch_request_creation() {
        let req = BatchRequest::new("test prompt");
        assert_eq!(req.prompt, "test prompt");
        assert!(req.config.is_none());
        assert!(req.id.is_none());

        let req = BatchRequest::new("test prompt").with_id("test-id");
        assert_eq!(req.id, Some("test-id".to_string()));

        let config = GenerationConfig::default();
        let req = BatchRequest::with_config("test prompt", config.clone());
        assert_eq!(req.prompt, "test prompt");
        assert!(req.config.is_some());
    }

    #[test]
    fn test_batch_item_result() {
        let result = GenerationResult::new("test text".to_string(), "model".to_string());
        let item = BatchItemResult::success(
            Some("id-1".to_string()),
            "prompt".to_string(),
            result.clone(),
        );
        
        assert!(item.is_success());
        assert!(!item.is_failure());
        assert_eq!(item.id, Some("id-1".to_string()));
        assert_eq!(item.prompt, "prompt");
        assert!(item.result.is_some());
        assert!(item.error.is_none());

        let error = Error::Api("test error".to_string());
        let item = BatchItemResult::failure(
            Some("id-2".to_string()),
            "prompt2".to_string(),
            error.clone(),
        );
        
        assert!(!item.is_success());
        assert!(item.is_failure());
        assert_eq!(item.id, Some("id-2".to_string()));
        assert!(item.result.is_none());
        assert!(item.error.is_some());
    }

    #[test]
    fn test_batch_generation_result() {
        let results = vec![
            BatchItemResult::success(
                Some("id-1".to_string()),
                "prompt1".to_string(),
                GenerationResult::new("result1".to_string(), "model".to_string()),
            ),
            BatchItemResult::success(
                Some("id-2".to_string()),
                "prompt2".to_string(),
                GenerationResult::new("result2".to_string(), "model".to_string()),
            ),
            BatchItemResult::failure(
                Some("id-3".to_string()),
                "prompt3".to_string(),
                Error::Api("error".to_string()),
            ),
        ];

        let batch_result = BatchGenerationResult::new(results, Duration::from_secs(1));
        
        assert_eq!(batch_result.total, 3);
        assert_eq!(batch_result.successful, 2);
        assert_eq!(batch_result.failed, 1);
        assert!(!batch_result.all_succeeded());
        assert!(batch_result.any_failed());
        
        let successes = batch_result.successes();
        assert_eq!(successes.len(), 2);
        
        let failures = batch_result.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "prompt3");
    }

    #[test]
    fn test_batch_generation_result_all_succeeded() {
        let results = vec![
            BatchItemResult::success(
                None,
                "prompt1".to_string(),
                GenerationResult::new("result1".to_string(), "model".to_string()),
            ),
            BatchItemResult::success(
                None,
                "prompt2".to_string(),
                GenerationResult::new("result2".to_string(), "model".to_string()),
            ),
        ];

        let batch_result = BatchGenerationResult::new(results, Duration::from_secs(1));
        
        assert_eq!(batch_result.total, 2);
        assert_eq!(batch_result.successful, 2);
        assert_eq!(batch_result.failed, 0);
        assert!(batch_result.all_succeeded());
        assert!(!batch_result.any_failed());
    }
}

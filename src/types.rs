//! Core types for WatsonX operations

use serde::{Deserialize, Serialize};
use std::time::Duration;

// Token constants are defined in models.rs to avoid conflicts

/// Configuration for text generation requests
#[derive(Clone, Debug, Serialize)]
pub struct GenerationConfig {
    /// Model ID to use for generation
    pub model_id: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Stop sequences to halt generation
    pub stop_sequences: Vec<String>,
    /// Temperature for generation (not used in current API)
    pub temperature: Option<f32>,
    /// Repetition penalty
    pub repetition_penalty: Option<f32>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            model_id: crate::models::DEFAULT_MODEL.to_string(),
            timeout: Duration::from_secs(120),
            max_tokens: crate::models::DEFAULT_MAX_TOKENS,
            top_k: Some(50),
            top_p: Some(1.0),
            stop_sequences: vec![],
            temperature: None,
            repetition_penalty: Some(1.1),
        }
    }
}

impl GenerationConfig {
    /// Create a config with maximum token support (128k)
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens.min(crate::models::MAX_TOKENS_LIMIT);
        self
    }

    /// Create a config optimized for long-form generation (128k tokens)
    pub fn long_form() -> Self {
        Self {
            max_tokens: crate::models::MAX_TOKENS_LIMIT,
            timeout: Duration::from_secs(300), // 5 minutes for long responses
            ..Default::default()
        }
    }

    /// Create a config optimized for quick responses
    pub fn quick_response() -> Self {
        Self {
            max_tokens: crate::models::QUICK_RESPONSE_MAX_TOKENS,
            timeout: Duration::from_secs(30),
            ..Default::default()
        }
    }

    /// Set the model ID
    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set top-k parameter
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set top-p parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set stop sequences
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = stop_sequences;
        self
    }

    /// Set repetition penalty
    pub fn with_repetition_penalty(mut self, penalty: f32) -> Self {
        self.repetition_penalty = Some(penalty);
        self
    }
}

/// Result of a text generation request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationResult {
    /// Generated text content
    pub text: String,
    /// Model ID used for generation
    pub model_id: String,
    /// Number of tokens used (if available)
    pub tokens_used: Option<u32>,
    /// Quality score (if calculated)
    pub quality_score: Option<f32>,
    /// Request ID for tracking
    pub request_id: Option<String>,
}

impl GenerationResult {
    /// Create a new generation result
    pub fn new(text: String, model_id: String) -> Self {
        Self {
            text,
            model_id,
            tokens_used: None,
            quality_score: None,
            request_id: None,
        }
    }

    /// Set the tokens used
    pub fn with_tokens_used(mut self, tokens: u32) -> Self {
        self.tokens_used = Some(tokens);
        self
    }

    /// Set the quality score
    pub fn with_quality_score(mut self, score: f32) -> Self {
        self.quality_score = Some(score);
        self
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Configuration for retry attempts
#[derive(Clone, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base timeout for requests
    pub base_timeout: Duration,
    /// Quality threshold for accepting results
    pub quality_threshold: f32,
    /// Delay between retries
    pub retry_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_timeout: Duration::from_secs(30),
            quality_threshold: 0.7,
            retry_delay: Duration::from_secs(1),
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Set the quality threshold
    pub fn with_quality_threshold(mut self, threshold: f32) -> Self {
        self.quality_threshold = threshold;
        self
    }

    /// Set the retry delay
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }
}

/// Information about an available model
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model ID
    pub model_id: String,
    /// Model name
    pub name: Option<String>,
    /// Model description
    pub description: Option<String>,
    /// Model provider
    pub provider: Option<String>,
    /// Model version
    pub version: Option<String>,
    /// Supported tasks
    pub supported_tasks: Option<Vec<String>>,
    /// Maximum context length
    pub max_context_length: Option<u32>,
    /// Whether the model is available
    pub available: Option<bool>,
}

impl ModelInfo {
    /// Create a new model info instance
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            name: None,
            description: None,
            provider: None,
            version: None,
            supported_tasks: None,
            max_context_length: None,
            available: None,
        }
    }

    /// Set the model name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the model description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the model provider
    pub fn with_provider(mut self, provider: String) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the model version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set supported tasks
    pub fn with_supported_tasks(mut self, tasks: Vec<String>) -> Self {
        self.supported_tasks = Some(tasks);
        self
    }

    /// Set maximum context length
    pub fn with_max_context_length(mut self, length: u32) -> Self {
        self.max_context_length = Some(length);
        self
    }

    /// Set availability status
    pub fn with_available(mut self, available: bool) -> Self {
        self.available = Some(available);
        self
    }
}

/// Information about a generation attempt
#[derive(Clone, Debug)]
pub struct GenerationAttempt {
    /// The prompt used for this attempt
    pub prompt: String,
    /// The generated result
    pub result: String,
    /// Quality score for this attempt
    pub quality_score: f32,
    /// Attempt number (1-based)
    pub attempt_number: u32,
    /// Duration of this attempt
    pub duration: Duration,
}

impl GenerationAttempt {
    /// Create a new generation attempt
    pub fn new(prompt: String, result: String, attempt_number: u32) -> Self {
        Self {
            prompt,
            result,
            quality_score: 0.0,
            attempt_number,
            duration: Duration::from_secs(0),
        }
    }

    /// Set the quality score
    pub fn with_quality_score(mut self, score: f32) -> Self {
        self.quality_score = score;
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// A single request in a batch generation operation
#[derive(Clone, Debug)]
pub struct BatchRequest {
    /// The prompt to generate text for
    pub prompt: String,
    /// Optional configuration (uses default if None)
    pub config: Option<GenerationConfig>,
    /// Optional identifier for tracking this request
    pub id: Option<String>,
}

impl BatchRequest {
    /// Create a new batch request with a prompt
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            config: None,
            id: None,
        }
    }

    /// Create a new batch request with prompt and config
    pub fn with_config(prompt: impl Into<String>, config: GenerationConfig) -> Self {
        Self {
            prompt: prompt.into(),
            config: Some(config),
            id: None,
        }
    }

    /// Set an identifier for this request
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Result for a single item in a batch generation operation
#[derive(Clone, Debug)]
pub struct BatchItemResult {
    /// The identifier for this request (if provided)
    pub id: Option<String>,
    /// The prompt that was used
    pub prompt: String,
    /// The generation result if successful
    pub result: Option<GenerationResult>,
    /// The error if the request failed
    pub error: Option<crate::error::Error>,
}

impl BatchItemResult {
    /// Create a successful batch item result
    pub fn success(id: Option<String>, prompt: String, result: GenerationResult) -> Self {
        Self {
            id,
            prompt,
            result: Some(result),
            error: None,
        }
    }

    /// Create a failed batch item result
    pub fn failure(id: Option<String>, prompt: String, error: crate::error::Error) -> Self {
        Self {
            id,
            prompt,
            result: None,
            error: Some(error),
        }
    }

    /// Check if this result is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if this result failed
    pub fn is_failure(&self) -> bool {
        self.error.is_some()
    }
}

/// Result of a batch generation operation
#[derive(Clone, Debug)]
pub struct BatchGenerationResult {
    /// Results for each item in the batch
    pub results: Vec<BatchItemResult>,
    /// Total number of requests
    pub total: usize,
    /// Number of successful requests
    pub successful: usize,
    /// Number of failed requests
    pub failed: usize,
    /// Total duration of the batch operation
    pub duration: Duration,
}

impl BatchGenerationResult {
    /// Create a new batch generation result
    pub fn new(results: Vec<BatchItemResult>, duration: Duration) -> Self {
        let successful = results.iter().filter(|r| r.is_success()).count();
        let failed = results.len() - successful;
        
        Self {
            total: results.len(),
            successful,
            failed,
            results,
            duration,
        }
    }

    /// Get all successful results
    pub fn successes(&self) -> Vec<&GenerationResult> {
        self.results
            .iter()
            .filter_map(|r| r.result.as_ref())
            .collect()
    }

    /// Get all failed results with their errors
    pub fn failures(&self) -> Vec<(&str, &crate::error::Error)> {
        self.results
            .iter()
            .filter_map(|r| r.error.as_ref().map(|e| (r.prompt.as_str(), e)))
            .collect()
    }

    /// Check if all requests succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }

    /// Check if any request failed
    pub fn any_failed(&self) -> bool {
        self.failed > 0
    }
}

/// A chat message with role and content
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender (system, user, or assistant)
    pub role: String,
    /// Content of the message
    pub content: String,
}

impl ChatMessage {
    /// Create a new chat message
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
}

/// Configuration for chat completion requests
#[derive(Clone, Debug, Serialize)]
pub struct ChatCompletionConfig {
    /// Model ID to use for completion
    pub model_id: String,
    /// Request timeout
    pub timeout: Duration,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Temperature for generation (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Stop sequences to halt generation
    pub stop_sequences: Vec<String>,
    /// Repetition penalty
    pub repetition_penalty: Option<f32>,
}

impl Default for ChatCompletionConfig {
    fn default() -> Self {
        Self {
            model_id: crate::models::DEFAULT_MODEL.to_string(),
            timeout: Duration::from_secs(120),
            max_tokens: crate::models::DEFAULT_MAX_TOKENS,
            temperature: Some(0.7),
            top_p: Some(1.0),
            top_k: Some(50),
            stop_sequences: vec![],
            repetition_penalty: Some(1.1),
        }
    }
}

impl ChatCompletionConfig {
    /// Set the model ID
    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens.min(crate::models::MAX_TOKENS_LIMIT);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.max(0.0).min(2.0));
        self
    }

    /// Set top-p parameter
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set top-k parameter
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Set stop sequences
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = stop_sequences;
        self
    }

    /// Set repetition penalty
    pub fn with_repetition_penalty(mut self, penalty: f32) -> Self {
        self.repetition_penalty = Some(penalty);
        self
    }
}

/// Result of a chat completion request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionResult {
    /// Generated message content
    pub message: ChatMessage,
    /// Model ID used for completion
    pub model_id: String,
    /// Number of prompt tokens used (if available)
    pub prompt_tokens: Option<u32>,
    /// Number of completion tokens used (if available)
    pub completion_tokens: Option<u32>,
    /// Total tokens used (if available)
    pub total_tokens: Option<u32>,
    /// Finish reason (if available)
    pub finish_reason: Option<String>,
    /// Request ID for tracking
    pub request_id: Option<String>,
}

impl ChatCompletionResult {
    /// Create a new chat completion result
    pub fn new(message: ChatMessage, model_id: String) -> Self {
        Self {
            message,
            model_id,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            finish_reason: None,
            request_id: None,
        }
    }

    /// Set token usage information
    pub fn with_tokens(mut self, prompt: u32, completion: u32, total: u32) -> Self {
        self.prompt_tokens = Some(prompt);
        self.completion_tokens = Some(completion);
        self.total_tokens = Some(total);
        self
    }

    /// Set finish reason
    pub fn with_finish_reason(mut self, reason: impl Into<String>) -> Self {
        self.finish_reason = Some(reason.into());
        self
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Get the content of the generated message
    pub fn content(&self) -> &str {
        &self.message.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::new("user", "Hello");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello");

        let system_msg = ChatMessage::system("You are a helpful assistant");
        assert_eq!(system_msg.role, "system");
        assert_eq!(system_msg.content, "You are a helpful assistant");

        let user_msg = ChatMessage::user("What is Rust?");
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "What is Rust?");

        let assistant_msg = ChatMessage::assistant("Rust is a systems programming language");
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(assistant_msg.content, "Rust is a systems programming language");
    }

    #[test]
    fn test_chat_completion_config_default() {
        let config = ChatCompletionConfig::default();
        assert_eq!(config.model_id, crate::models::DEFAULT_MODEL);
        assert_eq!(config.max_tokens, crate::models::DEFAULT_MAX_TOKENS);
        assert_eq!(config.timeout.as_secs(), 120);
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.top_p, Some(1.0));
        assert_eq!(config.top_k, Some(50));
        assert_eq!(config.repetition_penalty, Some(1.1));
    }

    #[test]
    fn test_chat_completion_config_builder() {
        let config = ChatCompletionConfig::default()
            .with_model("test-model")
            .with_max_tokens(1000)
            .with_temperature(0.9)
            .with_top_p(0.95)
            .with_top_k(40)
            .with_stop_sequences(vec!["\n".to_string(), "END".to_string()])
            .with_repetition_penalty(1.2)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.model_id, "test-model");
        assert_eq!(config.max_tokens, 1000);
        assert_eq!(config.temperature, Some(0.9));
        assert_eq!(config.top_p, Some(0.95));
        assert_eq!(config.top_k, Some(40));
        assert_eq!(config.stop_sequences.len(), 2);
        assert_eq!(config.repetition_penalty, Some(1.2));
        assert_eq!(config.timeout.as_secs(), 60);
    }

    #[test]
    fn test_chat_completion_config_temperature_clamping() {
        let config = ChatCompletionConfig::default().with_temperature(-1.0);
        assert_eq!(config.temperature, Some(0.0)); // Clamped to minimum

        let config = ChatCompletionConfig::default().with_temperature(3.0);
        assert_eq!(config.temperature, Some(2.0)); // Clamped to maximum
    }

    #[test]
    fn test_chat_completion_config_max_tokens_clamping() {
        let config = ChatCompletionConfig::default().with_max_tokens(200_000);
        assert_eq!(config.max_tokens, crate::models::MAX_TOKENS_LIMIT);
    }

    #[test]
    fn test_chat_completion_result_creation() {
        let message = ChatMessage::assistant("Hello, world!");
        let result = ChatCompletionResult::new(message.clone(), "test-model".to_string());

        assert_eq!(result.message.role, "assistant");
        assert_eq!(result.message.content, "Hello, world!");
        assert_eq!(result.model_id, "test-model");
        assert!(result.prompt_tokens.is_none());
        assert!(result.completion_tokens.is_none());
        assert!(result.total_tokens.is_none());
        assert!(result.finish_reason.is_none());
        assert!(result.request_id.is_none());
    }

    #[test]
    fn test_chat_completion_result_builder() {
        let message = ChatMessage::assistant("Response");
        let result = ChatCompletionResult::new(message, "model".to_string())
            .with_tokens(10, 20, 30)
            .with_finish_reason("stop")
            .with_request_id("req-123".to_string());

        assert_eq!(result.prompt_tokens, Some(10));
        assert_eq!(result.completion_tokens, Some(20));
        assert_eq!(result.total_tokens, Some(30));
        assert_eq!(result.finish_reason, Some("stop".to_string()));
        assert_eq!(result.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_chat_completion_result_content() {
        let message = ChatMessage::assistant("Test content");
        let result = ChatCompletionResult::new(message, "model".to_string());
        assert_eq!(result.content(), "Test content");
    }
}

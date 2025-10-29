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

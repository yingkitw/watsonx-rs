//! WatsonX Orchestrate types and data structures
//!
//! This module contains types specific to WatsonX Orchestrate functionality,
//! including custom assistants, agents, tools, skills, and document management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Configuration for WatsonX Orchestrate operations
/// Simplified to match wxo-client-main pattern - just needs instance_id
#[derive(Clone, Debug)]
pub struct OrchestrateConfig {
    /// Instance ID for Watson Orchestrate (required)
    pub instance_id: String,
    /// Region (defaults to us-south, can be set via WXO_REGION env var)
    pub region: String,
}

impl OrchestrateConfig {
    /// Create configuration from environment variables
    /// Reads: WXO_INSTANCE_ID (required), WXO_REGION (optional, defaults to us-south)
    pub fn from_env() -> Result<Self, String> {
        use std::env;
        
        let instance_id = env::var("WXO_INSTANCE_ID")
            .map_err(|_| "WXO_INSTANCE_ID must be set in environment variables".to_string())?;
        
        let region = env::var("WXO_REGION")
            .unwrap_or_else(|_| "us-south".to_string());
        
        Ok(Self {
            instance_id,
            region,
        })
    }

    /// Create a new Orchestrate configuration with instance ID
    pub fn new(instance_id: String) -> Self {
        Self {
            instance_id,
            region: "us-south".to_string(),
        }
    }

    /// Get the base URL (constructed from region and instance_id, matching wxo-client pattern)
    pub fn get_base_url(&self) -> String {
        format!(
            "https://api.{}.watson-orchestrate.cloud.ibm.com/instances/{}/v1/orchestrate",
            self.region, self.instance_id
        )
    }
}

/// Simple Agent information (matches Watson Orchestrate API response)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// Agent ID from API (field name: "id")
    #[serde(rename = "id")]
    pub agent_id: String,
    /// Agent display name from API (field name: "display_name")
    #[serde(rename = "display_name")]
    pub name: String,
}

/// Custom Assistant information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomAssistant {
    /// Assistant ID
    pub id: String,
    /// Assistant name
    pub name: String,
    /// Assistant description
    pub description: Option<String>,
    /// Assistant status
    pub status: AssistantStatus,
    /// Created timestamp
    pub created_at: Option<SystemTime>,
    /// Updated timestamp
    pub updated_at: Option<SystemTime>,
    /// Configuration settings
    pub config: AssistantConfig,
    /// Associated skills
    pub skills: Vec<Skill>,
    /// Associated tools
    pub tools: Vec<Tool>,
}

/// Assistant status enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AssistantStatus {
    /// Assistant is active and ready
    Active,
    /// Assistant is inactive
    Inactive,
    /// Assistant is being trained
    Training,
    /// Assistant has errors
    Error,
    /// Assistant is being deployed
    Deploying,
}

/// Assistant configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssistantConfig {
    /// Model ID to use
    pub model_id: String,
    /// System prompt
    pub system_prompt: Option<String>,
    /// Maximum tokens
    pub max_tokens: u32,
    /// Temperature setting
    pub temperature: f32,
    /// Top-p setting
    pub top_p: f32,
    /// Whether to enable streaming
    pub enable_streaming: bool,
    /// Custom parameters
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for AssistantConfig {
    fn default() -> Self {
        Self {
            model_id: "ibm/granite-3.0-8b-instruct".to_string(),
            system_prompt: None,
            max_tokens: 2048,
            temperature: 0.7,
            top_p: 0.9,
            enable_streaming: true,
            custom_params: HashMap::new(),
        }
    }
}

/// Skill definition for assistants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    /// Skill ID
    pub id: String,
    /// Skill name
    pub name: String,
    /// Skill description
    pub description: Option<String>,
    /// Skill type
    pub skill_type: SkillType,
    /// Skill configuration
    pub config: SkillConfig,
    /// Whether skill is enabled
    pub enabled: bool,
    /// Skill version
    pub version: Option<String>,
}

/// Skill type enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SkillType {
    /// Text processing skill
    TextProcessing,
    /// Code generation skill
    CodeGeneration,
    /// Data analysis skill
    DataAnalysis,
    /// Document processing skill
    DocumentProcessing,
    /// Custom skill
    Custom(String),
}

/// Skill configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillConfig {
    /// Input parameters
    pub input_params: HashMap<String, ParameterDefinition>,
    /// Output parameters
    pub output_params: HashMap<String, ParameterDefinition>,
    /// Execution timeout
    pub timeout: Duration,
    /// Retry configuration
    pub retry_config: Option<OrchestrateRetryConfig>,
    /// Custom settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Parameter definition for skills
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Whether parameter is required
    pub required: bool,
    /// Default value
    pub default_value: Option<serde_json::Value>,
    /// Parameter description
    pub description: Option<String>,
    /// Validation rules
    pub validation: Option<ValidationRules>,
}

/// Parameter type enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter
    String,
    /// Integer parameter
    Integer,
    /// Float parameter
    Float,
    /// Boolean parameter
    Boolean,
    /// Array parameter
    Array,
    /// Object parameter
    Object,
    /// File parameter
    File,
}

/// Validation rules for parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Minimum value (for numeric types)
    pub min_value: Option<f64>,
    /// Maximum value (for numeric types)
    pub max_value: Option<f64>,
    /// Minimum length (for string types)
    pub min_length: Option<usize>,
    /// Maximum length (for string types)
    pub max_length: Option<usize>,
    /// Allowed values
    pub allowed_values: Option<Vec<serde_json::Value>>,
    /// Regex pattern (for string types)
    pub pattern: Option<String>,
}

/// Tool definition for assistants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tool {
    /// Tool ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: Option<String>,
    /// Tool type
    pub tool_type: ToolType,
    /// Tool configuration
    pub config: ToolConfig,
    /// Whether tool is enabled
    pub enabled: bool,
    /// Tool version
    pub version: Option<String>,
}

/// Tool type enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ToolType {
    /// API tool
    Api,
    /// Database tool
    Database,
    /// File system tool
    FileSystem,
    /// Web scraping tool
    WebScraping,
    /// Custom tool
    Custom(String),
}

/// Tool configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Tool endpoint URL
    pub endpoint: Option<String>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request timeout
    pub timeout: Duration,
    /// Retry configuration
    pub retry_config: Option<OrchestrateRetryConfig>,
    /// Custom settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Authentication configuration for tools
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type
    pub auth_type: AuthType,
    /// Authentication credentials
    pub credentials: HashMap<String, String>,
}

/// Authentication type enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthType {
    /// API key authentication
    ApiKey,
    /// Bearer token authentication
    Bearer,
    /// Basic authentication
    Basic,
    /// OAuth authentication
    OAuth,
    /// No authentication
    None,
}

/// Document collection for knowledge base
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentCollection {
    /// Collection ID
    pub id: String,
    /// Collection name
    pub name: String,
    /// Collection description
    pub description: Option<String>,
    /// Collection status
    pub status: CollectionStatus,
    /// Created timestamp
    pub created_at: Option<SystemTime>,
    /// Updated timestamp
    pub updated_at: Option<SystemTime>,
    /// Document count
    pub document_count: u32,
    /// Vector index configuration
    pub vector_index: Option<VectorIndexConfig>,
}

/// Collection status enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CollectionStatus {
    /// Collection is active
    Active,
    /// Collection is inactive
    Inactive,
    /// Collection is being processed
    Processing,
    /// Collection has errors
    Error,
}

/// Vector index configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    /// Index ID
    pub id: String,
    /// Embedding model
    pub embedding_model: String,
    /// Vector dimensions
    pub dimensions: u32,
    /// Index type
    pub index_type: IndexType,
    /// Similarity metric
    pub similarity_metric: SimilarityMetric,
}

/// Index type enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IndexType {
    /// HNSW index
    Hnsw,
    /// IVF index
    Ivf,
    /// Flat index
    Flat,
}

/// Similarity metric enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SimilarityMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Inner product
    InnerProduct,
}

/// Document in a collection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Document content
    pub content: String,
    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Document type
    pub document_type: DocumentType,
    /// Created timestamp
    pub created_at: Option<SystemTime>,
    /// Updated timestamp
    pub updated_at: Option<SystemTime>,
    /// Vector embedding (if available)
    pub embedding: Option<Vec<f32>>,
}

/// Document type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    /// Text document
    Text,
    /// PDF document
    Pdf,
    /// Markdown document
    Markdown,
    /// HTML document
    Html,
    /// JSON document
    Json,
    /// CSV document
    Csv,
}

/// Simple message structure for Watson Orchestrate API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Message payload for Watson Orchestrate API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessagePayload {
    pub message: Message,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub additional_properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub context: HashMap<String, serde_json::Value>,
    pub agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// Thread information for conversation management
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID
    pub thread_id: String,
    /// Agent ID associated with the thread
    pub agent_id: Option<String>,
    /// Thread title or summary
    pub title: Option<String>,
    /// Created timestamp
    pub created_at: Option<String>,
    /// Updated timestamp
    pub updated_at: Option<String>,
    /// Message count
    pub message_count: Option<u32>,
}

/// Chat message for assistant conversations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message ID
    pub id: String,
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Message timestamp
    pub timestamp: SystemTime,
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Message role enumeration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
}

/// Chat session for assistant conversations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatSession {
    /// Session ID
    pub id: String,
    /// Assistant ID
    pub assistant_id: String,
    /// Session messages
    pub messages: Vec<ChatMessage>,
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: SystemTime,
    /// Updated timestamp
    pub updated_at: SystemTime,
}

/// Request to create a custom assistant
#[derive(Clone, Debug, Serialize)]
pub struct CreateAssistantRequest {
    /// Assistant name
    pub name: String,
    /// Assistant description
    pub description: Option<String>,
    /// Assistant configuration
    pub config: AssistantConfig,
    /// Initial skills
    pub skills: Option<Vec<String>>,
    /// Initial tools
    pub tools: Option<Vec<String>>,
}

/// Request to update a custom assistant
#[derive(Clone, Debug, Serialize)]
pub struct UpdateAssistantRequest {
    /// Assistant name
    pub name: Option<String>,
    /// Assistant description
    pub description: Option<String>,
    /// Assistant configuration
    pub config: Option<AssistantConfig>,
    /// Skills to add
    pub add_skills: Option<Vec<String>>,
    /// Skills to remove
    pub remove_skills: Option<Vec<String>>,
    /// Tools to add
    pub add_tools: Option<Vec<String>>,
    /// Tools to remove
    pub remove_tools: Option<Vec<String>>,
}

/// Request to send a chat message
#[derive(Clone, Debug, Serialize)]
pub struct ChatRequest {
    /// Message content
    pub message: String,
    /// Session ID (optional, creates new session if not provided)
    pub session_id: Option<String>,
    /// Message metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Whether to enable streaming
    pub stream: bool,
}

/// Response from chat request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response message
    pub message: String,
    /// Session ID
    pub session_id: String,
    /// Message ID
    pub message_id: String,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Tool calls (if any)
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool call information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub tool_name: String,
    /// Tool parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Tool result (if available)
    pub result: Option<serde_json::Value>,
}

/// Request to create a document collection
#[derive(Clone, Debug, Serialize)]
pub struct CreateCollectionRequest {
    /// Collection name
    pub name: String,
    /// Collection description
    pub description: Option<String>,
    /// Vector index configuration
    pub vector_index: Option<VectorIndexConfig>,
}

/// Request to add documents to a collection
#[derive(Clone, Debug, Serialize)]
pub struct AddDocumentsRequest {
    /// Documents to add
    pub documents: Vec<Document>,
    /// Whether to process documents asynchronously
    pub async_processing: bool,
}

/// Search request for document collections
#[derive(Clone, Debug, Serialize)]
pub struct SearchRequest {
    /// Search query
    pub query: String,
    /// Number of results to return
    pub limit: Option<u32>,
    /// Similarity threshold
    pub threshold: Option<f32>,
    /// Search metadata filters
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// Search result from document collection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub document_id: String,
    /// Document title
    pub title: String,
    /// Document content snippet
    pub content_snippet: String,
    /// Similarity score
    pub similarity_score: f32,
    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Search response from document collection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results
    pub total_results: u32,
    /// Search metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Retry configuration for Orchestrate operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrchestrateRetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f32,
    /// Retry on specific error types
    pub retry_on_errors: Vec<String>,
}

impl Default for OrchestrateRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retry_on_errors: vec!["timeout".to_string(), "network_error".to_string()],
        }
    }
}

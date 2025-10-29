//! Tests for WatsonX Orchestrate functionality

use crate::{
    OrchestrateClient, OrchestrateConfig, AssistantConfig,
    VectorIndexConfig, IndexType, SimilarityMetric,
    ChatRequest, Document, DocumentType, SearchRequest,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_orchestrate_client_creation() {
    let config = OrchestrateConfig::new("test-instance-id".to_string());
    let client = OrchestrateClient::new(config);
    
    assert_eq!(client.config().instance_id, "test-instance-id");
    assert_eq!(client.config().region, "us-south");
    assert!(!client.is_authenticated());
}

#[tokio::test]
async fn test_orchestrate_client_with_token() {
    let config = OrchestrateConfig::new("test-instance-id".to_string());
    let client = OrchestrateClient::new(config).with_token("test-token".to_string());
    
    assert!(client.is_authenticated());
}

#[tokio::test]
async fn test_assistant_config_default() {
    let config = AssistantConfig::default();
    
    assert_eq!(config.model_id, "ibm/granite-3.0-8b-instruct");
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.temperature, 0.7);
    assert_eq!(config.top_p, 0.9);
    assert!(config.enable_streaming);
}

#[tokio::test]
async fn test_vector_index_config() {
    let config = VectorIndexConfig {
        id: "test-index".to_string(),
        embedding_model: "test-model".to_string(),
        dimensions: 384,
        index_type: IndexType::Hnsw,
        similarity_metric: SimilarityMetric::Cosine,
    };
    
    assert_eq!(config.id, "test-index");
    assert_eq!(config.dimensions, 384);
}

#[tokio::test]
async fn test_document_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), serde_json::Value::String("test".to_string()));
    
    let document = Document {
        id: "test-doc-1".to_string(),
        title: "Test Document".to_string(),
        content: "This is a test document content.".to_string(),
        metadata,
        document_type: DocumentType::Text,
        created_at: None,
        updated_at: None,
        embedding: None,
    };
    
    assert_eq!(document.id, "test-doc-1");
    assert_eq!(document.title, "Test Document");
    assert_eq!(document.document_type, DocumentType::Text);
}

#[tokio::test]
async fn test_chat_request_creation() {
    let request = ChatRequest {
        message: "Hello, assistant!".to_string(),
        session_id: Some("session-123".to_string()),
        metadata: None,
        stream: false,
    };
    
    assert_eq!(request.message, "Hello, assistant!");
    assert_eq!(request.session_id, Some("session-123".to_string()));
    assert!(!request.stream);
}

#[tokio::test]
async fn test_search_request_creation() {
    let request = SearchRequest {
        query: "test query".to_string(),
        limit: Some(10),
        threshold: Some(0.8),
        filters: None,
    };
    
    assert_eq!(request.query, "test query");
    assert_eq!(request.limit, Some(10));
    assert_eq!(request.threshold, Some(0.8));
}

#[tokio::test]
async fn test_orchestrate_config_get_base_url() {
    let config = OrchestrateConfig::new("test-instance-123".to_string());
    let base_url = config.get_base_url();
    
    // Base URL should be from WXO_URL env var or default pattern
    assert!(base_url.contains("us-south"));
    assert!(base_url.contains("watson-orchestrate.cloud.ibm.com"));
    assert!(base_url.contains("/api/v1/"));
    assert!(base_url.starts_with("https://"));
}

#[tokio::test]
async fn test_orchestrate_config_region_default() {
    let config = OrchestrateConfig::new("test-instance-123".to_string());
    let base_url = config.get_base_url();
    
    assert!(base_url.contains("us-south")); // default region
    assert_eq!(config.region, "us-south");
}

#[tokio::test]
async fn test_orchestrate_retry_config_default() {
    use crate::OrchestrateRetryConfig;
    
    let config = OrchestrateRetryConfig::default();
    
    assert_eq!(config.max_attempts, 3);
    assert_eq!(config.backoff_multiplier, 2.0);
    assert_eq!(config.retry_on_errors.len(), 2);
}

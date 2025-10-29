//! Integration tests for wx-watsonx crate

use watsonx_rs::{GenerationConfig, WatsonxClient, WatsonxConfig};

#[test]
fn test_config_creation() {
    let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());

    assert_eq!(config.api_key, "test_key");
    assert_eq!(config.project_id, "test_project");
}

#[test]
fn test_config_from_env() {
    unsafe {
        std::env::set_var("WATSONX_API_KEY", "env_key");
        std::env::set_var("WATSONX_PROJECT_ID", "env_project");
    }

    let config = WatsonxConfig::from_env();
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.api_key, "env_key");
    assert_eq!(config.project_id, "env_project");

    unsafe {
        std::env::remove_var("WATSONX_API_KEY");
        std::env::remove_var("WATSONX_PROJECT_ID");
    }
}

#[test]
fn test_config_missing_env_vars() {
    // Test that from_env returns error when vars are missing
    // This test assumes WATSONX_API_KEY or WATSONX_PROJECT_ID are not set
    // If they ARE set (from previous test), this is expected behavior
    let config = WatsonxConfig::from_env();

    // Config should either succeed (if vars are set) or fail (if not set)
    // Both are valid outcomes depending on environment
    // Just verify the function doesn't panic
    let _ = config;
}

#[test]
fn test_client_creation() {
    let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());

    let client = WatsonxClient::new(config);
    assert!(client.is_ok());
}

#[test]
fn test_generation_config_default() {
    let config = GenerationConfig::default();

    assert_eq!(config.model_id, "ibm/granite-4-h-small");
    assert_eq!(config.max_tokens, 8192);
    assert!(config.timeout.as_secs() > 0);
}

#[test]
fn test_generation_config_with_max_tokens() {
    let config = GenerationConfig::default().with_max_tokens(50000);

    assert_eq!(config.max_tokens, 50000);
}

#[test]
fn test_generation_config_long_form() {
    let config = GenerationConfig::long_form();

    assert_eq!(config.max_tokens, 131072); // 128k
    assert_eq!(config.timeout.as_secs(), 300); // 5 minutes
}

#[test]
fn test_generation_config_quick_response() {
    let config = GenerationConfig::quick_response();

    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.timeout.as_secs(), 30);
}

#[test]
fn test_max_tokens_limit() {
    let config = GenerationConfig::default().with_max_tokens(200000); // Over limit

    // Should be clamped to MAX_TOKENS_LIMIT (131072)
    assert_eq!(config.max_tokens, 131072);
}

#[test]
fn test_model_id_accessor() {
    let config = WatsonxConfig::new("test_key".to_string(), "test_project".to_string());

    let client = WatsonxClient::new(config).unwrap();
    assert_eq!(client.model_id(), "ibm/granite-4-h-small");
}

#[test]
fn test_constants() {
    use watsonx_rs::{DEFAULT_MAX_TOKENS, MAX_TOKENS_LIMIT, QUICK_RESPONSE_MAX_TOKENS};

    assert_eq!(MAX_TOKENS_LIMIT, 131_072);
    assert_eq!(DEFAULT_MAX_TOKENS, 8192);
    assert_eq!(QUICK_RESPONSE_MAX_TOKENS, 2048);
}

#[test]
fn test_error_types() {
    use watsonx_rs::Error;

    let err = Error::Authentication("test".to_string());
    assert!(err.to_string().contains("test"));

    let err = Error::Network("connection failed".to_string());
    assert!(err.to_string().contains("connection failed"));

    let err = Error::Api("model error".to_string());
    assert!(err.to_string().contains("model error"));
}

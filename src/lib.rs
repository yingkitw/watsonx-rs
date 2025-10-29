//! # WatsonX-RS
//!
//! An unofficial Rust SDK for IBM WatsonX AI platform.
//!
//! This crate provides a high-level interface to interact with IBM WatsonX AI services,
//! including text generation, streaming responses, model management, and WatsonX Orchestrate
//! custom assistants and document collections.
//!
//! ## Features
//!
//! - **Text Generation**: Generate text using various WatsonX models
//! - **Streaming Support**: Real-time streaming responses with callbacks
//! - **Authentication**: Automatic token management and refresh
//! - **Error Handling**: Comprehensive error types and handling
//! - **Async/Await**: Full async support with Tokio
//! - **Configuration**: Flexible configuration options
//! - **WatsonX Orchestrate**: Custom assistant management and document collections
//! - **Chat Functionality**: Interactive chat with custom assistants
//! - **Document Management**: Vector search and knowledge base management
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use watsonx_rs::{WatsonxClient, WatsonxConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create configuration
//!     let config = WatsonxConfig::new(
//!         "your-api-key".to_string(),
//!         "your-project-id".to_string(),
//!     );
//!
//!     // Create client
//!     let mut client = WatsonxClient::new(config)?;
//!
//!     // Connect to WatsonX
//!     client.connect().await?;
//!
//!     // Generate text
//!     let result = client.generate("Hello, world!").await?;
//!     println!("Generated: {}", result.text);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Examples
//!
//! See the `examples/` directory for more detailed usage examples.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod orchestrate;
pub mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod orchestrate_tests;

// Re-export main types for convenience
pub use client::WatsonxClient;
pub use config::WatsonxConfig;
pub use error::{Error, Result};
pub use models::*;
pub use orchestrate::OrchestrateClient;
pub use orchestrate::{OrchestrateConfig, Agent, Message, MessagePayload};
pub use orchestrate::*;
pub use types::*;

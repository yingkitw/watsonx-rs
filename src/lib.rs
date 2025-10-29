//! # WatsonX-RS
//!
//! An unofficial Rust SDK for IBM WatsonX AI platform.
//!
//! This crate provides a high-level interface to interact with IBM WatsonX AI services,
//! including text generation, streaming responses, and model management.
//!
//! ## Features
//!
//! - **Text Generation**: Generate text using various WatsonX models
//! - **Streaming Support**: Real-time streaming responses with callbacks
//! - **Authentication**: Automatic token management and refresh
//! - **Error Handling**: Comprehensive error types and handling
//! - **Async/Await**: Full async support with Tokio
//! - **Configuration**: Flexible configuration options
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
pub mod types;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use client::WatsonxClient;
pub use config::WatsonxConfig;
pub use error::{Error, Result};
pub use models::*;
pub use types::*;

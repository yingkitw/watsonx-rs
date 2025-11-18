//! Watson Orchestrate SDK Module
//!
//! This module provides comprehensive support for IBM Watson Orchestrate,
//! including agent management, conversation threading, tool execution,
//! and document handling.
//!
//! ## Module Organization
//!
//! - `config` - Configuration management
//! - `client` - Core client implementation
//! - `types` - All types and data structures
//! - `agent` - Agent management operations
//! - `thread` - Thread management operations
//! - Additional modules for other operations

pub mod config;
pub mod client;
pub mod types;
pub mod agent;
pub mod thread;
pub mod tool;
pub mod run;
pub mod collection;
pub mod chat;
pub mod connection;

pub use config::OrchestrateConfig;
pub use client::OrchestrateClient;
pub use connection::OrchestrateConnection;
pub use types::*;

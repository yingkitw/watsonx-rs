//! Example demonstrating batch text generation with concurrent execution
//!
//! This example shows how to:
//! - Generate text for multiple prompts concurrently
//! - Use batch requests with custom IDs
//! - Handle batch results with per-item error handling
//! - Use the simple batch method for uniform configurations
//!
//! Different colors are used for each request to visualize parallel execution

use watsonx_rs::{WatsonxClient, WatsonxConfig, BatchRequest, GenerationConfig, models::models};
use std::collections::HashMap;

// ANSI color codes for terminal output
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

// Color palette for different threads/requests
const COLORS: &[&str] = &[
    "\x1b[31m", // Red
    "\x1b[32m", // Green
    "\x1b[33m", // Yellow
    "\x1b[34m", // Blue
    "\x1b[35m", // Magenta
    "\x1b[36m", // Cyan
    "\x1b[91m", // Bright Red
    "\x1b[92m", // Bright Green
    "\x1b[93m", // Bright Yellow
    "\x1b[94m", // Bright Blue
];

fn get_color(index: usize) -> &'static str {
    COLORS[index % COLORS.len()]
}

fn colorize(text: &str, color: &str) -> String {
    format!("{}{}{}{}", color, BOLD, text, RESET)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = WatsonxConfig::from_env()?;
    let mut client = WatsonxClient::new(config)?;
    
    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    client.connect().await?;
    println!("Connected!\n");

    // Create default configuration
    let default_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL)
        .with_max_tokens(200);

    // Example 1: Batch generation with custom IDs and configurations
    println!("{}", colorize("=== Example 1: Batch Generation with Custom IDs ===", "\x1b[1m"));
    println!("{}Note: Each request runs in a separate thread with different colors{}\n", 
        "\x1b[90m", RESET);
    
    let requests = vec![
        BatchRequest::new("Write a haiku about Rust programming")
            .with_id("haiku-1"),
        BatchRequest::new("Explain async/await in one sentence")
            .with_id("async-1"),
        BatchRequest::new("What is ownership in Rust?")
            .with_id("ownership-1"),
        BatchRequest::new("List three benefits of Rust")
            .with_id("benefits-1"),
    ];

    // Create a color map for each request ID
    let mut color_map: HashMap<String, &str> = HashMap::new();
    for (idx, req) in requests.iter().enumerate() {
        let id = req.id.as_ref().map(|s| s.clone())
            .unwrap_or_else(|| format!("req-{}", idx));
        color_map.insert(id, get_color(idx));
    }

    println!("{}Starting {} concurrent requests...{}\n", 
        "\x1b[90m", requests.len(), RESET);
    
    let batch_result = client.generate_batch(requests, &default_config).await?;
    
    println!("\n{}Batch completed in {:.2}s{}", 
        "\x1b[90m", batch_result.duration.as_secs_f64(), RESET);
    println!("Total: {}, Successful: {}, Failed: {}\n", 
        batch_result.total, batch_result.successful, batch_result.failed);

    // Display results with colors
    for item in batch_result.results {
        let id = item.id.as_ref().map(|s| s.as_str())
            .unwrap_or("unknown");
        let color = color_map.get(id).copied().unwrap_or(RESET);
        
        if let Some(result) = item.result {
            println!("{}[{}]{} {}", 
                color, 
                colorize(id, color),
                RESET,
                result.text);
        } else if let Some(error) = item.error {
            println!("{}[{}]{} {}Error: {}{}", 
                color,
                colorize(id, color),
                RESET,
                "\x1b[91m",
                error,
                RESET);
        }
    }

    println!("\n{}", colorize("=== Example 2: Simple Batch Generation ===", "\x1b[1m"));
    
    let prompts = vec![
        "What is a trait in Rust?".to_string(),
        "Explain the borrow checker in one sentence".to_string(),
        "What is pattern matching?".to_string(),
    ];

    println!("{}Starting {} concurrent requests...{}\n", 
        "\x1b[90m", prompts.len(), RESET);

    let batch_result = client.generate_batch_simple(prompts, &default_config).await?;
    
    println!("\n{}Batch completed in {:.2}s{}", 
        "\x1b[90m", batch_result.duration.as_secs_f64(), RESET);
    println!("Total: {}, Successful: {}, Failed: {}\n", 
        batch_result.total, batch_result.successful, batch_result.failed);

    // Display successful results with colors
    for (idx, result) in batch_result.successes().iter().enumerate() {
        let color = get_color(idx);
        let label = format!("Result {}", idx + 1);
        println!("{} {}", 
            colorize(&label, color),
            result.text);
    }

    // Display failures if any
    if batch_result.any_failed() {
        println!("\n{}Failures:{}", "\x1b[91m", RESET);
        for (idx, (prompt, error)) in batch_result.failures().iter().enumerate() {
            let color = get_color(batch_result.successful + idx);
            println!("  {}Prompt: '{}'{} - Error: {}", 
                color, prompt, RESET, error);
        }
    }

    println!("\n{}", colorize("=== Example 3: Mixed Configurations ===", "\x1b[1m"));
    
    let quick_config = GenerationConfig::quick_response()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let requests = vec![
        BatchRequest::with_config(
            "Write a short greeting",
            quick_config.clone()
        ).with_id("greeting-1"),
        BatchRequest::with_config(
            "Write a short farewell",
            quick_config.clone()
        ).with_id("farewell-1"),
        // This one uses default_config
        BatchRequest::new("Explain memory safety in Rust")
            .with_id("memory-1"),
    ];

    // Create color map for this batch
    let mut color_map: HashMap<String, &str> = HashMap::new();
    for (idx, req) in requests.iter().enumerate() {
        let id = req.id.as_ref().map(|s| s.clone())
            .unwrap_or_else(|| format!("req-{}", idx));
        color_map.insert(id, get_color(idx));
    }

    println!("{}Starting {} concurrent requests with mixed configs...{}\n", 
        "\x1b[90m", requests.len(), RESET);

    let batch_result = client.generate_batch(requests, &default_config).await?;
    
    println!("\n{}Batch completed in {:.2}s{}\n", 
        "\x1b[90m", batch_result.duration.as_secs_f64(), RESET);
    
    for item in batch_result.results {
        let id = item.id.as_ref().map(|s| s.as_str())
            .unwrap_or("unknown");
        let color = color_map.get(id).copied().unwrap_or(RESET);
        
        if let Some(result) = item.result {
            println!("{}[{}]{} {}", 
                color,
                colorize(id, color),
                RESET,
                result.text);
        }
    }

    Ok(())
}


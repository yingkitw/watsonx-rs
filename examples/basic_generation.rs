//! Basic WatsonX text generation example
//!
//! This example shows how to:
//! 1. Create a WatsonX client
//! 2. Connect to WatsonX
//! 3. Generate text with real-time streaming output

use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration from environment variables
    // Set WATSONX_API_KEY and WATSONX_PROJECT_ID in your .env file
    let config = WatsonxConfig::from_env()?;

    // Create client
    let mut client = WatsonxClient::new(config)?;

    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    client.connect().await?;
    println!("Connected successfully!");

    // Generate text with streaming for real-time output
    let prompt = "Explain the benefits of using Rust for web development.";
    println!("Generating text for prompt: {}", prompt);
    println!("Streaming response:");
    println!("---");
    
    let result = client.generate_text_stream(prompt, &GenerationConfig::default(), |chunk| {
        // Print each chunk as it arrives
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }).await?;
    
    println!("\n---");
    println!("Model used: {}", result.model_id);

    Ok(())
}

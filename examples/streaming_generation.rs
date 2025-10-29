//! Streaming text generation example
//!
//! This example shows how to:
//! 1. Use streaming generation with real-time callbacks
//! 2. Process text as it's generated
//! 3. Handle streaming responses

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

    // Create generation configuration
    let gen_config = GenerationConfig::default()
        .with_max_tokens(500);

    // Generate text with streaming callback
    let prompt = "Write a poem about the beauty of nature.";
    println!("Generating text for prompt: {}", prompt);
    println!("Streaming response:");
    println!("---");

    let result = client
        .generate_text_stream(prompt, &gen_config, |chunk| {
            // Print each chunk as it arrives
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        })
        .await?;

    println!("\n---");
    println!("Complete response: {}", result.text);
    println!("Model used: {}", result.model_id);

    Ok(())
}

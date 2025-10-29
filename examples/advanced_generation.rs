//! Advanced WatsonX text generation example
//!
//! This example shows how to:
//! 1. Use environment variables for configuration
//! 2. Configure generation parameters
//! 3. Use different models
//! 4. Handle errors properly

use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration from environment variables
    // Set WATSONX_API_KEY and WATSONX_PROJECT_ID in your .env file
    let config = WatsonxConfig::from_env()?;

    // Create client with a specific model
    let mut client = WatsonxClient::new(config)?
        .with_model(models::models::GRANITE_3_3_8B_INSTRUCT);

    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    client.connect().await?;
    println!("Connected successfully!");

    // Create custom generation configuration
    let gen_config = GenerationConfig::default()
        .with_max_tokens(1000)
        .with_top_k(40)
        .with_top_p(0.9)
        .with_timeout(Duration::from_secs(60))
        .with_stop_sequences(vec!["END".to_string()]);

    // Generate text with custom configuration and streaming
    let prompt = "Write a short story about a robot learning to paint.";
    println!("Generating text for prompt: {}", prompt);
    println!("Streaming response:");
    
    match client.generate_text_stream(prompt, &gen_config, |chunk| {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }).await {
        Ok(result) => {
            println!("\nModel used: {}", result.model_id);
            if let Some(request_id) = result.request_id {
                println!("Request ID: {}", request_id);
            }
        }
        Err(e) => {
            eprintln!("Error generating text: {}", e);
        }
    }

    Ok(())
}

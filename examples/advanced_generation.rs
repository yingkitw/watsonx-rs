//! Advanced WatsonX text generation example
//!
//! This example shows how to:
//! 1. Use environment variables for configuration
//! 2. Configure generation parameters
//! 3. Use different models
//! 4. Handle errors properly

use watsonx_rs::{WatsonxConnection, GenerationConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // One-line connection!
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
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

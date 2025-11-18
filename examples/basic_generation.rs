//! Basic WatsonX text generation example
//!
//! This example shows how to:
//! 1. Create a WatsonX client (simplified)
//! 2. Generate text with real-time streaming output

use watsonx_rs::{WatsonxConnection, GenerationConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // One-line connection!
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
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

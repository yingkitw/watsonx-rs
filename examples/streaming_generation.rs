//! Streaming text generation example
//!
//! This example shows how to:
//! 1. Use streaming generation with real-time callbacks
//! 2. Process text as it's generated
//! 3. Handle streaming responses

use watsonx_rs::{WatsonxConnection, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = WatsonxConnection::new().from_env().await?;

    // Create generation configuration
    let gen_config = GenerationConfig::default()
        .with_max_tokens(500)
        .with_model(models::GRANITE_4_H_SMALL);

    // Generate text with streaming callback
    let prompt = "Write a poem about the beauty of nature.";
    println!("Generating text for prompt: {}", prompt);
    println!("Streaming response:");
    println!("---");

    let result = (&client)
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

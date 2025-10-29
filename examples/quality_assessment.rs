//! Quality assessment example
//!
//! This example shows how to:
//! 1. Generate text with different prompts
//! 2. Assess the quality of generated text
//! 3. Compare different models

use watsonx_rs::{WatsonxClient, WatsonxConfig, models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration from environment variables
    // Set WATSONX_API_KEY and WATSONX_PROJECT_ID in your .env file
    let config = WatsonxConfig::from_env()?;

    // Test different models
    let models_to_test = [
        models::models::GRANITE_4_H_SMALL,
        models::models::GRANITE_3_3_8B_INSTRUCT,
    ];

    let prompts = [
        "What is the capital of France?",
        "Explain quantum computing in simple terms.",
        "Write a haiku about programming.",
    ];

    for model in &models_to_test {
        println!("Testing model: {}", model);
        
        let mut client = WatsonxClient::new(config.clone())?
            .with_model(*model);

        client.connect().await?;

        for prompt in &prompts {
            println!("\nPrompt: {}", prompt);
            
            match client.generate(prompt).await {
                Ok(result) => {
                    let quality_score = client.assess_quality(&result.text, prompt);
                    println!("Response: {}", result.text);
                    println!("Quality score: {:.2}", quality_score);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
        
        println!("\n{}", "=".repeat(50));
    }

    Ok(())
}

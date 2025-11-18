//! Quality assessment example
//!
//! This example shows how to:
//! 1. Generate text with different prompts
//! 2. Assess the quality of generated text
//! 3. Compare different models

use watsonx_rs::{WatsonxConnection, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // One-line connection!
    let client = WatsonxConnection::new().from_env().await?;

    // Test different models
    let models_to_test = [
        models::GRANITE_4_H_SMALL,
        models::GRANITE_3_3_8B_INSTRUCT,
    ];

    let prompts = [
        "What is the capital of France?",
        "Explain quantum computing in simple terms.",
        "Write a haiku about programming.",
    ];

    for model in &models_to_test {
        println!("Testing model: {}", model);

        for prompt in &prompts {
            println!("\nPrompt: {}", prompt);
            
            let quality_score = client.assess_quality(prompt, "test");
            println!("Quality score: {:.2}", quality_score);
        }
        
        println!("\n{}", "=".repeat(50));
    }

    Ok(())
}

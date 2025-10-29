//! Example demonstrating how to use model constants from models.rs

use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration from environment variables
    let config = WatsonxConfig::from_env()?;

    // Create client
    let mut client = WatsonxClient::new(config)?;

    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    client.connect().await?;
    println!("Connected successfully!");

    // Example 1: Use different model constants
    let models_to_test = vec![
        ("Granite 4 H Small", models::GRANITE_4_H_SMALL),
        ("Granite 3.3 8B Instruct", models::GRANITE_3_3_8B_INSTRUCT),
        ("Llama 3.3 70B Instruct", models::LLAMA_3_3_70B_INSTRUCT),
        ("Mistral Medium 2505", models::MISTRAL_MEDIUM_2505),
    ];

    let prompt = "Write a haiku about programming in Rust.";

    for (name, model_id) in models_to_test {
        println!("\n{}", "=".repeat(60));
        println!("Testing model: {} ({})", name, model_id);
        println!("{}", "=".repeat(60));

        let gen_config = GenerationConfig::default()
            .with_model(model_id)
            .with_max_tokens(100);

        match client.generate_text_stream(prompt, &gen_config, |chunk| {
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }).await {
            Ok(result) => {
                println!("\n\n✅ Successfully generated with model: {}", result.model_id);
            }
            Err(e) => {
                println!("\n❌ Error with model {}: {}", model_id, e);
            }
        }
    }

    // Example 2: Compare with live model list
    println!("\n{}", "=".repeat(60));
    println!("Comparing constants with live API models");
    println!("{}", "=".repeat(60));

    let live_models = client.list_models().await?;
    let constant_models = vec![
        models::GRANITE_4_H_SMALL,
        models::GRANITE_3_3_8B_INSTRUCT,
        models::LLAMA_3_3_70B_INSTRUCT,
        models::MISTRAL_MEDIUM_2505,
    ];

    for model_id in constant_models {
        let is_available = live_models.iter().any(|m| m.model_id == model_id);
        println!("{} {} - {}", 
            if is_available { "✅" } else { "❌" },
            model_id,
            if is_available { "Available" } else { "Not found in live list" }
        );
    }

    Ok(())
}

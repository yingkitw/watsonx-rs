//! Simplified WatsonX AI Example
//!
//! This example shows the easiest way to connect to WatsonX AI
//! and generate text.
//!
//! Setup:
//! 1. Create a .env file with:
//!    WATSONX_API_KEY=your-api-key
//!    WATSONX_PROJECT_ID=your-project-id
//!    MODEL_ID=ibm/granite-4-h-small (optional)
//!
//! 2. Run: cargo run --example basic_simple

use watsonx_rs::{WatsonxConnection, GenerationConfig};
use watsonx_rs::models::models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    println!("🚀 WatsonX AI - Simplified Connection");
    println!("=====================================\n");

    // ONE LINE CONNECTION - that's it!
    println!("📡 Connecting to WatsonX AI...");
    let client = match WatsonxConnection::new().from_env().await {
        Ok(client) => {
            println!("✅ Connected successfully!\n");
            client
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("\n📋 Setup required:");
            println!("   1. Create .env file with:");
            println!("      WATSONX_API_KEY=your-api-key");
            println!("      WATSONX_PROJECT_ID=your-project-id");
            println!("   2. Run: cargo run --example basic_simple");
            return Err(e.into());
        }
    };

    // Generate text with streaming
    println!("📝 Generating text...");
    let prompt = "Explain Rust ownership in one sentence.";
    println!("Prompt: {}\n", prompt);
    println!("Response:");
    println!("---");

    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);

    match (&client).generate_text_stream(prompt, &gen_config, |chunk| {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }).await {
        Ok(result) => {
            println!("\n---");
            println!("\n✅ Generation completed!");
            println!("   Model: {}", result.model_id);
        }
        Err(e) => {
            println!("❌ Generation failed: {}", e);
        }
    }

    println!("\n🎉 Example completed!");
    println!("\n💡 Next steps:");
    println!("   - Check examples/basic_generation.rs for more features");
    println!("   - Check examples/streaming_generation.rs for streaming options");
    println!("   - Check examples/batch_generation.rs for batch processing");

    Ok(())
}

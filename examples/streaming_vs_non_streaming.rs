//! Example demonstrating both streaming and non-streaming text generation

use watsonx_rs::{WatsonxConnection, GenerationConfig, models::models};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // One-line connection!
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
    println!("Connected successfully!");

    let prompt = "Write a short story about a robot learning to paint.";
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL)
        .with_max_tokens(200);

    println!("Prompt: {}", prompt);
    println!("{}", "=".repeat(80));

    // Example 1: Non-streaming generation
    println!("\n🔄 NON-STREAMING GENERATION");
    println!("{}", "-".repeat(40));
    
    let start_time = Instant::now();
    match client.generate_text(prompt, &gen_config).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            println!("✅ Non-streaming completed in {:.2}s", duration.as_secs_f64());
            println!("Model: {}", result.model_id);
            if let Some(request_id) = result.request_id {
                println!("Request ID: {}", request_id);
            }
            println!("\nGenerated text:");
            println!("{}", result.text);
        }
        Err(e) => {
            println!("❌ Non-streaming error: {}", e);
        }
    }

    println!("\n{}", "=".repeat(80));

    // Example 2: Streaming generation
    println!("\n🌊 STREAMING GENERATION");
    println!("{}", "-".repeat(40));
    
    let start_time = Instant::now();
    match client.generate_text_stream(prompt, &gen_config, |chunk| {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            println!("\n\n✅ Streaming completed in {:.2}s", duration.as_secs_f64());
            println!("Model: {}", result.model_id);
            if let Some(request_id) = result.request_id {
                println!("Request ID: {}", request_id);
            }
        }
        Err(e) => {
            println!("\n❌ Streaming error: {}", e);
        }
    }

    println!("\n{}", "=".repeat(80));

    // Example 3: Compare performance
    println!("\n⚡ PERFORMANCE COMPARISON");
    println!("{}", "-".repeat(40));
    
    let short_prompt = "Explain quantum computing in one sentence.";
    let quick_config = GenerationConfig::default()
        .with_model(models::GRANITE_3_3_8B_INSTRUCT)
        .with_max_tokens(50);

    // Non-streaming timing
    let start_time = Instant::now();
    let non_stream_result = client.generate_text(short_prompt, &quick_config).await;
    let non_stream_duration = start_time.elapsed();

    // Streaming timing
    let start_time = Instant::now();
    let stream_result = client.generate_text_stream(short_prompt, &quick_config, |_| {
        // Minimal callback to avoid I/O overhead
    }).await;
    let stream_duration = start_time.elapsed();

    println!("Non-streaming: {:.2}s", non_stream_duration.as_secs_f64());
    println!("Streaming: {:.2}s", stream_duration.as_secs_f64());
    
    if let (Ok(ns), Ok(s)) = (non_stream_result, stream_result) {
        println!("Both methods produced {} characters", ns.text.len());
        println!("Results match: {}", ns.text == s.text);
    }

    println!("\n{}", "=".repeat(80));
    println!("📝 SUMMARY");
    println!("{}", "-".repeat(40));
    println!("• Non-streaming: Returns complete response after generation finishes");
    println!("• Streaming: Provides real-time output as text is generated");
    println!("• Both methods use the same underlying parameters and configuration");
    println!("• Choose streaming for interactive applications, non-streaming for batch processing");

    Ok(())
}

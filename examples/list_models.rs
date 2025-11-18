//! List available models example
//!
//! This example shows how to:
//! 1. Connect to WatsonX
//! 2. List all available foundation models
//! 3. Display model information

use watsonx_rs::WatsonxConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // One-line connection!
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
    println!("Connected successfully!");

    // List available models
    println!("Fetching available models...");
    let models = client.list_models().await?;
    
    println!("Found {} available models:", models.len());
    println!("{}", "=".repeat(80));
    
    for model in models {
        println!("Model ID: {}", model.model_id);
        
        if let Some(name) = model.name {
            println!("  Name: {}", name);
        }
        
        if let Some(description) = model.description {
            println!("  Description: {}", description);
        }
        
        if let Some(provider) = model.provider {
            println!("  Provider: {}", provider);
        }
        
        if let Some(version) = model.version {
            println!("  Version: {}", version);
        }
        
        if let Some(tasks) = model.supported_tasks {
            println!("  Supported Tasks: {}", tasks.join(", "));
        }
        
        if let Some(context_length) = model.max_context_length {
            println!("  Max Context Length: {}", context_length);
        }
        
        if let Some(available) = model.available {
            println!("  Available: {}", available);
        }
        
        println!("{}", "-".repeat(40));
    }

    Ok(())
}

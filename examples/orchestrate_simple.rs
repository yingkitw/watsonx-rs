//! Simplified Watson Orchestrate Example
//!
//! This example shows the easiest way to connect to Watson Orchestrate
//! and perform basic operations.
//!
//! Setup:
//! 1. Create a .env file with:
//!    WXO_INSTANCE_ID=your-instance-id
//!    WXO_KEY=your-api-key
//!    WXO_REGION=us-south (optional, defaults to us-south)
//!
//! 2. Run: cargo run --example orchestrate_simple

use watsonx_rs::OrchestrateConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();

    println!("🚀 Watson Orchestrate - Simplified Connection");
    println!("==============================================\n");

    // ONE LINE CONNECTION - that's it!
    println!("📡 Connecting to Watson Orchestrate...");
    let client = match OrchestrateConnection::new().from_env().await {
        Ok(client) => {
            println!("✅ Connected successfully!");
            println!("   Base URL: {}\n", client.config().get_base_url());
            client
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("\n📋 Setup required:");
            println!("   1. Create .env file with:");
            println!("      WXO_INSTANCE_ID=your-instance-id");
            println!("      WXO_KEY=your-api-key");
            println!("   2. Run: cargo run --example orchestrate_simple");
            return Err(e.into());
        }
    };

    // Now use the client for any operations
    println!("📝 Listing assistants...");
    match client.list_assistants().await {
        Ok(assistants) => {
            if assistants.is_empty() {
                println!("ℹ️  No assistants found in this instance");
            } else {
                println!("✅ Found {} assistants:", assistants.len());
                for assistant in assistants {
                    println!("   - {} (ID: {})", assistant.name, assistant.id);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to list assistants: {}", e);
        }
    }

    println!("\n🎉 Example completed!");
    println!("\n💡 Next steps:");
    println!("   - Check examples/orchestrate_advanced.rs for more features");
    println!("   - Check examples/orchestrate_use_cases.rs for practical examples");

    Ok(())
}

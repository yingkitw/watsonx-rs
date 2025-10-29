//! Test script to check if agent has documents

use watsonx_rs::{OrchestrateClient, OrchestrateConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("🔍 Testing Agent Documents");
    println!("==========================\n");

    // Initialize client
    let config = OrchestrateConfig::from_env()
        .expect("Failed to load Orchestrate config from environment");

    // Get Watson Orchestrate API key
    let api_key = std::env::var("WXO_KEY")
        .or_else(|_| std::env::var("WATSONX_API_KEY"))
        .or_else(|_| std::env::var("IAM_API_KEY"))
        .or_else(|_| std::env::var("WO_API_KEY"))
        .expect("WXO_KEY or similar must be set");

    // Generate JWT token
    println!("🔐 Generating JWT token...");
    let token = OrchestrateClient::generate_jwt_token(&api_key).await?;
    println!("✅ Token generated\n");

    let client = OrchestrateClient::new(config).with_token(token);

    // Get agents
    println!("📋 Listing agents...");
    let agents = client.list_agents().await?;
    println!("Found {} agents\n", agents.len());

    // Try to list collections globally
    println!("📚 Checking Collections Endpoint:");
    match client.list_collections().await {
        Ok(collections) => {
            if collections.is_empty() {
                println!("  ℹ️  No collections available globally");
            } else {
                println!("  Found {} collections:", collections.len());
                for col in &collections {
                    println!("    - {} ({})", col.name, col.id);
                    println!("      Description: {}", col.description.as_deref().unwrap_or("N/A"));
                    
                    // Try to search in this collection
                    println!("      Searching for documents...");
                    let search_req = watsonx_rs::SearchRequest {
                        query: "*".to_string(),
                        limit: Some(10),
                        threshold: None,
                        filters: None,
                    };
                    
                    match client.search_documents(&col.id, search_req).await {
                        Ok(results) => {
                            println!("      Found {} documents:", results.results.len());
                            for doc in &results.results {
                                println!("        - {}", doc.title);
                            }
                        }
                        Err(e) => println!("      Error searching: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }
    println!();
    
    for agent in &agents {
        println!("Agent: {} ({})", agent.name, agent.agent_id);
        println!();
    }

    Ok(())
}

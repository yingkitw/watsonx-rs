//! WatsonX Orchestrate Chat Example
//!
//! This example demonstrates how to:
//! 1. List available agents
//! 2. Select an agent
//! 3. Chat with the agent (non-streaming)
//! 4. Stream responses from the agent
//! 5. Maintain conversation context with thread_id

use std::io::{self, Write};
use watsonx_rs::{OrchestrateClient, OrchestrateConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    dotenvy::dotenv().ok();
    
    // Load Orchestrate config
    let config = OrchestrateConfig::from_env()
        .expect("Failed to load Orchestrate config. Please set WXO_INSTANCE_ID in your .env file");
    
    // Get API key
    let api_key = std::env::var("WATSONX_API_KEY")
        .or_else(|_| std::env::var("IAM_API_KEY"))
        .or_else(|_| std::env::var("WO_API_KEY"))
        .expect("WATSONX_API_KEY, IAM_API_KEY, or WO_API_KEY must be set");

    // Create Orchestrate client
    let client = OrchestrateClient::new(config).with_token(api_key);
    
    println!("🚀 WatsonX Orchestrate Chat Example");
    println!("=====================================");
    println!("Using Orchestrate URL: {}", client.config().get_base_url());
    
    // Step 1: List available agents
    println!("\n📝 Step 1: Listing available agents...");
    let agents = match client.list_agents().await {
        Ok(agents) => {
            if agents.is_empty() {
                println!("❌ No agents available in this instance.");
                return Ok(());
            }
            println!("✅ Found {} agents:", agents.len());
            for agent in &agents {
                println!("  - {} (ID: {})", agent.name, agent.agent_id);
            }
            agents
        }
        Err(e) => {
            println!("❌ Failed to list agents: {}", e);
            return Ok(());
        }
    };

    // Select the first agent
    let agent = &agents[0];
    println!("\n✅ Selected agent: {} (ID: {})", agent.name, agent.agent_id);
    
    // Step 2: Send a message (non-streaming)
    println!("\n💬 Step 2: Sending a message (non-streaming)...");
    let message1 = "Hello! Can you introduce yourself?";
    println!("You: {}", message1);
    
    let mut thread_id = None;
    match client.send_message(&agent.agent_id, message1, thread_id).await {
        Ok((response, new_thread_id)) => {
            thread_id = new_thread_id;
            println!("\n🤖 Agent: {}", response);
            if let Some(ref tid) = thread_id {
                println!("   (Thread ID: {})", tid);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            return Ok(());
        }
    }
    
    // Step 3: Continue conversation with context
    println!("\n💬 Step 3: Continuing conversation with context...");
    let message2 = "What services do you provide?";
    println!("You: {}", message2);
    
    match client.send_message(&agent.agent_id, message2, thread_id.clone()).await {
        Ok((response, new_thread_id)) => {
            thread_id = new_thread_id;
            println!("\n🤖 Agent: {}", response);
            if let Some(ref tid) = thread_id {
                println!("   (Thread ID: {})", tid);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            return Ok(());
        }
    }
    
    // Step 4: Stream response
    println!("\n🌊 Step 4: Streaming response...");
    let message3 = "Tell me about Watson Orchestrate capabilities in a few sentences.";
    println!("You: {}", message3);
    print!("\n🤖 Agent (streaming): ");
    io::stdout().flush().unwrap();

    match client.stream_message(&agent.agent_id, message3, thread_id.clone(), |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
        // Use async sleep instead of blocking sleep
        // Small artificial delay to simulate real-time streaming effect
        std::thread::sleep(std::time::Duration::from_millis(5));
        Ok(())
    }).await {
        Ok(new_thread_id) => {
            thread_id = new_thread_id;
            println!("\n   ✅ Streaming completed");
            if let Some(ref tid) = thread_id {
                println!("   (Thread ID: {})", tid);
            }
        }
        Err(e) => {
            println!("\n❌ Error: {}", e);
        }
    }
    
    // Step 5: Another streaming example with a longer question
    println!("\n💬 Step 5: Another streaming example...");
    let message4 = "Explain how AI assistants work in simple terms.";
    println!("You: {}", message4);
    print!("\n🤖 Agent (streaming): ");
    
    match client.stream_message(&agent.agent_id, message4, thread_id.clone(), |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
        // Use async sleep instead of blocking sleep
        // Small artificial delay to simulate real-time streaming effect
        std::thread::sleep(std::time::Duration::from_millis(5));
        Ok(())
    }).await {
        Ok(new_thread_id) => {
            thread_id = new_thread_id;
            println!("\n   ✅ Streaming completed");
            if let Some(ref tid) = thread_id {
                println!("   (Thread ID: {})", tid);
            }
        }
        Err(e) => {
            println!("\n❌ Error: {}", e);
        }
    }
    
    println!("\n🎉 Chat example completed successfully!");
    println!("Note: The thread_id ({:?}) maintains conversation context", thread_id);
    
    Ok(())
}

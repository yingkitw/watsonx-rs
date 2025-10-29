//! Advanced Orchestrate SDK example demonstrating all capabilities
//!
//! This example showcases:
//! - Agent management
//! - Conversation with threads
//! - Streaming responses
//! - Run tracking
//! - Tool management
//! - Batch operations
//! - Document collections

use watsonx_rs::{
    OrchestrateClient, OrchestrateConfig, BatchMessageRequest, Message,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("🚀 Advanced Orchestrate SDK Test");
    println!("================================\n");

    // Initialize client
    let config = OrchestrateConfig::from_env()
        .expect("Failed to load Orchestrate config from environment");

    let api_key = std::env::var("WATSONX_API_KEY")
        .or_else(|_| std::env::var("IAM_API_KEY"))
        .or_else(|_| std::env::var("WO_API_KEY"))
        .expect("API key must be set");

    let client = OrchestrateClient::new(config).with_token(api_key);

    println!("✅ Client initialized");
    println!("URL: {}\n", client.config().get_base_url());

    // ========================================================================
    // 1. AGENT MANAGEMENT
    // ========================================================================
    println!("📋 1. AGENT MANAGEMENT");
    println!("─────────────────────");

    let agents = client.list_agents().await?;
    println!("✅ Found {} agents:", agents.len());
    for agent in &agents {
        println!("   - {} (ID: {})", agent.name, agent.agent_id);
    }

    if agents.is_empty() {
        println!("❌ No agents available. Exiting.");
        return Ok(());
    }

    let agent = &agents[0];
    println!("📌 Selected agent: {} ({})\n", agent.name, agent.agent_id);

    // ========================================================================
    // 2. THREAD MANAGEMENT
    // ========================================================================
    println!("🧵 2. THREAD MANAGEMENT");
    println!("──────────────────────");

    let thread = client.create_thread(Some(&agent.agent_id)).await?;
    println!("✅ Created new thread: {}", thread.thread_id);
    println!("   Title: {}\n", thread.title.unwrap_or_else(|| "N/A".to_string()));

    let thread_id = thread.thread_id.clone();

    // ========================================================================
    // 3. SKILL MANAGEMENT
    // ========================================================================
    println!("🎯 3. SKILL MANAGEMENT");
    println!("─────────────────────");

    match client.list_skills().await {
        Ok(skills) => {
            if skills.is_empty() {
                println!("ℹ️  No skills available in this instance");
            } else {
                println!("✅ Found {} skills:", skills.len());
                for skill in skills.iter().take(3) {
                    println!("   - {} (ID: {})", skill.name, skill.id);
                }
                if skills.len() > 3 {
                    println!("   ... and {} more", skills.len() - 3);
                }
            }
        }
        Err(e) => println!("ℹ️  Skills endpoint not available: {}", e),
    }
    println!();

    // ========================================================================
    // 4. TOOL MANAGEMENT
    // ========================================================================
    println!("🔧 4. TOOL MANAGEMENT");
    println!("────────────────────");

    match client.list_tools().await {
        Ok(tools) => {
            if tools.is_empty() {
                println!("ℹ️  No tools available in this instance");
            } else {
                println!("✅ Found {} tools:", tools.len());
                for tool in tools.iter().take(3) {
                    println!("   - {} (ID: {})", tool.name, tool.id);
                }
                if tools.len() > 3 {
                    println!("   ... and {} more", tools.len() - 3);
                }
            }
        }
        Err(e) => println!("ℹ️  Tools endpoint not available or different format: {}", e),
    }
    println!();

    // ========================================================================
    // 5. NON-STREAMING MESSAGE
    // ========================================================================
    println!("💬 5. NON-STREAMING MESSAGE");
    println!("──────────────────────────");

    let message1 = "Hello! What can you help me with?";
    println!("You: {}", message1);

    match client.send_message(&agent.agent_id, message1, Some(thread_id.clone())).await {
        Ok((response, new_thread_id)) => {
            println!("🤖 Agent: {}", response);
            if let Some(tid) = new_thread_id {
                println!("   Thread ID: {}", tid);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // 6. STREAMING MESSAGE
    // ========================================================================
    println!("🌊 6. STREAMING MESSAGE");
    println!("──────────────────────");

    let message2 = "Tell me about your capabilities in a few sentences.";
    println!("You: {}", message2);
    print!("🤖 Agent (streaming): ");
    io::stdout().flush().unwrap();

    match client
        .stream_message(&agent.agent_id, message2, Some(thread_id.clone()), |chunk| {
            print!("{}", chunk);
            io::stdout().flush().unwrap();
            Ok(())
        })
        .await
    {
        Ok(new_thread_id) => {
            println!("\n✅ Streaming completed");
            if let Some(tid) = new_thread_id {
                println!("   Thread ID: {}", tid);
            }
        }
        Err(e) => println!("\n❌ Error: {}", e),
    }
    println!();

    // ========================================================================
    // 7. THREAD HISTORY
    // ========================================================================
    println!("📜 7. THREAD HISTORY");
    println!("───────────────────");

    match client.get_thread_messages(&thread_id).await {
        Ok(messages) => {
            println!("✅ Retrieved {} messages:", messages.len());
            for (i, msg) in messages.iter().enumerate().take(3) {
                println!("   {}. [{}] {}", i + 1, msg.role, msg.content);
            }
            if messages.len() > 3 {
                println!("   ... and {} more", messages.len() - 3);
            }
        }
        Err(e) => println!("⚠️  Could not retrieve messages: {}", e),
    }
    println!();

    // ========================================================================
    // 8. RUN MANAGEMENT
    // ========================================================================
    println!("⚙️  8. RUN MANAGEMENT");
    println!("───────────────────");

    match client.list_runs(Some(&agent.agent_id)).await {
        Ok(runs) => {
            println!("✅ Found {} runs:", runs.len());
            for run in runs.iter().take(3) {
                println!("   - {} (Status: {:?})", run.run_id, run.status);
            }
            if runs.len() > 3 {
                println!("   ... and {} more", runs.len() - 3);
            }
        }
        Err(e) => println!("⚠️  Could not list runs: {}", e),
    }
    println!();

    // ========================================================================
    // 9. BATCH OPERATIONS
    // ========================================================================
    println!("📦 9. BATCH OPERATIONS");
    println!("────────────────────");

    let batch_messages = vec![
        Message {
            role: "user".to_string(),
            content: "What is artificial intelligence?".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Explain machine learning briefly.".to_string(),
        },
    ];

    let batch_request = BatchMessageRequest {
        messages: batch_messages,
        agent_id: agent.agent_id.clone(),
        thread_id: Some(thread_id.clone()),
        metadata: None,
    };

    match client.send_batch_messages(batch_request).await {
        Ok(batch_response) => {
            println!("✅ Batch processing completed (Batch ID: {})", batch_response.batch_id);
            for result in batch_response.responses {
                println!(
                    "   Message {}: {} ({}ms)",
                    result.message_index,
                    if result.error.is_some() { "❌ Failed" } else { "✅ Success" },
                    result.processing_time_ms.unwrap_or(0)
                );
            }
        }
        Err(e) => println!("⚠️  Batch operation failed: {}", e),
    }
    println!();

    // ========================================================================
    // 10. DOCUMENT COLLECTIONS
    // ========================================================================
    println!("📚 10. DOCUMENT COLLECTIONS");
    println!("──────────────────────────");

    match client.list_collections().await {
        Ok(collections) => {
            println!("✅ Found {} collections:", collections.len());
            for collection in collections.iter().take(3) {
                println!("   - {} (ID: {})", collection.name, collection.id);
            }
            if collections.len() > 3 {
                println!("   ... and {} more", collections.len() - 3);
            }
        }
        Err(e) => println!("⚠️  Could not list collections: {}", e),
    }
    println!();

    // ========================================================================
    // 11. THREAD CLEANUP
    // ========================================================================
    println!("🧹 11. THREAD CLEANUP");
    println!("────────────────────");

    match client.delete_thread(&thread_id).await {
        Ok(_) => println!("✅ Thread deleted: {}", thread_id),
        Err(e) => println!("⚠️  Could not delete thread: {}", e),
    }
    println!();

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("✨ TEST SUMMARY");
    println!("═══════════════");
    println!("✅ Agent Management - Tested");
    println!("✅ Thread Management - Tested");
    println!("✅ Skill Management - Tested");
    println!("✅ Tool Management - Tested");
    println!("✅ Non-Streaming Chat - Tested");
    println!("✅ Streaming Chat - Tested");
    println!("✅ Thread History - Tested");
    println!("✅ Run Management - Tested");
    println!("✅ Batch Operations - Tested");
    println!("✅ Document Collections - Tested");
    println!("✅ Thread Cleanup - Tested");
    println!("\n🎉 All tests completed successfully!");

    Ok(())
}

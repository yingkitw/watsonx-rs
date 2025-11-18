//! Chat with Documents Example
//!
//! This example demonstrates how to use the chat with documents feature
//! to ask questions about uploaded documents.

use watsonx_rs::{OrchestrateConnection, ChatWithDocsRequest};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("📄 Chat with Documents Example");
    println!("==============================\n");

    // One-line connection!
    println!("🔐 Connecting to Watson Orchestrate...");
    let client = OrchestrateConnection::new().from_env().await?;
    println!("✅ Connected\n");

    // Get first agent
    println!("📋 Listing agents...");
    let agents = client.list_agents().await?;
    if agents.is_empty() {
        println!("❌ No agents available");
        return Ok(());
    }

    let agent = &agents[0];
    println!("Using agent: {} ({})\n", agent.name, agent.agent_id);

    // Create a thread for conversation
    println!("🧵 Creating conversation thread...");
    let thread = client.create_thread(Some(&agent.agent_id)).await?;
    let thread_id = thread.thread_id.clone();
    println!("✅ Thread created: {}\n", thread_id);

    // Check chat with docs status
    println!("📊 Checking chat with documents status...");
    match client.get_chat_with_docs_status(&agent.agent_id, &thread_id).await {
        Ok(status) => {
            println!("✅ Status: {}", status.status);
            if let Some(count) = status.document_count {
                println!("   Documents: {}", count);
            }
            if let Some(updated) = status.last_updated {
                println!("   Last updated: {}", updated);
            }
        }
        Err(e) => {
            println!("ℹ️  Chat with documents feature is not available in this instance");
            println!("   This feature may require:");
            println!("   - Special instance configuration");
            println!("   - Premium plan or specific region");
            println!("   - Feature enablement in Watson Orchestrate settings");
            println!("   Error: {}", e);
        }
    }
    println!();

    // Example 1: Chat with document content
    println!("💬 Example 1: Chat with Document Content");
    println!("─────────────────────────────────────────\n");

    let doc_content = r#"
    Watson Orchestrate is an AI-powered platform that helps organizations 
    automate complex business processes. It integrates with various AI models 
    and services to provide intelligent automation capabilities.
    
    Key features include:
    - Multi-agent orchestration
    - Document processing
    - Skill composition
    - Knowledge management
    "#;

    let request = ChatWithDocsRequest {
        message: "What are the key features of Watson Orchestrate?".to_string(),
        document_content: Some(doc_content.to_string()),
        document_path: None,
        context: None,
    };

    println!("Question: {}", request.message);
    print!("Answer: ");
    io::stdout().flush().unwrap();

    match client.chat_with_docs(&agent.agent_id, &thread_id, request).await {
        Ok(response) => {
            println!("{}", response.message);
            if let Some(docs) = response.documents_used {
                println!("Documents used: {:?}", docs);
            }
            if let Some(confidence) = response.confidence {
                println!("Confidence: {:.2}%", confidence * 100.0);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
    println!();

    // Example 2: Stream chat with documents
    println!("🌊 Example 2: Streaming Chat with Documents");
    println!("────────────────────────────────────────────\n");

    let request = ChatWithDocsRequest {
        message: "Explain how Watson Orchestrate helps with automation.".to_string(),
        document_content: Some(doc_content.to_string()),
        document_path: None,
        context: None,
    };

    println!("Question: {}", request.message);
    print!("Answer: ");
    io::stdout().flush().unwrap();

    match client.stream_chat_with_docs(&agent.agent_id, &thread_id, request, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
        Ok(())
    }).await {
        Ok(_) => println!("\n✅ Streaming completed"),
        Err(e) => println!("\n❌ Error: {}", e),
    }
    println!();

    // Cleanup
    println!("🧹 Cleaning up...");
    match client.delete_thread(&thread_id).await {
        Ok(_) => println!("✅ Thread deleted"),
        Err(e) => println!("⚠️  Could not delete thread: {}", e),
    }

    println!("\n✨ Chat with documents example completed!");

    Ok(())
}

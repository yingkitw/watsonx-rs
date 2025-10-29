//! Orchestrate SDK Use Case Examples
//!
//! This example demonstrates practical use cases:
//! - Customer Support Bot
//! - Document Q&A
//! - Multi-turn Conversation
//! - Tool Integration

use watsonx_rs::{OrchestrateClient, OrchestrateConfig, SearchRequest};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("🎯 Orchestrate SDK Use Cases");
    println!("============================\n");

    // Initialize client
    let config = OrchestrateConfig::from_env()
        .expect("Failed to load Orchestrate config from environment");

    // Get Watson Orchestrate API key
    let api_key = std::env::var("WXO_KEY")
        .or_else(|_| std::env::var("WATSONX_API_KEY"))
        .or_else(|_| std::env::var("IAM_API_KEY"))
        .or_else(|_| std::env::var("WO_API_KEY"))
        .expect("WXO_KEY, WATSONX_API_KEY, IAM_API_KEY, or WO_API_KEY must be set");

    // Generate JWT token from API key
    println!("🔐 Generating JWT token from API key...");
    let token = match OrchestrateClient::generate_jwt_token(&api_key).await {
        Ok(t) => {
            println!("✅ JWT token generated successfully\n");
            t
        }
        Err(e) => {
            println!("❌ Failed to generate JWT token: {}", e);
            println!("\n⚠️  Please check:");
            println!("   - WXO_KEY is valid and not expired");
            println!("   - API key has correct permissions");
            println!("   - Network connectivity to iam.cloud.ibm.com");
            return Err(format!("JWT token generation failed: {}", e).into());
        }
    };

    let client = OrchestrateClient::new(config).with_token(token);

    // Get first agent
    let agents = client.list_agents().await?;
    if agents.is_empty() {
        println!("❌ No agents available");
        return Ok(());
    }

    let agent = &agents[0];
    println!("Using agent: {} ({})\n", agent.name, agent.agent_id);

    // ========================================================================
    // USE CASE 1: MULTI-TURN CONVERSATION
    // ========================================================================
    println!("📞 USE CASE 1: Multi-Turn Conversation");
    println!("─────────────────────────────────────\n");

    let thread = client.create_thread(Some(&agent.agent_id)).await?;
    let thread_id = thread.thread_id.clone();
    println!("Created conversation thread: {}\n", thread_id);

    let questions = vec![
        "What is your name?",
        "What services do you provide?",
        "Can you help me with technical issues?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("Q{}: {}", i + 1, question);
        print!("A{}: ", i + 1);
        io::stdout().flush().unwrap();

        match client
            .stream_message(&agent.agent_id, question, Some(thread_id.clone()), |chunk| {
                print!("{}", chunk);
                io::stdout().flush().unwrap();
                Ok(())
            })
            .await
        {
            Ok(_) => println!("\n"),
            Err(e) => println!("\n❌ Error: {}\n", e),
        }
    }

    // Show conversation history
    println!("📜 Conversation History:");
    match client.get_thread_messages(&thread_id).await {
        Ok(messages) => {
            println!("Total messages: {}", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                let role_emoji = if msg.role == "user" { "👤" } else { "🤖" };
                println!("  {}. [{}] {}", i + 1, role_emoji, msg.content);
            }
        }
        Err(e) => println!("Could not retrieve history: {}", e),
    }
    println!();

    // ========================================================================
    // USE CASE 2: DOCUMENT SEARCH & Q&A
    // ========================================================================
    println!("📚 USE CASE 2: Document Search & Q&A");
    println!("────────────────────────────────────\n");

    match client.list_collections().await {
        Ok(collections) => {
            if !collections.is_empty() {
                let collection = &collections[0];
                println!("Using collection: {} ({})\n", collection.name, collection.id);

                // Search for documents
                let search_query = "artificial intelligence";
                println!("Searching for: '{}'\n", search_query);

                let search_req = SearchRequest {
                    query: search_query.to_string(),
                    limit: Some(5),
                    threshold: Some(0.5),
                    filters: None,
                };

                match client.search_documents(&collection.id, search_req).await {
                    Ok(results) => {
                        println!("Found {} results:", results.results.len());
                        for (i, result) in results.results.iter().enumerate() {
                            println!(
                                "  {}. {} (Score: {:.2})",
                                i + 1,
                                result.title,
                                result.similarity_score
                            );
                            println!("     Snippet: {}", result.content_snippet);
                        }
                    }
                    Err(e) => println!("Search failed: {}", e),
                }
            } else {
                println!("ℹ️  No document collections available (endpoint may not be enabled)");
            }
        }
        Err(e) => println!("❌ Error listing collections: {}", e),
    }
    println!();

    // ========================================================================
    // USE CASE 3: TOOL INTEGRATION
    // ========================================================================
    println!("🔧 USE CASE 3: Tool Integration");
    println!("────────────────────────────────\n");

    match client.list_tools().await {
        Ok(tools) => {
            if !tools.is_empty() {
                println!("Available tools: {}", tools.len());
                for tool in tools.iter().take(5) {
                    println!("  - {} ({})", tool.name, tool.id);
                    if let Some(desc) = &tool.description {
                        println!("    Description: {}", desc);
                    }
                }

                // Demonstrate tool execution (if available)
                if let Some(tool) = tools.first() {
                    println!("\nTool execution example:");
                    println!("  Tool: {}", tool.name);
                    println!("  Status: Ready for execution");
                    println!("  (Actual execution depends on tool configuration)");
                }
            } else {
                println!("⚠️  No tools available");
            }
        }
        Err(e) => println!("Could not list tools: {}", e),
    }
    println!();

    // ========================================================================
    // USE CASE 4: RUN TRACKING
    // ========================================================================
    println!("⚙️  USE CASE 4: Run Tracking");
    println!("───────────────────────────\n");

    match client.list_runs(Some(&agent.agent_id)).await {
        Ok(runs) => {
            println!("Recent runs for agent: {}", agent.name);
            println!("Total runs: {}\n", runs.len());

            let mut status_counts = std::collections::HashMap::new();
            for run in &runs {
                *status_counts.entry(format!("{:?}", run.status)).or_insert(0) += 1;
            }

            println!("Run Status Summary:");
            for (status, count) in status_counts {
                println!("  - {}: {}", status, count);
            }

            // Show recent runs
            println!("\nRecent Runs:");
            for (i, run) in runs.iter().take(5).enumerate() {
                println!(
                    "  {}. {} - Status: {:?}",
                    i + 1,
                    run.run_id,
                    run.status
                );
                if let Some(created) = &run.created_at {
                    println!("     Created: {}", created);
                }
            }
        }
        Err(e) => println!("Could not list runs: {}", e),
    }
    println!();

    // ========================================================================
    // USE CASE 5: SKILL DISCOVERY
    // ========================================================================
    println!("🎯 USE CASE 5: Skill Discovery");
    println!("──────────────────────────────\n");

    match client.list_skills().await {
        Ok(skills) => {
            if skills.is_empty() {
                println!("ℹ️  No skills available (endpoint may not be enabled)");
            } else {
                println!("Available skills: {}", skills.len());
                for (i, skill) in skills.iter().take(10).enumerate() {
                    println!("  {}. {} ({})", i + 1, skill.name, skill.id);
                }
                if skills.len() > 10 {
                    println!("  ... and {} more skills", skills.len() - 10);
                }
            }
        }
        Err(e) => println!("❌ Error listing skills: {}", e),
    }
    println!();

    // ========================================================================
    // CLEANUP
    // ========================================================================
    println!("🧹 Cleaning up...");
    match client.delete_thread(&thread_id).await {
        Ok(_) => println!("✅ Thread deleted"),
        Err(e) => println!("⚠️  Could not delete thread: {}", e),
    }

    println!("\n✨ Use case demonstration completed!");

    Ok(())
}

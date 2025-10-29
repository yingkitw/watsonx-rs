//! WatsonX Orchestrate Example
//!
//! This example demonstrates how to use the WatsonX Orchestrate SDK to:
//! 1. Create and manage custom assistants
//! 2. Set up document collections for knowledge bases
//! 3. Chat with custom assistants
//! 4. Search documents using vector similarity

use watsonx_rs::{
    OrchestrateClient, OrchestrateConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    dotenvy::dotenv().ok();
    
    // Load Orchestrate config from environment variables
    // Required: WXO_INSTANCE_ID
    // Optional: WXO_REGION (defaults to us-south)
    let config = OrchestrateConfig::from_env()
        .expect("Failed to load Orchestrate config. Please set WXO_INSTANCE_ID in your .env file");
    
    // Get API key (supports multiple variable names for flexibility)
    let api_key = std::env::var("WATSONX_API_KEY")
        .or_else(|_| std::env::var("IAM_API_KEY"))
        .or_else(|_| std::env::var("WO_API_KEY"))
        .unwrap_or_else(|_| {
            eprintln!("❌ Error: API key not found!");
            eprintln!("\nPlease set one of the following in your .env file:");
            eprintln!("  - WATSONX_API_KEY");
            eprintln!("  - IAM_API_KEY");
            eprintln!("  - WO_API_KEY");
            std::process::exit(1);
        });

    // Create Orchestrate client
    let client = OrchestrateClient::new(config).with_token(api_key);
    
    // Display configuration info
    println!("Using Orchestrate URL: {}", client.config().get_base_url());

    println!("🚀 WatsonX Orchestrate SDK Example");
    println!("=====================================");

    // Example 1: List available agents
    println!("\n📝 Listing available agents...");
    
    let agents = match client.list_agents().await {
        Ok(agents) => {
            println!("✅ Found {} agents:", agents.len());
            for agent in &agents {
                println!("  - {} (ID: {})", agent.name, agent.agent_id);
            }
            agents
        }
        Err(e) => {
            println!("❌ Failed to list agents: {}", e);
            println!("Note: Make sure WXO_INSTANCE_ID and API key are correctly set in .env");
            return Ok(());
        }
    };

    if agents.is_empty() {
        println!("No agents available in this instance.");
        return Ok(());
    }

    let agent = &agents[0];
    println!("\n✅ Using agent: {} (ID: {})", agent.name, agent.agent_id);

    println!("\n🎉 Example completed successfully!");
    println!("Note: Chat and document collection features require additional API endpoints.");
    
    // Example 2: Create a document collection for knowledge base (not yet implemented)
    /*
    println!("\n📚 Creating a document collection...");
    
    let vector_config = VectorIndexConfig {
        id: "rust-docs-index".to_string(),
        embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimensions: 384,
        index_type: IndexType::Hnsw,
        similarity_metric: SimilarityMetric::Cosine,
    };

    let collection_request = CreateCollectionRequest {
        name: "Rust Documentation".to_string(),
        description: Some("Collection of Rust programming documentation and examples".to_string()),
        vector_index: Some(vector_config),
    };

    let collection = match client.create_collection(collection_request).await {
        Ok(collection) => {
            println!("✅ Created collection: {}", collection.name);
            collection
        }
        Err(e) => {
            println!("❌ Failed to create collection: {}", e);
            return Ok(());
        }
    };

    // Example 3: Add documents to the collection
    println!("\n📄 Adding documents to collection...");
    
    let documents = vec![
        Document {
            id: "rust-basics-1".to_string(),
            title: "Rust Ownership Basics".to_string(),
            content: "Ownership is Rust's most unique feature and has deep implications for the rest of the language. It enables Rust to make memory safety guarantees without needing a garbage collector.".to_string(),
            metadata: HashMap::from([
                ("category".to_string(), serde_json::Value::String("basics".to_string())),
                ("difficulty".to_string(), serde_json::Value::String("beginner".to_string())),
            ]),
            document_type: DocumentType::Text,
            created_at: None,
            updated_at: None,
            embedding: None,
        },
        Document {
            id: "rust-basics-2".to_string(),
            title: "Rust Error Handling".to_string(),
            content: "Rust groups errors into two major categories: recoverable and unrecoverable errors. For a recoverable error, such as a file not found error, we most likely just want to report the problem to the user and retry the operation.".to_string(),
            metadata: HashMap::from([
                ("category".to_string(), serde_json::Value::String("basics".to_string())),
                ("difficulty".to_string(), serde_json::Value::String("intermediate".to_string())),
            ]),
            document_type: DocumentType::Text,
            created_at: None,
            updated_at: None,
            embedding: None,
        },
    ];

    let add_docs_request = AddDocumentsRequest {
        documents,
        async_processing: false,
    };

    match client.add_documents(&collection.id, add_docs_request).await {
        Ok(docs) => {
            println!("✅ Added {} documents to collection", docs.len());
        }
        Err(e) => {
            println!("❌ Failed to add documents: {}", e);
        }
    }

    // Example 4: Search documents
    println!("\n🔍 Searching documents...");
    
    let search_request = SearchRequest {
        query: "memory safety".to_string(),
        limit: Some(5),
        threshold: Some(0.7),
        filters: None,
    };

    match client.search_documents(&collection.id, search_request).await {
        Ok(search_response) => {
            println!("✅ Found {} results:", search_response.total_results);
            for (i, result) in search_response.results.iter().enumerate() {
                println!("  {}. {} (score: {:.3})", i + 1, result.title, result.similarity_score);
                println!("     {}", result.content_snippet);
            }
        }
        Err(e) => {
            println!("❌ Failed to search documents: {}", e);
        }
    }

    // Example 5: Chat with the assistant
    println!("\n💬 Chatting with the assistant...");
    
    let chat_request = ChatRequest {
        message: "Can you explain Rust ownership in simple terms?".to_string(),
        session_id: None,
        metadata: None,
        stream: false,
    };

    match client.send_chat_message(&assistant.id, chat_request).await {
        Ok(response) => {
            println!("✅ Assistant response:");
            println!("{}", response.message);
        }
        Err(e) => {
            println!("❌ Failed to chat with assistant: {}", e);
        }
    }

    // Example 6: Streaming chat
    println!("\n🌊 Streaming chat example...");
    
    let stream_request = ChatRequest {
        message: "Write a simple Rust function that demonstrates ownership".to_string(),
        session_id: None,
        metadata: None,
        stream: true,
    };

    match client.send_chat_message_stream(&assistant.id, stream_request, |chunk| {
        print!("{}", chunk);
        Ok(())
    }).await {
        Ok(response) => {
            println!("\n✅ Streaming completed. Full response length: {} characters", response.message.len());
        }
        Err(e) => {
            println!("❌ Failed to stream chat: {}", e);
        }
    }

    // Example 7: List assistants and collections
    println!("\n📋 Listing assistants and collections...");
    
    match client.list_assistants().await {
        Ok(assistants) => {
            println!("✅ Found {} assistants:", assistants.len());
            for assistant in assistants {
                println!("  - {} ({})", assistant.name, assistant.id);
            }
        }
        Err(e) => {
            println!("❌ Failed to list assistants: {}", e);
        }
    }

    match client.list_collections().await {
        Ok(collections) => {
            println!("✅ Found {} collections:", collections.len());
            for collection in collections {
                println!("  - {} ({})", collection.name, collection.id);
            }
        }
        Err(e) => {
            println!("❌ Failed to list collections: {}", e);
        }
    }

    */
    
    Ok(())
}

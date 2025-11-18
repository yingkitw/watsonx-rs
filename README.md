# WatsonX-RS

A modern Rust SDK for IBM WatsonX AI platform, designed for simplicity and performance.

## 🎯 Vision

This SDK aims to provide comprehensive support for the entire IBM WatsonX ecosystem:

- **🤖 watsonx.ai** - AI and machine learning models (current focus)
- **📊 watsonx.data** - Data management and analytics
- **🛡️ watsonx.governance** - AI governance and compliance
- **⚙️ watsonx.orchestrate** - Agentic AI with orchestration and automation

Currently, we support both `watsonx.ai` (text generation) and `watsonx.orchestrate` (custom assistants and document management), with the architecture designed to expand across all WatsonX services.

## Reference:

watsonx Orchestrate API Reference: https://developer.ibm.com/apis/catalog/watsonorchestrate--custom-assistants/api

## 🚀 Quick Start (5 Minutes)

### 1. Add to Cargo.toml

```toml
[dependencies]
watsonx-rs = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Set up your credentials

Create a `.env` file in your project root:

```bash
# WatsonX AI
WATSONX_API_KEY=your_actual_api_key
WATSONX_PROJECT_ID=your_actual_project_id

# Watson Orchestrate (optional)
WXO_INSTANCE_ID=your_instance_id
WXO_KEY=your_orchestrate_api_key
```

### 3. Generate text with WatsonX AI (One-Line Connection!)

```rust
use watsonx_rs::{WatsonxConnection, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✨ One-line connection - that's it!
    let client = WatsonxConnection::new().from_env().await?;
    
    // Generate text with streaming
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let result = (&client).generate_text_stream(
        "Explain Rust ownership in one sentence.",
        &gen_config,
        |chunk| {
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    ).await?;
    
    println!("\n✅ Generated with model: {}", result.model_id);
    Ok(())
}
```

### 4. Chat with Watson Orchestrate (One-Line Connection!)

```rust
use watsonx_rs::OrchestrateConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✨ One-line connection - that's it!
    let client = OrchestrateConnection::new().from_env().await?;
    
    // List available agents
    let agents = client.list_agents().await?;
    
    if let Some(agent) = agents.first() {
        println!("✅ Found agent: {}", agent.name);
        
        // Create a conversation thread
        let thread = client.create_thread(Some(&agent.agent_id)).await?;
        
        // Send a message
        let response = client.send_message(
            &agent.agent_id,
            &thread.thread_id,
            "Hello! How can you help me?"
        ).await?;
        
        println!("Agent: {}", response.message);
    }
    
    Ok(())
}
```

## 📖 Core Usage Patterns

> **Important**: You must specify a model before generating text. Use `GenerationConfig::default().with_model(model_id)` to set the model.

### Pattern 1: Simple Text Generation

```rust
use watsonx_rs::{GenerationConfig, models::models};

// Set the model and generate text
let config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL);

let result = client.generate_text("Your prompt here", &config).await?;
println!("{}", result.text);
```

### Pattern 2: Streaming for Real-time Output

```rust
use watsonx_rs::{GenerationConfig, models::models};

// Perfect for interactive applications
let config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL);

let result = client.generate_text_stream("Your prompt", &config, |chunk| {
    print!("{}", chunk);  // Print as it generates
    std::io::Write::flush(&mut std::io::stdout()).unwrap();
}).await?;
```

### Pattern 3: Custom Configuration

```rust
use watsonx_rs::{GenerationConfig, models::models};

let config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL)
    .with_max_tokens(1000)
    .with_top_p(0.9);

let result = client.generate_text("Your prompt", &config).await?;
```

### Pattern 4: List Available Models

```rust
// Discover what models are available
let models = client.list_models().await?;
for model in models {
    println!("{} - {}", model.model_id, model.name.unwrap_or_default());
}
```

### Pattern 5: Quality Assessment

```rust
// Evaluate generated text quality
let score = client.assess_quality("Your generated text")?;
println!("Quality score: {:.2}", score);
```

## 🤖 Available Models

### Popular Models

```rust
use watsonx_rs::models::models;

// IBM Granite models
models::GRANITE_4_H_SMALL           // Default, best performance
models::GRANITE_3_3_8B_INSTRUCT     // Good balance of speed/quality
models::GRANITE_3_2_8B_INSTRUCT     // Fast generation

// Meta Llama models
models::LLAMA_3_3_70B_INSTRUCT      // High quality, slower
models::LLAMA_3_1_8B                // Good for most tasks

// Mistral models
models::MISTRAL_MEDIUM_2505          // Excellent quality
models::MISTRAL_SMALL_3_1_24B_INSTRUCT_2503  // Fast and efficient
```

### Discover Models Dynamically

```rust
// Get all available models
let models = client.list_models().await?;
for model in models {
    if model.available.unwrap_or(false) {
        println!("✅ {} - {}", model.model_id, model.name.unwrap_or_default());
    }
}
```

## 🎛️ Configuration Options

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `WATSONX_API_KEY` | ✅ | - | Your IBM Cloud API key |
| `WATSONX_PROJECT_ID` | ✅ | - | Your WatsonX project ID |
| `WATSONX_API_URL` | ❌ | `https://us-south.ml.cloud.ibm.com` | API base URL |
| `WATSONX_API_VERSION` | ❌ | `2023-05-29` | API version |
| `WATSONX_TIMEOUT_SECS` | ❌ | `120` | Request timeout |

### Generation Parameters

```rust
let config = GenerationConfig::default()
    .with_model("ibm/granite-4-h-small")  // Model to use
    .with_max_tokens(1000)                 // Max tokens to generate
    .with_top_p(0.9)                       // Nucleus sampling
    .with_top_k(50)                         // Top-k sampling
    .with_repetition_penalty(1.1)          // Reduce repetition
    .with_stop_sequences(vec!["END".to_string()]); // Stop tokens
```

## 🎯 When to Use Each Method

### Use `generate_text()` when:
- ✅ You need the complete response before processing
- ✅ Batch processing multiple prompts
- ✅ Building APIs that return complete responses
- ✅ Simple, synchronous-style workflows

### Use `generate_text_stream()` when:
- ✅ Building interactive chat applications
- ✅ Real-time user experience is important
- ✅ Processing long responses incrementally
- ✅ Building streaming APIs

### Use `generate_batch()` or `generate_batch_simple()` when:
- ✅ Processing multiple prompts concurrently
- ✅ Need to maximize throughput
- ✅ Want to collect all results at once
- ✅ Each request can succeed or fail independently

## 🔄 Batch Generation

Batch generation allows you to process multiple prompts concurrently, improving throughput and efficiency.

### Pattern 1: Simple Batch with Uniform Configuration

```rust
use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WatsonxConfig::from_env()?;
    let mut client = WatsonxClient::new(config)?;
    client.connect().await?;
    
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let prompts = vec![
        "Write a haiku about Rust".to_string(),
        "Explain async/await in one sentence".to_string(),
        "What is ownership in Rust?".to_string(),
    ];
    
    let batch_result = client.generate_batch_simple(prompts, &gen_config).await?;
    
    println!("Total: {}, Successful: {}, Failed: {}", 
        batch_result.total, batch_result.successful, batch_result.failed);
    
    for item in batch_result.results {
        if let Some(result) = item.result {
            println!("Generated: {}", result.text);
        }
    }
    
    Ok(())
}
```

### Pattern 2: Batch with Custom IDs and Mixed Configurations

```rust
use watsonx_rs::{WatsonxClient, WatsonxConfig, BatchRequest, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WatsonxConfig::from_env()?;
    let mut client = WatsonxClient::new(config)?;
    client.connect().await?;
    
    let default_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let quick_config = GenerationConfig::quick_response()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let requests = vec![
        BatchRequest::new("Write a haiku about Rust")
            .with_id("haiku-1"),
        BatchRequest::with_config("Quick response", quick_config)
            .with_id("quick-1"),
        BatchRequest::new("Long explanation")
            .with_id("long-1"),
    ];
    
    let batch_result = client.generate_batch(requests, &default_config).await?;
    
    // Process results
    for item in batch_result.results {
        if let Some(result) = item.result {
            println!("[{}] {}", 
                item.id.unwrap_or_default(), 
                result.text);
        } else if let Some(error) = item.error {
            println!("[{}] Error: {}", 
                item.id.unwrap_or_default(), 
                error);
        }
    }
    
    // Get only successful results
    for result in batch_result.successes() {
        println!("Success: {}", result.text);
    }
    
    // Check for failures
    if batch_result.any_failed() {
        for (prompt, error) in batch_result.failures() {
            eprintln!("Failed prompt '{}': {}", prompt, error);
        }
    }
    
    Ok(())
}
```

### Batch Result Features

- **Concurrent Execution**: All requests run in parallel for maximum throughput
- **Per-Item Error Handling**: Each request can succeed or fail independently
- **Result Tracking**: Track success/failure counts and duration
- **Flexible Configuration**: Use default config or per-request configs
- **Request IDs**: Optional IDs for tracking individual requests

## ⚙️ WatsonX Orchestrate

The SDK provides comprehensive support for WatsonX Orchestrate with the following capabilities:

### Core Features
- **Agent Management**: List, get, and interact with agents
- **Chat & Messaging**: Send messages and stream responses with thread management
- **Thread Management**: List threads and retrieve conversation history
- **Skills Management**: List and get skills available to agents
- **Tools Management**: List and get tools available to agents
- **Document Collections**: Create, manage, and search document collections
- **Knowledge Base**: Build and query knowledge bases with vector search

### Quick Start - Chat with Agents

```rust
use watsonx_rs::{OrchestrateClient, OrchestrateConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config from environment (WXO_INSTANCE_ID, WXO_REGION, WATSONX_API_KEY)
    let config = OrchestrateConfig::from_env()?;
    let client = OrchestrateClient::new(config).with_token("your-api-key".to_string());
    
    // List available agents
    let agents = client.list_agents().await?;
    let agent = &agents[0];
    
    // Send a message (non-streaming)
    let (response, thread_id) = client.send_message(&agent.agent_id, "Hello!", None).await?;
    println!("Agent: {}", response);
    
    // Continue conversation with context
    let (response2, _) = client.send_message(
        &agent.agent_id, 
        "What can you help me with?", 
        thread_id
    ).await?;
    println!("Agent: {}", response2);
    
    // Stream responses
    client.stream_message(&agent.agent_id, "Tell me a story", None, |chunk| {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        Ok(())
    }).await?;
    
    Ok(())
}
```

### Environment Setup for Orchestrate

Create a `.env` file with:

```bash
# Required
WXO_INSTANCE_ID=your-instance-id
WATSONX_API_KEY=your-api-key  # or IAM_API_KEY or WO_API_KEY

# Optional (defaults to us-south)
WXO_REGION=us-south
```

### Additional Orchestrate Capabilities

```rust
use watsonx_rs::{OrchestrateClient, OrchestrateConfig, ThreadInfo};

// Get specific agent details
let agent = client.get_agent(&agent_id).await?;
println!("Agent: {} ({})", agent.name, agent.agent_id);

// List all threads (optionally filter by agent)
let threads = client.list_threads(Some(&agent_id)).await?;
for thread in threads {
    println!("Thread: {} - {}", thread.thread_id, thread.title.unwrap_or_default());
}

// Get conversation history from a thread
let messages = client.get_thread_messages(&thread_id).await?;
for msg in messages {
    println!("{}: {}", msg.role, msg.content);
}

// List available skills
let skills = client.list_skills().await?;
for skill in skills {
    println!("Skill: {} - {}", skill.name, skill.id);
}

// List available tools
let tools = client.list_tools().await?;
for tool in tools {
    println!("Tool: {} - {}", tool.name, tool.id);
}

// Get document collection details
let collection = client.get_collection(&collection_id).await?;
println!("Collection: {} ({} documents)", collection.name, collection.document_count);

// Get specific document
let document = client.get_document(&collection_id, &document_id).await?;
println!("Document: {}", document.title);

// Delete document
client.delete_document(&collection_id, &document_id).await?;
```

### Document Collections & Knowledge Base

```rust
use watsonx_rs::{
    OrchestrateClient, CreateCollectionRequest, VectorIndexConfig, IndexType, SimilarityMetric,
    AddDocumentsRequest, Document, DocumentType, SearchRequest
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = OrchestrateConfig::new("your-project-id".to_string());
    let client = OrchestrateClient::new(config).with_token("your-api-key".to_string());
    
    // Create a document collection
    let vector_config = VectorIndexConfig {
        id: "docs-index".to_string(),
        embedding_model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        dimensions: 384,
        index_type: IndexType::Hnsw,
        similarity_metric: SimilarityMetric::Cosine,
    };
    
    let collection_request = CreateCollectionRequest {
        name: "Documentation".to_string(),
        description: Some("Technical documentation collection".to_string()),
        vector_index: Some(vector_config),
    };
    
    let collection = client.create_collection(collection_request).await?;
    
    // Add documents
    let documents = vec![
        Document {
            id: "doc-1".to_string(),
            title: "Rust Basics".to_string(),
            content: "Rust is a systems programming language...".to_string(),
            metadata: HashMap::new(),
            document_type: DocumentType::Text,
            created_at: None,
            updated_at: None,
            embedding: None,
        }
    ];
    
    let add_request = AddDocumentsRequest {
        documents,
        async_processing: false,
    };
    
    client.add_documents(&collection.id, add_request).await?;
    
    // Search documents
    let search_request = SearchRequest {
        query: "Rust programming".to_string(),
        limit: Some(5),
        threshold: Some(0.7),
        filters: None,
    };
    
    let results = client.search_documents(&collection.id, search_request).await?;
    for result in results.results {
        println!("Found: {} (score: {:.3})", result.title, result.similarity_score);
    }
    
    Ok(())
}
```

## 📚 Examples

Run these examples to see the SDK in action:

### WatsonX AI Examples

```bash
# Basic streaming generation
cargo run --example basic_generation

# Compare streaming vs non-streaming
cargo run --example streaming_vs_non_streaming

# List available models
cargo run --example list_models

# Use predefined model constants
cargo run --example model_constants

# Batch generation with concurrent execution
cargo run --example batch_generation
```

### WatsonX Orchestrate Examples

```bash
# Basic Orchestrate - list agents
cargo run --example orchestrate_example

# Chat with agents - streaming and non-streaming
cargo run --example orchestrate_chat

# Advanced capabilities - comprehensive feature test
cargo run --example orchestrate_advanced

# Practical use cases - real-world scenarios
cargo run --example orchestrate_use_cases

# Chat with documents - document-based Q&A
cargo run --example chat_with_documents

# Test agent documents - document discovery
cargo run --example test_agent_documents
```

### WatsonX Orchestrate Capabilities

The SDK provides comprehensive support for Watson Orchestrate with robust error handling and graceful degradation:

- **Agent Management**: List, retrieve, and interact with agents
- **Conversation Management**: Send messages (streaming and non-streaming) with thread context
- **Thread Management**: Create, list, and delete conversation threads
- **Run Management**: Track and cancel agent executions
- **Tool Management**: List, get, execute, update, delete, test, and track tool execution history
- **Tool Versioning**: Manage tool versions and rollbacks
- **Batch Operations**: Process multiple messages efficiently
- **Document Collections**: Manage knowledge bases with vector search
- **Chat with Documents**: Ask questions about uploaded documents
- **Skill Management**: List and retrieve available skills
- **Advanced Tool Features**: Test tools, track execution history, manage versions

**Key Features**:
- ✅ Real-time streaming with SSE parsing
- ✅ Flexible response parsing for API variations
- ✅ Graceful degradation for unavailable endpoints
- ✅ Comprehensive error handling
- ✅ Thread-based conversation context

See [ORCHESTRATE_CAPABILITIES.md](docs/ORCHESTRATE_CAPABILITIES.md) for detailed documentation and [TESTING_GUIDE.md](docs/TESTING_GUIDE.md) for testing instructions.

## 🔧 Error Handling

The SDK provides comprehensive error handling:

```rust
match client.generate_text("prompt", &config).await {
    Ok(result) => println!("Success: {}", result.text),
    Err(watsonx_rs::Error::Authentication(msg)) => {
        eprintln!("Auth error: {}", msg);
        // Handle authentication issues
    }
    Err(watsonx_rs::Error::Api(msg)) => {
        eprintln!("API error: {}", msg);
        // Handle API errors
    }
    Err(watsonx_rs::Error::Timeout(msg)) => {
        eprintln!("Timeout: {}", msg);
        // Handle timeouts
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🤖 WatsonX AI Quick Start

For simplified WatsonX AI connection, see **[WATSONX_AI_QUICK_START.md](docs/WATSONX_AI_QUICK_START.md)**.

### One-line connection:

```rust
let client = WatsonxConnection::new().from_env().await?;
```

### Setup:

```bash
# .env file
WATSONX_API_KEY=your-api-key
WATSONX_PROJECT_ID=your-project-id
```

### Run example:

```bash
cargo run --example basic_simple
```

For more details, see [docs/WATSONX_AI_QUICK_START.md](docs/WATSONX_AI_QUICK_START.md).

## 🤖 Watson Orchestrate Quick Start

For simplified Watson Orchestrate connection, see **[QUICK_START.md](docs/QUICK_START.md)**.

### One-line connection:

```rust
let client = OrchestrateConnection::new().from_env().await?;
```

### Setup:

```bash
# .env file
WXO_INSTANCE_ID=your-instance-id
WXO_KEY=your-api-key
```

### Run example:

```bash
cargo run --example orchestrate_simple
```

For more details, see [docs/QUICK_START.md](docs/QUICK_START.md).

## 🏗️ Architecture

The SDK is built with:

- **Async/Await**: Full async support with Tokio
- **Type Safety**: Strong typing throughout
- **Error Handling**: Comprehensive error types
- **Streaming**: Real-time Server-Sent Events processing
- **Configuration**: Environment-based setup

## 🚧 Roadmap

### Current (watsonx.ai)
- ✅ Text generation (streaming & non-streaming)
- ✅ Model discovery
- ✅ Quality assessment
- ✅ Configuration management

### Current (watsonx.orchestrate)
- ✅ Agent management and discovery
- ✅ Conversation with streaming support
- ✅ Thread lifecycle management
- ✅ Tool management (list, get, execute, update, delete, test)
- ✅ Tool versioning and execution history
- ✅ Run tracking and management
- ✅ Document collections and search
- ✅ Chat with documents (Q&A on uploaded docs)
- ✅ Batch message processing
- ✅ Graceful handling of unavailable endpoints
- ✅ Modular code organization (config, client, types)

### Planned (watsonx.ai)
- 🔄 Chat completion API
- 🔄 Embeddings generation
- 🔄 Fine-tuning support
- ✅ Batch processing

### Future (Full WatsonX Platform)
- 📊 **watsonx.data**: Data ingestion, processing, analytics
- 🛡️ **watsonx.governance**: Model governance, bias detection

## 🤝 Contributing

We welcome contributions! The SDK is designed to be extensible across the entire WatsonX platform.

## 📄 License

This project is licensed under the Apache License 2.0.
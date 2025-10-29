# Watson Orchestrate Advanced Capabilities

This document describes the advanced capabilities available in the Watson Orchestrate SDK.

## Overview

The Watson Orchestrate SDK provides comprehensive support for managing agents, threads, runs, tools, and batch operations. All methods follow the same authentication and error handling patterns.

## Core Capabilities

### 1. Agent Management

**List Agents**
```rust
let agents = client.list_agents().await?;
for agent in agents {
    println!("Agent: {} ({})", agent.name, agent.agent_id);
}
```

**Get Specific Agent**
```rust
let agent = client.get_agent("agent-id").await?;
```

### 2. Conversation Management

**Send Message (Non-Streaming)**
```rust
let (response, thread_id) = client.send_message(
    "agent-id",
    "Hello, how are you?",
    None  // No existing thread
).await?;
println!("Response: {}", response);
```

**Stream Message (Real-Time)**
```rust
client.stream_message(
    "agent-id",
    "Tell me about yourself",
    thread_id,
    |chunk| {
        print!("{}", chunk);
        Ok(())
    }
).await?;
```

**Create New Thread**
```rust
let thread = client.create_thread(Some("agent-id")).await?;
println!("Thread ID: {}", thread.thread_id);
```

**Delete Thread**
```rust
client.delete_thread("thread-id").await?;
```

### 3. Thread Management

**List Threads**
```rust
let threads = client.list_threads(Some("agent-id")).await?;
for thread in threads {
    println!("Thread: {} ({})", thread.title.unwrap_or_default(), thread.thread_id);
}
```

**Get Thread Messages**
```rust
let messages = client.get_thread_messages("thread-id").await?;
for msg in messages {
    println!("{}: {}", msg.role, msg.content);
}
```

### 4. Run Management

**Get Run Information**
```rust
let run = client.get_run("run-id").await?;
println!("Run Status: {:?}", run.status);
```

**List Runs**
```rust
let runs = client.list_runs(Some("agent-id")).await?;
for run in runs {
    println!("Run: {} - Status: {:?}", run.run_id, run.status);
}
```

**Cancel Run**
```rust
client.cancel_run("run-id").await?;
```

### 5. Tool Management

**List Available Tools**
```rust
let tools = client.list_tools().await?;
for tool in tools {
    println!("Tool: {} ({})", tool.name, tool.id);
}
```

**Get Tool Details**
```rust
let tool = client.get_tool("tool-id").await?;
```

**Execute Tool Directly**
```rust
use watsonx_rs::ToolExecutionRequest;
use std::collections::HashMap;

let mut params = HashMap::new();
params.insert("query".to_string(), serde_json::json!("search term"));

let request = ToolExecutionRequest {
    tool_id: "tool-id".to_string(),
    parameters: params,
    context: None,
};

let result = client.execute_tool(request).await?;
println!("Result: {}", result.result);
```

### 6. Skill Management

**List Skills**
```rust
let skills = client.list_skills().await?;
for skill in skills {
    println!("Skill: {} ({})", skill.name, skill.id);
}
```

**Get Skill Details**
```rust
let skill = client.get_skill("skill-id").await?;
```

### 7. Batch Operations

**Send Multiple Messages**
```rust
use watsonx_rs::{BatchMessageRequest, Message};

let messages = vec![
    Message {
        role: "user".to_string(),
        content: "What is AI?".to_string(),
    },
    Message {
        role: "user".to_string(),
        content: "Explain machine learning".to_string(),
    },
];

let request = BatchMessageRequest {
    messages,
    agent_id: "agent-id".to_string(),
    thread_id: None,
    metadata: None,
};

let response = client.send_batch_messages(request).await?;
for result in response.responses {
    println!("Message {}: {}", result.message_index, result.response);
}
```

### 8. Document Collections

**List Collections**
```rust
let collections = client.list_collections().await?;
for collection in collections {
    println!("Collection: {} ({})", collection.name, collection.id);
}
```

**Get Collection**
```rust
let collection = client.get_collection("collection-id").await?;
```

**Get Document**
```rust
let doc = client.get_document("collection-id", "document-id").await?;
println!("Document: {}", doc.title);
```

**Search Documents**
```rust
use watsonx_rs::SearchRequest;

let search_req = SearchRequest {
    query: "machine learning".to_string(),
    limit: Some(10),
    threshold: Some(0.7),
    filters: None,
};

let results = client.search_documents("collection-id", search_req).await?;
for result in results.results {
    println!("Found: {} (score: {})", result.title, result.similarity_score);
}
```

## Error Handling

All methods return `Result<T>` which contains either the result or an `Error`:

```rust
match client.get_agent("agent-id").await {
    Ok(agent) => println!("Agent: {}", agent.name),
    Err(Error::Authentication(msg)) => eprintln!("Auth error: {}", msg),
    Err(Error::Api(msg)) => eprintln!("API error: {}", msg),
    Err(Error::Network(msg)) => eprintln!("Network error: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Configuration

Initialize the client with environment variables:

```rust
use watsonx_rs::{OrchestrateClient, OrchestrateConfig};

// Requires WXO_INSTANCE_ID and optionally WXO_REGION (defaults to us-south)
let config = OrchestrateConfig::from_env()?;
let api_key = std::env::var("WATSONX_API_KEY")?;
let client = OrchestrateClient::new(config).with_token(api_key);
```

## Run Status Tracking

The `RunStatus` enum provides the following states:

- `Queued` - Run is waiting to be processed
- `InProgress` - Run is currently executing
- `Completed` - Run finished successfully
- `Failed` - Run encountered an error
- `Cancelled` - Run was cancelled by user

## Best Practices

1. **Thread Reuse**: Create a thread once and reuse it for multiple messages to maintain conversation context
2. **Error Handling**: Always handle errors appropriately, especially network and authentication errors
3. **Streaming**: Use `stream_message()` for long-running operations to provide real-time feedback
4. **Batch Operations**: Use batch operations for processing multiple messages efficiently
5. **Tool Execution**: Check tool availability before executing tools directly

## Examples

See the `examples/` directory for complete working examples:
- `orchestrate_example.rs` - Basic agent listing
- `orchestrate_chat.rs` - Complete chat workflow with streaming

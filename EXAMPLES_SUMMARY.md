# Orchestrate SDK Examples Summary

## Overview

The watsonx-rs SDK includes four comprehensive examples demonstrating all capabilities of the Watson Orchestrate API.

## Quick Start

```bash
# 1. Basic example - agent discovery
cargo run --example orchestrate_example

# 2. Chat workflow - streaming and non-streaming
cargo run --example orchestrate_chat

# 3. Advanced features - full capability test
cargo run --example orchestrate_advanced

# 4. Use cases - practical scenarios
cargo run --example orchestrate_use_cases
```

## Examples Overview

### 1. orchestrate_example.rs
**Difficulty:** Beginner  
**Duration:** < 1 second

**What it demonstrates:**
- Client initialization
- Agent discovery
- Basic API connectivity

**Key code patterns:**
```rust
let agents = client.list_agents().await?;
let agent = client.get_agent("agent-id").await?;
```

**Output:**
```
📝 Listing available agents...
✅ Found 3 agents:
  - test (ID: fdd98d33-2911-47f7-b540-b7020f34aa5c)
  - test (ID: bb5ca8dd-81f8-4ca7-9e88-ce4e410b2e6c)
  - hello agent (ID: e85560b1-6876-4991-9749-e89333e514a6)
```

---

### 2. orchestrate_chat.rs
**Difficulty:** Intermediate  
**Duration:** 5-10 seconds

**What it demonstrates:**
- Non-streaming message sending
- Streaming responses with real-time callbacks
- Thread management for conversation context
- Message history retrieval
- Multi-turn conversations

**Key code patterns:**
```rust
// Non-streaming
let (response, thread_id) = client.send_message(
    "agent-id",
    "Hello!",
    None
).await?;

// Streaming
client.stream_message(
    "agent-id",
    "Tell me about yourself",
    thread_id,
    |chunk| {
        print!("{}", chunk);
        Ok(())
    }
).await?;

// History
let messages = client.get_thread_messages(&thread_id).await?;
```

**Output:**
```
💬 Step 2: Sending a message (non-streaming)...
You: Hello! Can you introduce yourself?
🤖 Agent: Hello! I am watsonx Orchestrate...

🌊 Step 4: Streaming response...
You: Tell me about Watson Orchestrate capabilities...
🤖 Agent (streaming): I can assist with tasks...
```

---

### 3. orchestrate_advanced.rs
**Difficulty:** Advanced  
**Duration:** 10-15 seconds

**What it demonstrates:**
- Agent management (list, get)
- Thread creation and deletion
- Skill listing
- Tool management
- Non-streaming messages
- Streaming messages
- Thread history
- Run management (list, get, cancel)
- Batch operations
- Document collections

**Key code patterns:**
```rust
// Thread management
let thread = client.create_thread(Some("agent-id")).await?;
client.delete_thread(&thread_id).await?;

// Run tracking
let runs = client.list_runs(Some("agent-id")).await?;
let run = client.get_run("run-id").await?;

// Batch operations
let batch_request = BatchMessageRequest {
    messages: vec![...],
    agent_id: "agent-id".to_string(),
    thread_id: Some(thread_id),
    metadata: None,
};
let response = client.send_batch_messages(batch_request).await?;

// Document collections
let collections = client.list_collections().await?;
```

**Output:**
```
📋 1. AGENT MANAGEMENT
✅ Found 3 agents:
   - test (ID: fdd98d33-2911-47f7-b540-b7020f34aa5c)
   ...

🧵 2. THREAD MANAGEMENT
✅ Created new thread: 7d2a4ba8-95b5-4df7-9f87-79d644a5451c

... (11 sections total)

✨ TEST SUMMARY
✅ Agent Management - Tested
✅ Thread Management - Tested
✅ Streaming Chat - Tested
... (all features)
```

---

### 4. orchestrate_use_cases.rs
**Difficulty:** Intermediate  
**Duration:** 5-10 seconds

**What it demonstrates:**
- Multi-turn conversations with context
- Document search and Q&A
- Tool integration
- Run tracking and analytics
- Skill discovery

**Key code patterns:**
```rust
// Multi-turn conversation
for question in questions {
    client.stream_message(
        "agent-id",
        question,
        Some(thread_id.clone()),
        |chunk| { ... }
    ).await?;
}

// Document search
let search_req = SearchRequest {
    query: "artificial intelligence".to_string(),
    limit: Some(5),
    threshold: Some(0.5),
    filters: None,
};
let results = client.search_documents(&collection_id, search_req).await?;

// Run tracking
let runs = client.list_runs(Some("agent-id")).await?;
for run in runs {
    println!("Run: {} - Status: {:?}", run.run_id, run.status);
}
```

**Output:**
```
📞 USE CASE 1: Multi-Turn Conversation
Q1: What is your name?
A1: I am watsonx Orchestrate...

📚 USE CASE 2: Document Search & Q&A
Searching for: 'artificial intelligence'
Found 5 results:
  1. Document Title (Score: 0.95)
     Snippet: ...

🔧 USE CASE 3: Tool Integration
Available tools: 5
  - Tool Name (tool-id)
    Description: ...
```

---

## Feature Coverage Matrix

| Feature | Example 1 | Example 2 | Example 3 | Example 4 |
|---------|-----------|-----------|-----------|-----------|
| List Agents | ✅ | ✅ | ✅ | ✅ |
| Get Agent | ✅ | ✅ | ✅ | ✅ |
| Send Message | ❌ | ✅ | ✅ | ✅ |
| Stream Message | ❌ | ✅ | ✅ | ✅ |
| Create Thread | ❌ | ✅ | ✅ | ✅ |
| Delete Thread | ❌ | ✅ | ✅ | ✅ |
| Get Thread Messages | ❌ | ✅ | ✅ | ✅ |
| List Threads | ❌ | ✅ | ✅ | ❌ |
| List Runs | ❌ | ❌ | ✅ | ✅ |
| Get Run | ❌ | ❌ | ✅ | ❌ |
| Cancel Run | ❌ | ❌ | ✅ | ❌ |
| List Skills | ❌ | ❌ | ✅ | ✅ |
| Get Skill | ❌ | ❌ | ❌ | ❌ |
| List Tools | ❌ | ❌ | ✅ | ✅ |
| Get Tool | ❌ | ❌ | ❌ | ❌ |
| Execute Tool | ❌ | ❌ | ❌ | ❌ |
| Batch Messages | ❌ | ❌ | ✅ | ❌ |
| List Collections | ❌ | ❌ | ✅ | ✅ |
| Get Collection | ❌ | ❌ | ❌ | ✅ |
| Search Documents | ❌ | ❌ | ❌ | ✅ |

---

## Learning Path

### Beginner
1. Start with `orchestrate_example.rs` - understand basic client setup
2. Review the code to understand authentication and configuration

### Intermediate
1. Move to `orchestrate_chat.rs` - learn streaming and threading
2. Understand callback patterns and error handling
3. Practice with `orchestrate_use_cases.rs` - see practical applications

### Advanced
1. Study `orchestrate_advanced.rs` - comprehensive feature coverage
2. Understand batch operations and run tracking
3. Integrate patterns into your own application

---

## Common Patterns

### Error Handling
```rust
match client.list_agents().await {
    Ok(agents) => println!("Found {} agents", agents.len()),
    Err(Error::Authentication(msg)) => eprintln!("Auth error: {}", msg),
    Err(Error::Api(msg)) => eprintln!("API error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Streaming with Callbacks
```rust
client.stream_message(
    "agent-id",
    "Your question",
    thread_id,
    |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
        Ok(())
    }
).await?;
```

### Batch Operations
```rust
let request = BatchMessageRequest {
    messages: vec![
        Message { role: "user".to_string(), content: "Q1".to_string() },
        Message { role: "user".to_string(), content: "Q2".to_string() },
    ],
    agent_id: "agent-id".to_string(),
    thread_id: Some(thread_id),
    metadata: None,
};
let response = client.send_batch_messages(request).await?;
```

---

## Testing Checklist

- [ ] Run all four examples successfully
- [ ] Verify streaming outputs text in real-time
- [ ] Confirm thread IDs are maintained across messages
- [ ] Check that batch operations return multiple results
- [ ] Verify error handling for invalid inputs
- [ ] Test with different agents
- [ ] Verify document search returns results

---

## Next Steps

1. **Integrate into Your App**
   - Copy patterns from examples
   - Adapt to your specific use case
   - Add domain-specific logic

2. **Optimize for Production**
   - Add retry logic
   - Implement connection pooling
   - Add comprehensive logging

3. **Extend Functionality**
   - Create custom wrappers
   - Build domain-specific clients
   - Implement caching strategies

---

## Documentation

- **API Reference**: See [ORCHESTRATE_CAPABILITIES.md](docs/ORCHESTRATE_CAPABILITIES.md)
- **Testing Guide**: See [TESTING_GUIDE.md](docs/TESTING_GUIDE.md)
- **Architecture**: See [ARCHITECTURE.md](ARCHITECTURE.md)

---

## Support

For issues or questions:
1. Check the example code
2. Review the documentation
3. Check error messages carefully
4. Verify environment configuration
5. Open an issue on the repository

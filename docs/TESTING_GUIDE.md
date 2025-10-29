# Orchestrate SDK Testing Guide

This guide explains how to test the Watson Orchestrate SDK using the provided examples.

## Prerequisites

1. **Environment Setup**
   ```bash
   # Copy the example environment file
   cp .env.example .env
   
   # Edit with your actual credentials
   export WATSONX_API_KEY=your_api_key
   export WXO_INSTANCE_ID=your_instance_id
   export WXO_REGION=us-south  # or your region
   ```

2. **Verify Configuration**
   ```bash
   # Check that environment variables are set
   echo $WATSONX_API_KEY
   echo $WXO_INSTANCE_ID
   ```

## Running Examples

### 1. Basic Example - Agent Listing

**What it tests:**
- Client initialization
- Agent discovery
- Basic API connectivity

**Run:**
```bash
cargo run --example orchestrate_example
```

**Expected Output:**
```
🚀 WatsonX Orchestrate SDK Example
=====================================
📝 Listing available agents...
✅ Found 3 agents:
  - test (ID: fdd98d33-2911-47f7-b540-b7020f34aa5c)
  - ...
```

### 2. Chat Example - Streaming & Non-Streaming

**What it tests:**
- Non-streaming message sending
- Streaming responses with real-time callbacks
- Thread management for conversation context
- Message history retrieval

**Run:**
```bash
cargo run --example orchestrate_chat
```

**Expected Output:**
```
🚀 WatsonX Orchestrate Chat Example
💬 Step 2: Sending a message (non-streaming)...
🤖 Agent: Hello! I am watsonx Orchestrate...

🌊 Step 4: Streaming response...
🤖 Agent (streaming): I can assist with tasks...
```

### 3. Advanced Example - Full Capability Test

**What it tests:**
- Agent management
- Thread creation and deletion
- Skill listing
- Tool management
- Non-streaming messages
- Streaming messages
- Thread history
- Run management
- Batch operations
- Document collections

**Run:**
```bash
cargo run --example orchestrate_advanced
```

**Expected Output:**
```
🚀 Advanced Orchestrate SDK Test
================================

📋 1. AGENT MANAGEMENT
✅ Found 3 agents:
   - test (ID: fdd98d33-2911-47f7-b540-b7020f34aa5c)
   ...

🧵 2. THREAD MANAGEMENT
✅ Created new thread: 7d2a4ba8-95b5-4df7-9f87-79d644a5451c

... (more sections)

✨ TEST SUMMARY
✅ Agent Management - Tested
✅ Thread Management - Tested
✅ Streaming Chat - Tested
... (all features)
```

### 4. Use Cases Example - Practical Scenarios

**What it tests:**
- Multi-turn conversations with context
- Document search and Q&A
- Tool integration
- Run tracking and analytics
- Skill discovery

**Run:**
```bash
cargo run --example orchestrate_use_cases
```

**Expected Output:**
```
🎯 Orchestrate SDK Use Cases
============================

📞 USE CASE 1: Multi-Turn Conversation
Q1: What is your name?
A1: I am watsonx Orchestrate...

📚 USE CASE 2: Document Search & Q&A
Searching for: 'artificial intelligence'
Found 5 results:
  1. Document Title (Score: 0.95)
     Snippet: ...
```

## Testing Checklist

### Connectivity Tests
- [ ] Client initializes successfully
- [ ] API authentication works
- [ ] Base URL is correct

### Agent Tests
- [ ] List agents returns results
- [ ] Get specific agent works
- [ ] Agent properties are populated

### Conversation Tests
- [ ] Create thread succeeds
- [ ] Send non-streaming message works
- [ ] Stream message delivers chunks in real-time
- [ ] Thread ID is maintained across messages
- [ ] Get thread messages returns history

### Advanced Tests
- [ ] List runs returns results
- [ ] Get specific run works
- [ ] Cancel run succeeds
- [ ] List skills returns results
- [ ] List tools returns results
- [ ] Batch operations process multiple messages
- [ ] Document collections can be listed
- [ ] Document search returns results

## Troubleshooting

### Common Issues

**1. Authentication Error**
```
Error: Authentication("Not authenticated. Set access token first.")
```
**Solution:** Check that `WATSONX_API_KEY` is set correctly
```bash
echo $WATSONX_API_KEY
```

**2. Configuration Error**
```
Error: "Failed to load Orchestrate config from environment"
```
**Solution:** Verify `WXO_INSTANCE_ID` is set
```bash
echo $WXO_INSTANCE_ID
```

**3. Network Error**
```
Error: Network("Connection refused")
```
**Solution:** Check internet connectivity and API endpoint availability

**4. API Error (404)**
```
Error: Api("Failed to list skills: 404 Not Found")
```
**Solution:** This endpoint may not be available in your instance. This is expected for some features.

**5. Serialization Error**
```
Error: Serialization("error decoding response body")
```
**Solution:** The API response format may differ. Check the API documentation.

## Performance Testing

### Streaming Performance
```bash
# Test streaming with timing
time cargo run --example orchestrate_chat
```

### Batch Performance
```bash
# Monitor batch operations
cargo run --example orchestrate_advanced 2>&1 | grep "Batch"
```

## Integration Testing

### Test with Your Agent
Modify the examples to use a specific agent:

```rust
// In orchestrate_chat.rs or orchestrate_advanced.rs
let agent_id = "your-agent-id";
let agent = client.get_agent(agent_id).await?;
```

### Test with Your Collection
```rust
// In orchestrate_use_cases.rs
let collection_id = "your-collection-id";
let collection = client.get_collection(collection_id).await?;
```

## Continuous Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test
```bash
cargo test orchestrate_tests
```

### Run with Output
```bash
cargo test -- --nocapture
```

## Performance Metrics

### Expected Response Times
- List agents: < 1s
- Send message (non-streaming): 2-5s
- Stream message: Real-time chunks
- List runs: < 1s
- Batch messages: 5-10s (depends on batch size)

### Memory Usage
- Client initialization: ~5MB
- Streaming response: ~1MB (per stream)
- Batch operation: ~2MB (per 100 messages)

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run --example orchestrate_advanced
```

### Check API Responses
Add temporary logging in the examples:
```rust
println!("Response: {:?}", response);
```

### Verify Configuration
```bash
cargo run --example orchestrate_example 2>&1 | grep "URL:"
```

## Next Steps

1. **Integrate into Your Application**
   - Copy patterns from examples
   - Adapt to your use case
   - Handle errors appropriately

2. **Optimize for Production**
   - Add retry logic
   - Implement connection pooling
   - Add monitoring and logging

3. **Extend Functionality**
   - Create custom wrappers
   - Add domain-specific logic
   - Implement caching

## Support

For issues or questions:
1. Check the [ORCHESTRATE_CAPABILITIES.md](ORCHESTRATE_CAPABILITIES.md) documentation
2. Review the example code
3. Check the API documentation
4. Open an issue on the repository

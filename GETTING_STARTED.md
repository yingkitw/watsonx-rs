# Getting Started with Watson Orchestrate SDK

## ⚡ 5-Minute Quick Start

### Step 1: Create `.env` file (1 minute)

```bash
cat > .env << EOF
WXO_INSTANCE_ID=your-instance-id
WXO_KEY=your-api-key
EOF
```

### Step 2: Create Rust project (1 minute)

```bash
cargo new my_orchestrate_app
cd my_orchestrate_app
```

### Step 3: Add dependency (1 minute)

```bash
cargo add watsonx-rs tokio dotenvy
```

### Step 4: Write code (1 minute)

```rust
use watsonx_rs::OrchestrateConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    // One-line connection!
    let client = OrchestrateConnection::new().from_env().await?;
    
    // Use the client
    let assistants = client.list_assistants().await?;
    println!("Found {} assistants", assistants.len());
    
    Ok(())
}
```

### Step 5: Run (1 minute)

```bash
cargo run
```

**Done!** 🎉

---

## 📖 What's Next?

### Option 1: Learn by Example

```bash
# Run the simple example
cargo run --example orchestrate_simple

# Run the chat example
cargo run --example orchestrate_chat

# Run advanced features
cargo run --example orchestrate_advanced
```

### Option 2: Read the Docs

- **Quick Start**: `docs/QUICK_START.md`
- **API Reference**: `docs/ORCHESTRATE_CAPABILITIES.md`
- **Troubleshooting**: `docs/QUICK_START.md#troubleshooting`

### Option 3: Explore the Code

```bash
# See the connection builder
cat src/orchestrate/connection.rs

# See the simple example
cat examples/orchestrate_simple.rs

# See the chat example
cat examples/orchestrate_chat.rs
```

---

## 🎯 Common Tasks

### Task 1: List Assistants

```rust
let assistants = client.list_assistants().await?;
for assistant in assistants {
    println!("{}: {}", assistant.name, assistant.id);
}
```

### Task 2: Send a Message

```rust
let message = client.send_message(
    &agent_id,
    &thread_id,
    "Hello, assistant!",
    false  // not streaming
).await?;

println!("{}", message.content);
```

### Task 3: Stream a Message

```rust
client.stream_message(
    &agent_id,
    &thread_id,
    "Tell me a story",
    |chunk| {
        print!("{}", chunk);
        Ok(())
    }
).await?;
```

### Task 4: List Agents

```rust
let agents = client.list_agents().await?;
for agent in agents {
    println!("{}: {}", agent.name, agent.agent_id);
}
```

---

## 🔧 Three Ways to Connect

### Way 1: From Environment (Recommended)

```rust
let client = OrchestrateConnection::new().from_env().await?;
```

**Use when:** You have `.env` file

### Way 2: With Credentials

```rust
let client = OrchestrateConnection::new()
    .with_credentials("instance-id", "api-key", "us-south")
    .await?;
```

**Use when:** Credentials from config file or secrets manager

### Way 3: With Custom URL

```rust
let client = OrchestrateConnection::new()
    .with_custom_url(
        "instance-id",
        "api-key",
        "https://custom.domain.com/api/v1/"
    )
    .await?;
```

**Use when:** Non-standard deployment or custom domain

---

## ❌ Troubleshooting

### Problem: "WXO_INSTANCE_ID not found"

**Solution:** Add to `.env`:
```
WXO_INSTANCE_ID=your-instance-id
```

### Problem: "WXO_KEY not found"

**Solution:** Add to `.env`:
```
WXO_KEY=your-api-key
```

### Problem: "Failed to generate IAM token"

**Solution:** 
- Verify API key is correct
- Check network connectivity
- Ensure you're using Watson Orchestrate API key (not WatsonX)

### Problem: "Failed to list assistants: 401 Unauthorized"

**Solution:**
- Token generation failed
- Check API key format
- Verify instance ID is correct

### Problem: "Failed to list assistants: 404 Not Found"

**Solution:**
- Instance ID might be wrong
- Check region setting
- Verify endpoint is accessible

---

## 📚 Documentation

### Quick References
- **Quick Start**: `docs/QUICK_START.md` (5 min read)
- **Comparison**: `docs/CONNECTION_COMPARISON.md` (5 min read)
- **API Reference**: `docs/ORCHESTRATE_CAPABILITIES.md` (reference)

### Examples
- **Simple**: `examples/orchestrate_simple.rs`
- **Chat**: `examples/orchestrate_chat.rs`
- **Advanced**: `examples/orchestrate_advanced.rs`
- **Use Cases**: `examples/orchestrate_use_cases.rs`

### Architecture
- **Architecture**: `ARCHITECTURE.md`
- **Implementation**: `CONNECTION_SIMPLIFICATION.md`
- **Summary**: `SIMPLIFICATION_SUMMARY.md`

---

## 🚀 Running Examples

### Simple Example (Recommended First)

```bash
cargo run --example orchestrate_simple
```

Output:
```
🚀 Watson Orchestrate - Simplified Connection
==============================================

📡 Connecting to Watson Orchestrate...
✅ Connected successfully!
   Base URL: https://us-south.watson-orchestrate.cloud.ibm.com/api/v1/

📝 Listing assistants...
✅ Found 2 assistants:
   - My Assistant (ID: asst_123)
   - Another Assistant (ID: asst_456)

🎉 Example completed!
```

### Chat Example

```bash
cargo run --example orchestrate_chat
```

### Advanced Example

```bash
cargo run --example orchestrate_advanced
```

### Use Cases Example

```bash
cargo run --example orchestrate_use_cases
```

---

## 💡 Tips & Tricks

### Tip 1: Use `.env.example` as Template

```bash
cp .env.example .env
# Edit with your credentials
```

### Tip 2: Enable Debug Logging

```bash
RUST_LOG=debug cargo run --example orchestrate_simple
```

### Tip 3: Handle Errors Gracefully

```rust
match OrchestrateConnection::new().from_env().await {
    Ok(client) => {
        // Use client
    }
    Err(e) => {
        eprintln!("Connection failed: {}", e);
        // Error message tells you exactly what's missing
    }
}
```

### Tip 4: Use in Tests

```rust
#[tokio::test]
async fn test_connection() {
    let client = OrchestrateConnection::new()
        .with_credentials("test-id", "test-key", "us-south")
        .await
        .expect("Failed to create test client");
    
    // Test your code
}
```

---

## 📋 Checklist

Before you start, make sure you have:

- ✅ Rust installed (`rustup`)
- ✅ Watson Orchestrate instance ID
- ✅ Watson Orchestrate API key
- ✅ `.env` file with credentials
- ✅ Network access to Watson Orchestrate

---

## 🎓 Learning Resources

### Official Documentation
- [IBM Watson Orchestrate API](https://developer.ibm.com/apis/catalog/watsonorchestrate--custom-assistants/api)
- [IBM Cloud Documentation](https://cloud.ibm.com/docs)

### SDK Documentation
- `docs/QUICK_START.md` - Quick start guide
- `docs/ORCHESTRATE_CAPABILITIES.md` - API reference
- `docs/TESTING_GUIDE.md` - Testing guide
- `ARCHITECTURE.md` - Architecture overview

### Examples
- `examples/orchestrate_simple.rs` - Basic usage
- `examples/orchestrate_chat.rs` - Chat workflow
- `examples/orchestrate_advanced.rs` - Advanced features
- `examples/orchestrate_use_cases.rs` - Real-world scenarios

---

## 🎉 You're Ready!

You now have everything you need to:

1. ✅ Connect to Watson Orchestrate
2. ✅ List assistants and agents
3. ✅ Send and receive messages
4. ✅ Stream responses in real-time
5. ✅ Manage conversations
6. ✅ Execute tools
7. ✅ Search documents
8. ✅ And much more!

**Start with the simple example:**

```bash
cargo run --example orchestrate_simple
```

**Then explore the other examples and documentation.**

Happy coding! 🚀

---

## 📞 Need Help?

### Check the Docs
- `docs/QUICK_START.md` - Setup and connection
- `docs/ORCHESTRATE_CAPABILITIES.md` - API reference
- `docs/TESTING_GUIDE.md` - Testing and debugging

### Run Examples
```bash
cargo run --example orchestrate_simple
cargo run --example orchestrate_chat
cargo run --example orchestrate_advanced
```

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run --example orchestrate_simple
```

### Check Troubleshooting
See `docs/QUICK_START.md#troubleshooting`

---

**Version**: Latest
**Status**: ✅ Ready to use
**Last Updated**: 2024

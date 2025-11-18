# Watson Orchestrate - Quick Start Guide

## Simplified Connection Flow

The new `OrchestrateConnection` helper simplifies the connection process to **one line of code**.

### Before (Complex)

```rust
// Old way - multiple steps
let config = OrchestrateConfig::from_env()?;
let api_key = std::env::var("WXO_KEY")?;
let token = OrchestrateClient::generate_jwt_token(&api_key).await?;
let client = OrchestrateClient::new(config).with_token(token);
```

### After (Simple)

```rust
// New way - one line!
let client = OrchestrateConnection::new().from_env().await?;
```

## Setup (5 minutes)

### 1. Create `.env` file

```bash
# Required
WXO_INSTANCE_ID=your-instance-id-here
WXO_KEY=your-api-key-here

# Optional (defaults to us-south)
WXO_REGION=us-south
```

### 2. Run the example

```bash
cargo run --example orchestrate_simple
```

### 3. Expected output

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

## Connection Methods

### Method 1: From Environment Variables (Recommended)

```rust
let client = OrchestrateConnection::new()
    .from_env()
    .await?;
```

**Required env vars:**
- `WXO_INSTANCE_ID`
- `WXO_KEY`

**Optional env vars:**
- `WXO_REGION` (defaults to us-south)
- `WXO_URL` (custom base URL)

### Method 2: With Explicit Credentials

```rust
let client = OrchestrateConnection::new()
    .with_credentials(
        "instance-id-123",
        "api-key-xyz",
        "us-south"
    )
    .await?;
```

### Method 3: With Custom URL (Non-standard Deployments)

```rust
let client = OrchestrateConnection::new()
    .with_custom_url(
        "instance-id-123",
        "api-key-xyz",
        "https://custom.domain.com/api/v1/"
    )
    .await?;
```

## Common Operations

### List Assistants

```rust
let assistants = client.list_assistants().await?;
for assistant in assistants {
    println!("{}: {}", assistant.name, assistant.id);
}
```

### List Agents

```rust
let agents = client.list_agents().await?;
for agent in agents {
    println!("{}: {}", agent.name, agent.agent_id);
}
```

### Send a Message

```rust
let message = client.send_message(
    &agent_id,
    &thread_id,
    "Hello, assistant!",
    false  // not streaming
).await?;

println!("{}", message.content);
```

### Stream a Message

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

## Troubleshooting

### Connection Failed

**Error:** `WXO_INSTANCE_ID environment variable not set`

**Solution:** Add to `.env`:
```
WXO_INSTANCE_ID=your-instance-id
```

**Error:** `WXO_KEY environment variable not set`

**Solution:** Add to `.env`:
```
WXO_KEY=your-api-key
```

**Error:** `Failed to generate IAM token`

**Solution:** 
- Verify API key is correct
- Check network connectivity
- Ensure you're using a Watson Orchestrate API key (not WatsonX API key)

### API Errors

**Error:** `Failed to list assistants: 401 Unauthorized`

**Solution:**
- Token generation failed silently
- Check API key format
- Verify instance ID is correct

**Error:** `Failed to list assistants: 404 Not Found`

**Solution:**
- Instance ID might be wrong
- Check region setting
- Verify endpoint is accessible

## Examples

### Example 1: Simple Connection Test

```bash
cargo run --example orchestrate_simple
```

### Example 2: Advanced Features

```bash
cargo run --example orchestrate_advanced
```

### Example 3: Practical Use Cases

```bash
cargo run --example orchestrate_use_cases
```

### Example 4: Chat Workflow

```bash
cargo run --example orchestrate_chat
```

## Next Steps

1. ✅ Set up `.env` file
2. ✅ Run `orchestrate_simple` example
3. ✅ Explore `orchestrate_advanced` for more features
4. ✅ Check `orchestrate_use_cases` for practical examples
5. ✅ Read `ORCHESTRATE_CAPABILITIES.md` for full API reference

## API Reference

For complete API documentation, see:
- `docs/ORCHESTRATE_CAPABILITIES.md` - All methods with examples
- `docs/TESTING_GUIDE.md` - Testing and debugging tips
- `ARCHITECTURE.md` - SDK architecture and design

## Support

If you encounter issues:

1. Check `.env` file is in project root
2. Verify credentials are correct
3. Run with `RUST_LOG=debug` for more details:
   ```bash
   RUST_LOG=debug cargo run --example orchestrate_simple
   ```
4. Check network connectivity
5. Verify Watson Orchestrate instance is running

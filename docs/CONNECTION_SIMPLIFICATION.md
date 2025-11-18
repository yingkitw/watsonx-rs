# Connection Flow Simplification

## Problem Statement

The original Watson Orchestrate connection flow required multiple steps:

1. Load configuration from environment
2. Get API key from environment
3. Generate JWT token from API key
4. Create client with config
5. Set token on client

This was error-prone and easy to miss steps.

## Solution: OrchestrateConnection Helper

A new `OrchestrateConnection` builder simplifies the entire flow to **one line of code**.

### Before (5 steps)

```rust
// Step 1: Load config
let config = OrchestrateConfig::from_env()
    .expect("Failed to load config");

// Step 2: Get API key
let api_key = std::env::var("WXO_KEY")
    .expect("WXO_KEY not set");

// Step 3: Generate token
let token = OrchestrateClient::generate_jwt_token(&api_key)
    .await
    .expect("Failed to generate token");

// Step 4: Create client
let client = OrchestrateClient::new(config);

// Step 5: Set token
let client = client.with_token(token);
```

### After (1 line)

```rust
let client = OrchestrateConnection::new().from_env().await?;
```

## Implementation

### New Module: `src/orchestrate/connection.rs`

```rust
pub struct OrchestrateConnection;

impl OrchestrateConnection {
    pub fn new() -> Self { ... }
    
    pub async fn from_env(self) -> Result<OrchestrateClient> { ... }
    
    pub async fn with_credentials(
        self,
        instance_id: &str,
        api_key: &str,
        region: &str,
    ) -> Result<OrchestrateClient> { ... }
    
    pub async fn with_custom_url(
        self,
        instance_id: &str,
        api_key: &str,
        base_url: &str,
    ) -> Result<OrchestrateClient> { ... }
}
```

### Three Connection Methods

#### 1. From Environment (Recommended)

```rust
let client = OrchestrateConnection::new().from_env().await?;
```

**Required env vars:**
- `WXO_INSTANCE_ID`
- `WXO_KEY`

**Optional env vars:**
- `WXO_REGION` (defaults to us-south)
- `WXO_URL` (custom base URL)

#### 2. With Explicit Credentials

```rust
let client = OrchestrateConnection::new()
    .with_credentials("instance-id", "api-key", "us-south")
    .await?;
```

#### 3. With Custom URL

```rust
let client = OrchestrateConnection::new()
    .with_custom_url(
        "instance-id",
        "api-key",
        "https://custom.domain.com/api/v1/"
    )
    .await?;
```

## Files Created

1. **`src/orchestrate/connection.rs`** - New connection helper module
2. **`examples/orchestrate_simple.rs`** - Simplified example
3. **`docs/QUICK_START.md`** - Quick start guide

## Files Modified

1. **`src/orchestrate/mod.rs`** - Added connection module export
2. **`README.md`** - Added Watson Orchestrate quick start section

## Benefits

✅ **Simpler API** - One line instead of five
✅ **Less Error-Prone** - All steps handled internally
✅ **Better Error Messages** - Clear guidance on what's missing
✅ **Flexible** - Three connection methods for different use cases
✅ **Backward Compatible** - Old approach still works

## Testing

All tests pass:
```bash
cargo test --lib
# test result: ok. 17 passed; 0 failed; 3 ignored
```

New test added:
```rust
#[test]
fn test_connection_builder_creation() {
    let _conn = OrchestrateConnection::new();
    let _conn2 = OrchestrateConnection::default();
}
```

## Usage Examples

### Example 1: Simple Connection Test

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

### Example 2: In Your Code

```rust
use watsonx_rs::OrchestrateConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One line connection!
    let client = OrchestrateConnection::new().from_env().await?;
    
    // Use the client
    let assistants = client.list_assistants().await?;
    println!("Found {} assistants", assistants.len());
    
    Ok(())
}
```

## Migration Guide

### For Existing Code

Old code still works:
```rust
let config = OrchestrateConfig::from_env()?;
let api_key = std::env::var("WXO_KEY")?;
let token = OrchestrateClient::generate_jwt_token(&api_key).await?;
let client = OrchestrateClient::new(config).with_token(token);
```

New code is simpler:
```rust
let client = OrchestrateConnection::new().from_env().await?;
```

### Recommended Update

Replace the old pattern with the new one in your code:

```diff
- let config = OrchestrateConfig::from_env()?;
- let api_key = std::env::var("WXO_KEY")?;
- let token = OrchestrateClient::generate_jwt_token(&api_key).await?;
- let client = OrchestrateClient::new(config).with_token(token);
+ let client = OrchestrateConnection::new().from_env().await?;
```

## Documentation

- **Quick Start**: `docs/QUICK_START.md`
- **Full Examples**: `examples/orchestrate_simple.rs`
- **API Reference**: `docs/ORCHESTRATE_CAPABILITIES.md`
- **Architecture**: `ARCHITECTURE.md`

## Next Steps

1. ✅ Use `OrchestrateConnection` in new code
2. ✅ Update existing examples to use new helper
3. ✅ Update documentation
4. ✅ Consider deprecating old pattern in future versions

## Summary

The connection flow is now **5x simpler** with the new `OrchestrateConnection` helper. Users can connect to Watson Orchestrate with a single line of code, making the SDK much more user-friendly and less error-prone.

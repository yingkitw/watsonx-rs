# WatsonX AI - Quick Start Guide

## Simplified Connection Flow

The new `WatsonxConnection` helper simplifies the connection process to **one line of code**.

### Before (Complex)

```rust
// Old way - multiple steps
let config = WatsonxConfig::from_env()?;
let mut client = WatsonxClient::new(config)?;
client.connect().await?;
```

### After (Simple)

```rust
// New way - one line!
let mut client = WatsonxConnection::new().from_env().await?;
```

## Setup (5 minutes)

### 1. Create `.env` file

```bash
# Required
WATSONX_API_KEY=your-api-key-here
WATSONX_PROJECT_ID=your-project-id-here

# Optional (defaults provided)
WATSONX_API_URL=https://us-south.ml.cloud.ibm.com
IAM_IBM_CLOUD_URL=https://iam.cloud.ibm.com
WATSONX_API_VERSION=2023-05-29
WATSONX_TIMEOUT_SECS=120
```

### 2. Run the example

```bash
cargo run --example basic_simple
```

### 3. Expected output

```
🚀 WatsonX AI - Simplified Connection
=====================================

📡 Connecting to WatsonX AI...
✅ Connected successfully!

📝 Generating text...
Prompt: Explain Rust ownership in one sentence.

Response:
---
Rust's ownership system ensures memory safety by allowing only one owner of data at a time, automatically freeing memory when the owner goes out of scope.
---

✅ Generation completed!
   Model: ibm/granite-4-h-small

🎉 Example completed!
```

## Connection Methods

### Method 1: From Environment Variables (Recommended)

```rust
let mut client = WatsonxConnection::new()
    .from_env()
    .await?;
```

**Required env vars:**
- `WATSONX_API_KEY`
- `WATSONX_PROJECT_ID`

**Optional env vars:**
- `WATSONX_API_URL` (defaults to IBM Cloud)
- `IAM_IBM_CLOUD_URL` (defaults to IBM Cloud)
- `WATSONX_API_VERSION` (defaults to 2023-05-29)
- `WATSONX_TIMEOUT_SECS` (defaults to 120)

### Method 2: With Explicit Credentials

```rust
let mut client = WatsonxConnection::new()
    .with_credentials(
        "api-key-xyz",
        "project-id-123"
    )
    .await?;
```

### Method 3: With Custom Endpoints (Non-standard Deployments)

```rust
let mut client = WatsonxConnection::new()
    .with_custom_endpoints(
        "api-key-xyz",
        "project-id-123",
        "https://custom-iam.com",
        "https://custom-api.com"
    )
    .await?;
```

### Method 4: With Full Configuration (Advanced)

```rust
let config = WatsonxConfig {
    api_key: "api-key-xyz".to_string(),
    project_id: "project-id-123".to_string(),
    iam_url: "https://iam.cloud.ibm.com".to_string(),
    api_url: "https://us-south.ml.cloud.ibm.com".to_string(),
    api_version: "2023-05-29".to_string(),
    timeout_secs: 120,
};

let mut client = WatsonxConnection::new()
    .with_config(config)
    .await?;
```

## Common Operations

### Generate Text (Streaming)

```rust
use watsonx_rs::GenerationConfig;
use watsonx_rs::models::models;

let gen_config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL);

client.generate_text_stream(
    "Hello, world!",
    &gen_config,
    |chunk| {
        print!("{}", chunk);
        Ok(())
    }
).await?;
```

### Generate Text (Non-streaming)

```rust
let gen_config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL);

let result = client.generate_text(
    "Hello, world!",
    &gen_config
).await?;

println!("{}", result.text);
```

### Generate Text (Batch)

```rust
let prompts = vec![
    "What is Rust?".to_string(),
    "What is async/await?".to_string(),
    "What is ownership?".to_string(),
];

let gen_config = GenerationConfig::default()
    .with_model(models::GRANITE_4_H_SMALL);

let batch_result = client.generate_batch_simple(prompts, &gen_config).await?;

for item in batch_result.results {
    if let Some(result) = item.result {
        println!("{}", result.text);
    }
}
```

### List Available Models

```rust
let models = client.list_models().await?;
for model in models {
    println!("{}: {}", model.model_id, model.label.unwrap_or_default());
}
```

## Troubleshooting

### Problem: "WATSONX_API_KEY not found"

**Solution:** Add to `.env`:
```
WATSONX_API_KEY=your-api-key
```

### Problem: "WATSONX_PROJECT_ID not found"

**Solution:** Add to `.env`:
```
WATSONX_PROJECT_ID=your-project-id
```

### Problem: "Failed to connect to WatsonX"

**Solution:**
- Verify API key is correct
- Check network connectivity
- Verify project ID is correct
- Check if endpoint is accessible

### Problem: "Token generation failed"

**Solution:**
- Verify API key format
- Check IAM endpoint is accessible
- Ensure credentials are valid

## Examples

### Example 1: Simple Connection Test

```bash
cargo run --example basic_simple
```

### Example 2: Basic Generation

```bash
cargo run --example basic_generation
```

### Example 3: Streaming Generation

```bash
cargo run --example streaming_generation
```

### Example 4: Batch Generation

```bash
cargo run --example batch_generation
```

### Example 5: Model Listing

```bash
cargo run --example list_models
```

## API Reference

For complete API documentation, see:
- `docs/ORCHESTRATE_CAPABILITIES.md` - Full API reference
- `README.md` - Main documentation
- `ARCHITECTURE.md` - SDK architecture

## Support

If you encounter issues:

1. Check `.env` file is in project root
2. Verify credentials are correct
3. Run with `RUST_LOG=debug` for more details:
   ```bash
   RUST_LOG=debug cargo run --example basic_simple
   ```
4. Check network connectivity
5. Verify WatsonX instance is running

## Next Steps

1. ✅ Set up `.env` file
2. ✅ Run `basic_simple` example
3. ✅ Explore other examples
4. ✅ Read full API documentation
5. ✅ Build your application

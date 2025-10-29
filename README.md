# WatsonX-RS

A modern Rust SDK for IBM WatsonX AI platform, designed for simplicity and performance.

## 🎯 Vision

This SDK aims to provide comprehensive support for the entire IBM WatsonX ecosystem:

- **🤖 watsonx.ai** - AI and machine learning models (current focus)
- **📊 watsonx.data** - Data management and analytics
- **🛡️ watsonx.governance** - AI governance and compliance
- **⚙️ watsonx.orchestrate** - Workflow orchestration and automation

Currently, we focus on `watsonx.ai` with text generation capabilities, but the architecture is designed to expand across all WatsonX services.

## 🚀 Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
watsonx-rs = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 2. Set up your credentials

Create a `.env` file in your project root:

```bash
# Copy the example file
cp .env.example .env

# Edit with your actual values
WATSONX_API_KEY=your_actual_api_key
WATSONX_PROJECT_ID=your_actual_project_id
```

### 3. Generate text (streaming)

```rust
use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client from environment
    let config = WatsonxConfig::from_env()?;
    let mut client = WatsonxClient::new(config)?;
    
    // Connect to WatsonX
    client.connect().await?;
    
    // Set the model and generate text with real-time streaming
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let result = client.generate_text_stream("Hello, world!", &gen_config, |chunk| {
        print!("{}", chunk);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }).await?;
    
    println!("\nModel used: {}", result.model_id);
    Ok(())
}
```

### 4. Generate text (non-streaming)

```rust
use watsonx_rs::{WatsonxClient, WatsonxConfig, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WatsonxConfig::from_env()?;
    let mut client = WatsonxClient::new(config)?;
    client.connect().await?;
    
    // Set the model and generate complete response at once
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL);
    
    let result = client.generate_text("Write a haiku about programming.", &gen_config).await?;
    
    println!("Generated: {}", result.text);
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

## 📚 Examples

Run these examples to see the SDK in action:

```bash
# Basic streaming generation
cargo run --example basic_generation

# Compare streaming vs non-streaming
cargo run --example streaming_vs_non_streaming

# List available models
cargo run --example list_models

# Use predefined model constants
cargo run --example model_constants
```

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

### Planned (watsonx.ai)
- 🔄 Chat completion API
- 🔄 Embeddings generation
- 🔄 Fine-tuning support
- 🔄 Batch processing

### Future (Full WatsonX Platform)
- 📊 **watsonx.data**: Data ingestion, processing, analytics
- 🛡️ **watsonx.governance**: Model governance, bias detection
- ⚙️ **watsonx.orchestrate**: Workflow automation, pipeline management

## 🤝 Contributing

We welcome contributions! The SDK is designed to be extensible across the entire WatsonX platform.

## 📄 License

This project is licensed under the MIT License.
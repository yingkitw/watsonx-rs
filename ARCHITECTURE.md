# Architecture

## Overview

WatsonX-RS is a Rust SDK for IBM WatsonX AI platform, providing high-level interfaces for:
- **watsonx.ai**: Text generation with streaming support
- **watsonx.orchestrate**: Agent management and chat functionality

The architecture emphasizes:

- **Async/Await**: Full async support with Tokio
- **Streaming**: Real-time Server-Sent Events (SSE) processing
- **Security**: Environment-based configuration
- **Type Safety**: Strong typing throughout
- **Error Handling**: Comprehensive error types
- **Simplicity**: Following wxo-client-main patterns for consistency

## Architecture Diagram

```mermaid
graph TB
    A[User Application] --> B[WatsonxClient]
    A --> O[OrchestrateClient]
    
    B --> C[WatsonxConfig]
    B --> D[Authentication]
    B --> E[Generation API]
    B --> BA[Batch Generation]
    
    C --> F[Environment Variables]
    C --> G[Configuration Options]
    
    D --> H[IAM Token API]
    H --> I[Access Token]
    
    E --> J[Streaming Endpoint]
    E --> K[Generation Params]
    E --> N[Non-Streaming Endpoint]
    
    J --> L[SSE Parser]
    L --> M[Stream Callback]
    M --> A
    
    N --> NA[Complete Response]
    NA --> A
    
    BA --> BB[BatchRequest Queue]
    BB --> BC[tokio::spawn Tasks]
    BC --> BD[Concurrent HTTP Requests]
    BD --> BE[BatchGenerationResult]
    BE --> A
    
    B --> NQ[Quality Assessment]
    
    O --> P[OrchestrateConfig]
    O --> Q[Agent API]
    O --> R[Chat API]
    
    P --> F
    P --> S[Instance ID + Region]
    
    Q --> T[/agents endpoint]
    R --> U[/runs/stream endpoint]
    
    U --> V[Orchestrate SSE Parser]
    V --> W[Event Parser]
    W --> X[message.created/delta]
    X --> A
    
    style B fill:#e1f5ff
    style O fill:#ffe1f5
    style J fill:#fff4e1
    style L fill:#e8f5e9
    style V fill:#e8f5e9
    style BA fill:#fff9e1
    style BC fill:#ffe1f5
```

## Component Architecture

### Core Components

#### 1. `WatsonxClient` (src/client.rs)
The main client interface for interacting with WatsonX services.

**Responsibilities:**
- Manage authentication tokens
- Handle HTTP requests to WatsonX API
- Parse SSE streaming responses
- Provide quality assessment
- Execute concurrent batch operations with true parallelism

**Key Methods:**
- `new()` - Create client from configuration
- `from_env()` - Create client from environment variables
- `connect()` - Authenticate and get access token
- `generate()` - Generate text (uses streaming internally)
- `generate_with_config()` - Generate with custom configuration
- `generate_text()` - Standard text generation (returns complete response)
- `generate_text_stream()` - Real-time streaming generation
- `generate_batch()` - Concurrent batch generation with per-request configuration
- `generate_batch_simple()` - Concurrent batch generation with uniform configuration
- `list_models()` - Fetch available foundation models from API
- `assess_quality()` - Evaluate generated text quality

#### 2. `WatsonxConfig` (src/config.rs)
Configuration management for WatsonX client.

**Responsibilities:**
- Load configuration from environment variables
- Validate configuration
- Provide defaults
- Support multiple configuration sources

**Configuration Sources:**
1. Environment variables (`.env` file or system env)
2. Programmatic configuration
3. Default values

**Environment Variables:**
- `WATSONX_API_KEY` - IBM Cloud API key
- `WATSONX_PROJECT_ID` - WatsonX project ID
- `WATSONX_API_URL` - API base URL (default: us-south)
- `IAM_IBM_CLOUD_URL` - IAM authentication URL
- `WATSONX_API_VERSION` - API version
- `WATSONX_TIMEOUT_SECS` - Request timeout

#### 3. `GenerationConfig` (src/types.rs)
Configuration for text generation requests.

**Responsibilities:**
- Define generation parameters
- Provide factory methods for common configurations
- Validate parameter ranges

**Key Parameters:**
- `model_id` - Model to use
- `max_tokens` - Maximum tokens to generate
- `timeout` - Request timeout
- `top_k`, `top_p` - Sampling parameters
- `stop_sequences` - Stop generation triggers
- `repetition_penalty` - Penalty for repetition

**Configuration Presets:**
- `default()` - Standard configuration
- `long_form()` - For long responses (128k tokens)
- `quick_response()` - Fast responses (2k tokens)

#### 4. `Error` Types (src/error.rs)
Comprehensive error handling.

**Error Variants:**
- `Network` - Network/connection errors
- `Authentication` - Auth failures
- `Api` - API errors from WatsonX
- `Timeout` - Request timeouts
- `Serialization` - JSON parsing errors
- `Configuration` - Config validation errors
- `InvalidInput` - Invalid user input
- `RateLimit` - Rate limiting
- `ModelNotFound` - Invalid model
- `ProjectNotFound` - Invalid project

#### 5. Model Constants (src/models.rs)
Constants for available WatsonX models.

**Supported Models:**
- `ibm/granite-4-h-small` (default)
- `ibm/granite-3-3-8b-instruct`
- `ibm/granite-3-0-8b-instruct`
- `ibm/granite-3-0-70b-instruct`
- `ibm/granite-2-0-8b-instruct`
- `ibm/granite-2-0-70b-instruct`
- `ibm/granite-1-3-8b-instruct`
- `ibm/granite-1-3-70b-instruct`

#### 6. Model Information (src/types.rs)
Dynamic model information fetched from WatsonX API.

**ModelInfo Structure:**
- `model_id` - Unique model identifier
- `name` - Human-readable model name
- `description` - Model description
- `provider` - Model provider (e.g., IBM)
- `version` - Model version
- `supported_tasks` - List of supported tasks
- `max_context_length` - Maximum context length
- `available` - Availability status

#### 6.1. Batch Generation Types (src/types.rs)
Types for concurrent batch text generation operations.

**BatchRequest:**
- `prompt` - Text prompt for generation
- `config` - Optional per-request configuration (uses default if None)
- `id` - Optional identifier for tracking requests

**BatchItemResult:**
- `id` - Request identifier (if provided)
- `prompt` - Original prompt used
- `result` - Generation result if successful
- `error` - Error if request failed
- Helper methods: `is_success()`, `is_failure()`

**BatchGenerationResult:**
- `results` - Vector of all batch item results
- `total` - Total number of requests
- `successful` - Number of successful requests
- `failed` - Number of failed requests
- `duration` - Total execution time
- Helper methods: `successes()`, `failures()`, `all_succeeded()`, `any_failed()`

**Batch Execution Flow:**
1. Create `BatchRequest` instances with prompts and optional configs
2. Call `generate_batch()` or `generate_batch_simple()`
3. Each request is spawned as a separate `tokio::spawn` task
4. All tasks execute concurrently using shared HTTP client
5. Results are collected and returned as `BatchGenerationResult`
6. Per-item error handling allows partial success

#### 7. `OrchestrateClient` (src/orchestrate/client.rs)
Client for WatsonX Orchestrate agent management and chat functionality.

**Responsibilities:**
- Manage agent discovery and selection
- Handle chat interactions with agents
- Maintain conversation context (thread_id)
- Parse Orchestrate-specific SSE events
- Manage tools, threads, runs, and documents

**Key Methods:**
- `new()` - Create client from OrchestrateConfig
- `list_agents()` - Discover available agents
- `send_message()` - Send message and get response (non-streaming)
- `stream_message()` - Send message with real-time streaming response
- `update_tool()`, `delete_tool()`, `test_tool()` - Tool management
- `get_tool_versions()`, `get_tool_execution_history()` - Tool tracking
- `chat_with_docs()`, `stream_chat_with_docs()` - Document Q&A

**Configuration:**
- Simplified config (matching wxo-client-main pattern)
- Only requires: `instance_id` and `region`
- Loads from environment: `WXO_INSTANCE_ID`, `WXO_REGION`, `WATSONX_API_KEY`

#### 8. `OrchestrateConfig` (src/orchestrate/config.rs)
Simplified configuration for Watson Orchestrate operations.

**Responsibilities:**
- Load configuration from environment variables
- Construct base URL from instance_id and region
- Provide defaults (region defaults to "us-south")

**Environment Variables:**
- `WXO_INSTANCE_ID` - Watson Orchestrate instance ID (required)
- `WXO_REGION` - Region (optional, defaults to "us-south")
- `WATSONX_API_KEY` - API key for authentication (or `IAM_API_KEY`, `WO_API_KEY`)

**Base URL Construction:**
```
https://api.{region}.watson-orchestrate.cloud.ibm.com/instances/{instance_id}/v1/orchestrate
```

#### 9. Orchestrate Module Structure (src/orchestrate/)
Modular organization of Orchestrate functionality.

**Module Organization:**
- `mod.rs` - Module root and re-exports
- `config.rs` - Configuration management
- `client.rs` - Client implementation
- `types.rs` - All types and data structures

**Benefits:**
- Clear separation of concerns
- Easier to maintain and extend
- Logical namespace hierarchy
- Better code discoverability

## Data Flow

### Streaming Generation Flow

```mermaid
sequenceDiagram
    participant User
    participant Client
    participant IAM
    participant API
    participant SSE
    
    User->>Client: generate_text_stream()
    Client->>Client: Check access token
    alt No token
        Client->>IAM: Authenticate
        IAM-->>Client: Access token
    end
    Client->>API: POST /ml/v1/text/generation_stream
    API-->>Client: SSE stream
    loop Parse SSE chunks
        Client->>SSE: Read chunk
        SSE-->>Client: data: {JSON}
        Client->>Client: Parse JSON
        Client->>Client: Extract text
        Client->>User: Callback with chunk
    end
    Client-->>User: GenerationResult
```

### Batch Generation Flow

```mermaid
sequenceDiagram
    participant User
    participant Client
    participant Tokio
    participant API1
    participant API2
    participant API3
    
    User->>Client: generate_batch(requests, config)
    Client->>Client: Clone HTTP client & config
    loop For each request
        Client->>Tokio: tokio::spawn(async task)
        Tokio->>API1: Concurrent HTTP request 1
        Tokio->>API2: Concurrent HTTP request 2
        Tokio->>API3: Concurrent HTTP request 3
    end
    par Parallel execution
        API1-->>Tokio: Response 1
        API2-->>Tokio: Response 2
        API3-->>Tokio: Response 3
    end
    Tokio-->>Client: All tasks complete
    Client->>Client: Collect results
    Client->>Client: Create BatchGenerationResult
    Client-->>User: BatchGenerationResult
```

**Key Characteristics:**
- Each request spawns as separate `tokio::spawn` task
- True parallelism across multiple threads
- Shared HTTP client (reqwest::Client uses connection pooling)
- Per-item error handling (partial success supported)
- Results collected once all tasks complete

### Non-streaming Generation Flow

```mermaid
sequenceDiagram
    participant User
    participant Client
    participant IAM
    participant API
    
    User->>Client: generate_text()
    Client->>Client: Check access token
    alt No token
        Client->>IAM: Authenticate
        IAM-->>Client: Access token
    end
    Client->>API: POST /ml/v1/text/generation
    API-->>Client: Complete JSON response
    Client->>Client: Parse JSON
    Client->>Client: Extract generated text
    Client-->>User: GenerationResult
```

### Authentication Flow

```mermaid
sequenceDiagram
    participant Client
    participant IAM
    
    Client->>IAM: POST /identity/token
    Note over Client,IAM: grant_type=apikey
    Note over Client,IAM: apikey={key}
    IAM-->>Client: access_token
    Client->>Client: Store token
```

### Orchestrate Chat Flow (Non-streaming)

```mermaid
sequenceDiagram
    participant User
    participant Client
    participant API
    
    User->>Client: send_message(agent_id, message, thread_id?)
    Client->>API: POST /runs/stream with MessagePayload
    Note over Client,API: IAM-API_KEY header
    API-->>Client: SSE stream with events
    loop Parse SSE events
        Client->>Client: Parse message.created event
        Client->>Client: Extract response text and thread_id
    end
    Client-->>User: (response, new_thread_id)
```

### Orchestrate Chat Flow (Streaming)

```mermaid
sequenceDiagram
    participant User
    participant Client
    participant API
    
    User->>Client: stream_message(agent_id, message, thread_id?, callback)
    Client->>API: POST /runs/stream with MessagePayload
    Note over Client,API: IAM-API_KEY header
    API-->>Client: SSE stream with events
    loop Parse SSE events
        Client->>Client: Parse message.delta events
        Client->>Client: Extract incremental text chunks
        Client->>User: Callback with each chunk (real-time)
        Client->>Client: Track thread_id from events
    end
    Client-->>User: new_thread_id (for conversation continuity)
```

## SSE Parsing

The SDK implements proper Server-Sent Events parsing for streaming responses:

**Format:**
```
data: {"results":[{"generated_text":"chunk1"}]}

data: {"results":[{"generated_text":"chunk2"}]}

data: [DONE]
```

**Parsing Logic:**
1. Read stream chunks as bytes
2. Convert to UTF-8 strings
3. Buffer until complete lines
4. Parse `data:` prefixed lines
5. Extract JSON from data payload
6. Extract generated text from results
7. Call callback with each chunk
8. Accumulate total response

### Orchestrate SSE Event Parsing

The Orchestrate client parses Orchestrate-specific SSE events:

**Event Types:**
- `message.created` - Final complete message (for non-streaming)
- `message.delta` - Incremental text chunks (for streaming)

**Format:**
```
{"event":"message.created","data":{"message":{"content":[{"text":"Full response"}],"thread_id":"..."}}}
{"event":"message.delta","data":{"delta":{"content":[{"text":"chunk1"}]},"thread_id":"..."}}
{"event":"message.delta","data":{"delta":{"content":[{"text":"chunk2"}]},"thread_id":"..."}}
```

**Parsing Logic:**
1. Read SSE stream line by line
2. Parse JSON event data
3. For `message.created`: Extract full response text and thread_id
4. For `message.delta`: Extract incremental chunks and call callback
5. Maintain thread_id for conversation continuity

## Error Handling Strategy

### Error Propagation
- All async functions return `Result<T, Error>`
- Errors propagate up the call stack
- User can pattern match on error types

### Error Context
- Each error variant includes descriptive message
- Network errors include underlying cause
- API errors include HTTP status and response

### Error Recovery
- Authentication errors can be retried after reconnecting
- Network errors can be retried
- Timeout errors should use larger timeout
- Validation errors require user action

## Quality Assessment

Built-in quality scoring algorithm evaluates:

1. **Length Check** (30%): Non-empty and reasonable length (8-200 chars)
2. **Common Patterns** (20%): Contains common English words
3. **No Error Indicators** (20%): Doesn't contain error words
4. **Sentence Structure** (15%): Has proper sentences
5. **Word Count** (15%): Reasonable word count (3-100 words)

Returns score from 0.0 to 1.0

## Thread Safety

- Client is not thread-safe by design (async only)
- Configuration is thread-safe (immutable after creation)
- Error types are thread-safe
- Generation results are thread-safe

## Performance Considerations

### Streaming
- Uses async streams for real-time processing
- No buffering delays
- Callback fired immediately for each chunk

### Memory
- Streams processed incrementally
- Large responses don't require full memory allocation
- No unnecessary cloning

### Network
- Single connection for streaming
- Token cached in memory (not persisted)
- Connection timeout configurable

## Security

### Credentials
- Never hardcoded in examples
- Load from environment variables
- `.env` file excluded from version control

### API Communication
- HTTPS for all API calls
- Bearer token authentication
- Tokens not logged or exposed

### Dependencies
- All dependencies from crates.io
- Regular security updates
- No unsafe code blocks

## Testing Strategy

### Unit Tests
- Configuration validation
- Quality assessment logic
- Error handling

### Integration Tests
- Configuration from environment
- Client creation
- Model access
- Constant values

### Snapshot Tests
- Configuration snapshots
- Quality assessment results
- Model constants

## WatsonX AI SDK Enhancements (Latest)

### Batch Generation (Latest)
- ✅ Concurrent batch generation with `generate_batch()` and `generate_batch_simple()`
- ✅ True parallelism using `tokio::spawn` for each request
- ✅ Per-item error handling allowing partial success
- ✅ Flexible configuration (default or per-request)
- ✅ Result tracking with success/failure counts and duration
- ✅ Comprehensive batch types (`BatchRequest`, `BatchItemResult`, `BatchGenerationResult`)
- ✅ Color-coded example demonstrating parallel execution
- ✅ Extracted internal generation method for reusability
- ✅ Comprehensive unit tests

## Orchestrate SDK Enhancements

### Recent Improvements
- ✅ Flexible response parsing for API variations
- ✅ Graceful degradation for unavailable endpoints (404 handling)
- ✅ Optional Tool fields for compatibility with different API versions
- ✅ Comprehensive examples (basic, chat, advanced, use cases, chat with documents)
- ✅ Real-time streaming with proper SSE parsing
- ✅ Thread-based conversation context management
- ✅ Batch message processing support
- ✅ Tool execution and management (execute, update, delete, test)
- ✅ Tool versioning and execution history tracking
- ✅ Chat with documents (Q&A on uploaded documents)
- ✅ Modular code organization (config, client, types)
- ✅ Bearer token authentication with X-Instance-ID headers
- ✅ Multiple endpoint path fallbacks for robustness

### Robustness Features
- Multiple response format support (direct arrays, wrapped objects)
- Fallback parsing strategies for API variations
- Empty collection returns for unavailable endpoints (instead of errors)
- Consistent error handling across all endpoints
- Graceful degradation when optional features are unavailable
- Multiple endpoint path attempts for better compatibility
- Flexible document handling with fallback support

## Future Enhancements

### Potential Improvements
- Connection pooling for better performance
- Token caching with expiration
- Retry logic with exponential backoff
- Metrics and observability
- Enhanced document collection features (full CRUD operations)
- Session management abstraction (thread_id management)
- WebSocket support for Orchestrate (if available)
- Advanced streaming control options

### Architecture Considerations
- Keep client lightweight and simple
- Maintain async-first design
- Preserve type safety
- Keep error handling comprehensive
- Maintain streaming as primary interface for real-time features
- Follow established patterns (wxo-client-main, WatsonX API conventions)
- Keep configuration simple (environment-based, minimal fields)
- Prioritize graceful degradation over strict error handling


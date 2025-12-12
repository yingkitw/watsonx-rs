//! Code generation example
//!
//! This example demonstrates how to use WatsonX AI for code generation tasks
//! using both chat completion and text generation APIs.

use watsonx_rs::{WatsonxConnection, ChatMessage, ChatCompletionConfig, GenerationConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("=== WatsonX Code Generation Example ===\n");

    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
    println!("Connected successfully!\n");

    // Example 1: Code generation using chat completion
    println!("--- Example 1: Code Generation with Chat Completion ---");
    let chat_config = ChatCompletionConfig::default()
        .with_model(models::GRANITE_4_H_SMALL)
        .with_temperature(0.2) // Lower temperature for more deterministic code
        .with_max_tokens(1000);

    let code_messages = vec![
        ChatMessage::system("You are an expert Rust programmer. Generate clean, idiomatic Rust code."),
        ChatMessage::user("Write a function that calculates the factorial of a number using recursion."),
    ];

    let code_result = client.chat_completion(code_messages, &chat_config).await?;
    println!("Generated code:");
    println!("{}\n", code_result.content());

    // Example 2: Code explanation using chat completion
    println!("--- Example 2: Code Explanation ---");
    let explain_messages = vec![
        ChatMessage::system("You are a helpful programming tutor."),
        ChatMessage::user("Explain what this Rust code does:\n\n```rust\nfn fibonacci(n: u32) -> u32 {\n    match n {\n        0 => 0,\n        1 => 1,\n        _ => fibonacci(n - 1) + fibonacci(n - 2),\n    }\n}\n```"),
    ];

    let explain_result = client.chat_completion(explain_messages, &chat_config).await?;
    println!("Explanation:");
    println!("{}\n", explain_result.content());

    // Example 3: Code refactoring using chat completion
    println!("--- Example 3: Code Refactoring ---");
    let refactor_messages = vec![
        ChatMessage::system("You are a code review expert. Suggest improvements to make code more idiomatic and efficient."),
        ChatMessage::user("Refactor this Rust code to be more idiomatic:\n\n```rust\nfn sum(numbers: Vec<i32>) -> i32 {\n    let mut total = 0;\n    for num in numbers {\n        total += num;\n    }\n    total\n}\n```"),
    ];

    let refactor_result = client.chat_completion(refactor_messages, &chat_config).await?;
    println!("Refactored code:");
    println!("{}\n", refactor_result.content());

    // Example 4: Code generation using text generation API (alternative approach)
    println!("--- Example 4: Code Generation with Text Generation API ---");
    let gen_config = GenerationConfig::default()
        .with_model(models::GRANITE_4_H_SMALL)
        .with_max_tokens(500);

    let code_prompt = r#"Write a Rust function that implements a simple stack data structure with push, pop, and peek operations.

```rust
"#;

    println!("Generating code with text generation API...");
    let text_result = client.generate_text(code_prompt, &gen_config).await?;
    println!("Generated code:");
    println!("{}\n", text_result.text);

    // Example 5: Multi-step code generation (building on previous responses)
    println!("--- Example 5: Multi-step Code Generation ---");
    let mut step_messages = vec![
        ChatMessage::system("You are a helpful Rust programming assistant. Generate complete, working code."),
        ChatMessage::user("Create a struct called 'Person' with name and age fields."),
    ];

    let step1 = client.chat_completion(step_messages.clone(), &chat_config).await?;
    println!("Step 1 - User: Create a struct called 'Person' with name and age fields.");
    println!("Assistant: {}\n", step1.content());

    step_messages.push(ChatMessage::assistant(step1.content()));
    step_messages.push(ChatMessage::user("Now add a method to create a new Person instance."));

    let step2 = client.chat_completion(step_messages.clone(), &chat_config).await?;
    println!("Step 2 - User: Now add a method to create a new Person instance.");
    println!("Assistant: {}\n", step2.content());

    step_messages.push(ChatMessage::assistant(step2.content()));
    step_messages.push(ChatMessage::user("Finally, add a method to display the person's information."));

    let step3 = client.chat_completion(step_messages, &chat_config).await?;
    println!("Step 3 - User: Finally, add a method to display the person's information.");
    println!("Assistant: {}\n", step3.content());

    println!("=== Code Generation Examples Complete ===");

    Ok(())
}

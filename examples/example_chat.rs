//! Chat completion example
//!
//! This example demonstrates how to use the WatsonX AI chat completion API
//! for multi-turn conversations with system, user, and assistant messages.

use watsonx_rs::{WatsonxConnection, ChatMessage, ChatCompletionConfig, models::models};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    println!("=== WatsonX Chat Completion Example ===\n");

    // Connect to WatsonX
    println!("Connecting to WatsonX...");
    let client = WatsonxConnection::new().from_env().await?;
    println!("Connected successfully!\n");

    // Configure chat completion
    let chat_config = ChatCompletionConfig::default()
        .with_model(models::GRANITE_4_H_SMALL)
        .with_temperature(0.7)
        .with_max_tokens(500);

    // Example 1: Simple question-answer
    println!("--- Example 1: Simple Q&A ---");
    let messages = vec![
        ChatMessage::system("You are a helpful assistant that explains concepts clearly."),
        ChatMessage::user("What is async/await in Rust?"),
    ];

    let result = client.chat_completion(messages, &chat_config).await?;
    println!("User: What is async/await in Rust?");
    println!("Assistant: {}\n", result.content());

    // Example 2: Multi-turn conversation
    println!("--- Example 2: Multi-turn Conversation ---");
    let mut conversation = vec![
        ChatMessage::system("You are a helpful programming assistant."),
        ChatMessage::user("How do I create a vector in Rust?"),
    ];

    let response1 = client.chat_completion(conversation.clone(), &chat_config).await?;
    println!("User: How do I create a vector in Rust?");
    println!("Assistant: {}\n", response1.content());

    // Continue the conversation
    conversation.push(ChatMessage::assistant(response1.content()));
    conversation.push(ChatMessage::user("Can you show me an example with numbers?"));

    let response2 = client.chat_completion(conversation, &chat_config).await?;
    println!("User: Can you show me an example with numbers?");
    println!("Assistant: {}\n", response2.content());

    // Example 3: Streaming chat completion
    println!("--- Example 3: Streaming Chat Completion ---");
    let streaming_messages = vec![
        ChatMessage::system("You are a creative writing assistant."),
        ChatMessage::user("Write a short story about a robot learning to paint."),
    ];

    println!("User: Write a short story about a robot learning to paint.");
    print!("Assistant: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let streaming_result = client.chat_completion_stream(
        streaming_messages,
        &chat_config,
        |chunk| {
            print!("{}", chunk);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        },
    ).await?;

    println!("\n");
    if let Some(tokens) = streaming_result.total_tokens {
        println!("Total tokens used: {}", tokens);
    }

    Ok(())
}

//! Server-Sent Events (SSE) parsing utilities
//!
//! This module provides reusable functions for parsing SSE streams from WatsonX API responses.

use crate::error::{Error, Result};
use futures::StreamExt;
use reqwest::Response;
use serde_json::Value;

/// Parse SSE stream and extract text content
///
/// This function processes a streaming HTTP response and extracts text content
/// from SSE data events. It handles both complete and partial JSON data.
///
/// # Arguments
///
/// * `response` - HTTP response with SSE stream
/// * `callback` - Optional callback function called for each text chunk
///
/// # Returns
///
/// Complete accumulated text from all SSE data events
pub async fn parse_sse_stream<F>(
    response: Response,
    mut callback: Option<F>,
) -> Result<String>
where
    F: FnMut(&str),
{
    let mut answer = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    // Process stream chunks in real-time
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            Error::Network(format!(
                "Failed to read SSE stream chunk: {}. Check your network connection.",
                e
            ))
        })?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete lines from buffer
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if let Some(text_chunk) = parse_sse_line(&line)? {
                answer.push_str(&text_chunk);
                if let Some(ref mut cb) = callback {
                    cb(&text_chunk);
                }
            }
        }
    }

    // Process any remaining data in buffer
    if !buffer.is_empty() {
        if let Some(text_chunk) = parse_sse_line(&buffer)? {
            answer.push_str(&text_chunk);
            if let Some(ref mut cb) = callback {
                cb(&text_chunk);
            }
        }
    }

    Ok(answer)
}

/// Parse a single SSE line and extract text content if it's a data event
///
/// Returns `None` for non-data lines or empty data, `Some(text)` for valid data events.
pub(crate) fn parse_sse_line(line: &str) -> Result<Option<String>> {
    let trimmed = line.trim();

    // Skip empty lines, id lines, and event type lines
    if trimmed.is_empty()
        || trimmed.starts_with("id:")
        || trimmed.starts_with("event:")
    {
        return Ok(None);
    }

    // Process data lines
    if trimmed.starts_with("data:") {
        let json_data = if trimmed.starts_with("data: ") {
            &trimmed[6..]
        } else {
            &trimmed[5..]
        };

        // Skip empty data or done markers
        let trimmed_data = json_data.trim();
        if trimmed_data.is_empty() || trimmed_data == "[DONE]" {
            return Ok(None);
        }

        // Try to parse as JSON and extract text
        match serde_json::from_str::<Value>(trimmed_data) {
            Ok(data) => extract_text_from_json(&data),
            Err(e) => {
                // Log warning but don't fail - some SSE chunks may be malformed
                eprintln!("Warning: Failed to parse SSE data line: {}. Skipping.", e);
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

/// Extract text content from JSON data structure
///
/// Handles different response formats:
/// - `{results: [{generated_text: "..."}]}` - Text generation format
/// - `{choices: [{delta: {content: "..."}}]}` - Chat completion delta format
/// - `{choices: [{message: {content: "..."}}]}` - Chat completion message format
pub(crate) fn extract_text_from_json(data: &Value) -> Result<Option<String>> {
    // Try text generation format: {results: [{generated_text: "..."}]}
    if let Some(results) = data.get("results").and_then(|r| r.as_array()) {
        if let Some(result) = results.first() {
            if let Some(text) = result.get("generated_text").and_then(|t| t.as_str()) {
                return Ok(Some(text.to_string()));
            }
        }
    }

    // Try chat completion format: {choices: [{delta: {content: "..."}}]}
    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
        if let Some(choice) = choices.first() {
            // Try delta format first (streaming)
            if let Some(delta) = choice.get("delta") {
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                    return Ok(Some(content.to_string()));
                }
            }
            // Try message format (non-streaming)
            if let Some(message) = choice.get("message") {
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    return Ok(Some(content.to_string()));
                }
            }
        }
    }

    Ok(None)
}

/// Parse SSE stream for chat completion format
///
/// Similar to `parse_sse_stream` but specifically handles chat completion
/// response format with choices array.
pub async fn parse_chat_completion_sse<F>(
    response: Response,
    mut callback: F,
) -> Result<String>
where
    F: FnMut(&str),
{
    let mut answer = String::new();
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            Error::Network(format!(
                "Failed to read chat completion SSE stream: {}. Check your network connection.",
                e
            ))
        })?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("id:") || trimmed.starts_with("event:") {
                continue;
            }

            if trimmed.starts_with("data:") {
                let json_data = if trimmed.starts_with("data: ") {
                    &trimmed[6..]
                } else {
                    &trimmed[5..]
                };

                if json_data.trim().is_empty() || json_data.trim() == "[DONE]" {
                    continue;
                }

                match serde_json::from_str::<Value>(json_data) {
                    Ok(data) => {
                        if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
                            if let Some(choice) = choices.first() {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) =
                                        delta.get("content").and_then(|c| c.as_str())
                                    {
                                        answer.push_str(content);
                                        callback(content);
                                    }
                                } else if let Some(message) = choice.get("message") {
                                    if let Some(content) =
                                        message.get("content").and_then(|c| c.as_str())
                                    {
                                        answer.push_str(content);
                                        callback(content);
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Ignore parse errors for individual chunks
                        continue;
                    }
                }
            }
        }
    }

    // Process remaining buffer
    if !buffer.is_empty() {
        let trimmed = buffer.trim();
        if trimmed.starts_with("data:") {
            let json_data = if trimmed.starts_with("data: ") {
                &trimmed[6..]
            } else {
                &trimmed[5..]
            };

            if !json_data.trim().is_empty() && json_data.trim() != "[DONE]" {
                if let Ok(data) = serde_json::from_str::<Value>(json_data) {
                    if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
                        if let Some(choice) = choices.first() {
                            if let Some(delta) = choice.get("delta") {
                                if let Some(content) =
                                    delta.get("content").and_then(|c| c.as_str())
                                {
                                    answer.push_str(content);
                                    callback(content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_sse_line_empty() {
        assert!(parse_sse_line("").unwrap().is_none());
        assert!(parse_sse_line("   ").unwrap().is_none());
    }

    #[test]
    fn test_parse_sse_line_non_data() {
        assert!(parse_sse_line("id: 123").unwrap().is_none());
        assert!(parse_sse_line("event: message").unwrap().is_none());
        assert!(parse_sse_line(": comment").unwrap().is_none());
    }

    #[test]
    fn test_parse_sse_line_done() {
        assert!(parse_sse_line("data: [DONE]").unwrap().is_none());
        assert!(parse_sse_line("data:  [DONE]").unwrap().is_none());
    }

    #[test]
    fn test_parse_sse_line_text_generation_format() {
        let json = json!({
            "results": [{
                "generated_text": "Hello, world!"
            }]
        });
        let line = format!("data: {}", json);
        let result = parse_sse_line(&line).unwrap();
        assert_eq!(result, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_parse_sse_line_chat_completion_delta() {
        let json = json!({
            "choices": [{
                "delta": {
                    "content": "Hello"
                }
            }]
        });
        let line = format!("data: {}", json);
        let result = parse_sse_line(&line).unwrap();
        assert_eq!(result, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_line_chat_completion_message() {
        let json = json!({
            "choices": [{
                "message": {
                    "content": "Complete response"
                }
            }]
        });
        let line = format!("data: {}", json);
        let result = parse_sse_line(&line).unwrap();
        assert_eq!(result, Some("Complete response".to_string()));
    }

    #[test]
    fn test_extract_text_from_json_text_generation() {
        let data = json!({
            "results": [{
                "generated_text": "Generated text"
            }]
        });
        let result = extract_text_from_json(&data).unwrap();
        assert_eq!(result, Some("Generated text".to_string()));
    }

    #[test]
    fn test_extract_text_from_json_chat_delta() {
        let data = json!({
            "choices": [{
                "delta": {
                    "content": "Delta content"
                }
            }]
        });
        let result = extract_text_from_json(&data).unwrap();
        assert_eq!(result, Some("Delta content".to_string()));
    }

    #[test]
    fn test_extract_text_from_json_chat_message() {
        let data = json!({
            "choices": [{
                "message": {
                    "content": "Message content"
                }
            }]
        });
        let result = extract_text_from_json(&data).unwrap();
        assert_eq!(result, Some("Message content".to_string()));
    }

    #[test]
    fn test_extract_text_from_json_no_match() {
        let data = json!({
            "other": "data"
        });
        let result = extract_text_from_json(&data).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_sse_line_malformed_json() {
        let line = "data: {invalid json}";
        // Should not panic, just return None or log warning
        let result = parse_sse_line(line);
        // Result should be Ok(None) or handle gracefully
        assert!(result.is_ok());
    }
}

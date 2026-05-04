use ai_agent::agents::stream_handler::{DynStream, StreamHandler};
use futures::stream;
use rig::agent::MultiTurnStreamItem;
use rig::completion::{CompletionModel, CompletionRequest, GetTokenUsage, Usage};
use rig::message::{Message, Text, ToolCall, ToolFunction, ToolResult, ToolResultContent, Reasoning};
use rig::streaming::{StreamedAssistantContent, StreamedUserContent, ToolCallDeltaContent};
use rig::completion::CompletionError;
use rig::streaming::StreamingCompletionResponse;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct MockModel;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MockStreamingResponse {
    usage: Usage,
}

impl GetTokenUsage for MockStreamingResponse {
    fn token_usage(&self) -> Option<Usage> {
        Some(self.usage)
    }
}

impl CompletionModel for MockModel {
    type Response = serde_json::Value;
    type StreamingResponse = MockStreamingResponse;
    type Client = ();

    fn make(_client: &Self::Client, _model: impl Into<String>) -> Self {
        Self
    }

    async fn completion(
        &self,
        _request: CompletionRequest,
    ) -> Result<rig::completion::CompletionResponse<Self::Response>, CompletionError> {
        Err(CompletionError::ProviderError(
            "mock model".to_string(),
        ))
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError> {
        Err(CompletionError::ProviderError(
            "mock model".to_string(),
        ))
    }
}

fn make_stream(
    items: Vec<Result<MultiTurnStreamItem<MockStreamingResponse>, rig::agent::StreamingError>>,
) -> DynStream<MockModel> {
    Box::pin(stream::iter(items))
}

fn make_text_content(text: &str) -> MultiTurnStreamItem<MockStreamingResponse> {
    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(Text {
        text: text.to_string(),
    }))
}

fn make_reasoning_content(text: &str) -> MultiTurnStreamItem<MockStreamingResponse> {
    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(
        Reasoning::new(text).with_id("r1".to_string())
    ))
}

fn make_tool_call_content(tool_name: &str, args: serde_json::Value) -> MultiTurnStreamItem<MockStreamingResponse> {
    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall {
        tool_call: ToolCall {
            id: "tc_1".to_string(),
            call_id: Some("call_1".to_string()),
            function: ToolFunction {
                name: tool_name.to_string(),
                arguments: args,
            },
            signature: None,
            additional_params: None,
        },
        internal_call_id: "internal_1".to_string(),
    })
}

fn make_tool_result_content(result_text: &str) -> MultiTurnStreamItem<MockStreamingResponse> {
    MultiTurnStreamItem::StreamUserItem(StreamedUserContent::ToolResult {
        tool_result: ToolResult {
            id: "terminal".to_string(),
            call_id: Some("call_1".to_string()),
            content: OneOrMany::one(ToolResultContent::Text(Text {
                text: result_text.to_string(),
            })),
        },
        internal_call_id: "internal_1".to_string(),
    })
}

fn make_final_response() -> MultiTurnStreamItem<MockStreamingResponse> {
    MultiTurnStreamItem::FinalResponse(rig::agent::FinalResponse::empty())
}

use rig::OneOrMany;

// ============================================================
// TEST: Normal stream with text, reasoning, tool call, tool result
// ============================================================
#[tokio::test]
async fn test_normal_stream_with_text_and_reasoning() {
    let items = vec![
        Ok(make_text_content("Let me think")),
        Ok(make_reasoning_content("I need to analyze this")),
        Ok(make_text_content(" about that.")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Text chunks without tool calls between them get buffered together
    // and flushed as ONE message on FinalResponse. Reasoning does not produce a Message.
    assert_eq!(messages.len(), 1);

    // The single message should contain all text concatenated
    let text = extract_text_from_message(&messages[0]).unwrap();
    assert_eq!(text, "Let me think about that.");
}

// ============================================================
// TEST: Stream with tool call and tool result
// ============================================================
#[tokio::test]
async fn test_stream_with_tool_call_and_result() {
    let items = vec![
        Ok(make_text_content("I'll run that")),
        Ok(make_tool_call_content("terminal", serde_json::json!({"command": "echo hello"}))),
        Ok(make_tool_result_content("hello\n")),
        Ok(make_text_content("Done!")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Should have: assistant text, assistant tool call, user tool result, assistant text
    assert_eq!(messages.len(), 4);

    // First: "I'll run that"
    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "I'll run that");

    // Second: ToolCall message
    assert!(is_tool_call_message(&messages[1]));

    // Third: Tool result message
    let tool_result_text = extract_text_from_message(&messages[2]).unwrap();
    assert!(tool_result_text.contains("hello"));

    // Fourth: "Done!"
    assert_eq!(extract_text_from_message(&messages[3]).unwrap(), "Done!");
}

// ============================================================
// TEST: User interruption - tool execution blocked
// ============================================================
#[tokio::test]
async fn test_stream_with_user_interruption() {
    let items = vec![
        Ok(make_text_content("I'll run that")),
        Ok(make_tool_call_content("terminal", serde_json::json!({"command": "rm -rf /"}))),
        Ok(make_tool_result_content("Tool execution blocked by user")),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Should have: assistant text, tool call, interruption message
    // Then it should break, no more messages
    assert_eq!(messages.len(), 3);

    // First: "I'll run that"
    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "I'll run that");

    // Second: ToolCall message
    assert!(is_tool_call_message(&messages[1]));

    // Third: Tool result with blocked message
    let result_text = extract_text_from_message(&messages[2]).unwrap();
    assert!(result_text.contains("Tool execution blocked by user"));
}

// ============================================================
// TEST: Stream with only text (no tools)
// ============================================================
#[tokio::test]
async fn test_text_only_stream() {
    let items = vec![
        Ok(make_text_content("Hello")),
        Ok(make_text_content(" ")),
        Ok(make_text_content("world!")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // All text chunks get merged into one assistant message per block
    // But since there's no tool call between them, they should be in one message
    // Actually, each text chunk is buffered, then flushed on final response
    // Wait - the handler buffers text and flushes on tool call or final response
    // Since there are no tool calls, all text should be in ONE message
    assert_eq!(messages.len(), 1);
    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "Hello world!");
}

// ============================================================
// TEST: Multiple tool calls in sequence
// ============================================================
#[tokio::test]
async fn test_stream_with_multiple_tool_calls() {
    let items = vec![
        Ok(make_text_content("First")),
        Ok(make_tool_call_content("terminal", serde_json::json!({"command": "echo 1"}))),
        Ok(make_tool_result_content("1\n")),
        Ok(make_text_content("Second")),
        Ok(make_tool_call_content("terminal", serde_json::json!({"command": "echo 2"}))),
        Ok(make_tool_result_content("2\n")),
        Ok(make_text_content("All done")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Expected: text1, tool_call1, result1, text2, tool_call2, result2, text3
    assert_eq!(messages.len(), 7);

    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "First");
    assert!(is_tool_call_message(&messages[1]));
    assert!(extract_text_from_message(&messages[2]).unwrap().contains("1"));
    assert_eq!(extract_text_from_message(&messages[3]).unwrap(), "Second");
    assert!(is_tool_call_message(&messages[4]));
    assert!(extract_text_from_message(&messages[5]).unwrap().contains("2"));
    assert_eq!(extract_text_from_message(&messages[6]).unwrap(), "All done");
}

// ============================================================
// TEST: Stream with tool call delta
// ============================================================
#[tokio::test]
async fn test_stream_with_tool_delta() {
    let items = vec![
        Ok(make_text_content("Using tool")),
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::ToolCallDelta {
                id: "tc_1".to_string(),
                internal_call_id: "internal_1".to_string(),
                content: ToolCallDeltaContent::Name("terminal".to_string()),
            },
        )),
        Ok(make_tool_call_content("terminal", serde_json::json!({"command": "echo ok"}))),
        Ok(make_tool_result_content("ok\n")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Should have: text, tool call, result (delta is just logged, not a message)
    assert_eq!(messages.len(), 3);
    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "Using tool");
    assert!(is_tool_call_message(&messages[1]));
}

// ============================================================
// TEST: Empty stream
// ============================================================
#[tokio::test]
async fn test_empty_stream() {
    let items: Vec<Result<MultiTurnStreamItem<MockStreamingResponse>, rig::agent::StreamingError>> = vec![];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    assert_eq!(messages.len(), 0);
}

// ============================================================
// TEST: Stream with reasoning delta
// ============================================================
#[tokio::test]
async fn test_stream_with_reasoning_delta() {
    let items = vec![
        Ok(MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::ReasoningDelta {
                id: Some("r1".to_string()),
                reasoning: "thinking step by step".to_string(),
            },
        )),
        Ok(make_text_content("Answer")),
        Ok(make_final_response()),
    ];

    let mut stream = make_stream(items);
    let messages = StreamHandler::handle_stream::<MockModel>(&mut stream).await;

    // Reasoning delta is just logged, only text message should exist
    assert_eq!(messages.len(), 1);
    assert_eq!(extract_text_from_message(&messages[0]).unwrap(), "Answer");
}

// ============================================================
// Helpers
// ============================================================
fn extract_text_from_message(msg: &Message) -> Option<String> {
    match msg {
        Message::Assistant { content, .. } => {
            let items: Vec<_> = content.iter().collect();
            items.iter().find_map(|item| {
                if let rig::message::AssistantContent::Text(t) = item {
                    Some(t.text.clone())
                } else {
                    None
                }
            })
        }
        Message::User { content, .. } => {
            let items: Vec<_> = content.iter().collect();
            items.iter().find_map(|item| {
                match item {
                    rig::message::UserContent::Text(t) => Some(t.text.clone()),
                    rig::message::UserContent::ToolResult(tr) => {
                        tr.content.iter().find_map(|c| {
                            if let ToolResultContent::Text(t) = c {
                                Some(t.text.clone())
                            } else {
                                None
                            }
                        })
                    }
                    _ => None,
                }
            })
        }
        _ => None,
    }
}

fn is_tool_call_message(msg: &Message) -> bool {
    match msg {
        Message::Assistant { content, .. } => {
            content.iter().any(|item| {
                matches!(item, rig::message::AssistantContent::ToolCall(_))
            })
        }
        _ => false,
    }
}

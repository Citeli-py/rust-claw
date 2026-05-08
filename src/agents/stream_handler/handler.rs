use std::io;
use std::io::Write;
use futures::StreamExt;
use rig::OneOrMany;
use rig::message::{AssistantContent, Message, ToolCall};
use rig::streaming::{StreamedUserContent, StreamedAssistantContent};
use rig::completion::{CompletionModel, GetTokenUsage};

use super::types::{DynStream, UserInterruptionError};

pub struct StreamHandler {}

impl StreamHandler {

    pub async fn handle_stream<M>(stream: &mut DynStream<M>) -> Vec<Message>
    where
        M: CompletionModel + 'static,
        M::StreamingResponse: GetTokenUsage,
    {
        let mut messages = Vec::new();
        let mut current_text = String::new();
        let mut reasoning_text = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();

            match chunk {
                rig::agent::MultiTurnStreamItem::StreamAssistantItem(item) => {
                    Self::handle_assistant_item(item, &mut current_text, &mut reasoning_text, &mut messages);
                }

                rig::agent::MultiTurnStreamItem::StreamUserItem(user_item) => {
                    Self::flush_reasoning(&mut reasoning_text, &mut messages);
                    match Self::handle_user_item(user_item, &mut current_text, &mut messages) {
                        Ok(()) => {}
                        Err(e) => {
                            messages.push(e.message);
                            break;
                        }
                    }
                }

                rig::agent::MultiTurnStreamItem::FinalResponse(final_response) => {
                    Self::flush_reasoning(&mut reasoning_text, &mut messages);
                    Self::flush_text(&mut current_text, &mut messages);
                    Self::handle_final_response(&final_response);
                }

                _ => {}
            }
        }

        messages
    }

    fn handle_assistant_item<R>(
        item: StreamedAssistantContent<R>,
        current_text: &mut String,
        reasoning_text: &mut String,
        messages: &mut Vec<Message>,
    ) {
        match item {
            StreamedAssistantContent::Text(text) => {
                Self::flush_reasoning(reasoning_text, messages);
                Self::handle_text(&text.text, current_text);
            }

            StreamedAssistantContent::ToolCall { tool_call, internal_call_id } => {
                Self::flush_reasoning(reasoning_text, messages);
                Self::handle_tool_call(tool_call, internal_call_id, current_text, messages);
            }

            StreamedAssistantContent::ToolCallDelta { content, .. } => {
                Self::handle_tool_delta(content);
            }

            StreamedAssistantContent::Reasoning(reason) => {
                Self::flush_reasoning(reasoning_text, messages);
                Self::handle_reasoning(reason);
            }

            StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                Self::handle_reasoning_delta(reasoning, reasoning_text);
            }

            StreamedAssistantContent::Final(_) => {
                Self::flush_reasoning(reasoning_text, messages);
            }
        }
    }

    fn handle_text(text: &str, buffer: &mut String) {
        print!("{}", text);
        io::stdout().flush().unwrap();
        buffer.push_str(text);
    }

    fn handle_tool_call(
        tool_call: ToolCall,
        internal_call_id: String,
        current_text: &mut String,
        messages: &mut Vec<Message>,
    ) {
        Self::flush_text(current_text, messages);

        println!("\n[TOOL CALL]\nTool: {}\ncommand: {:#?}", tool_call.function.name, tool_call.function.arguments);
        messages.push(Message::Assistant {
            id: Some(internal_call_id),
            content: OneOrMany::one(
                AssistantContent::ToolCall(tool_call)
            ),
        });
    }

    fn handle_tool_delta(content: impl std::fmt::Debug) {
        println!("\n[Tool delta / possível resultado]");
        println!("{:?}", content);
    }

    fn handle_reasoning(reason: impl std::fmt::Debug) {
        println!("\n[Reasoning]");
        println!("{:?}", reason);
    }

    fn handle_reasoning_delta(reasoning: String, reasoning_text: &mut String) {
        if reasoning_text.is_empty() {
            println!("[THINKING]");
        }
        print!("{}", reasoning);
        io::stdout().flush().unwrap();
        reasoning_text.push_str(&reasoning);
    }

    fn handle_user_item(
        user_content: StreamedUserContent,
        current_text: &mut String,
        messages: &mut Vec<Message>,
    ) -> Result<(), UserInterruptionError> {
        let message = Self::parse_user_content(user_content)?;

        Self::flush_text(current_text, messages);
        messages.push(message);

        Ok(())
    }

    fn parse_user_content(user_content: StreamedUserContent) -> Result<Message, UserInterruptionError> {
        match user_content {
            StreamedUserContent::ToolResult { tool_result, internal_call_id: _ } => {
                let full_text = Self::extract_tool_result_text(&tool_result);
                
                println!("\n\n[Tool result]\n{}", full_text);
                io::stdout().flush().unwrap();

                let message = Message::tool_result_with_call_id(
                    tool_result.id,
                    tool_result.call_id,
                    full_text.clone()
                );

                if full_text.eq("Toolset error: ToolCallError: ToolCallError: [BLOCKED] Tool execution blocked by user") {
                    return Err(UserInterruptionError { message });
                }

                Ok(message)
            }
        }
    }

    fn extract_tool_result_text(tool_result: &rig::message::ToolResult) -> String {
        let text: String = tool_result.content.iter()
            .filter_map(|item| {
                if let rig::message::ToolResultContent::Text(t) = item {
                    Some(t.text.clone())
                } else {
                    None
                }
            })
            .collect();

        if text.is_empty() {
            format!("{:?}", tool_result.content)
        } else {
            text
        }
    }

    fn flush_text(buffer: &mut String, messages: &mut Vec<Message>) {
        if !buffer.is_empty() {
            messages.push(Message::assistant(buffer.clone()));
            buffer.clear();
        }
    }

    fn flush_reasoning(reasoning_text: &mut String, messages: &mut Vec<Message>) {
        if !reasoning_text.is_empty() {
            println!("\n[END THINKING]");
            io::stdout().flush().unwrap();
            messages.push(Message::assistant(reasoning_text.clone()));
            reasoning_text.clear();
        }
    }

    fn handle_final_response(final_response: &rig::agent::FinalResponse) {
        let usage = final_response.usage();

        println!("\n[Token usage -> input: {}, output: {}, total: {}]\n", usage.input_tokens, usage.output_tokens, usage.total_tokens);
    }
}

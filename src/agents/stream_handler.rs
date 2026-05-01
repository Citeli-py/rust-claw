use std::io;
use std::io::Write;
use futures::{Stream, StreamExt};

use rig::streaming::{ StreamedUserContent, StreamedAssistantContent};
use rig::completion::{CompletionModel, GetTokenUsage};

type DynStream<M> = std::pin::Pin<
    Box<
        dyn Stream<
                Item = std::result::Result<
                    rig::agent::MultiTurnStreamItem<
                        <M as CompletionModel>::StreamingResponse
                    >,
                    rig::agent::StreamingError
                >
            > + Send
    >
>;

#[derive(Debug)]
pub struct UserInterruptionError {
    text: String
}

impl std::fmt::Display for UserInterruptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation interrupted by user")
    }
}

impl std::error::Error for UserInterruptionError {}

pub struct StreamHandler {}


impl StreamHandler {

    pub async fn handle_stream<M>(stream: &mut DynStream<M>) -> String 
    where 
        M: CompletionModel + 'static,
        M::StreamingResponse: GetTokenUsage,

    {

        let mut output = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk_result: std::result::Result<String, UserInterruptionError> = StreamHandler::handle_chunk(chunk.unwrap());
            let is_interrupted = chunk_result.is_err();
            let text = match chunk_result {
                Ok(s) => s,
                Err(e) => e.text
            };

            output.push_str(&text);

            if is_interrupted {
                break;
            }
        }

        return output;
    }

    fn handle_chunk<R>(
        chunk: rig::agent::MultiTurnStreamItem<R>,
    ) -> anyhow::Result<String, UserInterruptionError> {

        let mut response_text = String::new();

        match chunk {

            // 🧠 Tudo vem como StreamAssistantItem
            rig::agent::MultiTurnStreamItem::StreamAssistantItem(assistant_content) => {
                let assistant_content_message = StreamHandler::handle_assistant_item(assistant_content);
                response_text.push_str(&assistant_content_message);
                
            }

            rig::agent::MultiTurnStreamItem::StreamUserItem(user_content) => {
                let user_content_message = StreamHandler::handle_user_content(user_content);

                let is_error = user_content_message.is_err();

                let text = match user_content_message {
                    Ok(s) => s,
                    Err(e) => e.text
                };

                println!("{}",text);
                response_text.push_str(&text);

                if is_error {
                    return Err(UserInterruptionError { text: response_text});
                }

            }

            rig::agent::MultiTurnStreamItem::FinalResponse(content) => {
                let usage = content.usage();
                println!("\n\n[input_tokens: {}, output_tokens: {}]", usage.input_tokens, usage.output_tokens);
            }

            _ => {}
        }

        Ok(response_text)
    }


    fn handle_user_content(user_content: StreamedUserContent) -> Result<String, UserInterruptionError>{

        let mut response_text = String::new();

        match user_content {
            StreamedUserContent::ToolResult { tool_result, internal_call_id,} => {

                let first = tool_result.content.first();
                // junta first + rest
                let mut items = vec![&first];
                
                let rest = tool_result.content.rest();
                items.extend(rest.iter());
                
                let mut full_text = String::new();
                for item in items {
                    match item {
                        rig::message::ToolResultContent::Text(t) => {
                            full_text.push_str(&t.text);
                        }
                        _ => {}
                    }
                }

                response_text.push_str("\n[TOOL RAW OUTPUT]\n");
                response_text.push_str(&full_text);
                response_text.push('\n');
            }
        }

        let blocked = response_text.contains("Tool execution blocked by user");

        if blocked {
            return Err(UserInterruptionError {text: response_text});
        }

        return Ok(response_text);
    }

    fn handle_assistant_item<R>(assistant_item: StreamedAssistantContent<R>) -> String {
        let mut response_text = String::new();
        match assistant_item {

            // 🔹 Texto normal
            StreamedAssistantContent::Text(text) => {
                print!("{text}");
                io::stdout().flush().unwrap();
                response_text.push_str(&text.text);
            }

            // 🔹 Tool sendo chamada
            StreamedAssistantContent::ToolCall { tool_call, .. } => {

                response_text.push_str("\n[Calling tool]\n");
                response_text.push_str(&format!("ID: {}\n", tool_call.function.name));
                response_text.push_str(&format!("Args: {:?}\n", tool_call.function.arguments));

                print!("{}", response_text);
            }

            // 🔹 DELTA da tool (isso aqui é importante!)
            StreamedAssistantContent::ToolCallDelta { content, .. } => {
                println!("\n[Tool delta / possível resultado]");
                println!("{:?}", content);
            }

            // 🔹 Reasoning
            StreamedAssistantContent::Reasoning(reason) => {
                println!("\n[Reasoning]");
                println!("{:?}", reason.display_text());
            }

            StreamedAssistantContent::ReasoningDelta { id, reasoning } => println!("ReasoningDelta event\n{}\nEnd reasoning", reasoning),

            StreamedAssistantContent::Final(_) => {}
        }

        return response_text;

    }
}
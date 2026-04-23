use anyhow::{Result};
use async_trait::async_trait;
use rig::{agent::{Agent, AgentBuilder, PromptHook}, completion::{Chat, CompletionModel}, message::{Message}};
use rig::streaming::StreamingChat;
use rig::streaming::StreamedAssistantContent;
use futures::StreamExt;
use std::io;
use std::io::Write;
use crate::agents::AgentInterface;

use std::result::Result::Ok;
use crate::tools::{TerminalTool, WebBrowserTool};

pub(super) struct AgentWrapper<M, P>
where 
    M: CompletionModel,
    P: PromptHook<M>,
{
    agent: Agent<M, P>,
    history: Vec<Message>
}


impl<M, P> AgentWrapper<M, P>
where 
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{
    pub(super) fn new(agent: Agent<M, P>, history: Vec<Message>) -> Self {
        Self { agent, history }
    }
}


#[async_trait]
impl <M, P> AgentInterface for AgentWrapper<M, P>
where 
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{
    
    async fn chat(&mut self, input: &str) -> Result<String> {
        let output = self.agent.chat(input, self.history.clone()).await?;
        self.history.push(Message::user(input));
        self.history.push(Message::assistant(output.clone()));
        Ok(output)
    }

    fn history(&self,) -> &[Message] {
        self.history.as_ref()
    }

    fn clean_history(&mut self) {
        self.history.clear();
    }

    async fn stream(&mut self, input: &str) -> Result<()> {
        let mut stream = self.agent.stream_chat(input, self.history.clone()).await;
        let mut output: String = String::new();
        while let Some(chunk) = stream.next().await {
            output.push_str(&handle_chunk(chunk?)?);
        }

        println!();
        self.history.push(Message::user(input));
        self.history.push(Message::assistant(output.clone()));

        Ok(())
    }

}

pub(super) async fn build_agent<M, P>(builder: AgentBuilder<M, P>, pre_prompt: &str, history: Vec<Message>) -> Box<dyn AgentInterface> 
where 
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let builder = builder
    .preamble(pre_prompt)
    .default_max_turns(20);

    let builder = builder.tool(TerminalTool);

    let builder = if let Ok(web_tool) = WebBrowserTool::new().await {
        builder.tool(web_tool)
    } else {
        println!("⚠️ Não foi possível iniciar o PinchTab, seguindo sem browser tool");
        builder
    };

    let agent = builder.build();
    Box::new(AgentWrapper::new(agent, history))
}

use rig::streaming::{ StreamedUserContent};

fn handle_chunk<R>(
    chunk: rig::agent::MultiTurnStreamItem<R>,
) -> anyhow::Result<String> {

    let mut response_text = String::new();

    match chunk {

        // 🧠 Tudo vem como StreamAssistantItem
        rig::agent::MultiTurnStreamItem::StreamAssistantItem(assistant_content) => {
            let assistant_content_message = handle_assistant_item(assistant_content);
            response_text.push_str(&assistant_content_message);
            
        }

        rig::agent::MultiTurnStreamItem::StreamUserItem(user_content) => {
            let user_content_message = handle_user_content(user_content);
            println!("{}", user_content_message);
            response_text.push_str(&user_content_message);
        }

        rig::agent::MultiTurnStreamItem::FinalResponse(content) => {
            let usage = content.usage();
            println!("\n\n[input_tokens: {}, output_tokens: {}]", usage.input_tokens, usage.output_tokens);
        }

        _ => {}
    }

    Ok(response_text)
}


fn handle_user_content(user_content: StreamedUserContent) -> String{

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

    return response_text
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
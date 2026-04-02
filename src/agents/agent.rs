use anyhow::{Result};
use async_trait::async_trait;
use rig::{agent::{Agent, AgentBuilder, PromptHook, stream_to_stdout}, client::builder, completion::{Chat, CompletionModel, GetTokenUsage}, message::{Message, ToolResult}};
use rig::streaming::StreamingChat;
use rig::streaming::StreamedAssistantContent;
use futures::StreamExt;
use std::io;
use std::io::Write;

use std::result::Result::Ok;

use crate::{agents::agent, tools::PinchTab};
use crate::tools::{TerminalTool, WebBrowserTool};

use crate::agents::PRE_PROMPT;


#[async_trait]
pub trait AgentInterface: Send {

    async fn chat(&mut self, input: &str) -> Result<String>;

    //retorna a stream pelo stdout
    async fn stream(&mut self, input: &str) -> Result<()>;

    fn history(&self,) -> &[Message];

    fn clean_history(&mut self);
}


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

pub(super) async fn build_agent<M, P>(builder: AgentBuilder<M, P>, history: Vec<Message>) -> Box<dyn AgentInterface> 
where 
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let builder = builder
    .preamble(PRE_PROMPT)
    .default_max_turns(10);

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


use rig::agent::MultiTurnStreamItem;
use rig::streaming::{ StreamedUserContent};

fn handle_chunk<R>(
    chunk: rig::agent::MultiTurnStreamItem<R>,
) -> anyhow::Result<String> {

    let mut response_text = String::new();

    match chunk {

        // 🧠 Tudo vem como StreamAssistantItem
        rig::agent::MultiTurnStreamItem::StreamAssistantItem(content) => {

            match content {

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

                StreamedAssistantContent::ReasoningDelta { .. } => println!("ReasoningDelta event"),

                StreamedAssistantContent::Final(_) => {}
            }
    
        }

        rig::agent::MultiTurnStreamItem::StreamUserItem(tool_content) => {

            match tool_content {

                StreamedUserContent::ToolResult {
                    tool_result,
                    internal_call_id,
                } => {

                    
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

                    println!("{}", response_text);
                }
            }
        }
        

        rig::agent::MultiTurnStreamItem::FinalResponse(content) => {}

        _ => {}
    }

    Ok(response_text)
}
use anyhow::{Result};
use async_trait::async_trait;
use rig::{agent::{Agent, AgentBuilder, PromptHook, WithBuilderTools}, completion::{Chat, CompletionModel}, message::Message};
use rig::streaming::StreamingChat;
use crate::agents::{AgentInterface, agent};

use std::result::Result::Ok;
use crate::tools::*;
use crate::tools::confirmed_tool::{ConfirmedTool, ConfirmationMode};
use crate::agents::stream_handler::StreamHandler;

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
        let mut output = StreamHandler::handle_stream::<M>(&mut stream).await;

        println!();
        self.history.push(Message::user(input));
        self.history.append(&mut output);

        Ok(())
    }

}

pub(super) async fn build_agent<M, P>(builder: AgentBuilder<M, P>, pre_prompt: &str, history: Vec<Message>, yolo: bool, tools: Vec<String>) -> Box<dyn AgentInterface>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let builder = builder
    .preamble(pre_prompt)
    .default_max_turns(20);

    let agent = build_with_tools(builder, tools, yolo);

    Box::new(AgentWrapper::new(agent, history))
}

pub(super) fn build_with_tools<M, P>(builder: AgentBuilder<M, P>, tools: Vec<String>, yolo: bool) -> Agent<M, P>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
{

    let confirmation_mode = if yolo {ConfirmationMode::AlwaysAllow} else {ConfirmationMode::Ask};

        let mut builder = Some(builder); // 👈 agora controlamos o move
    let mut agent_builder_tool: Option<AgentBuilder<M, P, WithBuilderTools>> = None;

    for tool_name in tools {
        match tool_name.as_str() {
            "terminal" => {
                let mut tool = ConfirmedTool::new(TerminalTool);
                tool.set_mode(confirmation_mode);
                agent_builder_tool = apply_tool(&mut builder, agent_builder_tool, tool);
            }
            _ => println!("Unknown tool: {}", tool_name),
        }
    }

    match agent_builder_tool {
        Some(agent_builder) => agent_builder.build(),
        None => builder.unwrap().build(),
    }

}

use rig::tool::Tool;

fn apply_tool<M, P, T>(
    builder: &mut Option<AgentBuilder<M, P>>,
    agent_builder_tool: Option<AgentBuilder<M, P, WithBuilderTools>>,
    tool: T,
) -> Option<AgentBuilder<M, P, WithBuilderTools>>
where
    M: CompletionModel + Send + Sync + 'static,
    P: PromptHook<M> + Send + Sync + 'static,
    T: Tool + 'static,
{
    Some(match agent_builder_tool {
        Some(agent) => agent.tool(tool),
        None => {
            let b = builder.take().expect("builder já foi usado");
            b.tool(tool)
        }
    })
}
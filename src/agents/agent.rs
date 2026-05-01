use anyhow::{Result};
use async_trait::async_trait;
use rig::{agent::{Agent, AgentBuilder, PromptHook}, completion::{Chat, CompletionModel}, message::{Message}};
use rig::streaming::StreamingChat;
use crate::agents::AgentInterface;

use std::result::Result::Ok;
use crate::tools::*;
use crate::tools::confirmed_tool::ConfirmedTool;
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
        let output: String = StreamHandler::handle_stream::<M>(&mut stream).await;

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

    let terminal_tool = ConfirmedTool::new(TerminalTool);
    let builder = builder.tool(terminal_tool);

    let agent = builder.build();
    Box::new(AgentWrapper::new(agent, history))
}
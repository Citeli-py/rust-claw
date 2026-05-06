use anyhow::Result;
use async_trait::async_trait;
use rig::{agent::{Agent, PromptHook}, completion::{Chat, CompletionModel}, message::Message};
use rig::streaming::StreamingChat;
use crate::agents::interface::AgentInterface;
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

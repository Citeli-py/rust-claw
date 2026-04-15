use anyhow::{Result};
use async_trait::async_trait;
use rig::message::Message;

#[async_trait]
pub trait AgentInterface: Send {

    async fn chat(&mut self, input: &str) -> Result<String>;

    //retorna a stream pelo stdout
    async fn stream(&mut self, input: &str) -> Result<()>;

    fn history(&self,) -> &[Message];

    fn clean_history(&mut self);
}
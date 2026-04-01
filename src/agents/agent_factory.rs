use rig::client::{CompletionClient, Nothing,};
use rig::providers::{gemini, groq, ollama};
use rig::completion::Message;

use crate::agents::agent::{AgentInterface, build_agent};


pub enum ModelProvider {
    Ollama,
    Groq,
    Gemini
}

pub struct AgentFactory {}

impl AgentFactory {

    pub async fn create_agent(provider: ModelProvider, model: &str, api_key: &str, history: Vec<Message>) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {

        match provider {
            ModelProvider::Gemini => {
                let client = gemini::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), history).await)
            }
            ModelProvider::Ollama => {
                let client = ollama::Client::new(Nothing)?;
                Ok(build_agent(client.agent(model), history).await)
            }
            ModelProvider::Groq => {
                let client = groq::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), history).await)
            }
        }
    }
}
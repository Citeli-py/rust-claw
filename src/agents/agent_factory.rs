use rig::client::{CompletionClient, Nothing,};
use rig::providers::{gemini, groq, ollama};
use rig::completion::Message;

use crate::AgentConfig;
use crate::agents::agent::{build_agent};
use crate::agents::AgentInterface;


pub enum ModelProvider {
    Ollama,
    Groq,
    Gemini
}

impl ModelProvider {
    pub fn to_string(&self, ) -> String {
        match self {
            ModelProvider::Gemini => {
                "gemini".to_string()
            }
            ModelProvider::Ollama => {
                "ollama".to_string()
            }
            ModelProvider::Groq => {
                "groq".to_string()
            }

            _ => String::new()
        }
    }
}

pub struct AgentFactory {}

impl AgentFactory {

    pub async fn create_agent(provider: ModelProvider, model: &str, api_key: &str, pre_prompt: &str, history: Vec<Message>) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {

        match provider {
            ModelProvider::Gemini => {
                let client = gemini::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), pre_prompt, history).await)
            }
            ModelProvider::Ollama => {
                let client = ollama::Client::new(Nothing)?;
                Ok(build_agent(client.agent(model), pre_prompt, history).await)
            }
            ModelProvider::Groq => {
                let client = groq::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), pre_prompt, history).await)
            }
        }
    }

    pub async fn from_config(config: AgentConfig, history: Vec<Message>) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {
        AgentFactory::create_agent(config.provider, &config.model, &config.api_key, &config.pre_prompt, history).await
    }

}
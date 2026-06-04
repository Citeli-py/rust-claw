use rig::client::{CompletionClient, Nothing};
use rig::providers::{gemini, groq, ollama, openrouter};
use rig::completion::Message;

use crate::config::AgentConfig;
use crate::agents::builder::build_agent;
use crate::agents::interface::AgentInterface;
use crate::provider::ModelProvider;


pub struct AgentFactory {}

impl AgentFactory {

    pub async fn from_config(config: AgentConfig, history: Vec<Message>) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {
        match config.provider {
            ModelProvider::Gemini => {
                let client = gemini::Client::new(&config.api_key)?;
                Ok(build_agent(client.agent(&config.model), config, history).await)
            }
            ModelProvider::Ollama => {
                let client = ollama::Client::new(Nothing)?;
                Ok(build_agent(client.agent(&config.model), config, history).await)
            }
            ModelProvider::Groq => {
                let client = groq::Client::new(&config.api_key)?;
                Ok(build_agent(client.agent(&config.model), config, history).await)
            }
            ModelProvider::OpenRouter => {
                let client = openrouter::Client::new(&config.api_key)?;
                Ok(build_agent(client.agent(&config.model), config, history).await)
            }
        }
    }

}

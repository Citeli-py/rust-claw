use rig::client::{CompletionClient, Nothing,};
use rig::providers::{gemini, groq, ollama, openrouter};
use rig::completion::Message;

use crate::AgentConfig;
use crate::agents::builder::build_agent;
use crate::agents::config::ToolsConfig;
use crate::agents::interface::AgentInterface;
use crate::provider::ModelProvider;


pub struct AgentFactory {}

impl AgentFactory {

    pub async fn create_agent(
        provider: ModelProvider,
        model: &str,
        api_key: &str,
        pre_prompt: &str,
        history: Vec<Message>,
        yolo: bool,
        tools: Vec<String>,
        tools_config: ToolsConfig,
    ) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {

        match provider {
            ModelProvider::Gemini => {
                let client = gemini::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), pre_prompt, history, yolo, tools, tools_config).await)
            }
            ModelProvider::Ollama => {
                let client = ollama::Client::new(Nothing)?;
                Ok(build_agent(client.agent(model), pre_prompt, history, yolo, tools, tools_config).await)
            }
            ModelProvider::Groq => {
                let client = groq::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), pre_prompt, history, yolo, tools, tools_config).await)
            }
            ModelProvider::OpenRouter => {
                let client = openrouter::Client::new(api_key)?;
                Ok(build_agent(client.agent(model), pre_prompt, history, yolo, tools, tools_config).await)
            }
        }
    }

    pub async fn from_config(config: AgentConfig, history: Vec<Message>) -> Result<Box<dyn AgentInterface>, rig::http_client::Error> {
        AgentFactory::create_agent(
            config.provider,
            &config.model,
            &config.api_key,
            &config.pre_prompt,
            history,
            config.yolo,
            config.tools,
            config.tools_config,
        ).await
    }

}

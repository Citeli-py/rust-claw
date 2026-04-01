use anyhow::Ok;

use rig::agent;
use rig::completion::{Chat, GetTokenUsage};
use rig::providers::{gemini, ollama, openrouter, groq};

pub enum ModelProvider {
    Ollama,
    Gemini,
    OpenRouter,
    Groq,
}

pub enum AgentProvider {
    Ollama(rig::agent::Agent<ollama::CompletionModel>),
    Gemini(rig::agent::Agent<gemini::CompletionModel>),
    OpenRouter(rig::agent::Agent<openrouter::CompletionModel>),
    Groq(rig::agent::Agent<groq::CompletionModel>)
}

impl AgentProvider {
    pub async fn chat(
        &self,
        prompt: String,
        history: Vec<rig::completion::message::Message>,
    ) -> anyhow::Result<String> {

        let response = match self {
            AgentProvider::Ollama(agent) => agent.chat(prompt, history).await,
            AgentProvider::Gemini(agent) => agent.chat(prompt, history).await,
            AgentProvider::OpenRouter(agent) => agent.chat(prompt, history).await,
            AgentProvider::Groq(agent ) => agent.chat(prompt, history).await,
        };

        Ok(response?)
    }
}
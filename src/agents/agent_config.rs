use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use std::fs;

use crate::ModelProvider;

#[derive(Serialize, Deserialize)]
pub struct AgentConfigJson {
    pub provider: String,
    pub model: String,
    pub api_key: String,
}

pub struct AgentConfig {
    pub provider: ModelProvider,
    pub model: String,
    pub api_key: String,
    pub pre_prompt: String,
}

impl AgentConfig {

    pub fn match_provider(provider_str: &str) -> Option<ModelProvider> {
        match provider_str.to_lowercase().as_str() {
            "gemini" => Some(ModelProvider::Gemini),
            "ollama" => Some(ModelProvider::Ollama),
            "groq" => Some(ModelProvider::Groq),
            _ => None,
        }
    }

    pub fn from_path(path: &str) -> Result<AgentConfig> {
        let config_path = format!("{}/config.json", path);
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Erro ao ler arquivo: {}", config_path))?;

        let config: AgentConfigJson = serde_json::from_str(&content)
            .with_context(|| "Erro ao fazer parse do JSON")?;

        let pre_prompt_path = format!("{}/PROMPT.md", path);
        let pre_prompt = fs::read_to_string(&pre_prompt_path)
            .with_context(|| format!("Erro ao ler arquivo: {}", pre_prompt_path))?;

        let agent_config = AgentConfig{
            provider: AgentConfig::match_provider(&config.provider).unwrap(),
            model: config.model,
            api_key: config.api_key,
            pre_prompt: pre_prompt
        };

        Ok(agent_config)
    }
}
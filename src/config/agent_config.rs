use anyhow::{Result, Context};
use std::fs;
use std::path::Path;

use crate::provider::ModelProvider;
use crate::config::ToolsConfig;
use crate::config::config_json::AgentConfigJson;

pub struct AgentConfig {
    pub name: String,
    pub provider: ModelProvider,
    pub model: String,
    pub api_key: String,
    pub pre_prompt: String,
    pub yolo: bool,
    pub tools_config: ToolsConfig,
    pub config_path: Option<String>,
}

impl AgentConfig {

    pub fn from_path(path: &str) -> Result<AgentConfig> {
        let config = Self::get_config(path)?;
        let pre_prompt = Self::get_pre_prompt(path)?;
        let name = Self::get_name(path)?;

        Ok(AgentConfig {
            name,
            provider: ModelProvider::from_str(&config.provider).unwrap(),
            model: config.model,
            api_key: config.api_key,
            pre_prompt,
            yolo: false,
            tools_config: ToolsConfig::from_json(
                config.tools,
                config.tools_config,
                config.tools_trusted_commands,
            ),
            config_path: Some(format!("{}/config.json", path)),
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}\n\tprovider: {}\n\tmodel: {}\n", self.name, self.provider.to_string(), self.model)
    }

    fn get_config(path: &str) -> Result<AgentConfigJson> {
        let config_path = format!("{}/config.json", path);
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Erro ao ler arquivo: {}", config_path))?;

        serde_json::from_str(&content)
            .with_context(|| "Erro ao fazer parse do JSON")
    }

    fn get_pre_prompt(path: &str) -> Result<String> {
        let pre_prompt_path = format!("{}/PROMPT.md", path);
        fs::read_to_string(&pre_prompt_path)
            .with_context(|| format!("Erro ao ler arquivo: {}", pre_prompt_path))
    }

    fn get_name(path: &str) -> Result<String> {
        Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .context("Erro ao extrair nome do agente a partir do path")
            .map(|s| s.to_string())
    }
}

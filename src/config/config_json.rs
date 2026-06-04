use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Serialize, Deserialize, Default)]
pub struct ToolsConfigJson {
    #[serde(default)]
    pub web_browser: WebBrowserConfigJson,
    #[serde(default)]
    pub terminal: TerminalConfigJson,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WebBrowserConfigJson {
    pub headless: bool,
}

impl Default for WebBrowserConfigJson {
    fn default() -> Self {
        Self { headless: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct TerminalConfigJson {}

#[derive(Serialize, Deserialize)]
pub struct AgentConfigJson {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub tools_config: ToolsConfigJson,
    #[serde(default)]
    pub tools_trusted_commands: HashMap<String, Vec<String>>,
}
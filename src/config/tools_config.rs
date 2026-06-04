use std::collections::HashMap;
use crate::config::config_json::ToolsConfigJson;

pub struct TerminalConfig {
    pub trusted: Vec<String>,
}

pub struct WebBrowserConfig {
    pub headless: bool,
    pub trusted: Vec<String>,
}

pub enum ToolConfig {
    Terminal(TerminalConfig),
    WebBrowser(WebBrowserConfig),
}

impl ToolConfig {
    pub fn name(&self) -> &str {
        match self {
            ToolConfig::Terminal(_) => "terminal",
            ToolConfig::WebBrowser(_) => "web_browser",
        }
    }

    pub fn trusted_commands(&self) -> &[String] {
        match self {
            ToolConfig::Terminal(c) => &c.trusted,
            ToolConfig::WebBrowser(c) => &c.trusted,
        }
    }
}

pub struct ToolsConfig {
    pub tools: Vec<ToolConfig>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        ToolsConfig { tools: vec![] }
    }
}

impl ToolsConfig {
    pub fn from_json(
        tool_names: Vec<String>,
        json: ToolsConfigJson,
        trusted_commands: HashMap<String, Vec<String>>,
    ) -> Self {
        let tools = tool_names.into_iter().filter_map(|name| {
            let trusted = trusted_commands.get(&name).cloned().unwrap_or_default();
            match name.as_str() {
                "terminal" => Some(ToolConfig::Terminal(TerminalConfig { trusted })),
                "web_browser" => Some(ToolConfig::WebBrowser(WebBrowserConfig {
                    headless: json.web_browser.headless,
                    trusted,
                })),
                other => {
                    println!("Unknown tool in config: {}", other);
                    None
                }
            }
        }).collect();

        ToolsConfig { tools }
    }
}

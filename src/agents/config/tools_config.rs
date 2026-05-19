use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct WebBrowserConfig {
    #[serde(default = "default_headless")]
    pub headless: bool,
}

impl Default for WebBrowserConfig {
    fn default() -> Self {
        Self { headless: default_headless() }
    }
}

fn default_headless() -> bool { true }

#[derive(Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    #[serde(default)]
    pub web_browser: WebBrowserConfig,
}
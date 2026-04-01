use std::time::Duration;

use rig::completion::ToolDefinition;
use rig::tool::Tool;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::pinchtab::{PinchTab, PinchTabOpenTabResponse};

#[derive(Debug)]
pub struct WebBrowserError {
    pub msg: String
}

impl std::fmt::Display for WebBrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.msg.clone();
        write!(f, "WebBrowser execution error: {msg}")
    }
}

impl std::error::Error for WebBrowserError {}

#[derive(Deserialize)]
pub struct WebBrowserArgs {

    pub command: String,

    pub tab_id: Option<String>,

    pub url: Option<String>,

    pub element_ref: Option<String>,

    pub text: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct WebBrowserOutput {

    pub result: String,
}

#[derive(Clone)]
pub struct WebBrowserTool {

    pub browser: PinchTab,
    pub tab_id: String,
}

impl WebBrowserTool {

    pub async fn new(browser: PinchTab) -> Self {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let res = browser.open_tab(None).await.unwrap();
        WebBrowserTool { browser, tab_id: res.tabId }
    }
}


impl Tool for WebBrowserTool {

    const NAME: &'static str = "web_browser";

    type Error = WebBrowserError;
    type Args = WebBrowserArgs;
    type Output = WebBrowserOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {

        ToolDefinition {
            name: "web_browser".to_string(),
            description: "Controla um navegador web usando PinchTab. Permite navegar, extrair texto, clicar em elementos e capturar screenshots.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {

                    "command": {
                        "type": "string",
                        "description": "Comando a executar",
                        "enum": [
                            "navigate",
                            "snapshot",
                            "text",
                            "click",
                            "fill",
                            "screenshot",
                            "pdf",
                        ]
                    },

                    "url": {
                        "type": "string",
                        "description": "URL para navegar"
                    },

                    "element_ref": {
                        "type": "string",
                        "description": "Referência do elemento obtida no snapshot"
                    },

                    "text": {
                        "type": "string",
                        "description": "Texto a preencher em um input"
                    }

                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {

        let result = match args.command.as_str() {

            "navigate" => {

                let url = args.url.ok_or(WebBrowserError{ msg: "url não informada".to_string()})?;

                self.browser
                    .navigate(self.tab_id.clone(), url)
                    .await
                    .map_err(|_| WebBrowserError{ msg: "Erro inesperado ao navegar".to_string()})?
            }

            "snapshot" => {

                self.browser
                    .snapshot(self.tab_id.clone())
                    .await
                    .map_err(|_| WebBrowserError{ msg: "Erro ao navegar".to_string()})?
            }

            "text" => {

                self.browser
                    .text(self.tab_id.clone())
                    .await
                    .map_err(|_| WebBrowserError{msg: "Erro inesperado".to_string()})?
            }

            "click" => {

                let element = args.element_ref.ok_or(WebBrowserError{ msg: "element não informada".to_string()})?;

                self.browser
                    .click(self.tab_id.clone(), element)
                    .await
                    .map_err(|_| WebBrowserError{msg: "Erro inesperado".to_string()})?
            }

            "fill" => {

                let element = args.element_ref.ok_or(WebBrowserError{ msg: "element não informada".to_string()})?;

                let text = args.text.ok_or(WebBrowserError{msg: "text não informado".to_string()})?;

                self.browser
                    .fill(self.tab_id.clone(), element, text)
                    .await
                    .map_err(|_| WebBrowserError{msg: "Erro inesperado".to_string()})?
            }

            "screenshot" => {

                self.browser
                    .screenshot(self.tab_id.clone())
                    .await
                    .map_err(|_| WebBrowserError{msg: "Erro inesperado".to_string()})?
            }

            "pdf" => {

                self.browser
                    .pdf(self.tab_id.clone())
                    .await
                    .map_err(|_| WebBrowserError{msg: "Erro inesperado".to_string()})?
            }

            _ => return Err(WebBrowserError{ msg: "comando não existe".to_string()}),
        };

        Ok(WebBrowserOutput { result })
    }
}
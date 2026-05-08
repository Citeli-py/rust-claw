use rig::completion::ToolDefinition;
use rig::tool::Tool;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::tools::web_browser::WebDriverHandler;
use crate::tools::web_browser::types::ElementInfo;

#[derive(Debug)]
pub struct WebBrowserError;

impl std::fmt::Display for WebBrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Web browser execution error")
    }
}

impl std::error::Error for WebBrowserError {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserAction {
    Goto,
    GetPageText,
    FindClickableElements,
    FindFillableElements,
    ClickElement,
    ClickBySelector,
    FillElement,
    FillBySelector,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebBrowserArgs {
    pub action: BrowserAction,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub element_json: Option<String>,

    #[serde(default)]
    pub css_selector: Option<String>,

    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebBrowserOutput {
    pub success: bool,
    pub message: String,

    #[serde(default)]
    pub page_text: Option<String>,

    #[serde(default)]
    pub elements: Option<Vec<ElementInfo>>,
}

pub struct WebBrowserTool {
    pub web_driver: WebDriverHandler,
}

impl WebBrowserTool {
    pub fn new(web_driver: WebDriverHandler) -> Self {
        Self { web_driver }
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
            description: concat!(
                "Controla um navegador web utilizando WebDriver. ",
                "Use 'find_clickable_elements' para listar botões, links e controles clicáveis. ",
                "Cada elemento retornado contém 'index', 'text' (texto visível), 'tag_name', 'css_selector', 'role' e 'bounding_box'. ",
                "Para clicar, prefira passar o 'element_json' exato ou use 'click_by_selector' com o css_selector. ",
                "Use 'find_fillable_elements' para campos de formulário. ",
                "Se houver múltiplos elementos similares, use 'index' ou 'text' para diferenciá-los."
            ).to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [
                            "goto",
                            "get_page_text",
                            "find_clickable_elements",
                            "find_fillable_elements",
                            "click_element",
                            "click_by_selector",
                            "fill_element",
                            "fill_by_selector"
                        ],
                        "description": "Ação a ser executada no navegador"
                    },
                    "url": {
                        "type": "string",
                        "description": "URL para navegar (usado com action=goto)"
                    },
                    "element_json": {
                        "type": "string",
                        "description": "JSON do elemento retornado por find_clickable_elements ou find_fillable_elements"
                    },
                    "css_selector": {
                        "type": "string",
                        "description": "CSS selector para encontrar o elemento (alternativa ao element_json)"
                    },
                    "text": {
                        "type": "string",
                        "description": "Texto para preencher em um campo (usado com action=fill_element ou fill_by_selector)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {

        let result = match args.action {
            BrowserAction::Goto => {
                let url = args.url.ok_or(WebBrowserError)?;

                self.web_driver
                    .goto(&url)
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: format!("Navegou para {}", url),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::GetPageText => {
                let text = self.web_driver
                    .get_page_text()
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: "Texto da página obtido com sucesso".to_string(),
                    page_text: Some(text),
                    elements: None,
                }
            }

            BrowserAction::FindClickableElements => {
                let elements = self.web_driver
                    .find_clickable_elements()
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: format!("{} elementos clicáveis encontrados", elements.len()),
                    page_text: None,
                    elements: Some(elements),
                }
            }

            BrowserAction::FindFillableElements => {
                let elements = self.web_driver
                    .find_fillable_elements()
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: format!("{} elementos preenchíveis encontrados", elements.len()),
                    page_text: None,
                    elements: Some(elements),
                }
            }

            BrowserAction::ClickElement => {
                let element_json = args.element_json.ok_or(WebBrowserError)?;

                self.web_driver
                    .click_element(&element_json)
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: "Elemento clicado com sucesso".to_string(),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::ClickBySelector => {
                let selector = args.css_selector.ok_or(WebBrowserError)?;

                self.web_driver
                    .click_by_selector(&selector)
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: format!("Elemento '{}' clicado com sucesso", selector),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::FillElement => {
                let element_json = args.element_json.ok_or(WebBrowserError)?;
                let text = args.text.ok_or(WebBrowserError)?;

                self.web_driver
                    .fill_element(&element_json, &text)
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: "Elemento preenchido com sucesso".to_string(),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::FillBySelector => {
                let selector = args.css_selector.ok_or(WebBrowserError)?;
                let text = args.text.ok_or(WebBrowserError)?;

                self.web_driver
                    .fill_by_selector(&selector, &text)
                    .await
                    .map_err(|_| WebBrowserError)?;

                WebBrowserOutput {
                    success: true,
                    message: format!("Elemento '{}' preenchido com sucesso", selector),
                    page_text: None,
                    elements: None,
                }
            }
        };

        Ok(result)
    }
}

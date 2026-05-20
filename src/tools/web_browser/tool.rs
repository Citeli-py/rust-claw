use rig::completion::ToolDefinition;
use rig::tool::Tool;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::tools::web_browser::WebDriverHandlerInterface;
use crate::tools::web_browser::types::Element;

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
    ClickBySelector,
    FillBySelector,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebBrowserArgs {
    pub action: BrowserAction,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub css_selector: Option<String>,

    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebBrowserOutput {
    pub success: bool,
    pub message: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_text: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<Element>>,
}

pub struct WebBrowserTool<D: WebDriverHandlerInterface> {
    pub web_driver: D,
}

impl<D: WebDriverHandlerInterface> WebBrowserTool<D> {
    pub fn new(web_driver: D) -> Self {
        Self { web_driver }
    }
}

impl<D: WebDriverHandlerInterface + Send + Sync + 'static> Tool for WebBrowserTool<D> {
    const NAME: &'static str = "web_browser";

    type Error = WebBrowserError;
    type Args = WebBrowserArgs;
    type Output = WebBrowserOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "web_browser".to_string(),
            description: concat!(
                "Controls a web browser. ",
                "Use 'find_clickable_elements' to list buttons, links and clickable controls — each has an index, type, text and css_selector. ",
                "Use 'find_fillable_elements' for form fields — each has an index, type, label and css_selector. ",
                "Use 'click_by_selector' or 'fill_by_selector' with the css_selector from those results."
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
                            "click_by_selector",
                            "fill_by_selector"
                        ],
                        "description": "Action to execute in the browser"
                    },
                    "url": {
                        "type": "string",
                        "description": "URL to navigate to (used with action=goto)"
                    },
                    "css_selector": {
                        "type": "string",
                        "description": "CSS selector from find_clickable_elements or find_fillable_elements results"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to fill in a field (used with action=fill_by_selector)"
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
                self.web_driver.goto(&url).await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: format!("Navigated to {url}"),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::GetPageText => {
                let text = self.web_driver.get_page_text().await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: "Page text retrieved".to_string(),
                    page_text: Some(text),
                    elements: None,
                }
            }

            BrowserAction::FindClickableElements => {
                let elements = self.web_driver.find_clickable_elements().await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: format!("{} clickable elements found", elements.len()),
                    page_text: None,
                    elements: Some(elements),
                }
            }

            BrowserAction::FindFillableElements => {
                let elements = self.web_driver.find_fillable_elements().await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: format!("{} fillable elements found", elements.len()),
                    page_text: None,
                    elements: Some(elements),
                }
            }

            BrowserAction::ClickBySelector => {
                let selector = args.css_selector.ok_or(WebBrowserError)?;
                self.web_driver.click_by_selector(&selector).await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: format!("Clicked '{selector}'"),
                    page_text: None,
                    elements: None,
                }
            }

            BrowserAction::FillBySelector => {
                let selector = args.css_selector.ok_or(WebBrowserError)?;
                let text = args.text.ok_or(WebBrowserError)?;
                self.web_driver.fill_by_selector(&selector, &text).await.map_err(|_| WebBrowserError)?;
                WebBrowserOutput {
                    success: true,
                    message: format!("Filled '{selector}'"),
                    page_text: None,
                    elements: None,
                }
            }
        };

        Ok(result)
    }
}

use ai_agent::config::WebBrowserConfig;
use ai_agent::tools::confirmed_tool::ConfirmationMode;
use ai_agent::tools::WebDriverHandlerInterface;
use ai_agent::tools::web_browser::{Element, WebBrowserTool, BrowserAction, WebBrowserArgs};
use async_trait::async_trait;
use rig::tool::Tool;
use std::sync::Mutex;

struct MockWebDriver {
    call_log: Mutex<Vec<String>>,
    page_text: String,
    clickable: Vec<Element>,
    fillable: Vec<Element>,
    should_fail: bool,
}

impl MockWebDriver {
    fn new() -> Self {
        Self {
            call_log: Mutex::new(Vec::new()),
            page_text: "Hello World".to_string(),
            clickable: vec![
                Element::Button {
                    index: 0,
                    selector: "#btn1".to_string(),
                    text: "Click Me".to_string(),
                },
            ],
            fillable: vec![
                Element::Input {
                    index: 0,
                    selector: "#search".to_string(),
                    label: Some("Search".to_string()),
                },
            ],
            should_fail: false,
        }
    }
}

#[async_trait]
impl WebDriverHandlerInterface for MockWebDriver {
    async fn goto(&self, url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("goto({})", url));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "goto failed")));
        }
        Ok(())
    }

    async fn get_page_text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push("get_page_text".to_string());
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "get_page_text failed")));
        }
        Ok(self.page_text.clone())
    }

    async fn find_clickable_elements(&self) -> Result<Vec<Element>, Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push("find_clickable_elements".to_string());
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "find_clickable_elements failed")));
        }
        Ok(self.clickable.clone())
    }

    async fn find_fillable_elements(&self) -> Result<Vec<Element>, Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push("find_fillable_elements".to_string());
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "find_fillable_elements failed")));
        }
        Ok(self.fillable.clone())
    }

    async fn click_by_selector(&self, selector: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("click_by_selector({})", selector));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "click_by_selector failed")));
        }
        Ok(())
    }

    async fn fill_by_selector(&self, selector: &str, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("fill_by_selector({}, {})", selector, text));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "fill_by_selector failed")));
        }
        Ok(())
    }
}

fn make_tool(mock: MockWebDriver) -> WebBrowserTool<MockWebDriver> {
    WebBrowserTool::new(mock)
}

#[tokio::test]
async fn test_definition() {
    let tool = make_tool(MockWebDriver::new());
    let def = tool.definition("prompt".to_string()).await;

    assert_eq!(def.name, "web_browser");
    assert!(def.description.contains("browser"));
    assert!(def.parameters["properties"]["action"]["enum"].is_array());
    assert_eq!(def.parameters["required"][0], "action");
}

mod action_goto {
    use super::*;

    #[tokio::test]
    async fn test_goto_with_url() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::Goto,
            url: Some("https://example.com".to_string()),
            css_selector: None,
            text: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("example.com"));
    }

    #[tokio::test]
    async fn test_goto_missing_url_returns_error() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::Goto,
            url: None,
            css_selector: None,
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }
}

mod action_get_page_text {
    use super::*;

    #[tokio::test]
    async fn test_get_page_text_returns_text() {
        let mut mock = MockWebDriver::new();
        mock.page_text = "Page content here".to_string();
        let tool = make_tool(mock);
        let args = WebBrowserArgs {
            action: BrowserAction::GetPageText,
            url: None,
            css_selector: None,
            text: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.page_text.unwrap(), "Page content here");
    }
}

mod action_find_clickable {
    use super::*;

    #[tokio::test]
    async fn test_find_clickable_returns_elements() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::FindClickableElements,
            url: None,
            css_selector: None,
            text: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        let elements = result.elements.unwrap();
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], Element::Button { text, .. } if text == "Click Me"));
    }
}

mod action_find_fillable {
    use super::*;

    #[tokio::test]
    async fn test_find_fillable_returns_elements() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::FindFillableElements,
            url: None,
            css_selector: None,
            text: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        let elements = result.elements.unwrap();
        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], Element::Input { .. }));
    }
}

mod action_click_by_selector {
    use super::*;

    #[tokio::test]
    async fn test_click_by_selector() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::ClickBySelector,
            url: None,
            css_selector: Some("#btn1".to_string()),
            text: None,
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("#btn1"));
    }

    #[tokio::test]
    async fn test_click_by_selector_missing_selector_returns_error() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::ClickBySelector,
            url: None,
            css_selector: None,
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }
}

mod action_fill_by_selector {
    use super::*;

    #[tokio::test]
    async fn test_fill_by_selector() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            css_selector: Some("#search".to_string()),
            text: Some("Rust".to_string()),
        };
        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("#search"));
    }

    #[tokio::test]
    async fn test_fill_by_selector_missing_selector_returns_error() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            css_selector: None,
            text: Some("Rust".to_string()),
        };
        assert!(tool.call(args).await.is_err());
    }

    #[tokio::test]
    async fn test_fill_by_selector_missing_text_returns_error() {
        let tool = make_tool(MockWebDriver::new());
        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            css_selector: Some("#search".to_string()),
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }
}

mod build {
    use super::*;

    fn cfg(trusted: Vec<&str>, headless: bool) -> WebBrowserConfig {
        WebBrowserConfig {
            headless,
            trusted: trusted.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[tokio::test]
    async fn test_build_sets_tool_name() {
        let tool = WebBrowserTool::build(cfg(vec![], true), ConfirmationMode::AlwaysAllow, None).await;
        assert_eq!(tool.tool_name, "web_browser");
    }

    #[tokio::test]
    async fn test_build_always_deny_blocks_action() {
        let tool = WebBrowserTool::build(cfg(vec![], true), ConfirmationMode::AlwaysDeny, None).await;
        let result = tool.call(WebBrowserArgs {
            action: BrowserAction::GetPageText,
            url: None,
            css_selector: None,
            text: None,
        }).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("blocked"));
    }

    #[tokio::test]
    async fn test_build_trusted_action_bypasses_confirmation() {
        let trusted_json = r#"{"action":"get_page_text"}"#;
        let tool = WebBrowserTool::build(cfg(vec![trusted_json], true), ConfirmationMode::Ask, None).await;
        let result = tool.call(WebBrowserArgs {
            action: BrowserAction::GetPageText,
            url: None,
            css_selector: None,
            text: None,
        }).await;
        // Vai falhar na execução real (sem driver), mas não deve ser bloqueado pelo trusted check
        assert!(result.is_err());
        assert!(!result.unwrap_err().to_string().contains("blocked"));
    }
}

mod error_propagation {
    use super::*;

    #[tokio::test]
    async fn test_goto_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);
        let args = WebBrowserArgs {
            action: BrowserAction::Goto,
            url: Some("https://example.com".to_string()),
            css_selector: None,
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }

    #[tokio::test]
    async fn test_get_page_text_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);
        let args = WebBrowserArgs {
            action: BrowserAction::GetPageText,
            url: None,
            css_selector: None,
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }

    #[tokio::test]
    async fn test_find_clickable_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);
        let args = WebBrowserArgs {
            action: BrowserAction::FindClickableElements,
            url: None,
            css_selector: None,
            text: None,
        };
        assert!(tool.call(args).await.is_err());
    }
}

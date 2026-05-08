use ai_agent::tools::WebDriverHandlerInterface;
use ai_agent::tools::web_browser::{
    ElementInfo, ElementType, BoundingRect,
    WebBrowserTool, BrowserAction, WebBrowserArgs,
};
use async_trait::async_trait;
use rig::tool::Tool;
use std::sync::Mutex;
use std::collections::HashMap;

struct MockWebDriver {
    call_log: Mutex<Vec<String>>,
    page_text: String,
    clickable: Vec<ElementInfo>,
    fillable: Vec<ElementInfo>,
    should_fail: bool,
}

impl MockWebDriver {
    fn new() -> Self {
        let mut attrs = HashMap::new();
        attrs.insert("id".to_string(), "search".to_string());

        Self {
            call_log: Mutex::new(Vec::new()),
            page_text: "Hello World".to_string(),
            clickable: vec![
                ElementInfo {
                    element_json: r#"{"element-6066-11e4-a52e-4f735466cecf":"btn1"}"#.to_string(),
                    index: 0,
                    tag_name: "button".to_string(),
                    text: "Click Me".to_string(),
                    css_selector: "#btn1".to_string(),
                    attributes: attrs.clone(),
                    element_type: ElementType::Button,
                    role: Some("button".to_string()),
                    is_clickable: true,
                    is_fillable: false,
                    is_displayed: true,
                    bounding_box: Some(BoundingRect { x: 0.0, y: 0.0, width: 100.0, height: 30.0 }),
                },
            ],
            fillable: vec![
                ElementInfo {
                    element_json: r#"{"element-6066-11e4-a52e-4f735466cecf":"inp1"}"#.to_string(),
                    index: 0,
                    tag_name: "input".to_string(),
                    text: "".to_string(),
                    css_selector: "#search".to_string(),
                    attributes: attrs,
                    element_type: ElementType::Input,
                    role: None,
                    is_clickable: false,
                    is_fillable: true,
                    is_displayed: true,
                    bounding_box: None,
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

    async fn find_clickable_elements(&self) -> Result<Vec<ElementInfo>, Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push("find_clickable_elements".to_string());
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "find_clickable_elements failed")));
        }
        Ok(self.clickable.clone())
    }

    async fn find_fillable_elements(&self) -> Result<Vec<ElementInfo>, Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push("find_fillable_elements".to_string());
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "find_fillable_elements failed")));
        }
        Ok(self.fillable.clone())
    }

    async fn click_element(&self, element_json: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("click_element({})", element_json));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "click_element failed")));
        }
        Ok(())
    }

    async fn click_by_selector(&self, selector: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("click_by_selector({})", selector));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "click_by_selector failed")));
        }
        Ok(())
    }

    async fn fill_element(&self, element_json: &str, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.call_log.lock().unwrap().push(format!("fill_element({}, {})", element_json, text));
        if self.should_fail {
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "fill_element failed")));
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
    let mock = MockWebDriver::new();
    let tool = make_tool(mock);
    let def = tool.definition("prompt".to_string()).await;

    assert_eq!(def.name, "web_browser");
    assert!(def.description.contains("navegador"));
    assert!(def.parameters["properties"]["action"]["enum"].is_array());
    assert_eq!(def.parameters["required"][0], "action");
}

mod action_goto {
    use super::*;

    #[tokio::test]
    async fn test_goto_with_url() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::Goto,
            url: Some("https://example.com".to_string()),
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("example.com"));
    }

    #[tokio::test]
    async fn test_goto_missing_url_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::Goto,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
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
            element_json: None,
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
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FindClickableElements,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        let elements = result.elements.unwrap();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].text, "Click Me");
    }
}

mod action_find_fillable {
    use super::*;

    #[tokio::test]
    async fn test_find_fillable_returns_elements() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FindFillableElements,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        let elements = result.elements.unwrap();
        assert_eq!(elements.len(), 1);
        assert!(elements[0].is_fillable);
    }
}

mod action_click_element {
    use super::*;

    #[tokio::test]
    async fn test_click_element_with_json() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::ClickElement,
            url: None,
            element_json: Some(r#"{"element-6066-11e4-a52e-4f735466cecf":"btn1"}"#.to_string()),
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_click_element_missing_json_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::ClickElement,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}

mod action_click_by_selector {
    use super::*;

    #[tokio::test]
    async fn test_click_by_selector() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::ClickBySelector,
            url: None,
            element_json: None,
            css_selector: Some("#btn1".to_string()),
            text: None,
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("#btn1"));
    }

    #[tokio::test]
    async fn test_click_by_selector_missing_selector_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::ClickBySelector,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}

mod action_fill_element {
    use super::*;

    #[tokio::test]
    async fn test_fill_element() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillElement,
            url: None,
            element_json: Some(r#"{"element-6066-11e4-a52e-4f735466cecf":"inp1"}"#.to_string()),
            css_selector: None,
            text: Some("hello".to_string()),
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_fill_element_missing_json_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillElement,
            url: None,
            element_json: None,
            css_selector: None,
            text: Some("hello".to_string()),
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fill_element_missing_text_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillElement,
            url: None,
            element_json: Some(r#"{"element-6066-11e4-a52e-4f735466cecf":"inp1"}"#.to_string()),
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}

mod action_fill_by_selector {
    use super::*;

    #[tokio::test]
    async fn test_fill_by_selector() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            element_json: None,
            css_selector: Some("#search".to_string()),
            text: Some("Rust".to_string()),
        };

        let result = tool.call(args).await.unwrap();
        assert!(result.success);
        assert!(result.message.contains("#search"));
    }

    #[tokio::test]
    async fn test_fill_by_selector_missing_selector_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            element_json: None,
            css_selector: None,
            text: Some("Rust".to_string()),
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fill_by_selector_missing_text_returns_error() {
        let mock = MockWebDriver::new();
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FillBySelector,
            url: None,
            element_json: None,
            css_selector: Some("#search".to_string()),
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
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
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_page_text_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::GetPageText,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_clickable_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::FindClickableElements,
            url: None,
            element_json: None,
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_click_element_failure_returns_error() {
        let mut mock = MockWebDriver::new();
        mock.should_fail = true;
        let tool = make_tool(mock);

        let args = WebBrowserArgs {
            action: BrowserAction::ClickElement,
            url: None,
            element_json: Some("{}".to_string()),
            css_selector: None,
            text: None,
        };

        let result = tool.call(args).await;
        assert!(result.is_err());
    }
}



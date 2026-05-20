use ai_agent::tools::WebDriverHandlerInterface;
use ai_agent::tools::web_browser::{WebDriverHandler, Element};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

fn shared_handler() -> &'static Arc<Mutex<WebDriverHandler>> {
    static HANDLER: OnceLock<Arc<Mutex<WebDriverHandler>>> = OnceLock::new();
    HANDLER.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new()
                .expect("Failed to create init runtime");
            Arc::new(Mutex::new(
                rt.block_on(WebDriverHandler::new(true))
                    .expect("Failed to create shared WebDriverHandler"),
            ))
        })
        .join()
        .expect("Thread panicked")
    })
}

#[tokio::test]
async fn test_new_webdriver_handler_without_errors() {
    let result = WebDriverHandler::new(true).await;
    assert!(result.is_ok());
    result.unwrap().close().await.unwrap();
}

#[tokio::test]
async fn test_webdriver_handler_should_goto_url() {
    let handler = WebDriverHandler::new(true).await.unwrap();
    let url = "https://www.wikipedia.org/";
    handler.goto(url).await.unwrap();
    let current_url = handler.driver.current_url().await.unwrap();
    handler.close().await.unwrap();
    assert_eq!(url, current_url.as_str());
}

mod unit_tests {
    use ai_agent::tools::web_browser::Element;
    use serde_json;

    #[test]
    fn test_element_button_serialization() {
        let elem = Element::Button {
            index: 0,
            selector: "button#submit".to_string(),
            text: "Submit".to_string(),
        };
        let json = serde_json::to_string(&elem).unwrap();
        assert!(json.contains("\"type\":\"Button\""));
        assert!(json.contains("Submit"));

        let de: Element = serde_json::from_str(&json).unwrap();
        assert!(matches!(de, Element::Button { .. }));
    }

    #[test]
    fn test_element_link_serialization() {
        let elem = Element::Link {
            index: 1,
            selector: "a.nav".to_string(),
            text: "Home".to_string(),
            href: Some("/home".to_string()),
        };
        let json = serde_json::to_string(&elem).unwrap();
        assert!(json.contains("\"type\":\"Link\""));
        assert!(json.contains("/home"));
    }

    #[test]
    fn test_element_input_serialization() {
        let elem = Element::Input {
            index: 2,
            selector: "input#q".to_string(),
            label: Some("Search".to_string()),
        };
        let json = serde_json::to_string(&elem).unwrap();
        assert!(json.contains("\"type\":\"Input\""));
    }

    #[test]
    fn test_element_checkbox_serialization() {
        let elem = Element::Checkbox {
            index: 3,
            selector: "#terms".to_string(),
            label: Some("Accept terms".to_string()),
            name: Some("terms".to_string()),
        };
        let json = serde_json::to_string(&elem).unwrap();
        assert!(json.contains("\"type\":\"Checkbox\""));
        assert!(json.contains("terms"));
    }

    #[test]
    fn test_to_llm_str_button() {
        let elem = Element::Button {
            index: 0,
            selector: "#btn".to_string(),
            text: "Search".to_string(),
        };
        let s = elem.to_llm_str();
        assert!(s.contains("[0]"));
        assert!(s.contains("Button"));
        assert!(s.contains("Search"));
        assert!(s.contains("#btn"));
    }

    #[test]
    fn test_to_llm_str_link_with_href() {
        let elem = Element::Link {
            index: 1,
            selector: "a.nav".to_string(),
            text: "Home".to_string(),
            href: Some("/home".to_string()),
        };
        let s = elem.to_llm_str();
        assert!(s.contains("-> /home"));
    }

    #[test]
    fn test_to_llm_str_radio_with_name() {
        let elem = Element::Radio {
            index: 5,
            selector: "input[name='color']".to_string(),
            label: Some("Red".to_string()),
            name: Some("color".to_string()),
        };
        let s = elem.to_llm_str();
        assert!(s.contains("(name=color)"));
    }
}

async fn reset_and_goto(handler: &WebDriverHandler, url: &str) {
    handler.driver.delete_all_cookies().await.ok();
    handler.goto(url).await.expect("Failed to goto");
}

#[tokio::test]
async fn test_get_page_elements_returns_valid_list() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;
    let elements = handler.get_page_elements().await.expect("Failed to get page elements");
    assert!(!elements.is_empty());
}

#[tokio::test]
async fn test_find_clickable_elements() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;
    let clickable = handler.find_clickable_elements().await.expect("Failed to find clickable elements");
    assert!(!clickable.is_empty());
    assert!(clickable.iter().any(|e| matches!(e, Element::Button { .. } | Element::Link { .. })));
}

#[tokio::test]
async fn test_find_fillable_elements() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;
    let fillable = handler.find_fillable_elements().await.expect("Failed to find fillable elements");
    assert!(!fillable.is_empty());
    assert!(fillable.iter().any(|e| matches!(e, Element::Input { .. } | Element::Textarea { .. })));
}

#[tokio::test]
async fn test_get_page_text_exposes_content() {
    let handler = WebDriverHandler::new(true).await.expect("Failed to create handler");
    handler.goto("https://www.wikipedia.org/").await.expect("Failed to goto");
    let text = handler.get_page_text().await.expect("Failed to get page text");
    assert!(text.contains("Wikipedia"));
    handler.close().await.expect("Failed to close");
}

#[tokio::test]
async fn test_click_and_fill_workflow() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;

    let fillable = handler.find_fillable_elements().await.expect("Failed to find fillable");
    let search_input = fillable.iter().find(|e| matches!(e, Element::Input { .. }))
        .expect("Should find search input");

    let selector = match search_input {
        Element::Input { selector, .. } => selector.clone(),
        _ => unreachable!(),
    };
    handler.fill_by_selector(&selector, "Rust programming").await.expect("Failed to fill input");

    let clickable = handler.find_clickable_elements().await.expect("Failed to find clickable");
    let search_btn = clickable.iter().find(|e| {
        if let Element::Button { text, .. } | Element::Link { text, .. } = e {
            text.to_lowercase().contains("search")
        } else { false }
    }).expect("Should find search button");

    let selector = match search_btn {
        Element::Button { selector, .. } | Element::Link { selector, .. } => selector.clone(),
        _ => unreachable!(),
    };
    handler.click_by_selector(&selector).await.expect("Failed to click button");

    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    let page_text = handler.get_page_text().await.expect("Failed to get text");
    assert!(page_text.contains("Rust"));
}

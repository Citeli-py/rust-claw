use ai_agent::tools::web_browser::{WebDriverHandler, ElementType, ElementInfo};
use std::collections::HashMap;
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
    let web_driver_result = WebDriverHandler::new(true).await;
    assert!(web_driver_result.is_ok());

    let web_driver = web_driver_result.unwrap();
    web_driver.close().await.unwrap();
}

#[tokio::test]
async fn test_webdriver_handler_should_goto_url() {
    let web_driver_result = WebDriverHandler::new(true).await.unwrap();

    let url = "https://www.wikipedia.org/";
    web_driver_result.goto(url).await.unwrap();

    let current_url = web_driver_result.driver.current_url().await.unwrap();
    web_driver_result.close().await.unwrap();
    
    assert_eq!(url, current_url.as_str());
}

mod unit_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_element_info_serialization() {
        let info = ElementInfo {
            element_json: r#"{"element-6066-11e4-a52e-4f735466cecf":"test-id"}"#.to_string(),
            index: 0,
            tag_name: "button".to_string(),
            text: "Click Me".to_string(),
            css_selector: "button".to_string(),
            attributes: HashMap::new(),
            element_type: ElementType::Button,
            role: Some("button".to_string()),
            is_clickable: true,
            is_fillable: false,
            is_displayed: true,
            bounding_box: None,
        };

        let serialized = serde_json::to_string(&info).expect("Failed to serialize ElementInfo");
        let deserialized: ElementInfo = serde_json::from_str(&serialized).expect("Failed to deserialize ElementInfo");

        assert!(deserialized.element_json.contains("test-id"));
        assert_eq!(deserialized.tag_name, "button");
        assert_eq!(deserialized.element_type, ElementType::Button);
        assert!(deserialized.is_clickable);
        assert!(!deserialized.is_fillable);
    }

    #[test]
    fn test_element_type_serialization() {
        let button = ElementType::Button;
        let serialized = serde_json::to_string(&button).unwrap();
        assert_eq!(serialized, "\"Button\"");

        let deserialized: ElementType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, ElementType::Button);
    }

    #[test]
    fn test_classify_element_button() {
        use ai_agent::tools::web_browser::classify_element;
        let attrs = HashMap::new();
        let elem_type = classify_element("button", &attrs);
        assert_eq!(elem_type, ElementType::Button);
    }

    #[test]
    fn test_classify_element_input_text() {
        use ai_agent::tools::web_browser::classify_element;
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "text".to_string());
        let elem_type = classify_element("input", &attrs);
        assert_eq!(elem_type, ElementType::Input);
    }

    #[test]
    fn test_classify_element_checkbox() {
        use ai_agent::tools::web_browser::classify_element;
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "checkbox".to_string());
        let elem_type = classify_element("input", &attrs);
        assert_eq!(elem_type, ElementType::Checkbox);
    }

    #[test]
    fn test_is_fillable_input_text() {
        use ai_agent::tools::web_browser::is_fillable_element;
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "text".to_string());
        assert!(is_fillable_element("input", &attrs));
    }

    #[test]
    fn test_is_fillable_checkbox() {
        use ai_agent::tools::web_browser::is_fillable_element;
        let mut attrs = HashMap::new();
        attrs.insert("type".to_string(), "checkbox".to_string());
        assert!(!is_fillable_element("input", &attrs));
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
    assert!(!elements.is_empty(), "Page elements should not be empty");
}

#[tokio::test]
async fn test_find_clickable_elements() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;
    let clickable = handler.find_clickable_elements().await.expect("Failed to find clickable elements");
    assert!(!clickable.is_empty(), "Should have clickable elements");
    assert!(clickable.iter().any(|e| matches!(e.element_type, ElementType::Button | ElementType::Link)), "Should have buttons or links");
}

#[tokio::test]
async fn test_find_fillable_elements() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;
    let fillable = handler.find_fillable_elements().await.expect("Failed to find fillable elements");
    assert!(!fillable.is_empty(), "Should have fillable elements");
    assert!(fillable.iter().any(|e| matches!(e.element_type, ElementType::Input | ElementType::TextArea)), "Should have inputs or textareas");
}

#[tokio::test]
async fn test_get_page_text_exposes_content() {
    let handler = WebDriverHandler::new(true).await.expect("Failed to create handler");
    handler.goto("https://www.wikipedia.org/").await.expect("Failed to goto");
    let text = handler.get_page_text().await.expect("Failed to get page text");
    assert!(text.contains("Wikipedia"), "Page text should contain 'Wikipedia'");
    handler.close().await.expect("Failed to close");
}

#[tokio::test]
async fn test_click_and_fill_workflow() {
    let handler = shared_handler().lock().await;
    reset_and_goto(&handler, "https://www.wikipedia.org/").await;

    // Find and fill search input
    let fillable = handler.find_fillable_elements().await.expect("Failed to find fillable");
    let search_input = fillable.iter().find(|e| e.element_type == ElementType::Input).expect("Should find search input");
    handler.fill_element(&search_input.element_json, "Rust programming").await.expect("Failed to fill input");

    // Find and click search button
    let clickable = handler.find_clickable_elements().await.expect("Failed to find clickable");
    let search_btn = clickable.iter().find(|e| e.text.to_lowercase().contains("search") || 
        e.attributes.get("aria-label").map(|s| s.to_lowercase().contains("search")).unwrap_or(false))
        .expect("Should find search button");
    handler.click_element(&search_btn.element_json).await.expect("Failed to click button");

    // Wait for page load
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Verify result
    let page_text = handler.get_page_text().await.expect("Failed to get text");
    assert!(page_text.contains("Rust"), "Page should contain 'Rust' after search");
}

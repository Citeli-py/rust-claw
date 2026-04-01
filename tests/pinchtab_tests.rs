use std::time::Duration;

use ai_agent::PinchTab;

#[tokio::test]
async fn test_create_instance() {

    let browser = PinchTab::new().await.unwrap();

    assert!(!browser.instance_id.is_empty());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_open_tab() {

    let browser = PinchTab::new().await.unwrap();

    let resp = browser
        .open_tab(Some("https://example.com".to_string()))
        .await;

    assert!(resp.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_navigation() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("about:blank".to_string()))
        .await
        .unwrap();

    println!("{:?}", tab);

    let result = browser
        .navigate(tab.tabId, "https://www.google.com/".to_string())
        .await;

    assert!(result.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_navigation_without_tab_id() {

    let browser = PinchTab::new().await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;

    let result = browser
        .navigate("".to_string(), "https://www.google.com/".to_string())
        .await;

    println!("{}", &result.as_ref().unwrap());

    let res = browser.snapshot("".to_string()).await;

    
    println!("{}", res.unwrap());

    assert!(result.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_snapshot() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("https://example.com".to_string()))
        .await
        .unwrap();

    let snapshot = browser.snapshot(tab.tabId).await;

    assert!(snapshot.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_page_text() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("https://example.com".to_string()))
        .await
        .unwrap();

    let text = browser.text(tab.tabId).await;

    assert!(text.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_screenshot() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("https://example.com".to_string()))
        .await
        .unwrap();

    let screenshot = browser.screenshot(tab.tabId).await;

    assert!(screenshot.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_pdf_export() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("https://example.com".to_string()))
        .await
        .unwrap();

    let pdf = browser.pdf(tab.tabId).await;

    assert!(pdf.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_close_tab() {

    let browser = PinchTab::new().await.unwrap();

    let tab = browser
        .open_tab(Some("https://example.com".to_string()))
        .await
        .unwrap();

    let result = browser.close_tab(tab.tabId).await;

    assert!(result.is_ok());
    let _ = browser.close().await;
}

#[tokio::test]
async fn test_list_tabs() {

    let browser = PinchTab::new().await.unwrap();

    //tokio::time::sleep(Duration::from_secs(2)).await;

    let res = browser.get_tabs().await;
    assert!(res.is_ok());

    let tabs = res.unwrap();

    assert!(tabs.len() > 0);
    assert!(tabs[0].id != String::new());

    let _ = browser.close().await;
}
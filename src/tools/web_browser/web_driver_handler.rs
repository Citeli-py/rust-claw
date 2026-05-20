use std::error::Error;
use thirtyfour::prelude::*;
use serde_json::Value;
use async_trait::async_trait;
use crate::tools::web_browser::types::*;

#[async_trait]
pub trait WebDriverHandlerInterface: Send + Sync {
    async fn goto(&self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn get_page_text(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
    async fn find_clickable_elements(&self) -> Result<Vec<Element>, Box<dyn Error + Send + Sync>>;
    async fn find_fillable_elements(&self) -> Result<Vec<Element>, Box<dyn Error + Send + Sync>>;
    async fn click_by_selector(&self, selector: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn fill_by_selector(&self, selector: &str, text: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub struct WebDriverHandler {
    pub driver: WebDriver,
}

#[async_trait]
impl WebDriverHandlerInterface for WebDriverHandler {
    async fn goto(&self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.goto(url).await?;
        Ok(())
    }

    async fn get_page_text(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = self.driver.find(By::Tag("body")).await?;
        Ok(body.text().await?)
    }

    async fn find_clickable_elements(&self) -> Result<Vec<Element>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css(CLICKABLE_SELECTOR)).await?;
        Ok(Self::elements_to_infos(&self.driver, elements).await)
    }

    async fn find_fillable_elements(&self) -> Result<Vec<Element>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css(FILLABLE_SELECTOR)).await?;
        Ok(Self::elements_to_infos(&self.driver, elements).await)
    }

    async fn click_by_selector(&self, selector: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let elem = self.driver.find(By::Css(selector)).await?;
        elem.click().await?;
        Ok(())
    }

    async fn fill_by_selector(&self, selector: &str, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let elem = self.driver.find(By::Css(selector)).await?;
        elem.clear().await?;
        elem.send_keys(text).await?;
        Ok(())
    }
}

impl WebDriverHandler {
    pub async fn new(headless: bool) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut caps = DesiredCapabilities::chrome();
        if headless {
            let _ = caps.set_headless();
        }
        Ok(Self { driver: WebDriver::managed(caps).await? })
    }

    pub async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.quit().await?;
        Ok(())
    }

    pub async fn get_page_elements(&self) -> Result<Vec<Element>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css("*")).await?;
        Ok(Self::elements_to_infos(&self.driver, elements).await)
    }

    async fn elements_to_infos(driver: &WebDriver, elements: Vec<WebElement>) -> Vec<Element> {
        if elements.is_empty() {
            return vec![];
        }

        let refs = Value::Array(
            elements.iter()
                .map(|e| serde_json::json!({ "element-6066-11e4-a52e-4f735466cecf": e.element_id().to_string() }))
                .collect(),
        );

        let data: Vec<Value> = match driver.execute(BATCH_SCRIPT, vec![refs]).await {
            Ok(ret) => serde_json::from_value(ret.json().clone()).unwrap_or_default(),
            Err(_)  => return vec![],
        };

        data.iter().enumerate()
            .filter_map(|(i, d)| to_element(d, i))
            .collect()
    }
}

fn to_element(d: &Value, index: usize) -> Option<Element> {
    if !d["displayed"].as_bool().unwrap_or(false) {
        return None;
    }

    let tag        = d["tag"].as_str()?;
    let text       = d["text"].as_str().unwrap_or("").to_string();
    let label      = d["label"].as_str().filter(|s| !s.is_empty()).map(str::to_string);
    let name       = d["name"].as_str().filter(|s| !s.is_empty()).map(str::to_string);
    let href       = d["href"].as_str().filter(|s| !s.is_empty()).map(str::to_string);
    let input_type = d["input_type"].as_str().unwrap_or("text");

    let selector = generate_css_selector(
        tag,
        d["id"].as_str(),
        d["class"].as_str(),
        d["name"].as_str(),
        index,
    );

    Some(match tag {
        "a"        => Element::Link     { index, selector, text, href },
        "button"   => Element::Button   { index, selector, text },
        "textarea" => Element::Textarea { index, selector, label },
        "select"   => Element::Select   { index, selector, label },
        "input"    => match input_type {
            "checkbox"                        => Element::Checkbox { index, selector, label, name },
            "radio"                           => Element::Radio    { index, selector, label, name },
            "button" | "submit" | "reset"     => Element::Button   { index, selector, text },
            _                                 => Element::Input    { index, selector, label },
        },
        _ => Element::Button { index, selector, text },
    })
}

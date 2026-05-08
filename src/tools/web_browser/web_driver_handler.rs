use std::error::Error;
use thirtyfour::prelude::*;
use std::collections::HashMap;
use serde_json::Value;
use crate::tools::web_browser::types::*;

const CLICKABLE_SELECTOR: &str = concat!(
    "button, a[href], input[type='button'], input[type='submit'], ",
    "[role='button'], [role='link'], [role='tab'], [role='menuitem'], ",
    "[role='option'], [onclick], ",
    "input[type='checkbox'], input[type='radio'], ",
    "[tabindex]:not([tabindex='-1'])"
);

const FILLABLE_SELECTOR: &str = concat!(
    "input:not([type='checkbox']):not([type='radio']):not([type='button']):not([type='submit']):not([type='reset']):not([type='image']):not([type='file']), ",
    "textarea, select"
);

pub struct WebDriverHandler {
    pub driver: WebDriver,
}

impl WebDriverHandler {
    pub async fn new(headless: bool) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut chrome_config = DesiredCapabilities::chrome();

        if headless {
            let _ = chrome_config.set_headless();
        }

        let driver = WebDriver::managed(chrome_config).await?;

        Ok(Self { driver })
    }

    pub async fn goto(&self, url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.goto(url).await?;
        Ok(())
    }

    pub async fn close(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.driver.quit().await?;
        Ok(())
    }

    pub async fn get_page_text(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = self.driver.find(By::Tag("body")).await?;
        let text = body.text().await?;
        Ok(text)
    }

    async fn elements_to_infos(elements: Vec<WebElement>) -> Vec<ElementInfo> {
        let mut result = Vec::new();
        for (i, elem) in elements.into_iter().enumerate() {
            if let Ok(info) = element_to_info(elem, i).await {
                result.push(info);
            }
        }
        result
    }

    pub async fn get_page_elements(&self) -> Result<Vec<ElementInfo>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css("*")).await?;
        let all = Self::elements_to_infos(elements).await;
        Ok(all.into_iter().filter(|e| e.is_displayed).collect())
    }

    pub async fn find_clickable_elements(&self) -> Result<Vec<ElementInfo>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css(CLICKABLE_SELECTOR)).await?;
        let all = Self::elements_to_infos(elements).await;
        Ok(all.into_iter().filter(|e| e.is_displayed).collect())
    }

    pub async fn find_fillable_elements(&self) -> Result<Vec<ElementInfo>, Box<dyn Error + Send + Sync>> {
        let elements = self.driver.find_all(By::Css(FILLABLE_SELECTOR)).await?;
        let all = Self::elements_to_infos(elements).await;
        Ok(all.into_iter().filter(|e| e.is_displayed).collect())
    }

    pub async fn click_element(&self, element_json: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let value: Value = serde_json::from_str(element_json)?;
        let handle = self.driver.handle().clone();
        let elem = WebElement::from_json(value, handle)?;
        elem.click().await?;
        Ok(())
    }

    pub async fn click_by_selector(&self, selector: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let elem = self.driver.find(By::Css(selector)).await?;
        elem.click().await?;
        Ok(())
    }

    pub async fn fill_element(&self, element_json: &str, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let value: Value = serde_json::from_str(element_json)?;
        let handle = self.driver.handle().clone();
        let elem = WebElement::from_json(value, handle)?;
        elem.clear().await?;
        elem.send_keys(text).await?;
        Ok(())
    }

    pub async fn fill_by_selector(&self, selector: &str, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let elem = self.driver.find(By::Css(selector)).await?;
        elem.clear().await?;
        elem.send_keys(text).await?;
        Ok(())
    }
}

async fn element_to_info(elem: WebElement, index: usize) -> Result<ElementInfo, Box<dyn Error + Send + Sync>> {
    let tag_name = elem.tag_name().await?;
    let text = elem.text().await.unwrap_or_default();
    let is_clickable = elem.is_clickable().await.unwrap_or(false);
    let is_displayed = elem.is_displayed().await.unwrap_or(false);

    let mut attributes = HashMap::new();

    for &attr_name in ["id", "class", "placeholder", "aria-label", "type", "name", "href", "value", "title", "role", "data-testid"].iter() {
        if let Ok(Some(val)) = elem.attr(attr_name).await {
            attributes.insert(attr_name.to_string(), val);
        }
    }

    let role = attributes.get("role").cloned().or_else(|| {
        let tag = tag_name.to_lowercase();
        if tag == "button" || tag == "a" {
            Some(tag)
        } else {
            None
        }
    });

    let rect = elem.rect().await.ok().map(|r| BoundingRect {
        x: r.x,
        y: r.y,
        width: r.width,
        height: r.height,
    });

    let element_type = classify_element(&tag_name, &attributes);
    let is_fillable = is_fillable_element(&tag_name, &attributes);
    let css_selector = generate_css_selector(&tag_name, &attributes, index);

    let element_id = elem.element_id();
    let element_json = serde_json::json!({
        "element-6066-11e4-a52e-4f735466cecf": element_id.to_string()
    }).to_string();

    Ok(ElementInfo {
        element_json,
        index,
        tag_name,
        text,
        css_selector,
        attributes,
        element_type,
        role,
        is_clickable,
        is_fillable,
        is_displayed,
        bounding_box: rect,
    })
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Button,
    Input,
    TextArea,
    Link,
    Select,
    Checkbox,
    Radio,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub element_json: String,
    pub index: usize,
    pub tag_name: String,
    pub text: String,
    pub css_selector: String,
    pub attributes: HashMap<String, String>,
    pub element_type: ElementType,
    pub role: Option<String>,
    pub is_clickable: bool,
    pub is_fillable: bool,
    pub is_displayed: bool,
    pub bounding_box: Option<BoundingRect>,
}

pub fn classify_element(tag_name: &str, attributes: &HashMap<String, String>) -> ElementType {
    match tag_name.to_lowercase().as_str() {
        "button" => ElementType::Button,
        "a" => ElementType::Link,
        "textarea" => ElementType::TextArea,
        "select" => ElementType::Select,
        "input" => {
            match attributes.get("type").map(|s| s.as_str()) {
                Some("checkbox") => ElementType::Checkbox,
                Some("radio") => ElementType::Radio,
                Some("button") | Some("submit") => ElementType::Button,
                _ => ElementType::Input,
            }
        }
        _ => ElementType::Other,
    }
}

pub fn is_fillable_element(tag_name: &str, attributes: &HashMap<String, String>) -> bool {
    match tag_name.to_lowercase().as_str() {
        "input" => {
            let input_type = attributes.get("type").map(|s| s.as_str()).unwrap_or("text");
            !matches!(input_type, "checkbox" | "radio" | "button" | "submit" | "reset" | "image" | "file")
        }
        "textarea" | "select" => true,
        _ => false,
    }
}

pub fn generate_css_selector(tag_name: &str, attributes: &HashMap<String, String>, index: usize) -> String {
    if let Some(id) = attributes.get("id") {
        if !id.is_empty() && !id.contains(char::is_whitespace) {
            return format!("#{}", id);
        }
    }
    let classes = attributes.get("class").map(|c| {
        let clean: Vec<&str> = c.split_whitespace().collect();
        clean.join(".")
    });
    if let Some(cls) = classes {
        if !cls.is_empty() {
            return format!("{}.{}", tag_name, cls);
        }
    }
    if let Some(name) = attributes.get("name") {
        if !name.is_empty() {
            return format!("{}[name='{}']", tag_name, name);
        }
    }
    format!("{}:nth-of-type({})", tag_name, index + 1)
}

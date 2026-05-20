use serde::{Deserialize, Serialize};

pub const CLICKABLE_SELECTOR: &str = concat!(
    "button, a[href], input[type='button'], input[type='submit'], ",
    "[role='button'], [role='link'], [role='tab'], [role='menuitem'], ",
    "[role='option'], [onclick], ",
    "input[type='checkbox'], input[type='radio'], ",
    "[tabindex]:not([tabindex='-1'])"
);

pub const FILLABLE_SELECTOR: &str = concat!(
    "input:not([type='checkbox']):not([type='radio']):not([type='button']):not([type='submit']):not([type='reset']):not([type='image']):not([type='file']), ",
    "textarea, select"
);

pub const BATCH_SCRIPT: &str = r#"
    var elems = arguments[0];
    var results = [];
    for (var i = 0; i < elems.length; i++) {
        var e = elems[i];
        var rect = e.getBoundingClientRect();
        var style = window.getComputedStyle(e);
        var displayed = style.display !== 'none'
            && style.visibility !== 'hidden'
            && style.opacity !== '0'
            && rect.width > 0 && rect.height > 0;
        results.push({
            tag:        e.tagName.toLowerCase(),
            text:       (e.textContent || '').trim().slice(0, 200),
            href:       e.getAttribute('href'),
            label:      e.getAttribute('aria-label') || e.getAttribute('placeholder'),
            name:       e.getAttribute('name'),
            input_type: e.getAttribute('type'),
            id:         e.getAttribute('id'),
            class:      e.getAttribute('class'),
            displayed:  displayed
        });
    }
    return results;
"#;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    Button   { index: usize, selector: String, text: String },
    Link     { index: usize, selector: String, text: String, href: Option<String> },
    Input    { index: usize, selector: String, label: Option<String> },
    Textarea { index: usize, selector: String, label: Option<String> },
    Select   { index: usize, selector: String, label: Option<String> },
    Checkbox { index: usize, selector: String, label: Option<String>, name: Option<String> },
    Radio    { index: usize, selector: String, label: Option<String>, name: Option<String> },
}

impl Element {
    pub fn to_llm_str(&self) -> String {
        match self {
            Element::Button { index, selector, text } =>
                format!("[{index}] Button   | {text} | {selector}"),
            Element::Link { index, selector, text, href } => {
                let dest = href.as_deref().map(|h| format!(" -> {h}")).unwrap_or_default();
                format!("[{index}] Link     | {text} | {selector}{dest}")
            }
            Element::Input { index, selector, label } =>
                format!("[{index}] Input    | {} | {selector}", label.as_deref().unwrap_or("")),
            Element::Textarea { index, selector, label } =>
                format!("[{index}] Textarea | {} | {selector}", label.as_deref().unwrap_or("")),
            Element::Select { index, selector, label } =>
                format!("[{index}] Select   | {} | {selector}", label.as_deref().unwrap_or("")),
            Element::Checkbox { index, selector, label, name } => {
                let grp = name.as_deref().map(|n| format!(" (name={n})")).unwrap_or_default();
                format!("[{index}] Checkbox | {} | {selector}{grp}", label.as_deref().unwrap_or(""))
            }
            Element::Radio { index, selector, label, name } => {
                let grp = name.as_deref().map(|n| format!(" (name={n})")).unwrap_or_default();
                format!("[{index}] Radio    | {} | {selector}{grp}", label.as_deref().unwrap_or(""))
            }
        }
    }
}

pub fn generate_css_selector(tag: &str, id: Option<&str>, class: Option<&str>, name: Option<&str>, index: usize) -> String {
    if let Some(id) = id.filter(|s| !s.is_empty() && !s.contains(char::is_whitespace)) {
        return format!("#{id}");
    }
    if let Some(cls) = class {
        let classes: Vec<&str> = cls.split_whitespace().collect();
        if !classes.is_empty() {
            return format!("{tag}.{}", classes.join("."));
        }
    }
    if let Some(name) = name.filter(|s| !s.is_empty()) {
        return format!("{tag}[name='{name}']");
    }
    format!("{tag}:nth-of-type({})", index + 1)
}

use std::collections::HashMap;
use serde_json::Value;
use std::sync::Mutex;

pub trait TrustedCommandsInterface: Send + Sync {
    fn is_trusted(&self, command: &str) -> bool;
    fn trust_command(&self, command: &str, save: bool);
}

pub struct TrustedCommands {
    tool_name: String,
    commands: Mutex<Vec<String>>,
    config_path: Option<String>,
}

impl TrustedCommands {
    pub fn new(tool_name: &str, trusted: HashMap<String, Vec<String>>) -> Self {
        TrustedCommands {
            commands: Mutex::new(trusted.get(tool_name).unwrap_or(&Vec::new()).to_vec()),
            tool_name: tool_name.to_string(),
            config_path: None,
        }
    }

    pub fn with_config_path(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    pub fn with_config_path_opt(mut self, path: Option<&str>) -> Self {
        self.config_path = path.map(|p| p.to_string());
        self
    }

    fn save_trust_command(&self, command: String, config_path: &String) {
        let content = std::fs::read_to_string(config_path).unwrap_or_default();
        let mut json: Value = serde_json::from_str(&content).unwrap_or(Value::Object(Default::default()));

        let trusted = json
            .as_object_mut()
            .and_then(|obj| {
                obj.entry("tools_trusted_commands")
                    .or_insert(Value::Object(Default::default()))
                    .as_object_mut()
            });

        if let Some(trusted) = trusted {
            let entry = trusted
                .entry(&self.tool_name)
                .or_insert(Value::Array(vec![]));

            if let Some(arr) = entry.as_array_mut() {
                if !arr.iter().any(|v| v.as_str() == Some(&command)) {
                    arr.push(Value::String(command));
                }
            }
        }

        let pretty = serde_json::to_string_pretty(&json).unwrap_or(content);
        let _ = std::fs::write(config_path, pretty);
    }

}

impl TrustedCommandsInterface for TrustedCommands {
    fn is_trusted(&self, command: &str) -> bool {
        let normalized = normalize(command);
        self.commands.lock().unwrap().iter().any(|c| normalize(c) == normalized)
    }

    fn trust_command(&self, command: &str, save: bool) {

        let normalized = normalize(command);
        self.commands.lock().unwrap().push(normalized.clone());

        let Some(config_path) = &self.config_path else {
            return;
        };

        if save {
            self.save_trust_command(normalized, config_path);
        }
    }
}

fn normalize(command: &str) -> String {
    serde_json::from_str::<Value>(command)
        .and_then(|v| serde_json::to_string(&v))
        .unwrap_or_else(|_| command.to_string())
}

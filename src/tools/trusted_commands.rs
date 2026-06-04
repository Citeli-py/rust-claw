use std::collections::HashMap;
use serde_json::Value;

pub trait TrustedCommandsInterface: Send + Sync {
    fn is_trusted(&self, command: &str) -> bool;
    fn trust_command(&self, command: &str);
}

pub struct TrustedCommands {
    tool_name: String,
    commands: Vec<String>,
    config_path: Option<String>,
}

impl TrustedCommands {
    pub fn new(tool_name: &str, trusted: HashMap<String, Vec<String>>) -> Self {
        TrustedCommands {
            commands: trusted.get(tool_name).unwrap_or(&Vec::new()).to_vec(),
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
}

impl TrustedCommandsInterface for TrustedCommands {
    fn is_trusted(&self, command: &str) -> bool {
        let normalized = normalize(command);
        self.commands.iter().any(|c| normalize(c) == normalized)
    }

    fn trust_command(&self, command: &str) {
        let Some(config_path) = &self.config_path else {
            return;
        };

        let normalized = normalize(command);

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
                if !arr.iter().any(|v| v.as_str() == Some(&normalized)) {
                    arr.push(Value::String(normalized));
                }
            }
        }

        let pretty = serde_json::to_string_pretty(&json).unwrap_or(content);
        let _ = std::fs::write(config_path, pretty);
    }
}

fn normalize(command: &str) -> String {
    serde_json::from_str::<Value>(command)
        .and_then(|v| serde_json::to_string(&v))
        .unwrap_or_else(|_| command.to_string())
}

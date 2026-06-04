use rig::completion::ToolDefinition;
use rig::tool::Tool;


use serde::{Deserialize, Serialize};
use serde_json::json;

use std::{collections::HashMap, process::Command};

#[derive(Debug)]
pub struct TerminalError;

impl std::fmt::Display for TerminalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Terminal execution error")
    }
}

impl std::error::Error for TerminalError {}

#[derive(Deserialize, Serialize)]
pub struct TerminalArgs {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Deserialize, Serialize)]
pub struct TerminalTool;

impl Tool for TerminalTool {
    const NAME: &'static str = "terminal";

    type Error = TerminalError;
    type Args = TerminalArgs;
    type Output = TerminalOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "terminal".to_string(),
            description: "Executes commands in the Linux terminal".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Command to execute in the terminal"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(&args.command)
            .output()
            .map_err(|_| TerminalError)?;

        Ok(TerminalOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

use crate::config::TerminalConfig;
use crate::tools::{TrustedCommands, confirmed_tool::{ConfirmationMode, ConfirmedTool}};

impl TerminalTool {
    pub fn build(
        cfg: TerminalConfig,
        mode: ConfirmationMode,
        config_path: Option<&str>,
    ) -> ConfirmedTool<TerminalTool, TrustedCommands> {
        let trusted = TrustedCommands::new("terminal", HashMap::from([
            ("terminal".to_string(), cfg.trusted),
        ])).with_config_path_opt(config_path);

        let mut tool = ConfirmedTool::new(TerminalTool, trusted);
        tool.set_mode(mode);
        tool
    }
}
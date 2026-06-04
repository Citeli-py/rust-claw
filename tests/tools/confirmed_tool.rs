use ai_agent::tools::confirmed_tool::{ConfirmedTool, ConfirmationMode, ConfirmedToolError};
use ai_agent::tools::trusted_commands::TrustedCommands;
use rig::tool::Tool;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StringProcessorArgs {
    pub text: String,
    pub operation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StringProcessorOutput {
    pub result: String,
}

use rig::completion::ToolDefinition;
use serde_json::json;

pub struct StringProcessorTool;

impl Tool for StringProcessorTool {
    const NAME: &'static str = "string_processor";

    type Error = ConfirmedToolError;
    type Args = StringProcessorArgs;
    type Output = StringProcessorOutput;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Processa strings (uppercase, lowercase, reverse)".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "operation": { "type": "string" }
                },
                "required": ["text", "operation"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = match args.operation.as_str() {
            "uppercase" => args.text.to_uppercase(),
            "lowercase" => args.text.to_lowercase(),
            "reverse" => args.text.chars().rev().collect(),
            _ => return Err(ConfirmedToolError { message: "Unknown operation".to_string() }),
        };

        Ok(StringProcessorOutput { result })
    }
}

#[tokio::test]
async fn test_wrapped_tool_must_show_their_description() {
    let trusted = TrustedCommands::new("string_processor", HashMap::new());
    let wrapped = ConfirmedTool::new(StringProcessorTool, trusted);

    // 🔍 Verifica se definition ainda funciona
    let def = wrapped.definition("".to_string()).await;

    assert_eq!(def.name, "string_processor");
    assert!(def.description.contains("Processa strings (uppercase, lowercase, reverse)"));

    // 🔍 Verifica se parâmetros existem
    let params = def.parameters;

    assert!(params["properties"]["text"].is_object());
    assert!(params["properties"]["operation"].is_object());
}

#[tokio::test]
async fn test_wrapped_tool_must_return_the_same_as_tool_in_success() {
    let trusted = TrustedCommands::new("string_processor", HashMap::new());
    let mut wrapped = ConfirmedTool::new(StringProcessorTool, trusted);
    wrapped.set_mode(ConfirmationMode::AlwaysAllow);

    let args = serde_json::json!({
        "text": "hello",
        "operation": "uppercase"
    });

    let args: StringProcessorArgs = serde_json::from_value(args).unwrap();

    let wrapped_result = wrapped.call(args.clone()).await.unwrap();

    let result = StringProcessorTool.call(args).await.unwrap();
    assert_eq!(wrapped_result.result, result.result);
}

#[tokio::test]
async fn test_wrapped_tool_must_return_the_same_error_as_tool() {
    let trusted = TrustedCommands::new("string_processor", HashMap::new());
    let mut wrapped = ConfirmedTool::new(StringProcessorTool, trusted);
    wrapped.set_mode(ConfirmationMode::AlwaysAllow);

    let args = serde_json::json!({
        "text": "hello",
        "operation": "invalid_operation"
    });

    let args: StringProcessorArgs = serde_json::from_value(args).unwrap();

    let result = wrapped.call(args).await;

    assert!(result.is_err());

    let err = result.err().unwrap().to_string();

    assert!(err.contains("Unknown operation"));
}

#[tokio::test]
async fn test_wrap_tool_confirmation_denied() {
    let trusted = TrustedCommands::new("string_processor", HashMap::new());
    let mut wrapped = ConfirmedTool::new(StringProcessorTool, trusted);
    wrapped.set_mode(ConfirmationMode::AlwaysDeny);

    let args = StringProcessorArgs {
        text: "hello".into(),
        operation: "reverse".into(),
    };

    let result = wrapped.call(args).await;

    assert!(result.is_err());

    let err = result.unwrap_err();

    assert!(err.to_string().contains("blocked"));
}
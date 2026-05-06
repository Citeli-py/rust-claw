use rig::tool::Tool;
use rig::completion::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::{io, io::Write};

#[derive(Debug)]
pub struct ConfirmedToolError {
    pub message: String,
}

impl std::fmt::Display for ConfirmedToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConfirmedToolError {}

#[derive(PartialEq, Clone, Copy)]
pub enum ConfirmationMode {
    Ask,          // comportamento normal
    AlwaysAllow,  // usado em testes ou bypass
    AlwaysDeny,   // útil pra testes e segurança
}

pub struct ConfirmedTool<T>
where
    T: Tool,
    T::Args: Serialize,
    T::Error: std::fmt::Display,
{
    inner: T,
    pub tool_name: String,
    pub mode: ConfirmationMode,
}

impl<T> ConfirmedTool<T>
where
    T: Tool,
    T::Args: Serialize,
    T::Error: std::fmt::Display,
{
    pub fn new(tool: T) -> Self {
        Self {
            tool_name: T::NAME.to_string(),
            inner: tool,
            mode: ConfirmationMode::Ask,
        }
    }

    pub fn set_mode(&mut self, mode: ConfirmationMode) {
        self.mode = mode;
    }
}

impl<T> Tool for ConfirmedTool<T>
where
    T: Tool + Send + Sync,
    T::Args: Serialize + for<'de> Deserialize<'de> + Send,
    T::Output: Serialize + for<'de> Deserialize<'de>,
    T::Error: std::fmt::Display + std::error::Error + Send + Sync + 'static,
{
    const NAME: &'static str = T::NAME;

    type Error = ConfirmedToolError;
    type Args = T::Args;
    type Output = T::Output;

    async fn definition(&self, prompt: String) -> ToolDefinition {
        self.inner.definition(prompt).await
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if self.mode == ConfirmationMode::AlwaysDeny {
            return Err(ConfirmedToolError {
                message: "[BLOCKED] Tool execution blocked by user".to_string(),
            });
        }

        let command = serde_json::to_string_pretty(&args)
            .unwrap_or("Unknown command".to_string());

        if self.mode == ConfirmationMode::AlwaysAllow
            || confirm_tool_use(&self.tool_name, &command)
        {
            return self.inner.call(args).await.map_err(|e| ConfirmedToolError {
                message: e.to_string(),
            });
        }

        Err(ConfirmedToolError {
            message: "[BLOCKED] Tool execution blocked by user".to_string(),
        })
    }
}

pub fn confirm_tool_use(tool: &str, command: &str) -> bool {
    println!("\n🤖 Pensando...\n");

    println!("⚠️ O agente quer usar a ferramenta:\n");
    println!("Tool: {}", tool);
    println!("Comando: {}\n", command);

    println!("[y] sim | [n] não");

    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim(), "y")
}

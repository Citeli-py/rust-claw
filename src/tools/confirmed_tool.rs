use rig::tool::Tool;
use rig::completion::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

use crate::tools::TrustedCommandsInterface;

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

enum ConfirmationDecision {
    Allow,
    Deny,
    Trust,
}

pub struct ConfirmedTool<T, C>
where
    T: Tool,
    T::Args: Serialize,
    T::Error: std::fmt::Display,
    C: TrustedCommandsInterface,
{
    inner: T,
    pub tool_name: String,
    pub mode: ConfirmationMode,
    pub trusted_commands: C,
}

impl<T, C> ConfirmedTool<T, C>
where
    T: Tool,
    T::Args: Serialize,
    T::Error: std::fmt::Display,
    C: TrustedCommandsInterface,
{
    pub fn new(tool: T, trusted_commands: C) -> Self {
        Self {
            tool_name: T::NAME.to_string(),
            inner: tool,
            mode: ConfirmationMode::Ask,
            trusted_commands,
        }
    }

    pub fn set_mode(&mut self, mode: ConfirmationMode) {
        self.mode = mode;
    }
}

impl<T, C> Tool for ConfirmedTool<T, C>
where
    T: Tool + Send + Sync,
    T::Args: Serialize + for<'de> Deserialize<'de> + Send,
    T::Output: Serialize + for<'de> Deserialize<'de>,
    T::Error: std::fmt::Display + std::error::Error + Send + Sync + 'static,
    C: TrustedCommandsInterface + Send + Sync,
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

        let is_trusted = self.trusted_commands.is_trusted(&command);

        let should_run = self.mode == ConfirmationMode::AlwaysAllow || is_trusted || {
            match confirm_tool_use(&self.tool_name, &command) {
                ConfirmationDecision::Allow => true,
                ConfirmationDecision::Trust => {
                    self.trusted_commands.trust_command(&command);
                    true
                }
                ConfirmationDecision::Deny => false,
            }
        };

        if should_run {
            return self.inner.call(args).await.map_err(|e| ConfirmedToolError {
                message: e.to_string(),
            });
        }

        Err(ConfirmedToolError {
            message: "[BLOCKED] Tool execution blocked by user".to_string(),
        })
    }
}

fn confirm_tool_use(tool: &str, command: &str) -> ConfirmationDecision {
    println!("\n🤖 Pensando...\n");
    println!("⚠️ O agente quer usar a ferramenta:\n");
    println!("Tool: {}", tool);
    println!("Comando: {}\n", command);
    println!("[y] sim | [n] não | [t] sim e salvar como confiável");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "y" => return ConfirmationDecision::Allow,
            "n" => return ConfirmationDecision::Deny,
            "t" => return ConfirmationDecision::Trust,
            _ => println!("Opção inválida. Use y, n ou t."),
        }
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "rustclaw",
    version,
    about = "CLI for agent-based automation",
    long_about = "Rustclaw is a CLI tool to create, manage and interact with AI agents."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new agent project
    Create {
        /// Name of the agent
        name: String,
    },

    /// Run a one-time command with an agent
    Run {
        /// Agent name
        name: String,

        /// Message to send to the agent
        message: String,
    },

    /// Start an interactive chat session
    Chat {
        /// Agent name
        name: String,
    },

    /// List all available agents
    List,
}
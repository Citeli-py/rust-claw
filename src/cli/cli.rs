use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustclaw")]
#[command(about = "CLI para automação com agentes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Cria um novo projeto
    Create {
        nome: String,
    },

    /// Executa um comando no projeto
    Run {
        nome: String,
        mensagem: String,
    },

    /// Abre chat interativo
    Chat {
        nome: String,
    },

    // Lista os seus agentes
    List,
}

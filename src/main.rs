use std::io::{self, Write};

use ai_agent::{AgentFactory, ModelProvider};
use ai_agent::cli::{chat::chat, create_agent::create_agent, load_agent::load_agent};

const GEMINI_MODEL: &str = "gemini-2.5-flash-lite";
const OLLAMA_MODEL: &str = "qwen3.5:2b";

use dotenvy::dotenv;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rustclaw")]
#[command(about = "CLI para automação com agentes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv()?;

    println!("Model loaded!");

    let cli = Cli::parse();

    match cli.command {
        Commands::Create { nome } => {
            create_agent(&nome);
        }

        Commands::Run { nome, mensagem } => {
            println!("Rodando {} com mensagem: {}", nome, mensagem);
        }

        Commands::Chat { nome } => {
            println!("Abrindo chat para: {}", nome);
            let mut agent = load_agent(&nome).await;
            chat(&mut agent).await?;
        }
    }

    Ok(())

}
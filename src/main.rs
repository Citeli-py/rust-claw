
use ai_agent::cli::{
    chat::chat, 
    create_agent::create_agent, 
    load_agent::load_agent, 
    run_agent::run_agent,
    list_agents::list_agents,
};

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
    List {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cli = Cli::try_parse()?;

    match cli.command {
        Commands::Create { nome } => {
            create_agent(&nome);
        }

        Commands::Run { nome, mensagem } => {
            println!("Rodando {} com mensagem: {}", nome, mensagem);
            run_agent(&nome, &mensagem).await;
        }

        Commands::Chat { nome } => {
            println!("Abrindo chat para: {}", nome);
            let mut agent = load_agent(&nome).await;
            chat(&mut agent).await?;
        }

        Commands::List{} => {
            list_agents();
        }
    }

    Ok(())

}
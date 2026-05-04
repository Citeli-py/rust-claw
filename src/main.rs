
use ai_agent::cli::{
    chat::chat, 
    create_agent::create_agent, 
    load_agent::load_agent, 
    run_agent::run_agent,
    list_agents::list_agents,
};

use ai_agent::{Cli, Commands};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let cli = Cli::parse();

    match cli.command {
        Commands::Create { name } => {
            create_agent(&name);
        }

        Commands::Run { name, message } => {
            println!("Running {} agent", name);
            run_agent(&name, &message).await;
        }

        Commands::Chat { name } => {
            println!("Opening chat with {} agent", name);
            let mut agent = load_agent(&name).await;
            chat(&mut agent).await?;
        }

        Commands::List{} => {
            list_agents();
        }
    }

    Ok(())

}
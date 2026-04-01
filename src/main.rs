use std::io::{self, Write};

mod tools;
mod agents;

use ai_agent::{AgentFactory, ModelProvider};

//mod multi_provider_agent;
//use crate::multi_provider_agent::{MultiProviderAgent};

mod pinchtab;

const GEMINI_MODEL: &str = "gemini-2.5-flash-lite";
const OLLAMA_MODEL: &str = "qwen3.5:2b";

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv()?;
    let config = ai_agent::Config::from_env();
    println!("Loading model...");

    let mut agent = AgentFactory::create_agent(
        ModelProvider::Ollama,
        OLLAMA_MODEL, 
        &config.gemini_api_key, 
        Vec::new()
    ).await?;

    println!("Model loaded!");


    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut question = String::new();
        io::stdin().read_line(&mut question)?;
        let question = question.trim();

        let resposta = agent.stream(question).await;

        if let Err(e) = resposta {
            eprintln!("failed to generate response: {e}");
            continue;
        }

        // println!("AI> {}\n", resposta.unwrap());
        //println!("{:?}", agent.history());
    }
}
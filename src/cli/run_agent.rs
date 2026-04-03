use crate::AgentConfig;
use crate::cli::load_agent::load_agent;

pub async fn run_agent(name: &str, prompt: &str) {

    let mut agent = load_agent(name).await;
    let resp = agent.chat(prompt).await;

    match resp {
        Ok(resp) => println!("{}", resp),
        Err(e) => eprintln!("Error running agent {}:\n\t{e}", name)
    }
}
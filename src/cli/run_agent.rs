use crate::cli::load_agent::load_agent;

pub async fn run_agent(name: &str, prompt: &str) {

    let mut agent = load_agent(name).await;
    let result = agent.stream(prompt).await;

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Error running agent\n\t{e}")
    };
}
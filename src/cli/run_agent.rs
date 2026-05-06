use crate::cli::load_agent::load_agent;

pub async fn run_agent(name: &str, prompt: &str, yolo: bool) {

    let mut agent = load_agent(name, yolo).await;
    let result = agent.stream(prompt).await;

    match result {
        Ok(_) => {}
        Err(e) => eprintln!("Error running agent\n\t{e}")
    };
}
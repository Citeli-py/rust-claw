use crate::agents::AgentInterface;
use crate::config::AgentConfig;
use crate::AgentFactory;


pub async fn load_agent(name: &str, yolo: bool) -> Box<dyn AgentInterface> {

    println!("Loading model...");
    let mut config = AgentConfig::from_path(&format!("agents/{}", name)).unwrap();
    config.yolo = yolo; // Sobrescreve com o valor da CLI

    let agent = AgentFactory::from_config(
        config,
        Vec::new()
    ).await.unwrap();

    agent
}
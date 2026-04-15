use crate::agents::{AgentInterface, AgentConfig};
use crate::{AgentFactory, ModelProvider};
use std::fs;



pub async fn load_agent(name: &str) -> Box<dyn AgentInterface> {

    println!("Loading model...");
    let config = AgentConfig::from_path(&format!("agents/{}", name)).unwrap();


    let mut agent = AgentFactory::from_config(
        config,
        Vec::new()
    ).await.unwrap();

    agent
}
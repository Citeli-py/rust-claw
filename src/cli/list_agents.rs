use crate::AgentConfig;
use std::fs;

pub fn list_agents() {
    println!("Agents:\n");
    let paths = fs::read_dir("agents").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let path_str = path.to_str().unwrap();
        let agent_config = AgentConfig::from_path(path_str).unwrap();

        println!("{}", agent_config.to_string());
    }
}
use ai_agent::AgentFactory;
use ai_agent::AgentConfig;
use ai_agent::AgentConfigJson;
use ai_agent::ModelProvider;
use dotenvy;
use dotenvy::dotenv;
use tempfile::TempDir;
use std::fs;

#[tokio::test]
async fn test_create_ollama_agent() {
    let agent = AgentFactory::create_agent(
        ModelProvider::Ollama, 
        "qwen3.5:0.8b", 
        "", 
        "",
        Vec::new(),
        false,
        vec![],
        Default::default()
    ).await;

    assert!(agent.is_err() == false);
}

#[tokio::test]
async fn test_create_gemini_agent() {
    dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").unwrap();
    let agent = AgentFactory::create_agent(
        ModelProvider::Gemini, 
        "gemini-2.5-flash-lite", 
        &api_key, 
        "",
        Vec::new(),
        false,
        vec![],
        Default::default()
    ).await;

    assert!(agent.is_err() == false);
}

#[tokio::test]
async fn test_history_returns_messages() {
    let result_agent = AgentFactory::create_agent(
        ModelProvider::Ollama,
        "qwen3.5:2b",
        "",
        "",
        Vec::new(),
        false,
        vec![],
        Default::default()
    ).await;

    let mut agent = match result_agent {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error to create agent:\n{e}");
            return;
        }
    };

    assert!(agent.history().is_empty());

    let _ = agent.chat("say hi").await;

    assert!(!agent.history().is_empty());
    assert_eq!(agent.history().len() % 2, 0);
}

#[tokio::test]
async fn test_clean_history_clears_messages() {
    let result_agent = AgentFactory::create_agent(
        ModelProvider::Ollama,
        "qwen3.5:2b",
        "",
        "",
        Vec::new(),
        false,
        vec![],
        Default::default()
    ).await;

    let mut agent = match result_agent {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error to create agent:\n{e}");
            return;
        }
    };

    let _ = agent.chat("say hi").await;
    assert!(!agent.history().is_empty());

    agent.clean_history();
    assert!(agent.history().is_empty());
}

#[tokio::test]
async fn test_factory_from_config_propagates_yolo() {
    dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").unwrap();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    let config_json = AgentConfigJson {
        provider: "gemini".to_string(),
        model: "gemini-2.5-flash-lite".to_string(),
        api_key: api_key.clone(),
        tools: vec![],
        tools_config: Default::default()
    };

    fs::write(&config_path, serde_json::to_string_pretty(&config_json).unwrap()).unwrap();
    fs::write(&prompt_path, "You are a test agent").unwrap();

    let mut config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();
    config.yolo = true;

    let agent = AgentFactory::from_config(config, Vec::new()).await;
    assert!(agent.is_ok());
}

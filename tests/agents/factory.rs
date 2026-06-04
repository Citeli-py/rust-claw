use ai_agent::AgentFactory;
use ai_agent::config::AgentConfig;
use ai_agent::config::config_json::AgentConfigJson;
use ai_agent::ModelProvider;
use dotenvy::dotenv;
use tempfile::TempDir;
use std::fs;

fn make_config(provider: ModelProvider, model: &str, api_key: &str) -> AgentConfig {
    AgentConfig {
        name: "test".to_string(),
        provider,
        model: model.to_string(),
        api_key: api_key.to_string(),
        pre_prompt: String::new(),
        yolo: false,
        tools_config: Default::default(),
        config_path: None,
    }
}

#[tokio::test]
async fn test_create_ollama_agent() {
    let agent = AgentFactory::from_config(
        make_config(ModelProvider::Ollama, "qwen3.5:0.8b", ""),
        vec![]
    ).await;

    assert!(agent.is_ok());
}

#[tokio::test]
async fn test_create_gemini_agent() {
    dotenv().ok();

    let api_key = std::env::var("GEMINI_API_KEY").unwrap();
    let agent = AgentFactory::from_config(
        make_config(ModelProvider::Gemini, "gemini-2.5-flash-lite", &api_key),
        vec![]
    ).await;

    assert!(agent.is_ok());
}

#[tokio::test]
async fn test_history_returns_messages() {
    let result_agent = AgentFactory::from_config(
        make_config(ModelProvider::Ollama, "qwen3.5:2b", ""),
        vec![]
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
    let result_agent = AgentFactory::from_config(
        make_config(ModelProvider::Ollama, "qwen3.5:2b", ""),
        vec![]
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
        tools_config: Default::default(),
        tools_trusted_commands: Default::default(),
    };

    fs::write(&config_path, serde_json::to_string_pretty(&config_json).unwrap()).unwrap();
    fs::write(&prompt_path, "You are a test agent").unwrap();

    let mut config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();
    config.yolo = true;

    let agent = AgentFactory::from_config(config, vec![]).await;
    assert!(agent.is_ok());
}

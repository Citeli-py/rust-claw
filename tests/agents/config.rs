use ai_agent::AgentConfig;
use ai_agent::ModelProvider;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_match_provider_invalid_returns_none() {
    assert!(ModelProvider::from_str("").is_none());
    assert!(ModelProvider::from_str("unknown").is_none());
    assert!(ModelProvider::from_str("openai").is_none());
    assert!(ModelProvider::from_str("azure").is_none());
    assert!(ModelProvider::from_str("123").is_none());
}

#[test]
fn test_agent_config_with_tools() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    // Cria config.json com tools especificadas
    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": ["terminal"]
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Cria PROMPT.md necessário
    fs::write(&prompt_path, "# Test Agent").unwrap();

    // Carrega config
    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    // Verifica se tools foi carregado corretamente
    assert_eq!(config.tools.len(), 1);
    assert_eq!(config.tools[0], "terminal");
    assert!(!config.yolo);
}

#[test]
fn test_agent_config_without_tools_uses_default() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    // Cria config.json SEM o campo tools
    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key"
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Cria PROMPT.md necessário
    fs::write(&prompt_path, "# Test Agent").unwrap();

    // Carrega config
    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    // Verifica se usou o default (["terminal"])
    assert_eq!(config.tools.len(), 1);
    assert_eq!(config.tools[0], "terminal");
}

#[test]
fn test_agent_config_with_multiple_tools() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    // Cria config.json com múltiplas tools
    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": ["terminal", "browser", "file"]
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Cria PROMPT.md necessário
    fs::write(&prompt_path, "# Test Agent").unwrap();

    // Carrega config
    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    // Verifica se todas as tools foram carregadas
    assert_eq!(config.tools.len(), 3);
    assert_eq!(config.tools[0], "terminal");
    assert_eq!(config.tools[1], "browser");
    assert_eq!(config.tools[2], "file");
}

#[test]
fn test_agent_config_empty_tools_array() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    // Cria config.json com tools vazio
    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": []
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Cria PROMPT.md necessário
    fs::write(&prompt_path, "# Test Agent").unwrap();

    // Carrega config
    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    // Verifica se tools está vazio
    assert!(config.tools.is_empty());
}

#[test]
fn test_from_path_invalid_json_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    // Cria config.json com JSON malformado
    fs::write(&config_path, "{ provider: broken, }").unwrap();
    fs::write(&prompt_path, "# Test").unwrap();

    let result = AgentConfig::from_path(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_from_path_missing_prompt_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Cria config.json válido mas NÃO cria PROMPT.md
    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key"
    }"#;
    fs::write(&config_path, config_content).unwrap();

    let result = AgentConfig::from_path(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

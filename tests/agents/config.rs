use ai_agent::config::AgentConfig;
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

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": ["terminal"]
    }"#;
    fs::write(&config_path, config_content).unwrap();
    fs::write(&prompt_path, "# Test Agent").unwrap();

    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    assert_eq!(config.tools_config.tools.len(), 1);
    assert_eq!(config.tools_config.tools[0].name(), "terminal");
    assert!(!config.yolo);
}

#[test]
fn test_agent_config_without_tools_uses_default() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key"
    }"#;
    fs::write(&config_path, config_content).unwrap();
    fs::write(&prompt_path, "# Test Agent").unwrap();

    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    assert!(config.tools_config.tools.is_empty());
}

#[test]
fn test_agent_config_with_multiple_tools() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": ["terminal", "web_browser"]
    }"#;
    fs::write(&config_path, config_content).unwrap();
    fs::write(&prompt_path, "# Test Agent").unwrap();

    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    assert_eq!(config.tools_config.tools.len(), 2);
    assert_eq!(config.tools_config.tools[0].name(), "terminal");
    assert_eq!(config.tools_config.tools[1].name(), "web_browser");
}

#[test]
fn test_agent_config_empty_tools_array() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": []
    }"#;
    fs::write(&config_path, config_content).unwrap();
    fs::write(&prompt_path, "# Test Agent").unwrap();

    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    assert!(config.tools_config.tools.is_empty());
}

#[test]
fn test_agent_config_trusted_commands_loaded() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key",
        "tools": ["terminal"],
        "tools_trusted_commands": {
            "terminal": ["{\"command\":\"free -h\"}"]
        }
    }"#;
    fs::write(&config_path, config_content).unwrap();
    fs::write(&prompt_path, "# Test Agent").unwrap();

    let config = AgentConfig::from_path(temp_dir.path().to_str().unwrap()).unwrap();

    let terminal = &config.tools_config.tools[0];
    assert_eq!(terminal.name(), "terminal");
    assert_eq!(terminal.trusted_commands().len(), 1);
}

#[test]
fn test_from_path_invalid_json_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let prompt_path = temp_dir.path().join("PROMPT.md");

    fs::write(&config_path, "{ provider: broken, }").unwrap();
    fs::write(&prompt_path, "# Test").unwrap();

    let result = AgentConfig::from_path(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_from_path_missing_prompt_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
        "provider": "gemini",
        "model": "gemini-pro",
        "api_key": "test-key"
    }"#;
    fs::write(&config_path, config_content).unwrap();

    let result = AgentConfig::from_path(temp_dir.path().to_str().unwrap());
    assert!(result.is_err());
}

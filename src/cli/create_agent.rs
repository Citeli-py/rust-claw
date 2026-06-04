use std::fs::File;
use std::io::{Write, stdin, stdout};
use crate::config::config_json::AgentConfigJson;
use crate::provider::ModelProvider;

fn create_files(base_path: &str) {
    let prompt_path = format!("{}/PROMPT.md", base_path);
    let mut prompt_file = File::create(prompt_path)
        .expect("Erro ao criar PROMPT.md");
    prompt_file.write_all(b"# System Prompt\n\nDescribe your agent here.").unwrap();

    let config_path = format!("{}/config.json", base_path);
    let mut config_file = File::create(config_path)
        .expect("Erro ao criar config.json");
    config_file.write_all(b"{}").unwrap();
}

fn ask(question: &str) -> String {
    print!("{}", question);
    stdout().flush().unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn save_config(base_path: &str, provider: &str, model: &str, api_key: &str) {
    let config_path = format!("{}/config.json", base_path);

    let config = AgentConfigJson {
        model: model.to_string(),
        provider: provider.to_string(),
        api_key: api_key.to_string(),
        tools: vec![],
        tools_config: Default::default(),
        tools_trusted_commands: Default::default(),
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    std::fs::write(config_path, json).expect("Erro ao salvar config");
}

pub fn create_agent(name: &str) {
    let base_path = format!("agents/{}", name);

    std::fs::create_dir_all(&base_path).expect("Erro ao criar diretório");

    let provider = ModelProvider::from_str(
        &ask("Provider (gemini/ollama/openrouter/groq): ")
    ).unwrap();

    let model = ask("Model: ");

    create_files(&base_path);
    save_config(&base_path, &provider.to_string(), &model, "");

    println!("✅ Agente '{}' criado com sucesso!", name);
}

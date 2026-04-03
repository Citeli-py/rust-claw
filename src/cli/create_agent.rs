use std::fs::File;
use std::io::{Write, stdin, stdout};
use crate::{AgentConfig, AgentConfigJson};

fn create_files(base_path: &str) {
    // PROMPT.md
    let prompt_path = format!("{}/PROMPT.md", base_path);
    let mut prompt_file = File::create(prompt_path)
        .expect("Erro ao criar PROMPT.md");

    prompt_file.write_all(b"# System Prompt\n\nDescribe your agent here.")
        .unwrap();

    // config.json (vazio por enquanto)
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


fn save_config(base_path: &str, config: &AgentConfig) {
    let config_path = format!("{}/config.json", base_path);

    let config = AgentConfigJson {
        model: config.model.clone(),
        provider: config.provider.to_string(),
        api_key: config.api_key.clone()
    };

    let json = serde_json::to_string_pretty(&config).unwrap();

    std::fs::write(config_path, json)
        .expect("Erro ao salvar config");
}

pub fn create_agent(name: &str) {
    let base_path = format!("agents/{}", name);

    // cria pasta
    std::fs::create_dir_all(&base_path)
        .expect("Erro ao criar diretório");

    // pergunta config
    let provider = AgentConfig::match_provider(
        &ask("Provider (gemini/ollama/openrouter/groq): ")
    ).unwrap();

    let model = ask("Model: ");

    let config = AgentConfig {
        provider,
        model,
        api_key: String::new(),
        pre_prompt: String::new(),
    };

    // cria arquivos
    create_files(&base_path);

    // salva config
    save_config(&base_path, &config);

    println!("✅ Agente '{}' criado com sucesso!", name);
}
use ai_agent::AgentFactory;
use ai_agent::ModelProvider;
use ai_agent::AgentConfig;

#[tokio::test]
async fn test_create_ollama_agent() {

    let agent = AgentFactory::create_agent(
        ModelProvider::Ollama, 
        "qwen3.5:0.8b", 
        "", 
        Vec::new()
    ).await;

    assert!(agent.is_err() == false);
}


#[tokio::test]
async fn test_create_gemini_agent() {
    let config = Config::from_env();

    let agent = AgentFactory::create_agent(
        ModelProvider::Gemini, 
        "gemini-2.5-flash-lite", 
        &config.gemini_api_key, 
        Vec::new()
    ).await;

    assert!(agent.is_err() == false);
}

#[tokio::test]
async fn test_response_from_gemini() {

   let config = Config::from_env();
    
    let result_agent = AgentFactory::create_agent(
        ModelProvider::Gemini, 
        "gemini-2.5-flash-lite", 
        &config.gemini_api_key, 
        Vec::new()
    ).await;

    let mut agent = match result_agent {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error to create agent:\n{e}");
            return;
        }
    };

    let result = agent.chat("Seja breve, fale apenas oi").await;

    match result {
        Ok(resp) =>  assert!(resp.to_lowercase().contains("oi")),
        Err(_) => panic!()
    }

}


#[tokio::test]
async fn test_response_from_ollama() {

    let result_agent = AgentFactory::create_agent(
        ModelProvider::Ollama, 
        "qwen3.5:0.8b", 
        "", 
        Vec::new()
    ).await;

    let mut agent = match result_agent {
        Ok(agent) => agent,
        Err(e) => {
            eprintln!("Error to create agent:\n{e}");
            return;
        }
    };

    let result = agent.chat("Seja breve, fale apenas oi").await;

    match result {
        Ok(resp) =>  assert!(resp.to_lowercase().contains("oi")),
        Err(_) => panic!()
    }

}
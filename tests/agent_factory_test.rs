use ai_agent::AgentFactory;
use ai_agent::ModelProvider;
use ai_agent::AgentConfig;
use dotenvy;
use dotenvy::dotenv;

#[tokio::test]
async fn test_create_ollama_agent() {

    let agent = AgentFactory::create_agent(
        ModelProvider::Ollama, 
        "qwen3.5:0.8b", 
        "", 
        "",
        Vec::new()
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
        Vec::new()
    ).await;

    assert!(agent.is_err() == false);
}

#[tokio::test]
async fn test_response_from_gemini() {
    dotenv().ok();
    let api_key = std::env::var("GEMINI_API_KEY").unwrap();
    
    let result_agent = AgentFactory::create_agent(
        ModelProvider::Gemini, 
        "gemini-2.5-flash-lite", 
        &api_key, 
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

    let result = agent.chat("Be quick, say \"TEST\"").await;

    match result {
        Ok(resp) =>  assert!(resp.to_lowercase().contains("test")),
        Err(e) => {
            eprintln!("Error trying to generate response with gemini:\n\t{e}");
            panic!()
        }
    }

}


#[tokio::test]
async fn test_response_from_ollama() {

    let result_agent = AgentFactory::create_agent(
        ModelProvider::Ollama, 
        "qwen3.5:2b", 
        "", 
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

    let result = agent.chat("Be quick, say \"TEST\"").await;

    match result {
        Ok(resp) =>  {
            println!("Response from ollama: {}", resp);
            assert!(resp.to_lowercase().contains("test"))
        }    
        Err(e) => {
            eprintln!("Error trying to generate response with ollama:\n\t{e}");
            panic!()
        }
    }

}
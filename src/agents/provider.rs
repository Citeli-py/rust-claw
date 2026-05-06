pub enum ModelProvider {
    Ollama,
    Groq,
    Gemini,
    OpenRouter,
}

impl ModelProvider {
    pub fn to_string(&self) -> String {
        match self {
            ModelProvider::Gemini => "gemini".to_string(),
            ModelProvider::Ollama => "ollama".to_string(),
            ModelProvider::Groq => "groq".to_string(),
            ModelProvider::OpenRouter => "openrouter".to_string(),
        }
    }
}

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

    pub fn from_str(provider_str: &str) -> Option<ModelProvider> {
        match provider_str.to_lowercase().as_str() {
            "gemini" => Some(ModelProvider::Gemini),
            "ollama" => Some(ModelProvider::Ollama),
            "groq" => Some(ModelProvider::Groq),
            "openrouter" => Some(ModelProvider::OpenRouter),
            _ => None,
        }
    }
}

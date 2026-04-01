use std::env;

pub struct Config {
    pub gemini_api_key: String,
    pub openrouter_api_key: String,
    pub groq_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            gemini_api_key: env::var("GEMINI_API_KEY")
                .unwrap_or("".to_string()),
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .unwrap_or("".to_string()),
            groq_api_key: env::var("GROQ_API_KEY")
                .unwrap_or("".to_string()),
        }
    }
}
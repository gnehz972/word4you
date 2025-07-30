use anyhow::Result;

#[async_trait::async_trait]
pub trait AiClient {
    async fn get_text_explanation(&self, text: &str, prompt_template: &str) -> Result<String>;
    async fn test_connection(&self) -> Result<bool>;
}

pub enum AiProvider {
    Gemini,
    Qwen,
}

impl std::str::FromStr for AiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gemini" => Ok(AiProvider::Gemini),
            "qwen" => Ok(AiProvider::Qwen),
            _ => Err(format!("Unknown AI provider: {}", s)),
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Gemini => write!(f, "gemini"),
            AiProvider::Qwen => write!(f, "qwen"),
        }
    }
}

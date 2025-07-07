use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub gemini_api_key: String,
    pub vocabulary_notebook_file: String,
    pub git_remote_url: Option<String>,
    pub gemini_prompt_template: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        let _ = dotenv();

        let gemini_api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow!("GEMINI_API_KEY not found in environment variables. Please set it in your .env file or environment."))?;

        let vocabulary_notebook_file = env::var("VOCABULARY_NOTEBOOK_FILE").unwrap_or_else(|_| "vocabulary_notebook.md".to_string());
        let git_remote_url = env::var("GIT_REMOTE_URL").ok();

        let gemini_prompt_template = r#"
Please provide a comprehensive explanation for the English word "{word}" in the following format:

## {word}

*[IPA phonetic symbols]*

> [Brief English definition]

**[Simplified Chinese meaning, NO pinyin]**

- [One example sentence using the word]
- [Chinese translation using only Simplified Chinese characters, NO pinyin]

*[one usage note or tip]*

Important formatting rules:
- Use only Simplified Chinese characters for Chinese translations
- Do NOT include pinyin (romanized Chinese) in any Chinese text
- Ensure the response is in proper markdown format
"#.to_string();

        Ok(Config {
            gemini_api_key,
            vocabulary_notebook_file,
            git_remote_url,
            gemini_prompt_template,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.gemini_api_key.is_empty() {
            return Err(anyhow!("GEMINI_API_KEY cannot be empty"));
        }
        Ok(())
    }
} 
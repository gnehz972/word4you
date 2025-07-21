use anyhow::{anyhow, Result};
use dotenvy::dotenv;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub gemini_api_key: String,
    pub gemini_model_name: String,
    pub gemini_prompt_template: String,
    pub vocabulary_notebook_file: String,
    pub git_enabled: bool,
    pub git_remote_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        let _ = dotenv();

        let gemini_api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow!("GEMINI_API_KEY not found in environment variables. Please set it in your .env file or environment."))?;

        // Gemini model name - default to gemini-2.0-flash-001
        let gemini_model_name =
            env::var("GEMINI_MODEL_NAME").unwrap_or_else(|_| "gemini-2.0-flash-001".to_string());

        let gemini_prompt_template = r#"
Please provide a comprehensive explanation for the English word "{word}" in the following format:

## {word}

*/{IPA phonetic symbols}/*

> {Brief English definition}

**{Simplified Chinese meaning, NO pinyin}**

- {One example sentence using the word}
- {Chinese translation of the English example}

*{one usage note or tip}*

Important formatting rules:
- Use Simplified Chinese, no Pinyin(romanized Chinese) included
- Ensure the response is in proper markdown format
"#
        .to_string();

        let vocabulary_base_dir = env::var("VOCABULARY_BASE_DIR")
            .map(|path| expand_tilde_path(&path))
            .unwrap_or_else(|_| {
                // Default to home directory
                env::var("HOME")
                    .unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()))
            });

        // Create word4you subdirectory path
        let mut word4you_dir = PathBuf::from(vocabulary_base_dir);
        word4you_dir.push("word4you");

        // Create vocabulary notebook file path
        let mut vocabulary_notebook_file = word4you_dir.clone();
        vocabulary_notebook_file.push("vocabulary_notebook.md");

        let vocabulary_notebook_file = vocabulary_notebook_file.to_string_lossy().to_string();

        // Git enabled control - default to false
        let git_enabled = env::var("GIT_ENABLED")
            .map(|val| val.to_lowercase() == "true" || val == "1" || val.to_lowercase() == "yes")
            .unwrap_or(false);

        let git_remote_url = env::var("GIT_REMOTE_URL").ok();

        Ok(Config {
            gemini_api_key,
            gemini_model_name,
            gemini_prompt_template,
            vocabulary_notebook_file,
            git_enabled,
            git_remote_url,
        })
    }
}

fn expand_tilde_path(path: &str) -> String {
    if path.starts_with('~') {
        let home_dir = env::var("HOME")
            .unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));
        path.replacen('~', &home_dir, 1)
    } else {
        path.to_string()
    }
}

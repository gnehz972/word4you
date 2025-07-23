use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

use crate::config_manager::{ConfigManager, UserConfig};

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
        // Load configuration from file if it exists, otherwise use defaults
        let user_config = if ConfigManager::config_exists() {
            ConfigManager::load_config()?
        } else {
            UserConfig::default()
        };
        
        // Get API key from environment variable first, then config file
        let gemini_api_key = env::var("WORD4YOU_GEMINI_API_KEY")
            .unwrap_or_else(|_| user_config.gemini_api_key.clone());
        
        if gemini_api_key.is_empty() {
            return Err(anyhow!("Gemini API key not found. Set WORD4YOU_GEMINI_API_KEY environment variable or run 'word4you config' to set up."));
        }

        // Get model name from environment variable first, then config file
        let gemini_model_name = env::var("WORD4YOU_GEMINI_MODEL_NAME")
            .unwrap_or_else(|_| user_config.gemini_model_name.clone());

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

        // Get vocabulary base directory from environment variable first, then config file
        let vocabulary_base_dir_raw = env::var("WORD4YOU_VOCABULARY_BASE_DIR")
            .unwrap_or_else(|_| user_config.vocabulary_base_dir.clone());
        let vocabulary_base_dir = expand_tilde_path(&vocabulary_base_dir_raw);

        // Create word4you subdirectory path
        let mut word4you_dir = PathBuf::from(vocabulary_base_dir);
        word4you_dir.push("word4you");
        
        // Create the directory if it doesn't exist
        if !word4you_dir.exists() {
            std::fs::create_dir_all(&word4you_dir)?;
        }

        // Create vocabulary notebook file path
        let mut vocabulary_notebook_file = word4you_dir.clone();
        vocabulary_notebook_file.push("vocabulary_notebook.md");

        let vocabulary_notebook_file = vocabulary_notebook_file.to_string_lossy().to_string();

        // Get git enabled from environment variable first, then config file
        let git_enabled = env::var("WORD4YOU_GIT_ENABLED")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(user_config.git_enabled);

        // Get git remote URL from environment variable first, then config file
        let git_remote_url = env::var("WORD4YOU_GIT_REMOTE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .or(user_config.git_remote_url);

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

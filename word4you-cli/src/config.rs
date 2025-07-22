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
        // Check if config file exists
        if !ConfigManager::config_exists() {
            return Err(anyhow!("Configuration not found. Run 'word4you config' to set up."));
        }
        
        // Load configuration from file
        let user_config = ConfigManager::load_config()?;
        
        // Get API key from config file
        let gemini_api_key = user_config.gemini_api_key.clone();
        if gemini_api_key.is_empty() {
            return Err(anyhow!("Gemini API key not found in configuration. Run 'word4you config' to set up."));
        }

        // Get model name from config file
        let gemini_model_name = user_config.gemini_model_name.clone();

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

        // Get vocabulary base directory from config
        let vocabulary_base_dir = expand_tilde_path(&user_config.vocabulary_base_dir);

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

        // Get git enabled from config
        let git_enabled = user_config.git_enabled;

        // Get git remote URL from config
        let git_remote_url = user_config.git_remote_url;

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

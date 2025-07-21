use anyhow::{anyhow, Result};
use dotenvy::dotenv;
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
        // First try to load from config file
        let user_config = if ConfigManager::config_exists() {
            ConfigManager::load_config()?
        } else {
            // If config file doesn't exist, try environment variables
            // Load .env file if it exists
            let _ = dotenv();
            
            // Create default config that will be populated from env vars
            UserConfig::default()
        };
        
        // Get API key from config file or environment
        let gemini_api_key = if !user_config.gemini_api_key.is_empty() {
            user_config.gemini_api_key
        } else {
            env::var("GEMINI_API_KEY")
                .map_err(|_| anyhow!("GEMINI_API_KEY not found in configuration or environment variables. Run 'word4you config' to set up."))?
        };

        // Get model name from config file or environment
        let gemini_model_name = if !user_config.gemini_model_name.is_empty() {
            user_config.gemini_model_name
        } else {
            env::var("GEMINI_MODEL_NAME").unwrap_or_else(|_| "gemini-2.0-flash-001".to_string())
        };

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

        // Get vocabulary base directory from config or environment
        let vocabulary_base_dir = if !user_config.vocabulary_base_dir.is_empty() {
            expand_tilde_path(&user_config.vocabulary_base_dir)
        } else {
            env::var("VOCABULARY_BASE_DIR")
                .map(|path| expand_tilde_path(&path))
                .unwrap_or_else(|_| {
                    // Default to home directory
                    env::var("HOME")
                        .unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()))
                })
        };

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
        
        // Create the vocabulary notebook file if it doesn't exist
        if !vocabulary_notebook_file.exists() {
            std::fs::write(&vocabulary_notebook_file, "# Word4You Vocabulary Notebook\n\n")?;
        }

        let vocabulary_notebook_file = vocabulary_notebook_file.to_string_lossy().to_string();

        // Get git enabled from config or environment
        let git_enabled = if ConfigManager::config_exists() {
            user_config.git_enabled
        } else {
            env::var("GIT_ENABLED")
                .map(|val| val.to_lowercase() == "true" || val == "1" || val.to_lowercase() == "yes")
                .unwrap_or(false)
        };

        // Get git remote URL from config or environment
        let git_remote_url = if ConfigManager::config_exists() {
            user_config.git_remote_url
        } else {
            env::var("GIT_REMOTE_URL").ok()
        };

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

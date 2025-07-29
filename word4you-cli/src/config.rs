use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

use crate::config_manager::ConfigManager;

#[derive(Debug, Clone)]
pub struct Config {
    pub ai_provider: String,
    pub gemini_api_key: String,
    pub gemini_model_name: String,
    pub qwen_api_key: String,
    pub qwen_model_name: String,
    pub prompt_template: String,
    pub vocabulary_notebook_file: String,
    pub git_enabled: bool,
    pub git_remote_url: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Check if WORD4YOU_GEMINI_API_KEY environment variable is set
        // If it is, load all configuration from environment variables
        // If not, fallback to loading all configuration from TOML config file
        
        let ai_provider = env::var("WORD4YOU_AI_PROVIDER")
            .unwrap_or_else(|_| "gemini".to_string());
        let gemini_api_key = env::var("WORD4YOU_GEMINI_API_KEY");
        let _qwen_api_key = env::var("WORD4YOU_QWEN_API_KEY");

        let (ai_provider, gemini_api_key, gemini_model_name, qwen_api_key, qwen_model_name, vocabulary_base_dir_raw, git_enabled, git_remote_url) = 
            if let Ok(gemini_key) = gemini_api_key {
                // Load all configuration from environment variables
                let gemini_model = env::var("WORD4YOU_GEMINI_MODEL_NAME")
                    .unwrap_or_else(|_| "gemini-2.0-flash-001".to_string());
                let qwen_key = env::var("WORD4YOU_QWEN_API_KEY")
                    .unwrap_or_else(|_| "".to_string());
                let qwen_model = env::var("WORD4YOU_QWEN_MODEL_NAME")
                    .unwrap_or_else(|_| "qwen-turbo".to_string());
                let vocab_dir = env::var("WORD4YOU_VOCABULARY_BASE_DIR")
                    .unwrap_or_else(|_| "~".to_string());
                let git_enabled = env::var("WORD4YOU_GIT_ENABLED")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(false);
                let git_url = env::var("WORD4YOU_GIT_REMOTE_URL")
                    .ok()
                    .filter(|s| !s.is_empty());
                
                (ai_provider, gemini_key, gemini_model, qwen_key, qwen_model, vocab_dir, git_enabled, git_url)
            } else {
                // Fallback to loading all configuration from TOML config file
                if !ConfigManager::config_exists() {
                    return Err(anyhow!(
                        "Configuration not found. Run 'word4you config' to update your configuration."
                    ));
                }
                
                let user_config = ConfigManager::load_config()?;
                
                // Check if we have at least one API key
                if user_config.gemini_api_key.is_empty() && user_config.qwen_api_key.is_empty() {
                    return Err(anyhow!(
                        "No API key found in configuration. Run 'word4you config' to update your configuration."
                    ));
                }
                
                (
                    user_config.ai_provider,
                    user_config.gemini_api_key,
                    user_config.gemini_model_name,
                    user_config.qwen_api_key,
                    user_config.qwen_model_name,
                    user_config.vocabulary_base_dir,
                    user_config.git_enabled,
                    user_config.git_remote_url,
                )
            };

        let prompt_template = r#"
**Role:** You are a bilingual dictionary assistant that provides structured explanations.
**Input Handling:**
- Detect language automatically (English/Chinese)
- Detect if input is a word, phrase, sentence, or paragraph

**Output Handling:**
- For words: Full structured output
- For phrases: Omit phonetics
- For sentences/paragraphs: Return only the original input and translation

**Output Structure:**
## [INPUT]

*/Phonetics/*

> Definition in English

**Translation**

- Example (source language)
- Example (target language)

*Usage Tip in English*

**Word Example:**
## resilience

*/rɪˈzɪliəns/*

> Capacity to recover quickly from difficulties.

**韧性；恢复力**

- Her resilience helped her overcome the crisis.
- 她的韧性帮助她度过了危机。

*Often describes emotional or physical toughness.*

Provide any word/phrase/sentence to generate the structured output:
[INSERT TEXT HERE]
"#
        .to_string();

        // Expand tilde path for vocabulary base directory
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

        Ok(Config {
            ai_provider,
            gemini_api_key,
            gemini_model_name,
            qwen_api_key,
            qwen_model_name,
            prompt_template,
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

use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

use crate::config_manager::ConfigManager;

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
        // Check if WORD4YOU_GEMINI_API_KEY environment variable is set
        // If it is, load all configuration from environment variables
        // If not, fallback to loading all configuration from TOML config file
        
        let gemini_api_key = env::var("WORD4YOU_GEMINI_API_KEY");

        let (gemini_api_key, gemini_model_name, vocabulary_base_dir_raw, git_enabled, git_remote_url) = 
            if let Ok(api_key) = gemini_api_key {
                // Load all configuration from environment variables
                let model_name = env::var("WORD4YOU_GEMINI_MODEL_NAME")
                    .unwrap_or_else(|_| "gemini-2.0-flash-001".to_string());
                let vocab_dir = env::var("WORD4YOU_VOCABULARY_BASE_DIR")
                    .unwrap_or_else(|_| "~".to_string());
                let git_enabled = env::var("WORD4YOU_GIT_ENABLED")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(false);
                let git_url = env::var("WORD4YOU_GIT_REMOTE_URL")
                    .ok()
                    .filter(|s| !s.is_empty());
                
                (api_key, model_name, vocab_dir, git_enabled, git_url)
            } else {
                // Fallback to loading all configuration from TOML config file
                if !ConfigManager::config_exists() {
                    return Err(anyhow!(
                        "Configuration not found. Run 'word4you config' to update your configuration."
                    ));
                }
                
                let user_config = ConfigManager::load_config()?;
                
                if user_config.gemini_api_key.is_empty() {
                    return Err(anyhow!(
                        "Gemini API key not found in configuration. Run 'word4you config' to update your configuration."
                    ));
                }
                
                (
                    user_config.gemini_api_key,
                    user_config.gemini_model_name,
                    user_config.vocabulary_base_dir,
                    user_config.git_enabled,
                    user_config.git_remote_url,
                )
            };

        let gemini_prompt_template = r#"
You are a helpful bilingual dictionary assistant. Your task is to provide a structured explanation for a given English word.

The user will provide a word. You must generate a response that strictly adheres to the following Markdown format and content structure:

1.  **Heading:** The word itself, as a Level 2 Markdown heading (`##`).
2.  **Phonetics:** The IPA phonetic transcription, in italics (`*/.../*`).
3.  **Definition:** A concise, primary English definition, as a Markdown blockquote (`> ...`).
4.  **Translation:** The main Chinese translation(s), in bold (`**...**`).
5.  **Example (EN):** A simple English example sentence, as a bullet point (`- ...`).
6.  **Example (CN):** The Chinese translation of the example sentence, also as a bullet point (`- ...`).
7.  **Usage Tip:** A brief, helpful tip about usage, nuance, or synonyms, in italics (`*...*`).

Do not include any introductory phrases like "Sure, here is the explanation...". Your response must start directly with the Level 2 heading.

---
**EXAMPLE OUTPUT FOR "NICE":**
## nice

*/naɪs/*

> Pleasant or agreeable.

**好的; 友善的; 美好的**

- She is a very nice person.
- 她是个非常好的人。

*Avoid overusing "nice" as it can be vague. Use more descriptive words to express your specific meaning.*
---

Now, generate the output for the following word: **[INSERT WORD HERE]**
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

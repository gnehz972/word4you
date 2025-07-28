use anyhow::{anyhow, Result};
use console::{style, Term};
use dialoguer::{Confirm, Input, Password};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub ai_provider: String,
    pub gemini_api_key: String,
    pub gemini_model_name: String,
    pub qwen_api_key: String,
    pub qwen_model_name: String,
    pub vocabulary_base_dir: String,
    pub git_enabled: bool,
    pub git_remote_url: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            ai_provider: "gemini".to_string(),
            gemini_api_key: String::new(),
            gemini_model_name: "gemini-2.0-flash-001".to_string(),
            qwen_api_key: String::new(),
            qwen_model_name: "qwen-turbo".to_string(),
            vocabulary_base_dir: "~".to_string(),
            git_enabled: false,
            git_remote_url: None,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    /// Get the config directory path
    pub fn get_config_dir() -> Result<PathBuf> {
        // Try to get HOME environment variable (Unix/macOS)
        let home_dir = std::env::var("HOME").or_else(|_| {
            // Fallback to USERPROFILE for Windows
            std::env::var("USERPROFILE")
        }).map_err(|_| anyhow!("Could not determine home directory"))?;
        
        let config_dir = PathBuf::from(home_dir).join(".config").join("word4you");
        Ok(config_dir)
    }

    /// Get the config file path
    pub fn get_config_file_path() -> Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    /// Check if config file exists
    pub fn config_exists() -> bool {
        if let Ok(config_path) = Self::get_config_file_path() {
            config_path.exists()
        } else {
            false
        }
    }

    /// Load configuration from file with backward compatibility
    pub fn load_config() -> Result<UserConfig> {
        let config_path = Self::get_config_file_path()?;
        
        if !config_path.exists() {
            return Ok(UserConfig::default());
        }

        let config_str = fs::read_to_string(config_path)?;
        
        // Try to parse with new format first
        match toml::from_str::<UserConfig>(&config_str) {
            Ok(config) => Ok(config),
            Err(_) => {
                // If that fails, try to parse with old format and migrate
                eprintln!("ℹ️  Migrating configuration from old format to new format...");
                let migrated_config = Self::migrate_old_config(&config_str)?;
                
                // Save the migrated configuration in new format
                if let Err(e) = Self::save_config(&migrated_config) {
                    eprintln!("Warning: Could not save migrated configuration: {}", e);
                } else {
                    eprintln!("✅ Configuration migrated successfully");
                }
                
                Ok(migrated_config)
            }
        }
    }

    /// Migrate old configuration format to new format
    fn migrate_old_config(config_str: &str) -> Result<UserConfig> {
        // Define the old config structure
        #[derive(Debug, Deserialize)]
        struct OldUserConfig {
            pub gemini_api_key: String,
            pub gemini_model_name: String,
            pub vocabulary_base_dir: String,
            pub git_enabled: bool,
            pub git_remote_url: Option<String>,
        }

        // Try to parse with old format
        let old_config: OldUserConfig = toml::from_str(config_str)?;
        
        // Convert to new format
        let new_config = UserConfig {
            ai_provider: "gemini".to_string(), // Default to gemini for old configs
            gemini_api_key: old_config.gemini_api_key,
            gemini_model_name: old_config.gemini_model_name,
            qwen_api_key: String::new(), // Empty for old configs
            qwen_model_name: "qwen-turbo".to_string(), // Default value
            vocabulary_base_dir: old_config.vocabulary_base_dir,
            git_enabled: old_config.git_enabled,
            git_remote_url: old_config.git_remote_url,
        };

        Ok(new_config)
    }

    /// Save configuration to file
    pub fn save_config(config: &UserConfig) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.toml");
        let config_str = toml::to_string_pretty(config)?;
        fs::write(config_path, config_str)?;
        
        Ok(())
    }

    /// Run the interactive configuration setup
    pub fn run_setup(term: &Term) -> Result<()> {
        term.write_line(&style("🔧 Word4You Configuration Setup").cyan().bold().to_string())?;
        term.write_line("Let's set up your Word4You configuration.")?;
        term.write_line("")?;

        // Load existing config if available
        let mut config = if Self::config_exists() {
            Self::load_config()?
        } else {
            UserConfig::default()
        };

        // AI Provider Selection
        term.write_line(&style("1. AI Provider Selection").yellow().to_string())?;
        term.write_line("Choose your preferred AI provider:")?;
        
        let provider_choices = vec!["gemini", "qwen"];
        let provider_selection = dialoguer::Select::new()
            .with_prompt("Select AI provider")
            .items(&provider_choices)
            .default(if config.ai_provider == "qwen" { 1 } else { 0 })
            .interact()?;
            
        config.ai_provider = provider_choices[provider_selection].to_string();
        term.write_line("")?;

        // Configuration based on selected provider
        if config.ai_provider == "gemini" {
            // Gemini Configuration
            term.write_line(&style("2. Gemini Configuration").yellow().to_string())?;
            term.write_line("You need a Google Gemini API key to use Gemini.")?;
            term.write_line("Get one at: https://aistudio.google.com/app/apikey")?;
            
            let gemini_api_key = if !config.gemini_api_key.is_empty() {
                let masked_key = format!("{}...", &config.gemini_api_key[..4]);
                
                if Confirm::new()
                    .with_prompt(format!("Current Gemini API key: {}. Update it?", masked_key))
                    .default(false)
                    .interact()?
                {
                    Password::new()
                        .with_prompt("Enter your Gemini API key")
                        .interact()?
                } else {
                    config.gemini_api_key.clone()
                }
            } else {
                Password::new()
                    .with_prompt("Enter your Gemini API key")
                    .interact()?
            };
            
            config.gemini_api_key = gemini_api_key;
            term.write_line("")?;

            // Gemini Model
            term.write_line("Select the Gemini model to use:")?;
            
            let model = Input::<String>::new()
                .with_prompt("Gemini model name")
                .default(config.gemini_model_name.clone())
                .interact()?;
                
            config.gemini_model_name = model;
            term.write_line("")?;
        } else if config.ai_provider == "qwen" {
            // QWEN Configuration
            term.write_line(&style("2. QWEN Configuration").yellow().to_string())?;
            term.write_line("You need a QWEN API key to use QWEN.")?;
            term.write_line("Get one at: https://dashscope.console.aliyun.com/")?;
            
            let qwen_api_key = if !config.qwen_api_key.is_empty() {
                let masked_key = format!("{}...", &config.qwen_api_key[..4]);
                
                if Confirm::new()
                    .with_prompt(format!("Current QWEN API key: {}. Update it?", masked_key))
                    .default(false)
                    .interact()?
                {
                    Password::new()
                        .with_prompt("Enter your QWEN API key")
                        .interact()?
                } else {
                    config.qwen_api_key.clone()
                }
            } else {
                Password::new()
                    .with_prompt("Enter your QWEN API key")
                    .interact()?
            };
            
            config.qwen_api_key = qwen_api_key;
            term.write_line("")?;

            // QWEN Model
            term.write_line("Select the QWEN model to use:")?;
            
            let qwen_model = Input::<String>::new()
                .with_prompt("QWEN model name")
                .default(config.qwen_model_name.clone())
                .interact()?;
                
            config.qwen_model_name = qwen_model;
            term.write_line("")?;
        }



        // Vocabulary Directory
        term.write_line(&style("3. Vocabulary Storage Location").yellow().to_string())?;
        term.write_line("Where do you want to store your vocabulary notebook?")?;
        
        let vocab_dir = Input::<String>::new()
            .with_prompt("Vocabulary base directory (~ for home directory)")
            .default(config.vocabulary_base_dir.clone())
            .interact()?;
            
        config.vocabulary_base_dir = vocab_dir;
        
        // Create vocabulary directory structure
        let expanded_path = if config.vocabulary_base_dir.starts_with('~') {
            let home_dir = std::env::var("HOME")
                .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));
            config.vocabulary_base_dir.replacen('~', &home_dir, 1)
        } else {
            config.vocabulary_base_dir.clone()
        };
        
        let mut word4you_dir = PathBuf::from(expanded_path);
        word4you_dir.push("word4you");
        
        // Create the directory if it doesn't exist
        if !word4you_dir.exists() {
            term.write_line(&format!("Creating directory: {}", word4you_dir.display()))?;
            fs::create_dir_all(&word4you_dir)?;
        }

        term.write_line("")?;

        // Git Integration
        term.write_line(&style("4. Git Integration").yellow().to_string())?;
        term.write_line("Would you like to enable Git integration for version control?")?;
        
        let git_enabled = Confirm::new()
            .with_prompt("Enable Git integration?")
            .default(config.git_enabled)
            .interact()?;
            
        config.git_enabled = git_enabled;

        // Git Remote URL (if Git is enabled)
        if git_enabled {
            let default_url = config.git_remote_url.clone().unwrap_or_else(|| "".to_string());
            
            let git_url = if default_url.is_empty() {
                Input::<String>::new()
                    .with_prompt("Git remote URL (leave empty to skip)")
                    .allow_empty(true)
                    .interact()?
            } else {
                Input::<String>::new()
                    .with_prompt("Git remote URL (leave empty to skip)")
                    .default(default_url)
                    .allow_empty(true)
                    .interact()?
            };
                
            config.git_remote_url = if git_url.is_empty() { None } else { Some(git_url) };
            
            // Initialize Git repository if enabled
            if let Some(url) = &config.git_remote_url {
                term.write_line(&format!("Git integration enabled with remote: {}", url))?;
                // Note: actual Git initialization is handled by the git_utils module
            }
        } else {
            config.git_remote_url = None;
        }

        // Save the configuration
        Self::save_config(&config)?;

        term.write_line("")?;
        term.write_line(&style("✅ Configuration saved successfully!").green().to_string())?;
        term.write_line("")?;
        term.write_line("You can now use Word4You with your configuration.")?;
        term.write_line("Run 'word4you' to start learning words!")?;

        Ok(())
    }

    /// Test the configuration
    pub fn test_config(term: &Term) -> Result<()> {
        if !Self::config_exists() {
            return Err(anyhow!("Configuration not found. Run 'word4you config' to set up."));
        }

        let config = Self::load_config()?;
        
        term.write_line(&style("📋 Current Configuration:").cyan().to_string())?;
        term.write_line(&format!("• AI Provider: {}", config.ai_provider))?;
        term.write_line(&format!("• Gemini API Key: {}...", &config.gemini_api_key[..4]))?;
        term.write_line(&format!("• Gemini Model: {}", config.gemini_model_name))?;
        term.write_line(&format!("• QWEN API Key: {}...", &config.qwen_api_key[..4]))?;
        term.write_line(&format!("• QWEN Model: {}", config.qwen_model_name))?;
        term.write_line(&format!("• Vocabulary Directory: {}", config.vocabulary_base_dir))?;
        term.write_line(&format!("• Git Integration: {}", if config.git_enabled { "Enabled" } else { "Disabled" }))?;
        
        if let Some(url) = &config.git_remote_url {
            term.write_line(&format!("• Git Remote URL: {}", url))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_old_config() {
        let old_config_str = r#"
gemini_api_key = "test_key_123"
gemini_model_name = "gemini-1.5-flash"
vocabulary_base_dir = "~/Documents"
git_enabled = true
git_remote_url = "https://github.com/user/repo.git"
"#;

        let migrated_config = ConfigManager::migrate_old_config(old_config_str).unwrap();
        
        assert_eq!(migrated_config.ai_provider, "gemini");
        assert_eq!(migrated_config.gemini_api_key, "test_key_123");
        assert_eq!(migrated_config.gemini_model_name, "gemini-1.5-flash");
        assert_eq!(migrated_config.qwen_api_key, "");
        assert_eq!(migrated_config.qwen_model_name, "qwen-turbo");
        assert_eq!(migrated_config.vocabulary_base_dir, "~/Documents");
        assert_eq!(migrated_config.git_enabled, true);
        assert_eq!(migrated_config.git_remote_url, Some("https://github.com/user/repo.git".to_string()));
    }

    #[test]
    fn test_migrate_old_config_invalid_toml() {
        let invalid_config_str = r#"
gemini_api_key = "test_key_123"
invalid_field = "should_fail"
"#;

        let result = ConfigManager::migrate_old_config(invalid_config_str);
        assert!(result.is_err());
    }
}
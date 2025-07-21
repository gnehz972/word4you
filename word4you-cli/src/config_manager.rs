use anyhow::{anyhow, Result};
use console::{style, Term};
use dialoguer::{Confirm, Input, Password};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub gemini_api_key: String,
    pub gemini_model_name: String,
    pub vocabulary_base_dir: String,
    pub git_enabled: bool,
    pub git_remote_url: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            gemini_api_key: String::new(),
            gemini_model_name: "gemini-2.0-flash-001".to_string(),
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

    /// Load configuration from file
    pub fn load_config() -> Result<UserConfig> {
        let config_path = Self::get_config_file_path()?;
        
        if !config_path.exists() {
            return Ok(UserConfig::default());
        }

        let config_str = fs::read_to_string(config_path)?;
        let config: UserConfig = toml::from_str(&config_str)?;
        Ok(config)
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

        // Gemini API Key
        term.write_line(&style("1. Gemini API Key").yellow().to_string())?;
        term.write_line("You need a Google Gemini API key to use Word4You.")?;
        term.write_line("Get one at: https://aistudio.google.com/app/apikey")?;
        
        let api_key = if !config.gemini_api_key.is_empty() {
            let masked_key = format!("{}...", &config.gemini_api_key[..4]);
            
            if Confirm::new()
                .with_prompt(format!("Current API key: {}. Update it?", masked_key))
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
        
        config.gemini_api_key = api_key;
        term.write_line("")?;

        // Gemini Model
        term.write_line(&style("2. Gemini Model").yellow().to_string())?;
        term.write_line("Select the Gemini model to use:")?;
        
        let model = Input::<String>::new()
            .with_prompt("Gemini model name")
            .with_initial_text(&config.gemini_model_name)
            .default(config.gemini_model_name.clone())
            .interact()?;
            
        config.gemini_model_name = model;
        term.write_line("")?;

        // Vocabulary Directory
        term.write_line(&style("3. Vocabulary Storage Location").yellow().to_string())?;
        term.write_line("Where do you want to store your vocabulary notebook?")?;
        
        let vocab_dir = Input::<String>::new()
            .with_prompt("Vocabulary base directory (~ for home directory)")
            .with_initial_text(&config.vocabulary_base_dir)
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
        
        // Create vocabulary notebook file if it doesn't exist
        let mut vocabulary_file = word4you_dir.clone();
        vocabulary_file.push("vocabulary_notebook.md");
        
        if !vocabulary_file.exists() {
            term.write_line(&format!("Creating vocabulary notebook: {}", vocabulary_file.display()))?;
            fs::write(&vocabulary_file, "# Word4You Vocabulary Notebook\n\n")?;
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
            
            let git_url = Input::<String>::new()
                .with_prompt("Git remote URL (leave empty to skip)")
                .allow_empty(true)
                .with_initial_text(&default_url)
                .interact()?;
                
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
        term.write_line(&format!("• Gemini API Key: {}...", &config.gemini_api_key[..4]))?;
        term.write_line(&format!("• Gemini Model: {}", config.gemini_model_name))?;
        term.write_line(&format!("• Vocabulary Directory: {}", config.vocabulary_base_dir))?;
        term.write_line(&format!("• Git Integration: {}", if config.git_enabled { "Enabled" } else { "Disabled" }))?;
        
        if let Some(url) = &config.git_remote_url {
            term.write_line(&format!("• Git Remote URL: {}", url))?;
        }
        
        Ok(())
    }
}
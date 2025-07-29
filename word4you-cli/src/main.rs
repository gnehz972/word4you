use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use text_processor::TextProcessor;

mod ai_client;
mod config;
mod config_manager;
mod gemini_client;
mod qwen_client;
mod git_section_sync;
mod git_utils;
mod prompt_templates;
mod utils;
mod text_processor;

use config::Config;
use config_manager::ConfigManager;

const INTRO: &str = r#"
Word4You - Language Learning Tool

Features:
• AI-powered explanations for words, phrases, and sentences using Google Gemini or QWEN
• Translations between English and Chinese with phonetic symbols
• Example sentences in both English and Chinese
• Automatic Git integration for version control
• Markdown-formatted vocabulary notebook
• Content update functionality (delete and replace)

Usage:
  word4you                           # Interactive mode (enter text one by one)
    word4you query <text>              # Learn a new English or Chinese word, phrase, or sentence
  word4you query <text> --provider gemini  # Use Gemini AI provider
  word4you query <text> --provider qwen    # Use QWEN AI provider
  word4you test                      # Test API connection
  word4you config                    # Set up or update configuration
  word4you config --show-vob-path      # Show the vocabulary notebook path
  word4you save <text> --content <content>  # Save content to vocabulary notebook
  word4you delete <text> [--timestamp <timestamp>]  # Delete content from vocabulary notebook, optionally by specific timestamp
  word4you update <text> --content <content> [--timestamp <timestamp>]  # Update content (delete if exists, then save)

Options:
  --raw                              # Output raw response from API without user interaction
  --provider <provider>              # AI provider to use (gemini or qwen)
"#;

#[derive(Parser)]
#[command(
    name = "word4you",
    about = "Learn English words with AI-powered explanations using Google Gemini",
    long_about = INTRO,
    version = "1.0.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Query a word, phrase, or sentence for learning
    Query {
        /// The text to learn (word, phrase, or sentence)
        word: String,

        /// Output raw response from API without user interaction
        #[arg(long)]
        raw: bool,

        /// AI provider to use (gemini or qwen)
        #[arg(long, value_enum)]
        provider: Option<String>,
    },

    /// Save content to vocabulary notebook
    Save {
        /// The text to save (word, phrase, or sentence)
        word: String,

        /// The content to save
        #[arg(long)]
        content: String,
    },

    /// Delete content from vocabulary notebook
    Delete {
        /// The text to delete (word, phrase, or sentence)
        word: String,

        /// Optional timestamp for the specific entry to delete
        #[arg(long)]
        timestamp: Option<String>,
    },

    /// Update content: delete if exists, then save new content
    Update {
        /// The text to update (word, phrase, or sentence)
        word: String,

        /// The new content to save
        #[arg(long)]
        content: String,

        /// Optional timestamp for the specific entry to update
        #[arg(long)]
        timestamp: Option<String>,
    },

    /// Test the API connection
    Test,
    
    /// Set up or update configuration
    Config {
        /// Show the vocabulary notebook path
        #[arg(long)]
        show_vob_path: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let term = Term::stdout();

    // Check if configuration is available
    // Priority: Environment variables > Config file
    let has_env_config = std::env::var("WORD4YOU_GEMINI_API_KEY").is_ok() || std::env::var("WORD4YOU_QWEN_API_KEY").is_ok();
    let has_file_config = ConfigManager::config_exists();
    
    // If neither environment variables nor config file exists, and not running config command, run onboarding
    if !has_env_config && !has_file_config && !matches!(cli.command, Some(Commands::Config { .. })) {
        term.write_line(&style("👋 Welcome to Word4You!").cyan().bold().to_string())?;
        term.write_line("It looks like this is your first time running Word4You.")?;
        term.write_line("Let's set up your configuration before we begin.")?;
        term.write_line("")?;
        
        if let Err(e) = ConfigManager::run_setup(&term) {
            eprintln!("❌ Configuration error: {}", e);
            term.write_line("You can run 'word4you config' later to set up your configuration.")?;
            return Ok(());
        }
        
        term.write_line("")?;
    }

    // Handle subcommands
    match &cli.command {
                Some(Commands::Query { word, raw, provider }) => {
            if let Err(e) = query_text(&term, word, *raw, provider.as_deref()).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Test) => {
            if let Err(e) = test_api_connection(&term).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Save { word, content }) => {
            if let Err(e) = save_text(&term, word, content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Delete { word, timestamp }) => {
            if let Err(e) = delete_text(&term, word, timestamp.as_deref()).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Update {
            word,
            content,
            timestamp,
        }) => {
            if let Err(e) = update_text(&term, word, content, timestamp.as_deref()).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Config { show_vob_path }) => {
            if *show_vob_path {
                // Show the vocabulary notebook path
                if let Err(e) = show_vocabulary_path(&term) {
                    eprintln!("❌ Error: {}", e);
                    return Ok(());
                }
            } else {
                // Run the regular configuration setup
                if let Err(e) = ConfigManager::run_setup(&term) {
                    eprintln!("❌ Configuration error: {}", e);
                    return Ok(());
                }
            }
        }
        None => {
            // Enter interactive mode when no subcommand provided
            if let Err(e) = interactive_mode(&term).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn query_text(term: &Term, text: &str, raw: bool, provider: Option<&str>) -> anyhow::Result<()> {
    // Validate configuration
    let mut config = Config::load()?;

    // Override provider if specified
    if let Some(provider) = provider {
        config.ai_provider = provider.to_string();
    }

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Process the text (prompt template is now determined automatically based on classification)
    processor.process_text(term, text, raw, "").await?;

    Ok(())
}

async fn save_text(term: &Term, text: &str, content: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Save the text
    processor.save_text(term, text, content)?;

    Ok(())
}

async fn delete_text(term: &Term, text: &str, timestamp: Option<&str>) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Delete the text
    processor.delete_text(term, text, timestamp)?;

    Ok(())
}

async fn update_text(
    term: &Term,
    text: &str,
    content: &str,
    timestamp: Option<&str>,
) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Update the text (delete if exists, then save)
    processor.update_text(term, text, content, timestamp)?;

    Ok(())
}

async fn test_api_connection(term: &Term) -> anyhow::Result<()> {
    let config = Config::load()?;
    let processor = TextProcessor::new(config);

    term.write_line("🔍 Testing API connection...")?;

    match processor.test_api_connection().await {
        Ok(true) => {
            term.write_line(
                &style("✅ API connection successful")
                    .green()
                    .to_string(),
            )?;
            Ok(())
        }
        Ok(false) => {
            term.write_line(&style("❌ API connection failed").red().to_string())?;
            return Ok(());
        }
        Err(e) => {
            term.write_line(&format!("❌ API connection error: {}", e))?;
            return Ok(());
        }
    }
}

fn show_vocabulary_path(_term: &Term) -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load()?;
    
    // Simply print the vocabulary notebook file path without any formatting
    // This makes it easier for scripts and other programs to parse the output
    println!("{}", config.vocabulary_notebook_file);
    
    Ok(())
}

async fn interactive_mode(term: &Term) -> anyhow::Result<()> {
    // Validate configuration first
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    term.write_line(
        &style("🎯 Welcome to Word4You Interactive Mode!")
            .cyan()
            .to_string(),
    )?;
    term.write_line("Enter words, phrases, or sentences to learn, or type 'exit' to quit.")?;
    term.write_line("")?;

    loop {
        // Get text input from user
        let input_text = match dialoguer::Input::<String>::new()
            .with_prompt("Enter text to learn (word, phrase, or sentence)")
            .allow_empty(false)
            .interact_text()
        {
            Ok(input) => input.trim().to_string(),
            Err(_) => {
                term.write_line("👋 Goodbye!")?;
                break;
            }
        };

        // Check for exit command
        let lower_input = input_text.to_lowercase();
        if lower_input == "exit" || lower_input == "quit" || lower_input == "q" {
            term.write_line("👋 Goodbye!")?;
            break;
        }

        // Skip empty input
        if input_text.is_empty() {
            term.write_line("❌ Please enter valid text.")?;
            continue;
        }

        // Process the text using the new classification system
        if let Err(e) = processor.process_text(term, &input_text, false, "").await {
            term.write_line(&format!("❌ Error processing text '{}': {}", input_text, e))?;
            term.write_line("Please try again with different text.")?;
            continue;
        }

        // After processing (save/skip), continue to next text
        term.write_line("")?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;
        term.write_line("")?;
    }

    Ok(())
}

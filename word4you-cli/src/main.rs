use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use word_processor::WordProcessor;

mod config;
mod gemini_client;
mod utils;
mod word_processor;
mod git_utils;
mod git_section_detector;
mod git_section_sync;

use config::Config;

const INTRO: &str = r#"
Word4You - English Word Learning Tool

Features:
• AI-powered word explanations using Google Gemini
• Chinese translations and phonetic symbols
• Example sentences in both English and Chinese
• Automatic Git integration for version control
• Markdown-formatted vocabulary notebook
• Word update functionality (delete and replace)

Usage:
  word4you                           # Interactive mode (enter words one by one)
  word4you query <word>              # Learn a new word
  word4you test                      # Test API connection
  word4you info                      # Show this information
  word4you learn <word>              # Learn a specific word
  word4you save <word> --content <content>  # Save word to vocabulary
  word4you delete <word> [--timestamp <timestamp>]  # Delete word from vocabulary, optionally by specific timestamp
  word4you update <word> --content <content> [--timestamp <timestamp>]  # Update word (delete if exists, then save)

Options:
  --raw                              # Output raw response from API without user interaction
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
    /// Query a word for learning
    Query {
        /// The word to learn
        word: String,
        
        /// Output raw response from API without user interaction
        #[arg(long)]
        raw: bool,
    },
    
    /// Test the API connection
    Test,
    
    /// Display application information
    Info,
    
    /// Learn a specific word
    Learn {
        /// The word to learn
        word: String,
        
        /// Output raw response from API without user interaction
        #[arg(long)]
        raw: bool,
    },
    
    /// Save word to vocabulary notebook
    Save {
        /// The word to save
        word: String,
        
        /// The content to save
        #[arg(long)]
        content: String,
    },
    
    /// Delete a word from vocabulary notebook
    Delete {
        /// The word to delete
        word: String,
        
        /// Optional timestamp for the specific word entry to delete
        #[arg(long)]
        timestamp: Option<String>,
    },
    
    /// Update a word: delete if exists, then save new content
    Update {
        /// The word to update
        word: String,
        
        /// The new content to save
        #[arg(long)]
        content: String,
        
        /// Optional timestamp for the specific word entry to update
        #[arg(long)]
        timestamp: Option<String>,
    },
    
    /// Synchronize vocabulary with remote using section-based logic
    Sync {
        /// Force sync even if there are conflicts
        #[arg(long)]
        force: bool,
    },
}



#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();
    let term = Term::stdout();

    // Handle subcommands
    match &cli.command {
        Some(Commands::Query { word, raw }) => {
            if let Err(e) = query_word(&term, word, *raw).await {
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
        Some(Commands::Info) => {
            show_info(&term);
        }
        Some(Commands::Learn { word, raw }) => {
            if let Err(e) = query_word(&term, word, *raw).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Save { word, content }) => {
            if let Err(e) = save_word(&term, word, content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Delete { word, timestamp }) => {
            if let Err(e) = delete_word(&term, word, timestamp.as_deref()).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Update { word, content, timestamp }) => {
            if let Err(e) = update_word(&term, word, content, timestamp.as_deref()).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Sync { force }) => {
            if let Err(e) = sync_vocabulary(&term, *force).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
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

async fn query_word(term: &Term, word: &str, raw: bool) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Process the word
    processor.process_word(term, word, raw).await?;
    
    Ok(())
}

async fn save_word(term: &Term, word: &str, content: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Save the word
    processor.save_word(term, word, content)?;
    
    Ok(())
}

async fn delete_word(term: &Term, word: &str, timestamp: Option<&str>) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Delete the word
    processor.delete_word(term, word, timestamp)?;
    
    Ok(())
}

async fn update_word(term: &Term, word: &str, content: &str, timestamp: Option<&str>) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Update the word (delete if exists, then save)
    processor.update_word(term, word, content, timestamp)?;
    
    Ok(())
}

async fn test_api_connection(term: &Term) -> anyhow::Result<()> {
    let config = Config::load()?;
    let processor = WordProcessor::new(config);
    
    term.write_line("🔍 Testing Gemini API connection...")?;
    
    match processor.test_api_connection().await {
        Ok(true) => {
            term.write_line(&style("✅ Gemini API connection successful").green().to_string())?;
            Ok(())
        }
        Ok(false) => {
            term.write_line(&style("❌ Gemini API connection failed").red().to_string())?;
            return Ok(());
        }
        Err(e) => {
            term.write_line(&format!("❌ API connection error: {}", e))?;
            return Ok(());
        }
    }
}

fn show_info(term: &Term) {
    term.write_line(&style(INTRO).cyan().to_string()).unwrap();
}

async fn sync_vocabulary(term: &Term, _force: bool) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    term.write_line("🔄 Starting section-aware vocabulary synchronization...")?;
    
    // Use section-aware synchronization
    git_utils::sync_with_section_awareness(
        &config.vocabulary_notebook_file,
        config.git_remote_url.as_deref(),
        config.ssh_private_key_path.as_deref(),
        config.ssh_public_key_path.as_deref()
    )?;
    
    Ok(())
}

async fn interactive_mode(term: &Term) -> anyhow::Result<()> {
    // Validate configuration first
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    term.write_line(&style("🎯 Welcome to Word4You Interactive Mode!").cyan().to_string())?;
    term.write_line("Enter words to learn, or type 'exit' to quit.")?;
    term.write_line("")?;
    
    loop {
        // Get word input from user
        let word = match dialoguer::Input::<String>::new()
            .with_prompt("Enter a word to learn")
            .allow_empty(false)
            .interact_text()
        {
            Ok(input) => input.trim().to_lowercase(),
            Err(_) => {
                term.write_line("👋 Goodbye!")?;
                break;
            }
        };
        
        // Check for exit command
        if word == "exit" || word == "quit" || word == "q" {
            term.write_line("👋 Goodbye!")?;
            break;
        }
        
        // Skip empty input
        if word.is_empty() {
            term.write_line("❌ Please enter a valid word.")?;
            continue;
        }
        
        // Process the word using existing functionality
        if let Err(e) = processor.process_word(term, &word, false).await {
            term.write_line(&format!("❌ Error processing word '{}': {}", word, e))?;
            term.write_line("Please try another word.")?;
            continue;
        }
        
        // After processing (save/skip), continue to next word
        term.write_line("")?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;
        term.write_line("")?;
    }
    
    Ok(())
}

 
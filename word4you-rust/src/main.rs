use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use word_processor::WordProcessor;

mod config;
mod gemini_client;
mod utils;
mod word_processor;

use config::Config;

#[derive(Parser)]
#[command(
    name = "word4you",
    about = "Learn English words with AI-powered explanations using Google Gemini",
    version = "1.0.0"
)]
struct Cli {
    /// The word to learn
    word: Option<String>,

    /// Output raw response from API without user interaction
    #[arg(long)]
    raw: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test the API connection
    Test,
    /// Display application information
    Info,
    /// Learn a specific word
    Learn {
        /// The word to learn
        word: String,
    },
    /// Save raw response to vocabulary notebook
    Save {
        /// The word being saved
        word: String,
        /// The raw response content to save
        content: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();
    let term = Term::stdout();

    match cli.command {
        Some(Commands::Test) => {
            if let Err(e) = test_api_connection(&term).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Info) => {
            show_info(&term);
        }
        Some(Commands::Learn { word }) => {
            if let Err(e) = query_word(&term, &word, cli.raw).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Save { word, content }) => {
            if let Err(e) = save_word(&term, &word, &content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        None => {
            if let Some(word) = cli.word {
                if let Err(e) = query_word(&term, &word, cli.raw).await {
                    eprintln!("❌ Error: {}", e);
                    return Ok(());
                }
            } else {
                // Show help if no word provided
                let _ = clap::Command::new("word4you").print_help();
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
    let info = r#"
Word4You - English Word Learning Tool

Features:
• AI-powered word explanations using Google Gemini
• Chinese translations and phonetic symbols
• Example sentences in both English and Chinese
• Automatic Git integration for version control
• Markdown-formatted vocabulary notebook

Usage:
  word4you <word>           # Learn a new word
  word4you test             # Test API connection
  word4you info             # Show this information
"#;

    term.write_line(&style(info).cyan().to_string()).unwrap();
}

 
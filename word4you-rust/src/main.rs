use clap::{Parser, Subcommand};
use console::{style, Term};
use std::process;

mod config;
mod gemini_client;
mod word_processor;
mod utils;

use config::Config;
use word_processor::WordProcessor;

#[derive(Parser)]
#[command(
    name = "word4you",
    about = "Learn English words with AI-powered explanations using Google Gemini",
    version = "1.0.0"
)]
struct Cli {
    /// The word to learn
    word: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Test the API connection
    Test,
    /// Display application information
    Info,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let term = Term::stdout();

    match cli.command {
        Some(Commands::Test) => {
            if let Err(e) = test_api_connection(&term).await {
                eprintln!("❌ Error: {}", e);
                process::exit(1);
            }
        }
        Some(Commands::Info) => {
            show_info(&term);
        }
        None => {
            if let Some(word) = cli.word {
                if let Err(e) = learn_word(&term, &word).await {
                    eprintln!("❌ Error: {}", e);
                    process::exit(1);
                }
            } else {
                // Show help if no word provided
                let _ = clap::Command::new("word4you").print_help();
            }
        }
    }
}

async fn learn_word(term: &Term, word: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Process the word
    processor.process_word(term, word).await?;
    
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
            process::exit(1);
        }
        Err(e) => {
            term.write_line(&format!("❌ API connection error: {}", e))?;
            process::exit(1);
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
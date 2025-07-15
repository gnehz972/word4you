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

    /// Test the API connection
    #[arg(long)]
    test: bool,

    /// Display application information
    #[arg(long)]
    info: bool,

    /// Learn a specific word
    #[arg(long)]
    learn: Option<String>,

    /// Save raw response to vocabulary notebook
    #[arg(long)]
    save: Option<String>,

    /// The content to save (used with --save)
    #[arg(long)]
    content: Option<String>,

    /// Delete a word from vocabulary notebook
    #[arg(long)]
    delete: Option<String>,
}



#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();
    let term = Term::stdout();

    // Handle different arguments
    if cli.test {
        if let Err(e) = test_api_connection(&term).await {
            eprintln!("❌ Error: {}", e);
            return Ok(());
        }
    } else if cli.info {
        show_info(&term);
    } else if let Some(word) = cli.learn {
        if let Err(e) = query_word(&term, &word, cli.raw).await {
            eprintln!("❌ Error: {}", e);
            return Ok(());
        }
    } else if let Some(word) = cli.save {
        if let Some(content) = cli.content {
            if let Err(e) = save_word(&term, &word, &content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        } else {
            eprintln!("❌ Error: --content is required when using --save");
            return Ok(());
        }
    } else if let Some(word) = cli.delete {
        if let Err(e) = delete_word(&term, &word).await {
            eprintln!("❌ Error: {}", e);
            return Ok(());
        }
    } else if let Some(word) = cli.word {
        if let Err(e) = query_word(&term, &word, cli.raw).await {
            eprintln!("❌ Error: {}", e);
            return Ok(());
        }
    } else {
        // Show help if no arguments provided
        let _ = clap::Command::new("word4you").print_help();
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

async fn delete_word(term: &Term, word: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;
    
    // Initialize word processor
    let processor = WordProcessor::new(config);
    
    // Delete the word
    processor.delete_word(term, word)?;
    
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
  word4you <word>                    # Learn a new word
  word4you --test                    # Test API connection
  word4you --info                    # Show this information
  word4you --learn <word>            # Learn a specific word
  word4you --save <word> --content <content>  # Save word to vocabulary
  word4you --delete <word>           # Delete word from vocabulary
"#;

    term.write_line(&style(info).cyan().to_string()).unwrap();
}

 
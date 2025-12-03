use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{style, Term};
use text_processor::TextProcessor;

mod ai_client;
mod config;
mod config_manager;
mod gemini_client;
mod git_section_sync;
mod git_utils;
mod prompt_templates;
mod qwen_client;
mod text_processor;
mod utils;

use config::Config;
use config_manager::ConfigManager;

const INTRO: &str = r#"
Word4You - Language Learning Tool

Features:
• AI-powered explanations for words, phrases, and sentences using Google Gemini or QWEN
• Translations between English and Chinese with phonetic symbols
• Example sentences in both English and Chinese
• Compose sentences using two words with AI
• Automatic Git integration for version control
• Markdown-formatted vocabulary notebook
• Content update functionality (delete and replace)

Usage:
  word4you                           # Interactive mode (enter text one by one)
  word4you query <text>              # Learn a new English or Chinese word, phrase, or sentence
  word4you compose                   # Interactive compose mode (random words from saved vocabulary)
  word4you compose <word1> <word2>   # Compose a sentence using two specific words
  word4you test                      # Test API connection
  word4you config                    # Set up or update configuration
  word4you config --show-vob-path    # Show the vocabulary notebook path
  word4you save <content>            # Save content to vocabulary notebook
  word4you delete <timestamp>        # Delete content from vocabulary notebook by timestamp
  word4you update <timestamp> --content <content>  # Update content (delete entry by timestamp, then save)

Options:
  --raw                              # Output raw response from API without user interaction
"#;

#[derive(Parser)]
#[command(
    name = "word4you",
    about = "Learn English words with AI-powered explanations using Google Gemini",
    long_about = INTRO,
    version = "1.2.0"
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
    },

    /// Save content to vocabulary notebook
    Save {
        /// The content to save
        content: String,
    },

    /// Delete content from vocabulary notebook by timestamp
    Delete {
        /// Timestamp of the entry to delete
        timestamp: String,
    },

    /// Update content: delete entry by timestamp, then save new content
    Update {
        /// Timestamp of the entry to update
        timestamp: String,

        /// The new content to save
        #[arg(long)]
        content: String,
    },

    /// Compose a sentence using two given words (or random words if not provided)
    Compose {
        /// First word to use in the sentence (optional, uses random if not provided)
        word1: Option<String>,

        /// Second word to use in the sentence (optional, uses random if not provided)
        word2: Option<String>,
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
    let has_env_config = std::env::var("WORD4YOU_GEMINI_API_KEY").is_ok()
        || std::env::var("WORD4YOU_QWEN_API_KEY").is_ok();
    let has_file_config = ConfigManager::config_exists();

    // If neither environment variables nor config file exists, and not running config command, run onboarding
    if !has_env_config && !has_file_config && !matches!(cli.command, Some(Commands::Config { .. }))
    {
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
        Some(Commands::Query { word, raw }) => {
            if let Err(e) = query_text(&term, word, *raw).await {
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
        Some(Commands::Save { content }) => {
            if let Err(e) = save_text(&term, content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Delete { timestamp }) => {
            if let Err(e) = delete_text(&term, timestamp).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Update { timestamp, content }) => {
            if let Err(e) = update_text(&term, timestamp, content).await {
                eprintln!("❌ Error: {}", e);
                return Ok(());
            }
        }
        Some(Commands::Compose { word1, word2 }) => {
            match (word1, word2) {
                (Some(w1), Some(w2)) => {
                    // Both words provided, compose directly
                    if let Err(e) = compose_sentence(&term, w1, w2).await {
                        eprintln!("❌ Error: {}", e);
                        return Ok(());
                    }
                }
                _ => {
                    // No words or partial words, enter interactive mode
                    if let Err(e) = interactive_compose_mode(&term).await {
                        eprintln!("❌ Error: {}", e);
                        return Ok(());
                    }
                }
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

async fn query_text(term: &Term, text: &str, raw: bool) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Process the text (prompt template is now determined automatically based on classification)
    processor.process_text(term, text, raw, "").await?;

    Ok(())
}

async fn save_text(term: &Term, content: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Save the content
    processor.save_text(term, content)?;

    Ok(())
}

async fn delete_text(term: &Term, timestamp: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Delete by timestamp
    processor.delete_text(term, timestamp)?;

    Ok(())
}

async fn update_text(term: &Term, timestamp: &str, content: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Update the entry (delete by timestamp, then save)
    processor.update_text(term, timestamp, content)?;

    Ok(())
}

async fn compose_sentence(_term: &Term, word1: &str, word2: &str) -> anyhow::Result<()> {
    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config);

    // Compose a sentence using both words
    let result = processor.compose_sentence(word1, word2).await?;

    // Print result directly (for CLI parsing)
    println!("{}", result);

    Ok(())
}

async fn interactive_compose_mode(term: &Term) -> anyhow::Result<()> {
    use crate::utils::{get_random_single_words, parse_saved_words, prepend_to_vocabulary_notebook};
    use termimad::*;

    // Validate configuration
    let config = Config::load()?;

    // Initialize text processor
    let processor = TextProcessor::new(config.clone());

    term.write_line(
        &style("✍️  Welcome to Word4You Compose Mode!")
            .cyan()
            .bold()
            .to_string(),
    )?;
    term.write_line("Compose sentences using your saved words.\n")?;

    // Parse saved words from vocabulary notebook
    let all_words = parse_saved_words(&config.vocabulary_notebook_file)?;
    let single_words: Vec<String> = all_words
        .iter()
        .filter(|w| !w.contains(" + "))
        .cloned()
        .collect();

    if single_words.len() < 2 {
        term.write_line(&style("❌ You need at least 2 saved words to compose sentences.")
            .red()
            .to_string())?;
        term.write_line("Please use 'word4you query <word>' to save more words first.")?;
        return Ok(());
    }

    term.write_line(&format!(
        "📚 Found {} saved words (excluding composed sentences)\n",
        single_words.len()
    ))?;

    // Create a markdown skin for rendering
    let skin = MadSkin::default();

    // Get initial random words
    let mut current_words = get_random_single_words(&single_words, 2);
    if current_words.len() < 2 {
        return Err(anyhow::anyhow!("Not enough words to compose"));
    }

    loop {
        let word1 = &current_words[0];
        let word2 = &current_words[1];

        term.write_line(&format!(
            "🎲 Composing: {} + {}",
            style(word1).yellow(),
            style(word2).yellow()
        ))?;
        term.write_line(&format!(
            "🤖 Querying {} API...\n",
            config.ai_provider.to_uppercase()
        ))?;

        // Generate sentence
        let result = processor.compose_sentence(word1, word2).await?;

        // Display result
        term.write_line(&style("=".repeat(50)).blue().to_string())?;
        let rendered = FmtText::from(&skin, &result, None);
        term.write_line(&rendered.to_string())?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;

        // Show action menu
        term.write_line("\nChoose an action:")?;
        term.write_line(
            &format!(
                "{} - Regenerate with same words ({} + {})",
                style("r").green(),
                word1,
                word2
            ),
        )?;
        term.write_line(
            &format!(
                "{} - Generate with new random words",
                style("n").yellow()
            ),
        )?;
        term.write_line(
            &format!(
                "{} - Save to vocabulary notebook",
                style("s").cyan()
            ),
        )?;
        term.write_line(
            &format!(
                "{} - Exit",
                style("e").red()
            ),
        )?;
        term.write_line("")?;

        let choices = vec!["r", "n", "s", "e"];
        let selection = dialoguer::Select::new()
            .with_prompt("Enter your choice")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => {
                // Regenerate with same words - just continue the loop
                term.write_line("\n🔄 Regenerating with same words...\n")?;
                continue;
            }
            1 => {
                // Generate with new random words
                term.write_line("\n🎲 Selecting new random words...\n")?;
                current_words = get_random_single_words(&single_words, 2);
                if current_words.len() < 2 {
                    term.write_line("❌ Not enough words available")?;
                    break;
                }
                continue;
            }
            2 => {
                // Save to vocabulary notebook
                term.write_line("\n💾 Saving to vocabulary notebook...")?;
                prepend_to_vocabulary_notebook(&config.vocabulary_notebook_file, &result)?;
                term.write_line(&style("✅ Sentence saved!").green().to_string())?;
                
                // Ask what to do next
                term.write_line("\nWhat would you like to do next?")?;
                let next_choices = vec!["Continue composing", "Exit"];
                let next_selection = dialoguer::Select::new()
                    .with_prompt("Choose")
                    .items(&next_choices)
                    .default(0)
                    .interact()?;
                
                if next_selection == 1 {
                    term.write_line("\n👋 Goodbye!")?;
                    break;
                }
                
                // Get new words for next round
                term.write_line("\n🎲 Selecting new random words...\n")?;
                current_words = get_random_single_words(&single_words, 2);
                if current_words.len() < 2 {
                    term.write_line("❌ Not enough words available")?;
                    break;
                }
                continue;
            }
            3 => {
                // Exit
                term.write_line("\n👋 Goodbye!")?;
                break;
            }
            _ => {
                term.write_line("❓ Invalid choice. Please try again.")?;
                continue;
            }
        }
    }

    Ok(())
}

async fn test_api_connection(term: &Term) -> anyhow::Result<()> {
    let config = Config::load()?;
    let processor = TextProcessor::new(config);

    term.write_line("🔍 Testing API connection...")?;

    match processor.test_api_connection().await {
        Ok(true) => {
            term.write_line(&style("✅ API connection successful").green().to_string())?;
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

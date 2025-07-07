use anyhow::Result;
use console::{style, Term};
use dialoguer::Select;
use crate::gemini_client::GeminiClient;
use crate::utils::{commit_and_push_changes, format_commit_message, prepend_to_wordbook, validate_word};
use crate::config::Config;

pub struct WordProcessor {
    gemini_client: GeminiClient,
    config: Config,
}

impl WordProcessor {
    pub fn new(config: Config) -> Self {
        let gemini_client = GeminiClient::new(config.gemini_api_key.clone());
        Self {
            gemini_client,
            config,
        }
    }

    pub async fn process_word(&self, term: &Term, word: &str) -> Result<()> {
        // Validate word
        validate_word(word)?;
        
        term.write_line(&format!("🔍 Processing word: {}", word))?;
        
        // Get explanation from Gemini
        term.write_line("🤖 Querying Gemini API...")?;
        let mut explanation = Box::new(self.gemini_client
            .get_word_explanation(word, &self.config.gemini_prompt_template)
            .await?);
        
        // Display the explanation
        term.write_line("\n📖 Word Explanation:")?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;
        term.write_line(&explanation)?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;
        
        // Ask for user confirmation with options
        loop {
            term.write_line("\nChoose an action:")?;
            term.write_line(format!("{} - Save to wordbook", style("s").green().to_string()).as_str())?;
            term.write_line(format!("{} - Regenerate explanation", style("r").yellow().to_string()).as_str())?;
            term.write_line(format!("{} - Preview what will be saved", style("p").blue().to_string()).as_str())?;
            term.write_line(format!("{} - Skip this word", style("k").red().to_string()).as_str())?;
            term.write_line("")?;
            
            let choices = vec!["s", "r", "p", "k"];
            let selection = Select::new()
                .with_prompt("Enter your choice")
                .items(&choices)
                .default(0)
                .interact()?;
            
            match selection {
                0 => {
                    // Save to wordbook
                    term.write_line("\n💾 Saving to wordbook...")?;
                    prepend_to_wordbook(&self.config.wordbook_file, &explanation)?;
                    
                    // Commit and push changes
                    term.write_line("📝 Committing changes...")?;
                    let commit_message = format_commit_message(word);
                    commit_and_push_changes(&commit_message, self.config.git_remote_url.as_deref())?;
                    
                    term.write_line(&format!("✅ Successfully processed and saved word: {}", word))?;
                    return Ok(());
                }
                1 => {
                    // Regenerate explanation
                    term.write_line("🔄 Regenerating explanation...")?;
                    let new_explanation = self.gemini_client
                        .get_word_explanation(word, &self.config.gemini_prompt_template)
                        .await?;
                    explanation = Box::new(new_explanation);
                    term.write_line("\n📖 New Word Explanation:")?;
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;
                    term.write_line(&explanation)?;
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;
                    continue; // Ask again
                }
                2 => {
                    // Preview
                    term.write_line("\n📋 Preview of what will be saved:")?;
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;
                    
                    let lines: Vec<&str> = explanation.lines().collect();
                    let preview_lines = if lines.len() > 10 {
                        &lines[..10]
                    } else {
                        &lines
                    };
                    
                    for line in preview_lines {
                        term.write_line(line)?;
                    }
                    
                    if lines.len() > 10 {
                        term.write_line(&format!("\n... and {} more lines", lines.len() - 10))?;
                    }
                    
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;
                    continue;
                }
                3 => {
                    // Skip
                    term.write_line("❌ Word explanation skipped.")?;
                    return Ok(());
                }
                _ => {
                    term.write_line("❓ Invalid choice. Please try again.")?;
                    continue;
                }
            }
        }
    }

    pub async fn test_api_connection(&self) -> Result<bool> {
        self.gemini_client.test_connection().await
    }
} 
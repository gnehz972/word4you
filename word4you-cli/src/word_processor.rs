use crate::config::Config;
use crate::gemini_client::GeminiClient;
use crate::git_utils::{commit, sync_with_remote};
use crate::utils::{
    delete_from_vocabulary_notebook, prepend_to_vocabulary_notebook, validate_word
};
use anyhow::Result;
use console::{style, Term};
use dialoguer::Select;
use termimad::*;

pub struct WordProcessor {
    gemini_client: GeminiClient,
    config: Config,
}

impl WordProcessor {
    pub fn new(config: Config) -> Self {
        let gemini_client = GeminiClient::new(
            config.gemini_api_key.clone(),
            config.gemini_model_name.clone(),
        );
        Self {
            gemini_client,
            config,
        }
    }

    pub async fn process_word(&self, term: &Term, word: &str, raw: bool) -> Result<()> {
        // Validate word
        validate_word(word)?;

        if !raw {
            term.write_line(&format!("🔍 Processing word: {}", word))?;
            term.write_line("🤖 Querying Gemini API...")?;
        }

        // Get explanation from Gemini
        let mut explanation = Box::new(
            self.gemini_client
                .get_word_explanation(word, &self.config.gemini_prompt_template)
                .await?,
        );

        // If raw mode, just print the response and return
        if raw {
            println!("{}", explanation);
            return Ok(());
        }

        // Display the explanation with beautiful markdown rendering
        term.write_line("\n📖 Word Explanation:")?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;

        // Create a markdown skin for beautiful rendering
        let skin = make_skin();

        // Render the markdown
        let text = FmtText::from(&skin, &explanation, None);
        term.write_line(&text.to_string())?;

        term.write_line(&style("=".repeat(50)).blue().to_string())?;

        // Ask for user confirmation with options
        loop {
            term.write_line("\nChoose an action:")?;
            term.write_line(
                format!(
                    "{} - Save to vocabulary notebook",
                    style("s").green().to_string()
                )
                .as_str(),
            )?;
            term.write_line(
                format!(
                    "{} - Regenerate explanation",
                    style("r").yellow().to_string()
                )
                .as_str(),
            )?;
            term.write_line(
                format!(
                    "{} - Preview what will be saved",
                    style("p").blue().to_string()
                )
                .as_str(),
            )?;
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
                    // Save to vocabulary notebook using the shared method
                    self.save_word(term, word, &explanation)?;
                    return Ok(());
                }
                1 => {
                    // Regenerate explanation
                    term.write_line("🔄 Regenerating explanation...")?;
                    let new_explanation = self
                        .gemini_client
                        .get_word_explanation(word, &self.config.gemini_prompt_template)
                        .await?;
                    explanation = Box::new(new_explanation);

                    term.write_line("\n📖 New Word Explanation:")?;
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;

                    // Render the new markdown
                    let text = FmtText::from(&skin, &explanation, None);
                    term.write_line(&text.to_string())?;

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

    pub fn save_word(&self, term: &Term, word: &str, content: &str) -> Result<()> {
        // Validate word
        validate_word(word)?;

        term.write_line(&format!(
            "💾 Saving word '{}' to vocabulary notebook...",
            word
        ))?;
        // Save to vocabulary notebook
        prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, content)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully saved word locally")?;

        if self.config.git_enabled {
            // Commit changes locally
            term.write_line("📝 Committing changes locally...")?;
            self.commit_local_changes(word, "Add")?;

            // Then perform section-aware sync
            term.write_line("🔄 Synchronizing with section awareness...")?;
            sync_with_remote(
                &self.config.vocabulary_notebook_file,
                self.config.git_remote_url.as_deref(),
            )?;

        } else {
            term.write_line("ℹ️  Git operations disabled (GIT_ENABLED=false)")?;
        }

        Ok(())
    }

    pub fn delete_word(&self, term: &Term, word: &str, timestamp: Option<&str>) -> Result<()> {
        // Validate word
        validate_word(word)?;

        term.write_line(&format!(
            "🗑️  Deleting word '{}' from vocabulary notebook...",
            word
        ))?;

        // Delete from vocabulary notebook, optionally with timestamp
        delete_from_vocabulary_notebook(&self.config.vocabulary_notebook_file, word, timestamp)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully deleted word locally")?;

        if self.config.git_enabled {
            // Commit changes locally
            term.write_line("📝 Committing changes locally...")?;
            self.commit_local_changes(word, "Delete")?;

            // Then perform section-aware sync
            term.write_line("🔄 Synchronizing with section awareness...")?;
            sync_with_remote(
                &self.config.vocabulary_notebook_file,
                self.config.git_remote_url.as_deref(),
            )?;
        } else {
            term.write_line("ℹ️  Git operations disabled (GIT_ENABLED=false)")?;
        }

        Ok(())
    }

    pub fn update_word(
        &self,
        term: &Term,
        word: &str,
        content: &str,
        timestamp: Option<&str>,
    ) -> Result<()> {
        // Validate word
        validate_word(word)?;

        term.write_line(&format!(
            "🔄 Updating word '{}' in vocabulary notebook...",
            word
        ))?;

        // First, try to delete the word if it exists (ignore error if word doesn't exist)
        match delete_from_vocabulary_notebook(
            &self.config.vocabulary_notebook_file,
            word,
            timestamp,
        ) {
            Ok(_) => {
                term.write_line(&format!("🗑️  Deleted existing entry for '{}'", word))?;
            }
            Err(_) => {
                term.write_line(&format!(
                    "ℹ️  No existing entry found for '{}', creating new entry",
                    word
                ))?;
            }
        }

        // Then save the new content
        term.write_line(&format!("💾 Saving updated content for '{}'...", word))?;
        prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, content)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully updated word locally")?;

        if self.config.git_enabled {
            // Commit changes locally
            term.write_line("📝 Committing changes locally...")?;
            self.commit_local_changes(word, "Update")?;

            // Then perform section-aware sync
            term.write_line("🔄 Synchronizing with section awareness...")?;
            sync_with_remote(
                &self.config.vocabulary_notebook_file,
                self.config.git_remote_url.as_deref(),
            )?;
        } else {
            term.write_line("ℹ️  Git operations disabled (GIT_ENABLED=false)")?;
        }

        Ok(())
    }

    /// Helper method to commit local changes before sync
    fn commit_local_changes(&self, word: &str, operation: &str) -> Result<()> {
        let commit_message = format!(
            "{} word: {} - {}",
            operation,
            word,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        commit(
            &commit_message,
            &self.config.vocabulary_notebook_file,
        )?;

        Ok(())
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    // Configure colors for different markdown elements
    skin.set_headers_fg(rgb(255, 187, 0));
    skin.bold.set_fg(rgb(255, 187, 0));
    skin.italic.set_fg(rgb(215, 255, 135));
    skin.bullet = StyledChar::from_fg_char(rgb(255, 187, 0), '•');
    skin.quote_mark = StyledChar::from_fg_char(rgb(0, 187, 255), '│');
    skin.quote_mark.set_fg(rgb(0, 187, 255));
    skin.inline_code.set_fg(rgb(255, 119, 119));
    skin.inline_code.set_bg(rgb(40, 44, 52));
    skin.code_block.set_bg(rgb(40, 44, 52));
    skin.code_block.set_fg(rgb(255, 119, 119));

    skin
}

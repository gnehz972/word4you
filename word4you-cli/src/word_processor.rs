use crate::config::Config;
use crate::gemini_client::GeminiClient;
use crate::qwen_client::QwenClient;
use crate::ai_client::AiClient;
use crate::git_section_sync::{GitSectionSynchronizer, SyncResult};
use crate::git_utils::{commit, init_git_repo};
use crate::utils::{
    delete_from_vocabulary_notebook, get_work_dir, prepend_to_vocabulary_notebook, validate_word,
};
use anyhow::Result;
use console::{style, Term};
use dialoguer::Select;
use termimad::*;

pub struct WordProcessor {
    ai_client: Box<dyn AiClient + Send + Sync>,
    config: Config,
}

impl WordProcessor {
    pub fn new(config: Config) -> Self {
        let ai_client: Box<dyn AiClient + Send + Sync> = match config.ai_provider.as_str() {
            "qwen" => {
                if config.qwen_api_key.is_empty() {
                    panic!("QWEN API key not configured");
                }
                Box::new(QwenClient::new(
                    config.qwen_api_key.clone(),
                    config.qwen_model_name.clone(),
                ))
            }
            "gemini" | _ => {
                if config.gemini_api_key.is_empty() {
                    panic!("Gemini API key not configured");
                }
                Box::new(GeminiClient::new(
                    config.gemini_api_key.clone(),
                    config.gemini_model_name.clone(),
                ))
            }
        };
        
        Self {
            ai_client,
            config,
        }
    }

    pub async fn process_word(&self, term: &Term, word: &str, raw: bool) -> Result<()> {
        // Validate word
        validate_word(word)?;

        if !raw {
            term.write_line(&format!("🔍 Processing word: {}", word))?;
            term.write_line(&format!("🤖 Querying {} API...", self.config.ai_provider.to_uppercase()))?;
        }

        // Get explanation from AI provider
        let mut explanation = Box::new(
            self.ai_client
                .get_word_explanation(word, &self.config.prompt_template)
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
                    "{} - Skip this word",
                    style("k").red().to_string()
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
            term.write_line("")?;

            let choices = vec!["s", "k", "r"];
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
                    // Skip
                    term.write_line("✔️ Word explanation skipped.")?;
                    return Ok(());
                }
                2 => {
                    // Regenerate explanation
                    term.write_line("🔄 Regenerating explanation...")?;
                    let new_explanation = self
                        .ai_client
                        .get_word_explanation(word, &self.config.prompt_template)
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

                _ => {
                    term.write_line("❓ Invalid choice. Please try again.")?;
                    continue;
                }
            }
        }
    }

    pub async fn test_api_connection(&self) -> Result<bool> {
        self.ai_client.test_connection().await
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

        self.commit_and_push(term, word, "Save")?;

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

        self.commit_and_push(term, word, "Delete")?;

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
        delete_from_vocabulary_notebook(&self.config.vocabulary_notebook_file, word, timestamp)?;

        // Then save the new content
        prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, content)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully updated word locally")?;

        self.commit_and_push(term, word, "Update")?;

        Ok(())
    }

    fn commit_and_push(&self, term: &Term, word: &str, operation: &str) -> Result<()> {
        if self.config.git_enabled {
            let work_dir = get_work_dir(&self.config.vocabulary_notebook_file)?;
            // Initialize git repository if it doesn't exist
            init_git_repo(&work_dir, self.config.git_remote_url.as_deref())?;
            // Commit changes locally
            term.write_line("📝 Committing changes locally...")?;
            self.commit_local_changes(word, operation)?;

            if self.config.git_remote_url.is_some() {
                // Then perform section-aware sync
                self.sync_with_remote()?;
            }
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
        commit(&commit_message, &self.config.vocabulary_notebook_file)?;

        Ok(())
    }

    /// Section-aware synchronization that uses git's change detection
    fn sync_with_remote(&self) -> Result<()> {
        // Create section synchronizer
        let synchronizer = GitSectionSynchronizer::new(self.config.clone())?;

        // Perform section-aware sync
        match synchronizer.sync_with_remote() {
            Ok(SyncResult::Success) => {
                println!("✅ Successfully synchronized vocabulary notebook");
                Ok(())
            }
            Ok(SyncResult::FAIL) => {
                println!("ℹ️ Failed to synchronize");
                Ok(())
            }
            Err(e) => Err(e),
        }
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

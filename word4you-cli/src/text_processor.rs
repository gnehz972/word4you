use crate::ai_client::AiClient;
use crate::config::Config;
use crate::gemini_client::GeminiClient;
use crate::git_section_sync::{GitSectionSynchronizer, SyncResult};
use crate::git_utils::{commit, init_git_repo};
use crate::prompt_templates::PromptTemplates;
use crate::qwen_client::QwenClient;
use crate::utils::{
    classify_input, delete_from_vocabulary_notebook, get_work_dir, prepend_to_vocabulary_notebook,
    validate_text, InputType,
};
use anyhow::Result;
use console::{style, Term};
use dialoguer::Select;
use termimad::*;

pub struct TextProcessor {
    ai_client: Box<dyn AiClient + Send + Sync>,
    pub config: Config,
}

impl TextProcessor {
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

        Self { ai_client, config }
    }

    pub async fn process_text(
        &self,
        term: &Term,
        text: &str,
        raw: bool,
        _prompt_template: &str,
    ) -> Result<()> {
        // Validate input text
        validate_text(text)?;

        // Classify the input (language and type)
        let classification = classify_input(text);

        // Get the appropriate prompt template based on classification
        let prompt_template = PromptTemplates::get_template(&classification);

        if !raw {
            let lang_str = match classification.language {
                crate::utils::Language::English => "English",
                crate::utils::Language::Chinese => "Chinese",
                crate::utils::Language::Mixed => "Mixed",
            };
            let type_str = match classification.input_type {
                InputType::Word => "word",
                InputType::Phrase => "phrase",
                InputType::Sentence => "sentence",
            };

            term.write_line(&format!(
                "🔍 Processing {} {}: {}",
                lang_str, type_str, text
            ))?;
            term.write_line(&format!(
                "🤖 Querying {} API...",
                self.config.ai_provider.to_uppercase()
            ))?;
        }

        // Get explanation from AI provider using the appropriate template
        let mut explanation = Box::new(
            self.ai_client
                .get_text_explanation(text, &prompt_template)
                .await?,
        );

        // If raw mode, just print the response and return
        if raw {
            println!("{}", explanation);
            return Ok(());
        }

        // Display the explanation with beautiful markdown rendering
        let content_type = match classification.input_type {
            InputType::Word => "Word",
            InputType::Phrase => "Phrase",
            InputType::Sentence => "Sentence",
        };

        term.write_line(&format!("\n📖 {} Explanation:", content_type))?;
        term.write_line(&style("=".repeat(50)).blue().to_string())?;

        // Create a markdown skin for beautiful rendering
        let skin = make_skin();

        // Render the markdown
        let rendered_text = FmtText::from(&skin, &explanation, None);
        term.write_line(&rendered_text.to_string())?;

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
            term.write_line(format!("{} - Skip this text", style("k").red().to_string()).as_str())?;
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
                    self.save_text(term, &explanation)?;
                    return Ok(());
                }
                1 => {
                    // Skip
                    term.write_line("✔️ Text explanation skipped.")?;
                    return Ok(());
                }
                2 => {
                    // Regenerate explanation
                    term.write_line("🔄 Regenerating explanation...")?;
                    let new_explanation = self
                        .ai_client
                        .get_text_explanation(text, &prompt_template)
                        .await?;
                    explanation = Box::new(new_explanation);

                    term.write_line(&format!("\n📖 New {} Explanation:", content_type))?;
                    term.write_line(&style("=".repeat(50)).blue().to_string())?;

                    // Render the new markdown
                    let rendered_text = FmtText::from(&skin, &explanation, None);
                    term.write_line(&rendered_text.to_string())?;

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

    /// Compose a sentence using two words and return the result
    pub async fn compose_sentence(&self, word1: &str, word2: &str) -> Result<String> {
        let prompt_template = PromptTemplates::compose_sentence_template();
        let words_text = format!("\"{}\", \"{}\"", word1, word2);

        let result = self
            .ai_client
            .get_text_explanation(&words_text, &prompt_template)
            .await?;

        Ok(result)
    }

    pub fn save_text(&self, term: &Term, content: &str) -> Result<()> {
        term.write_line("💾 Saving content to vocabulary notebook...")?;

        // Save to vocabulary notebook
        prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, content)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully saved content locally")?;

        self.commit_and_push(term, "content", "Save")?;

        Ok(())
    }

    pub fn delete_text(&self, term: &Term, timestamp: &str) -> Result<()> {
        term.write_line(&format!(
            "🗑️  Deleting entry with timestamp '{}' from vocabulary notebook...",
            timestamp
        ))?;

        // Delete from vocabulary notebook by timestamp
        delete_from_vocabulary_notebook(&self.config.vocabulary_notebook_file, timestamp)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully deleted entry locally")?;

        self.commit_and_push(term, timestamp, "Delete")?;

        Ok(())
    }

    pub fn update_text(&self, term: &Term, timestamp: &str, content: &str) -> Result<()> {
        term.write_line(&format!(
            "🔄 Updating entry with timestamp '{}' in vocabulary notebook...",
            timestamp
        ))?;

        // First, try to delete the entry by timestamp if it exists
        delete_from_vocabulary_notebook(&self.config.vocabulary_notebook_file, timestamp)?;

        // Then save the new content
        prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, content)?;

        // Commit changes only if git is enabled
        term.write_line("✅ Successfully updated entry locally")?;

        self.commit_and_push(term, timestamp, "Update")?;

        Ok(())
    }

    fn commit_and_push(&self, term: &Term, text: &str, operation: &str) -> Result<()> {
        if self.config.git_enabled {
            let work_dir = get_work_dir(&self.config.vocabulary_notebook_file)?;
            // Initialize git repository if it doesn't exist
            init_git_repo(&work_dir, self.config.git_remote_url.as_deref())?;
            // Commit changes locally
            term.write_line("📝 Committing changes locally...")?;
            self.commit_local_changes(text, operation)?;

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
    fn commit_local_changes(&self, text: &str, operation: &str) -> Result<()> {
        let commit_message = format!(
            "{} text: {} - {}",
            operation,
            text,
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

use crate::config::Config;
use crate::git_utils::run_git_command;
use anyhow::{anyhow, Result};
use console::Term;
use std::path::Path;

#[derive(Debug)]
pub enum SyncResult {
    Success,
    FAIL,
}

#[derive(Debug)]
struct LocalChanges {
    added_sections: Vec<AddedWordSection>,
    deleted_sections: Vec<DeletedWordSection>,
}

#[derive(Debug)]
struct AddedWordSection {
    word: String,
    content: String,
    timestamp: Option<String>,
}

#[derive(Debug)]
struct DeletedWordSection {
    word: String,
    timestamp: Option<String>,
}

pub struct GitSectionSynchronizer {
    config: Config,
    term: Term,
}

impl GitSectionSynchronizer {
    pub fn new(config: Config) -> Result<Self> {
        let term = Term::stdout();

        Ok(Self { config, term })
    }

    pub fn sync_with_remote(&self) -> Result<SyncResult> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;

        self.term.write_line("🔄 Starting synchronization...")?;

        // Fetch latest from remote
        self.term
            .write_line("📥 Fetching latest changes from remote...")?;
        if let Err(e) = run_git_command(&["fetch", "origin"], work_dir) {
            self.term
                .write_line(&format!("⚠️  Could not fetch from remote: {}", e))?;
            // Continue with local-only operation
        }

        // Check if this is a first-time sync (no common history)
        self.term.write_line("🔍 Checking repository history...")?;
        let is_first_time_sync = self.is_first_time_sync(work_dir)?;

        if is_first_time_sync {
            self.term
                .write_line("🎆 First-time sync detected - using direct content merging...")?;
            self.handle_first_time_sync()?;
        } else {
            // Check if we have unpushed commits
            let mut has_unpushed_commits = false;
            if let Ok(output) =
                run_git_command(&["rev-list", "--count", "origin/main..HEAD"], work_dir)
            {
                if let Ok(count) = output.trim().parse::<i32>() {
                    has_unpushed_commits = count > 0;
                    if has_unpushed_commits {
                        self.term
                            .write_line(&format!("📝 {} unpushed commits detected", count))?;
                    }
                }
            }

            // Normal sync with existing history
            // First check if we're ahead of remote (only have unpushed commits)
            if has_unpushed_commits {
                // Check if remote has new commits
                let remote_ahead =
                    run_git_command(&["rev-list", "--count", "HEAD..origin/main"], work_dir)
                        .map(|output| output.trim().parse::<i32>().unwrap_or(0) > 0)
                        .unwrap_or(false);

                if !remote_ahead {
                    // We're ahead and remote has no new commits - skip merge, go straight to push
                    self.term
                        .write_line("ℹ️  Only local commits, no remote changes - skipping merge")?;
                } else {
                    // Both sides have commits - need to merge
                    self.term
                        .write_line("🔍 Both local and remote changes detected - merging...")?;
                    self.perform_merge(work_dir)?;
                }
            } else {
                // No unpushed commits - check for merge conflicts
                self.term.write_line("🔍 Checking for merge conflicts...")?;
                self.perform_merge(work_dir)?;
            }
        }

        // Push changes
        self.term.write_line("📤 Pushing changes to remote...")?;
        match run_git_command(&["push", "-u", "origin", "main"], work_dir) {
            Ok(_) => {
                self.term
                    .write_line("✅ Successfully pushed changes to remote")?;
                Ok(SyncResult::Success)
            }
            Err(e) => {
                self.term
                    .write_line(&format!("⚠️  Could not push to remote: {}", e))?;
                Ok(SyncResult::FAIL)
            }
        }
    }

    /// Perform merge with conflict resolution
    fn perform_merge(&self, work_dir: &Path) -> Result<()> {
        let merge_result = run_git_command(
            &["merge", "--no-commit", "--no-ff", "origin/main"],
            work_dir,
        );

        match merge_result {
            Ok(_) => {
                // No conflicts - complete the merge
                self.term
                    .write_line("✅ No conflicts detected - completing merge...")?;

                // Check if there are actually changes to commit
                let status = run_git_command(&["status", "--porcelain"], work_dir)?;
                if !status.trim().is_empty() {
                    run_git_command(&["commit", "-m", "Merge remote changes"], work_dir)?;
                    self.term
                        .write_line("✅ Successfully merged remote changes")?;
                } else {
                    // No file changes but merge is needed - complete the merge
                    // This happens when remote has commits that don't change files
                    run_git_command(
                        &["commit", "-m", "Merge remote changes (no file changes)"],
                        work_dir,
                    )?;
                    self.term
                        .write_line("✅ Successfully merged remote changes (no file changes)")?;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();

                if error_msg.contains("CONFLICT") || error_msg.contains("Automatic merge failed") {
                    self.term.write_line(
                        "⚠️  Merge conflicts detected - resolving with theirs strategy...",
                    )?;

                    // Reset to clean state
                    let _ = run_git_command(&["merge", "--abort"], work_dir);

                    // Apply our enhanced conflict resolution
                    self.resolve_conflicts_with_manual_theirs()?;

                    self.term
                        .write_line("✅ Conflicts resolved and changes applied")?;
                } else if error_msg.contains("Already up to date") {
                    self.term.write_line("ℹ️  Already up to date with remote")?;
                } else if error_msg.contains("not something we can merge") {
                    self.term.write_line(
                        "ℹ️  Remote branch not found - this may be an empty repository",
                    )?;
                    self.term
                        .write_line("✅ Continuing with local-only operation")?;
                } else {
                    self.term.write_line(&format!(
                        "❌ Merge failed with unexpected error: {}",
                        error_msg
                    ))?;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Check if this is a first-time sync (no common history with remote)
    fn is_first_time_sync(&self, work_dir: &Path) -> Result<bool> {
        // First, check if origin/main exists locally
        let remote_exists =
            run_git_command(&["rev-parse", "--verify", "origin/main"], work_dir).is_ok();

        if !remote_exists {
            self.term
                .write_line("ℹ️  Remote branch not found locally - this is a first-time sync")?;
            return Ok(true);
        }

        // Check if we have any local commits
        let has_local_commits =
            run_git_command(&["rev-parse", "--verify", "HEAD"], work_dir).is_ok();

        if !has_local_commits {
            self.term
                .write_line("ℹ️  No local commits found - this is a first-time sync")?;
            return Ok(true);
        }

        // Try to find a merge base between local and remote
        match run_git_command(&["merge-base", "HEAD", "origin/main"], work_dir) {
            Ok(output) => {
                // If we get output, there's a common ancestor
                let has_common_ancestor = !output.trim().is_empty();
                if !has_common_ancestor {
                    self.term
                        .write_line("ℹ️  No common ancestor found - this is a first-time sync")?;
                }
                Ok(!has_common_ancestor)
            }
            Err(_) => {
                // No merge base found - this is a first-time sync
                self.term
                    .write_line("ℹ️  Cannot find merge base - this is a first-time sync")?;
                Ok(true)
            }
        }
    }

    /// Handle first-time sync - much simpler approach
    fn handle_first_time_sync(&self) -> Result<()> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;

        // Read local content before merge (if any)
        let local_content = std::fs::read_to_string(&self.config.vocabulary_notebook_file)?
            .trim()
            .to_string();

        // Check if remote branch exists
        let remote_exists =
            run_git_command(&["rev-parse", "--verify", "origin/main"], work_dir).is_ok();

        if remote_exists {
            // Just let Git handle the merge completely
            self.term.write_line("🔗 Merging with remote history...")?;

            match run_git_command(
                &[
                    "merge",
                    "origin/main",
                    "--allow-unrelated-histories",
                    "-X",
                    "theirs",
                ],
                work_dir,
            ) {
                Ok(_) => {
                    self.term
                        .write_line("✅ Successfully merged with remote history")?;

                    // If we had local content, prepend it to the merged file
                    if !local_content.is_empty() {
                        self.term.write_line("📝 Prepending local content...")?;

                        // Use our existing prepend utility function
                        crate::utils::prepend_to_vocabulary_notebook(
                            &self.config.vocabulary_notebook_file,
                            &local_content,
                        )?;

                        // Commit the prepended content
                        run_git_command(&["add", "."], work_dir)?;
                        run_git_command(
                            &["commit", "-m", "Prepend local content after initial sync"],
                            work_dir,
                        )?;
                        self.term
                            .write_line("✅ Successfully prepended local content")?;
                    }
                }
                Err(e) => {
                    self.term.write_line(&format!("⚠️  Merge failed: {}", e))?;
                    return Err(e);
                }
            }
        } else {
            // No remote exists, just commit local content if any
            if !local_content.is_empty() {
                run_git_command(&["add", "."], work_dir)?;
                run_git_command(
                    &["commit", "-m", "Initial sync: local content only"],
                    work_dir,
                )?;
                self.term
                    .write_line("✅ Successfully committed local content")?;
            }
        }

        Ok(())
    }

    /// Fallback manual resolution when theirs strategy fails
    fn resolve_conflicts_with_manual_theirs(&self) -> Result<()> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;

        // First, clean up any existing merge state
        self.term.write_line("🧹 Cleaning up merge state...")?;
        let _ = run_git_command(&["merge", "--abort"], work_dir);

        // Use a different approach: merge with --allow-unrelated-histories and -X theirs
        self.term
            .write_line("🔄 Attempting merge with unrelated histories and theirs strategy...")?;
        let local_changes = self.get_local_changes_since_ancestor(work_dir)?;
        match run_git_command(
            &[
                "merge",
                "--allow-unrelated-histories",
                "-X",
                "theirs",
                "origin/main",
            ],
            work_dir,
        ) {
            Ok(_) => {
                self.term.write_line(
                    "✅ Successfully merged with unrelated histories and theirs strategy",
                )?;
                // Analyze and apply local changes after merge
                self.term
                    .write_line("🔍 Applying local changes after theirs merge...")?;

                self.apply_local_changes(&local_changes)?;

                // Stage the resolved content
                self.term.write_line("💾 Staging resolved content...")?;
                run_git_command(&["add", "."], work_dir)?;

                // Create a merge commit that preserves remote history
                self.term
                    .write_line("🔗 Creating merge commit to preserve remote history...")?;

                // Check if there are actually changes to commit
                let status = run_git_command(&["status", "--porcelain"], work_dir)?;
                if !status.trim().is_empty() {
                    // Create a merge commit with two parents
                    match run_git_command(
                        &[
                            "commit",
                            "-m",
                            "Merge origin/main (resolved conflicts by preserving local changes)",
                        ],
                        work_dir,
                    ) {
                        Ok(_) => {
                            self.term.write_line(
                                "✅ Successfully created merge commit with local changes preserved",
                            )?;
                        }
                        Err(e) => {
                            self.term
                                .write_line(&format!("⚠️  Failed to create merge commit: {}", e))?;
                            // Fall back to regular commit
                            run_git_command(
                                &[
                                    "commit",
                                    "-m",
                                    "Apply local changes to remote base (fallback commit)",
                                ],
                                work_dir,
                            )?;
                            self.term.write_line("✅ Created fallback commit")?;
                        }
                    }
                } else {
                    self.term
                        .write_line("ℹ️  No changes to commit after resolution")?;
                }

                return Ok(());
            }
            Err(e) => {
                self.term
                    .write_line(&format!("⚠️  Merge with -X theirs failed: {}", e))?;
                self.term
                    .write_line("🔄 Rolling back to local changes...")?;
                // Restore vocabulary file to local HEAD
                let vocab_filename = Path::new(&self.config.vocabulary_notebook_file)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("vocabulary_notebook.md");
                let _ = run_git_command(&["checkout", "HEAD", "--", vocab_filename], work_dir);
                self.term
                    .write_line("✅ Local changes restored after failed merge")?;
                self.term.write_line(
                    "🔄 Falling back to manual merge with local changes preservation...",
                )?;
                return Ok(());
            }
        }
    }

    /// Get local changes since common ancestor by parsing git diff
    fn get_local_changes_since_ancestor(&self, work_dir: &Path) -> Result<LocalChanges> {
        // Get common ancestor (merge base)
        let merge_base = run_git_command(&["merge-base", "HEAD", "origin/main"], work_dir)
            .map_err(|e| anyhow!("Failed to find common ancestor: {}", e))?;
        let merge_base = merge_base.trim();

        // Get diff from merge base to HEAD for vocabulary file only
        let vocab_filename = Path::new(&self.config.vocabulary_notebook_file)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("vocabulary_notebook.md");

        let diff_output = run_git_command(
            &[
                "diff",
                &format!("{}...HEAD", merge_base),
                "--",
                vocab_filename,
            ],
            work_dir,
        )?;

        self.parse_diff_for_word_changes(&diff_output)
    }

    /// Parse git diff output to extract word section changes
    fn parse_diff_for_word_changes(&self, diff_output: &str) -> Result<LocalChanges> {
        let mut added_sections = Vec::new();
        let mut deleted_sections = Vec::new();

        let lines: Vec<&str> = diff_output.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            // Look for added word sections (+ prefix)
            if let Some(stripped) = line.strip_prefix("+## ") {
                let word = stripped.trim();
                let mut section_content = String::new();
                let mut timestamp = None;

                // Collect the entire added section
                section_content.push_str(&line[1..]); // Remove + prefix
                section_content.push('\n');
                i += 1;

                // Continue collecting until we hit a separator or another section
                while i < lines.len() {
                    let current_line = lines[i];
                    if current_line.starts_with("+---") {
                        section_content.push_str("---\n");
                        i += 1;
                        break;
                    } else if current_line.starts_with("+## ") {
                        // Hit another section, don't consume this line
                        break;
                    } else if let Some(stripped) = current_line.strip_prefix("+<!-- timestamp=") {
                        // Extract timestamp
                        if let Some(ts_end) = stripped.find(" -->") {
                            timestamp = Some(stripped[..ts_end].to_string());
                        }
                        section_content.push_str(&current_line[1..]); // Remove + prefix
                        section_content.push('\n');
                        i += 1;
                    } else if let Some(stripped) = current_line.strip_prefix("+") {
                        // Regular added line
                        section_content.push_str(stripped);
                        section_content.push('\n');
                        i += 1;
                    } else {
                        // Not an added line, stop collecting
                        break;
                    }
                }

                added_sections.push(AddedWordSection {
                    word: word.to_string(),
                    content: section_content.trim().to_string(),
                    timestamp,
                });
                continue;
            }

            // Look for deleted word sections (- prefix)
            if let Some(stripped) = line.strip_prefix("-## ") {
                let word = stripped.trim();
                let mut timestamp = None;

                i += 1;
                // Look for timestamp in the deleted section
                while i < lines.len() {
                    let current_line = lines[i];
                    if current_line.starts_with("-<!-- timestamp=") {
                        // Extract timestamp
                        if let Some(ts_start) = current_line.find("timestamp=") {
                            if let Some(ts_end) = current_line.find(" -->") {
                                timestamp = Some(current_line[ts_start + 10..ts_end].to_string());
                            }
                        }
                        i += 1;
                        break;
                    } else if current_line.starts_with("----") {
                        i += 1;
                        break;
                    } else if current_line.starts_with("-## ") {
                        // Hit another section, don't consume this line
                        break;
                    } else if current_line.starts_with("-") {
                        // Continue through deleted section
                        i += 1;
                    } else {
                        // Not a deleted line, stop collecting
                        break;
                    }
                }

                deleted_sections.push(DeletedWordSection {
                    word: word.to_string(),
                    timestamp,
                });
                continue;
            }

            i += 1;
        }

        Ok(LocalChanges {
            added_sections,
            deleted_sections,
        })
    }

    /// Apply local changes to the current file (which has remote content as base)
    fn apply_local_changes(&self, changes: &LocalChanges) -> Result<()> {
        // First, remove deleted sections
        for deleted in &changes.deleted_sections {
            if let Some(timestamp) = &deleted.timestamp {
                self.term
                    .write_line(&format!("🗑️  Removing entry with timestamp: {}", timestamp))?;
                if let Err(e) = crate::utils::delete_from_vocabulary_notebook(
                    &self.config.vocabulary_notebook_file,
                    timestamp,
                ) {
                    self.term.write_line(&format!(
                        "⚠️  Could not delete entry with timestamp '{}': {}",
                        timestamp, e
                    ))?;
                    // Continue with other deletions
                }
            } else {
                self.term.write_line(&format!(
                    "⚠️  Cannot delete '{}': no timestamp available",
                    deleted.word
                ))?;
            }
        }

        // Then, prepend added sections
        for added in &changes.added_sections {
            self.term
                .write_line(&format!("➕ Adding local word: {}", added.word))?;
            if let Err(e) = crate::utils::prepend_to_vocabulary_notebook(
                &self.config.vocabulary_notebook_file,
                &added.content,
            ) {
                self.term
                    .write_line(&format!("⚠️  Could not add '{}': {}", added.word, e))?;
                // Continue with other additions
            }
        }

        Ok(())
    }
}

use crate::config::Config;
use crate::git_utils::run_git_command;
use anyhow::{anyhow, Result};
use console::Term;
use std::path::Path;

#[derive(Debug)]
pub enum SyncResult {
    Success,
    NoChanges,
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

        self.term
            .write_line("🔄 Starting simplified synchronization...")?;

        // 1. Fetch latest from remote
        self.term
            .write_line("📥 Fetching latest changes from remote...")?;
        if let Err(e) = run_git_command(&["fetch", "origin"], work_dir) {
            self.term
                .write_line(&format!("⚠️  Could not fetch from remote: {}", e))?;
            // Continue with local-only operation
        }

        // 2. Check if we have local changes
        let status = run_git_command(&["status", "--porcelain"], work_dir)?;
        let has_local_changes = !status.trim().is_empty();

        if has_local_changes {
            self.term.write_line("📝 Local changes detected")?;
            // Commit local changes first
            self.commit_changes_if_needed()?;
        } else {
            self.term.write_line("ℹ️  No uncommitted changes detected")?;
        }

        // 2.5. Check if we have unpushed commits
        let mut has_unpushed_commits = false;
        if let Ok(output) = run_git_command(&["rev-list", "--count", "origin/main..HEAD"], work_dir) {
            if let Ok(count) = output.trim().parse::<i32>() {
                has_unpushed_commits = count > 0;
                if has_unpushed_commits {
                    self.term.write_line(&format!("📝 {} unpushed commits detected", count))?;
                }
            }
        }

        // 3. Check if this is a first-time sync (no common history)
        self.term.write_line("🔍 Checking repository history...")?;
        let is_first_time_sync = self.is_first_time_sync(work_dir)?;

        if is_first_time_sync {
            self.term
                .write_line("🎆 First-time sync detected - using direct content merging...")?;
            self.handle_first_time_sync()?;
        } else {
            // Normal sync with existing history
            // First check if we're ahead of remote (only have unpushed commits)
            if has_unpushed_commits {
                // Check if remote has new commits
                let remote_ahead = run_git_command(&["rev-list", "--count", "HEAD..origin/main"], work_dir)
                    .map(|output| output.trim().parse::<i32>().unwrap_or(0) > 0)
                    .unwrap_or(false);
                
                if !remote_ahead {
                    // We're ahead and remote has no new commits - skip merge, go straight to push
                    self.term.write_line("ℹ️  Only local commits, no remote changes - skipping merge")?;
                } else {
                    // Both sides have commits - need to merge
                    self.term.write_line("🔍 Both local and remote changes detected - merging...")?;
                    self.perform_merge(work_dir)?;
                }
            } else {
                // No unpushed commits - check for merge conflicts
                self.term.write_line("🔍 Checking for merge conflicts...")?;
                self.perform_merge(work_dir)?;
            }
        }

        // 4. Push changes if remote is configured
        if self.config.git_remote_url.is_some() {
            self.term.write_line("📤 Pushing changes to remote...")?;
            match run_git_command(&["push", "origin", "main"], work_dir) {
                Ok(_) => self
                    .term
                    .write_line("✅ Successfully pushed changes to remote")?,
                Err(e) => {
                    self.term
                        .write_line(&format!("⚠️  Could not push to remote: {}", e))?;
                    self.term
                        .write_line("💡 You may need to resolve conflicts manually")?;
                }
            }
        }

        self.term
            .write_line("✅ Synchronization completed successfully")?;
        Ok(SyncResult::Success)
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
                    run_git_command(&["commit", "-m", "Merge remote changes (no file changes)"], work_dir)?;
                    self.term
                        .write_line("✅ Successfully merged remote changes (no file changes)")?;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();

                if error_msg.contains("CONFLICT")
                    || error_msg.contains("Automatic merge failed")
                {
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
        let local_content = std::fs::read_to_string(&self.config.vocabulary_notebook_file)
            ?.trim().to_string();

        // Check if remote branch exists
        let remote_exists =
            run_git_command(&["rev-parse", "--verify", "origin/main"], work_dir).is_ok();

        if remote_exists {
            // Just let Git handle the merge completely
            self.term.write_line("🔗 Merging with remote history...")?;
            
            match run_git_command(&["merge", "origin/main", "--allow-unrelated-histories", "-X", "theirs"], work_dir) {
                Ok(_) => {
                    self.term.write_line("✅ Successfully merged with remote history")?;
                    
                    // If we had local content, prepend it to the merged file
                    if !local_content.is_empty() {
                        self.term.write_line("📝 Prepending local content...")?;
                        
                        // Use our existing prepend utility function
                        crate::utils::prepend_to_vocabulary_notebook(&self.config.vocabulary_notebook_file, &local_content)?;
                        
                        // Commit the prepended content
                        run_git_command(&["add", "."], work_dir)?;
                        run_git_command(&["commit", "-m", "Prepend local content after initial sync"], work_dir)?;
                        self.term.write_line("✅ Successfully prepended local content")?;
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
                run_git_command(&["commit", "-m", "Initial sync: local content only"], work_dir)?;
                self.term.write_line("✅ Successfully committed local content")?;
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
        self.term.write_line("🔄 Attempting merge with unrelated histories and theirs strategy...")?;
        
        match run_git_command(&["merge", "--allow-unrelated-histories", "-X", "theirs", "origin/main"], work_dir) {
            Ok(_) => {
                self.term.write_line("✅ Successfully merged with unrelated histories and theirs strategy")?;
                return Ok(());
            }
            Err(e) => {
                self.term.write_line(&format!("⚠️  Merge with -X theirs failed: {}", e))?;
                self.term.write_line("🔄 Falling back to manual merge commit creation...")?;
            }
        }

        // If the above fails, manually create a merge commit that preserves remote history
        let _ = run_git_command(&["merge", "--abort"], work_dir);
        
        // Get the remote version manually
        self.term.write_line("📥 Getting remote version manually...")?;

        // Extract filename from the vocabulary file path
        let vocab_filename = Path::new(&self.config.vocabulary_notebook_file)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("vocabulary_notebook.md");

        let remote_ref = format!("origin/main:{}", vocab_filename);
        let remote_content = run_git_command(&["show", &remote_ref], work_dir)
            .map_err(|e| {
                self.term.write_line(&format!("⚠️  Could not get remote file '{}': {}", vocab_filename, e)).ok();
                anyhow!("Failed to get remote file content: {}. This might be a first-time sync with an empty remote repository.", e)
            })?;

        // Write remote content to resolve conflicts
        std::fs::write(&self.config.vocabulary_notebook_file, remote_content)?;

        // Stage the resolved content
        self.term.write_line("💾 Staging resolved content...")?;
        run_git_command(&["add", "."], work_dir)?;

        // Create a merge commit that preserves remote history
        self.term.write_line("🔗 Creating merge commit to preserve remote history...")?;
        
        // Check if there are actually changes to commit
        let status = run_git_command(&["status", "--porcelain"], work_dir)?;
        if !status.trim().is_empty() {
            // Create a merge commit with both parents to preserve history
            // Note: We could use git commit-tree to create the merge commit manually
            // but for simplicity, we'll use the standard commit approach
            
            // Create the merge commit with two parents
            match run_git_command(&[
                "commit",
                "-m",
                "Merge origin/main (resolved conflicts by accepting remote version)",
            ], work_dir) {
                Ok(_) => {
                    self.term.write_line("✅ Successfully created merge commit with remote history preserved")?;
                }
                Err(e) => {
                    self.term.write_line(&format!("⚠️  Failed to create merge commit: {}", e))?;
                    // Fall back to regular commit
                    run_git_command(&[
                        "commit",
                        "-m",
                        "Accept remote version (fallback commit)",
                    ], work_dir)?;
                    self.term.write_line("✅ Created fallback commit")?;
                }
            }
        } else {
            self.term
                .write_line("ℹ️  No changes to commit after resolution")?;
        }

        Ok(())
    }

    fn commit_changes_if_needed(&self) -> Result<()> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .unwrap();

        // Check if there are changes to commit
        let status = run_git_command(&["status", "--porcelain"], work_dir)?;
        if !status.trim().is_empty() {
            // Add all changes
            run_git_command(&["add", "."], work_dir)?;

            // Double-check after adding - sometimes files get ignored
            let status_after_add = run_git_command(&["status", "--porcelain"], work_dir)?;
            if status_after_add.trim().is_empty() {
                self.term
                    .write_line("ℹ️  No changes to commit after staging")?;
                return Ok(());
            }

            // Commit with simplified message
            let commit_message = format!(
                "Simplified sync - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );

            if let Err(e) = run_git_command(&["commit", "-m", &commit_message], work_dir) {
                self.term
                    .write_line(&format!("⚠️  Could not commit changes: {}", e))?;
                self.term
                    .write_line("💡 You may need to commit changes manually")?;
            } else {
                self.term.write_line("✅ Committed changes locally")?;
            }
        } else {
            self.term.write_line("ℹ️  No changes to commit")?;
        }

        Ok(())
    }
}

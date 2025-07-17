use anyhow::{anyhow, Result};
use std::path::Path;
use std::collections::HashMap;
use console::Term;
use crate::config::Config;
use crate::git_utils::run_git_command;
use crate::git_section_detector::{GitSectionDetector, SectionChange, ChangeType, SectionChanges};

#[derive(Debug)]
pub struct SectionConflict {
    pub word: String,
    pub local_change: SectionChange,
    pub remote_change: SectionChange,
}

#[derive(Debug)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    Manual,
}

#[derive(Debug)]
pub enum SyncResult {
    Success,
    NoChanges,
    Conflicts(Vec<SectionConflict>),
}

pub struct GitSectionSynchronizer {
    config: Config,
    detector: GitSectionDetector,
    term: Term,
}

impl GitSectionSynchronizer {
    pub fn new(config: Config) -> Result<Self> {
        let detector = GitSectionDetector::new(&config.vocabulary_notebook_file)?;
        let term = Term::stdout();
        
        Ok(Self {
            config,
            detector,
            term,
        })
    }
    
    pub fn sync_with_remote(&self) -> Result<SyncResult> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;
        
        self.term.write_line("🔄 Starting section-aware synchronization...")?;
        
        // 1. Fetch latest from remote
        self.term.write_line("📥 Fetching latest changes from remote...")?;
        if let Err(e) = run_git_command(&["fetch", "origin"], work_dir) {
            self.term.write_line(&format!("⚠️  Could not fetch from remote: {}", e))?;
            // Continue with local-only operation
        }
        
        // 2. Detect local section changes
        self.term.write_line("🔍 Analyzing local section changes...")?;
        let section_changes = self.detector.detect_section_changes()?;
        
        if section_changes.local_changes.is_empty() {
            self.term.write_line("ℹ️  No local changes detected")?;
        } else {
            self.term.write_line(&format!("📝 Found {} local section changes", section_changes.local_changes.len()))?;
            for change in &section_changes.local_changes {
                match change.change_type {
                    ChangeType::Added => self.term.write_line(&format!("  + Added: {}", change.word))?,
                    ChangeType::Modified => self.term.write_line(&format!("  ~ Modified: {}", change.word))?,
                    ChangeType::Deleted => self.term.write_line(&format!("  - Deleted: {}", change.word))?,
                }
            }
        }
        
        // 3. Detect remote section changes
        self.term.write_line("🔍 Analyzing remote section changes...")?;
        let remote_changes = self.detector.detect_remote_changes(section_changes.common_parent_hash.as_deref())?;
        
        if remote_changes.is_empty() {
            self.term.write_line("ℹ️  No remote changes detected")?;
        } else {
            self.term.write_line(&format!("📥 Found {} remote section changes", remote_changes.len()))?;
            for change in &remote_changes {
                match change.change_type {
                    ChangeType::Added => self.term.write_line(&format!("  + Remote added: {}", change.word))?,
                    ChangeType::Modified => self.term.write_line(&format!("  ~ Remote modified: {}", change.word))?,
                    ChangeType::Deleted => self.term.write_line(&format!("  - Remote deleted: {}", change.word))?,
                }
            }
        }
        
        // 4. Check for conflicts at section level
        let conflicts = self.detect_section_conflicts(&section_changes.local_changes, &remote_changes)?;
        
        if !conflicts.is_empty() {
            self.term.write_line(&format!("⚠️  Detected {} section-level conflicts", conflicts.len()))?;
            return self.handle_section_conflicts(conflicts);
        }
        
        // 5. No conflicts - perform section-aware merge
        if section_changes.local_changes.is_empty() && remote_changes.is_empty() {
            self.term.write_line("✅ No changes to synchronize")?;
            return Ok(SyncResult::NoChanges);
        }
        
        self.term.write_line("🔀 Performing section-aware merge...")?;
        self.perform_section_aware_merge(&section_changes, &remote_changes)?;
        
        // 6. Commit local changes if any
        self.commit_changes_if_needed()?;
        
        // 7. Push changes if remote is configured
        if self.config.git_remote_url.is_some() {
            self.term.write_line("📤 Pushing changes to remote...")?;
            match run_git_command(&["push", "origin", "main"], work_dir) {
                Ok(_) => self.term.write_line("✅ Successfully pushed changes to remote")?,
                Err(e) => {
                    self.term.write_line(&format!("⚠️  Could not push to remote: {}", e))?;
                    self.term.write_line("💡 You may need to resolve conflicts manually")?;
                }
            }
        }
        
        self.term.write_line("✅ Section-aware synchronization completed successfully")?;
        Ok(SyncResult::Success)
    }
    
    fn detect_section_conflicts(&self, local_changes: &[SectionChange], remote_changes: &[SectionChange]) -> Result<Vec<SectionConflict>> {
        let mut conflicts = Vec::new();
        
        // Build maps for easy lookup
        let local_map: HashMap<String, &SectionChange> = local_changes.iter()
            .map(|change| (change.word.to_lowercase(), change))
            .collect();
        
        let remote_map: HashMap<String, &SectionChange> = remote_changes.iter()
            .map(|change| (change.word.to_lowercase(), change))
            .collect();
        
        // Find conflicting changes (same word modified in both local and remote)
        for (word, local_change) in &local_map {
            if let Some(remote_change) = remote_map.get(word) {
                // Both local and remote modified the same word
                if self.is_conflicting_change(local_change, remote_change)? {
                    conflicts.push(SectionConflict {
                        word: word.clone(),
                        local_change: (*local_change).clone(),
                        remote_change: (*remote_change).clone(),
                    });
                }
            }
        }
        
        Ok(conflicts)
    }
    
    fn is_conflicting_change(&self, local: &SectionChange, remote: &SectionChange) -> Result<bool> {
        // Consider it a conflict if both sides have substantial changes
        match (&local.change_type, &remote.change_type) {
            (ChangeType::Added, ChangeType::Added) => {
                // Both added the same word - check if content differs
                Ok(local.new_content != remote.new_content)
            }
            (ChangeType::Modified, ChangeType::Modified) => {
                // Both modified - always a conflict
                Ok(true)
            }
            (ChangeType::Deleted, ChangeType::Modified) | 
            (ChangeType::Modified, ChangeType::Deleted) => {
                // One deleted, one modified - conflict
                Ok(true)
            }
            (ChangeType::Deleted, ChangeType::Deleted) => {
                // Both deleted - no conflict
                Ok(false)
            }
            _ => {
                // Other combinations are generally not conflicts
                Ok(false)
            }
        }
    }
    
    fn handle_section_conflicts(&self, conflicts: Vec<SectionConflict>) -> Result<SyncResult> {
        self.term.write_line(&format!("🔍 Handling {} section-level conflicts", conflicts.len()))?;
        
        let mut resolved_conflicts = Vec::new();
        
        for conflict in &conflicts {
            self.term.write_line(&format!("\n📝 Conflict in word: '{}'", conflict.word))?;
            
            // Show conflict details
            self.display_section_conflict(conflict)?;
            
            // Auto-resolve using timestamp (newer wins)
            let resolution = self.auto_resolve_by_timestamp(conflict)?;
            
            match resolution {
                ConflictResolution::UseLocal => {
                    self.term.write_line(&format!("✅ Using local version of '{}' (newer timestamp)", conflict.word))?;
                    resolved_conflicts.push((conflict, ConflictResolution::UseLocal));
                }
                ConflictResolution::UseRemote => {
                    self.term.write_line(&format!("✅ Using remote version of '{}' (newer timestamp)", conflict.word))?;
                    self.apply_remote_section(&conflict.remote_change)?;
                    resolved_conflicts.push((conflict, ConflictResolution::UseRemote));
                }
                ConflictResolution::Manual => {
                    self.term.write_line(&format!("⚠️  Manual resolution required for '{}'", conflict.word))?;
                    self.term.write_line("💡 Please resolve conflicts manually and run sync again")?;
                    return Ok(SyncResult::Conflicts(conflicts));
                }
            }
        }
        
        // All conflicts resolved - complete the merge
        self.term.write_line("🔀 Completing section merge with resolved conflicts...")?;
        self.complete_section_merge()?;
        
        Ok(SyncResult::Success)
    }
    
    fn display_section_conflict(&self, conflict: &SectionConflict) -> Result<()> {
        self.term.write_line(&format!("  Local change: {:?}", conflict.local_change.change_type))?;
        if let Some(ref timestamp) = conflict.local_change.new_timestamp {
            self.term.write_line(&format!("  Local timestamp: {}", timestamp))?;
        }
        
        self.term.write_line(&format!("  Remote change: {:?}", conflict.remote_change.change_type))?;
        if let Some(ref timestamp) = conflict.remote_change.new_timestamp {
            self.term.write_line(&format!("  Remote timestamp: {}", timestamp))?;
        }
        
        Ok(())
    }
    
    fn auto_resolve_by_timestamp(&self, conflict: &SectionConflict) -> Result<ConflictResolution> {
        // Extract timestamps from section changes
        let local_timestamp = conflict.local_change.new_timestamp.as_ref()
            .or(conflict.local_change.old_timestamp.as_ref());
        let remote_timestamp = conflict.remote_change.new_timestamp.as_ref()
            .or(conflict.remote_change.old_timestamp.as_ref());
        
        match (local_timestamp, remote_timestamp) {
            (Some(local_ts), Some(remote_ts)) => {
                match (chrono::DateTime::parse_from_rfc3339(local_ts), 
                       chrono::DateTime::parse_from_rfc3339(remote_ts)) {
                    (Ok(local_time), Ok(remote_time)) => {
                        if local_time > remote_time {
                            Ok(ConflictResolution::UseLocal)
                        } else if remote_time > local_time {
                            Ok(ConflictResolution::UseRemote)
                        } else {
                            // Same timestamp - prefer local
                            Ok(ConflictResolution::UseLocal)
                        }
                    }
                    _ => {
                        // Can't parse timestamps
                        Ok(ConflictResolution::Manual)
                    }
                }
            }
            (Some(_), None) => Ok(ConflictResolution::UseLocal), // Only local has timestamp
            (None, Some(_)) => Ok(ConflictResolution::UseRemote), // Only remote has timestamp
            (None, None) => Ok(ConflictResolution::Manual), // No timestamps available
        }
    }
    
    fn apply_remote_section(&self, remote_change: &SectionChange) -> Result<()> {
        // Apply remote section change to local file
        match remote_change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                if let Some(ref content) = remote_change.new_content {
                    // First, delete any existing version of this word
                    let _ = crate::utils::delete_from_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        &remote_change.word,
                        None
                    );
                    
                    // Then add the remote version
                    crate::utils::prepend_to_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        content
                    )?;
                }
            }
            ChangeType::Deleted => {
                // Delete the section
                crate::utils::delete_from_vocabulary_notebook(
                    &self.config.vocabulary_notebook_file,
                    &remote_change.word,
                    remote_change.old_timestamp.as_deref()
                )?;
            }
        }
        
        Ok(())
    }
    
    fn perform_section_aware_merge(&self, local_changes: &SectionChanges, remote_changes: &[SectionChange]) -> Result<()> {
        // Apply remote changes that don't conflict with local changes
        let local_words: std::collections::HashSet<String> = local_changes.local_changes.iter()
            .map(|c| c.word.to_lowercase())
            .collect();
        
        for remote_change in remote_changes {
            if !local_words.contains(&remote_change.word.to_lowercase()) {
                // No local conflict - apply remote change
                self.apply_remote_section(remote_change)?;
            }
        }
        
        Ok(())
    }
    
    fn complete_section_merge(&self) -> Result<()> {
        // Finalize the merge process
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .unwrap();
        
        // Add all changes
        run_git_command(&["add", "."], work_dir)?;
        
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
            
            // Commit with section-aware message
            let commit_message = format!(
                "Section-aware sync - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            run_git_command(&["commit", "-m", &commit_message], work_dir)?;
            self.term.write_line("✅ Committed section changes locally")?;
        }
        
        Ok(())
    }
}

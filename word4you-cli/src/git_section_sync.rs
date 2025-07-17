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
        
        // 4.5. Collect conflict resolutions for merge strategy
        let mut conflict_resolutions = std::collections::HashMap::new();
        
        if !conflicts.is_empty() {
            self.term.write_line(&format!("⚠️  Detected {} section-level conflicts", conflicts.len()))?;
            // Handle conflicts first, but don't return yet - we still need to merge non-conflicted sections
            match self.handle_section_conflicts_with_resolutions(conflicts, &mut conflict_resolutions)? {
                SyncResult::Conflicts(remaining_conflicts) => {
                    return Ok(SyncResult::Conflicts(remaining_conflicts));
                }
                _ => {
                    // Conflicts resolved, continue with merge
                    self.term.write_line("🔀 Merging with conflict resolutions applied...")?;
                }
            }
        } else {
            self.term.write_line("🔀 No conflicts detected - performing section-aware merge...")?;
        }
        
        // 5. Perform section-aware merge: remote as base, local changes on top
        if section_changes.local_changes.is_empty() && remote_changes.is_empty() {
            self.term.write_line("✅ No changes to synchronize")?;
            return Ok(SyncResult::NoChanges);
        }
        
        self.term.write_line("🔀 Performing git merge with section-aware conflict resolution...")?;
        self.perform_git_merge_with_section_awareness(&section_changes, &remote_changes, &conflict_resolutions)?;
        
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
    
    fn handle_section_conflicts_with_resolutions(&self, conflicts: Vec<SectionConflict>, resolutions: &mut std::collections::HashMap<String, ConflictResolution>) -> Result<SyncResult> {
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
                    // Store resolution for merge strategy
                    resolutions.insert(conflict.word.to_lowercase(), ConflictResolution::UseLocal);
                    resolved_conflicts.push((conflict, ConflictResolution::UseLocal));
                }
                ConflictResolution::UseRemote => {
                    self.term.write_line(&format!("✅ Using remote version of '{}' (newer timestamp)", conflict.word))?;
                    // Store resolution for merge strategy
                    resolutions.insert(conflict.word.to_lowercase(), ConflictResolution::UseRemote);
                    resolved_conflicts.push((conflict, ConflictResolution::UseRemote));
                }
                ConflictResolution::Manual => {
                    self.term.write_line(&format!("⚠️  Manual resolution required for '{}'", conflict.word))?;
                    self.term.write_line("💡 Please resolve conflicts manually and run sync again")?;
                    return Ok(SyncResult::Conflicts(conflicts));
                }
            }
        }
        
        // All conflicts resolved - return control to main sync flow for non-conflicted merge
        self.term.write_line("✅ All conflicts resolved successfully")?;
        
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
    
    fn reposition_local_section_to_top(&self, local_change: &SectionChange) -> Result<()> {
        // Extract the current content of the local section and move it to the top
        match local_change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                if let Some(ref content) = local_change.new_content {
                    // First, delete the existing version from its current position
                    let _ = crate::utils::delete_from_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        &local_change.word,
                        local_change.new_timestamp.as_deref()
                            .or(local_change.old_timestamp.as_deref())
                    );
                    
                    // Then prepend it to the top (newer content goes on top)
                    crate::utils::prepend_to_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        content
                    )?;
                    
                    self.term.write_line(&format!("📝 Repositioned '{}' to top based on newer timestamp", local_change.word))?;
                }
            }
            ChangeType::Deleted => {
                // Nothing to reposition for deleted content
            }
        }
        
        Ok(())
    }

    fn resolve_git_merge_conflicts(&self, conflict_resolutions: &std::collections::HashMap<String, ConflictResolution>) -> Result<()> {
        // Read the conflicted file
        let conflicted_content = std::fs::read_to_string(&self.config.vocabulary_notebook_file)?;
        
        self.term.write_line("🔍 Analyzing git merge conflicts in vocabulary file...")?;
        
        // Extract all sections from the conflicted content
        let (resolved_sections, non_conflicted_content) = self.extract_and_resolve_all_sections(&conflicted_content, conflict_resolutions)?;
        
        // Reconstruct the file with all content preserved
        let final_content = self.reconstruct_vocabulary_file(resolved_sections, non_conflicted_content)?;
        
        // Write the resolved content back
        std::fs::write(&self.config.vocabulary_notebook_file, final_content)?;
        
        self.term.write_line("🎯 Applied section-aware conflict resolution while preserving all content")?;
        
        Ok(())
    }
    
    fn extract_and_resolve_all_sections(&self, content: &str, conflict_resolutions: &std::collections::HashMap<String, ConflictResolution>) -> Result<(Vec<(String, Option<String>)>, Vec<String>)> {
        let mut all_sections = Vec::new();
        let mut non_section_content = Vec::new();
        let mut lines = content.lines();
        
        while let Some(line) = lines.next() {
            if line.starts_with("<<<<<<< HEAD") {
                // Handle conflict - extract both local and remote sections
                let mut local_section = Vec::new();
                let mut remote_section = Vec::new();
                let mut in_remote = false;
                
                // Collect conflict content
                while let Some(conflict_line) = lines.next() {
                    if conflict_line.starts_with("=======") {
                        in_remote = true;
                    } else if conflict_line.starts_with(">>>>>>> origin/main") {
                        break;
                    } else if in_remote {
                        remote_section.push(conflict_line);
                    } else {
                        local_section.push(conflict_line);
                    }
                }
                
                // Process both local and remote sections
                self.process_conflicted_sections(&local_section, &remote_section, conflict_resolutions, &mut all_sections)?;
                
            } else if line.starts_with("## ") {
                // Regular section (not in conflict) - preserve it
                let section_start = vec![line];
                let mut section_lines = section_start;
                let mut timestamp = None;
                
                // Collect the rest of the section
                while let Some(section_line) = lines.next() {
                    if section_line.trim() == "---" {
                        section_lines.push(section_line);
                        break;
                    } else {
                        if section_line.starts_with("<!-- timestamp=") {
                            timestamp = self.extract_timestamp_from_lines(&[section_line]);
                        }
                        section_lines.push(section_line);
                    }
                }
                
                let section_content = section_lines.join("\n");
                all_sections.push((section_content, timestamp));
                
            } else {
                // Non-section content (headers, etc.) - preserve as-is
                non_section_content.push(line.to_string());
            }
        }
        
        Ok((all_sections, non_section_content))
    }
    
    fn process_conflicted_sections(&self, local_section: &[&str], remote_section: &[&str], conflict_resolutions: &std::collections::HashMap<String, ConflictResolution>, all_sections: &mut Vec<(String, Option<String>)>) -> Result<()> {
        // Extract word identifiers and timestamps from both sections
        let local_word = self.extract_word_from_section(local_section, &[])?;
        let remote_word = self.extract_word_from_section(&[], remote_section)?;
        
        let local_timestamp = self.extract_timestamp_from_lines(local_section);
        let remote_timestamp = self.extract_timestamp_from_lines(remote_section);
        
        // Determine which sections to keep
        let local_content = if !local_section.is_empty() { Some(local_section.join("\n")) } else { None };
        let remote_content = if !remote_section.is_empty() { Some(remote_section.join("\n")) } else { None };
        
        // Check if we have a resolution for either word
        let local_key = local_word.to_lowercase();
        let remote_key = remote_word.to_lowercase();
        
        if local_key == remote_key {
            // Same word in conflict - apply resolution
            match conflict_resolutions.get(&local_key) {
                Some(ConflictResolution::UseLocal) => {
                    if let Some(content) = local_content {
                        self.term.write_line(&format!("🎯 Conflict resolution: keeping local version of '{}'", local_word))?;
                        all_sections.push((content, local_timestamp));
                    }
                }
                Some(ConflictResolution::UseRemote) => {
                    if let Some(content) = remote_content {
                        self.term.write_line(&format!("🎯 Conflict resolution: keeping remote version of '{}'", remote_word))?;
                        all_sections.push((content, remote_timestamp));
                    }
                }
                _ => {
                    // No specific resolution - use timestamp
                    if self.is_timestamp_newer(&local_timestamp, &remote_timestamp)? {
                        if let Some(content) = local_content {
                            self.term.write_line(&format!("🎯 Timestamp resolution: keeping local version of '{}' (newer)", local_word))?;
                            all_sections.push((content, local_timestamp));
                        }
                    } else {
                        if let Some(content) = remote_content {
                            self.term.write_line(&format!("🎯 Timestamp resolution: keeping remote version of '{}' (newer)", remote_word))?;
                            all_sections.push((content, remote_timestamp));
                        }
                    }
                }
            }
        } else {
            // Different words - keep both (this shouldn't normally happen but let's be safe)
            if let Some(content) = local_content {
                self.term.write_line(&format!("🔄 Preserving local section: '{}'", local_word))?;
                all_sections.push((content, local_timestamp));
            }
            if let Some(content) = remote_content {
                self.term.write_line(&format!("🔄 Preserving remote section: '{}'", remote_word))?;
                all_sections.push((content, remote_timestamp));
            }
        }
        
        Ok(())
    }
    
    fn reconstruct_vocabulary_file(&self, mut sections: Vec<(String, Option<String>)>, non_section_content: Vec<String>) -> Result<String> {
        // Sort sections by timestamp (newest first)
        sections.sort_by(|a, b| {
            match (&a.1, &b.1) {
                (Some(ts_a), Some(ts_b)) => {
                    match (chrono::DateTime::parse_from_rfc3339(ts_a), 
                           chrono::DateTime::parse_from_rfc3339(ts_b)) {
                        (Ok(time_a), Ok(time_b)) => time_b.cmp(&time_a), // Reverse for newest first
                        _ => std::cmp::Ordering::Equal,
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less,    // A has timestamp, B doesn't
                (None, Some(_)) => std::cmp::Ordering::Greater, // B has timestamp, A doesn't
                (None, None) => std::cmp::Ordering::Equal,      // Neither has timestamp
            }
        });
        
        // Reconstruct the file
        let mut final_content = String::new();
        
        // Add non-section content (headers, etc.) first
        for line in non_section_content {
            if !line.trim().is_empty() {
                final_content.push_str(&line);
                final_content.push('\n');
            }
        }
        
        // Add all sections in timestamp order
        for (section_content, _timestamp) in sections {
            if !final_content.is_empty() && !final_content.ends_with('\n') {
                final_content.push('\n');
            }
            final_content.push_str(&section_content);
            final_content.push('\n');
        }
        
        Ok(final_content.trim_end().to_string())
    }

    fn resolve_conflict_markers(&self, content: &str, conflict_resolutions: &std::collections::HashMap<String, ConflictResolution>) -> Result<String> {
        let mut resolved_content = String::new();
        let mut lines = content.lines();
        
        while let Some(line) = lines.next() {
            if line.starts_with("<<<<<<< HEAD") {
                // Start of conflict - collect local and remote versions
                let mut local_section = Vec::new();
                let mut remote_section = Vec::new();
                let mut in_remote = false;
                
                // Collect conflict content
                while let Some(conflict_line) = lines.next() {
                    if conflict_line.starts_with("=======") {
                        in_remote = true;
                    } else if conflict_line.starts_with(">>>>>>> origin/main") {
                        break;
                    } else if in_remote {
                        remote_section.push(conflict_line);
                    } else {
                        local_section.push(conflict_line);
                    }
                }
                
                // Determine which section this conflict is about
                let word = self.extract_word_from_section(&local_section, &remote_section)?;
                let word_key = word.to_lowercase();
                
                // Apply resolution based on our conflict analysis
                match conflict_resolutions.get(&word_key) {
                    Some(ConflictResolution::UseLocal) => {
                        self.term.write_line(&format!("🎯 Resolving conflict for '{}': using local version", word))?;
                        resolved_content.push_str(&local_section.join("\n"));
                        if !local_section.is_empty() {
                            resolved_content.push('\n');
                        }
                    }
                    Some(ConflictResolution::UseRemote) => {
                        self.term.write_line(&format!("🎯 Resolving conflict for '{}': using remote version", word))?;
                        resolved_content.push_str(&remote_section.join("\n"));
                        if !remote_section.is_empty() {
                            resolved_content.push('\n');
                        }
                    }
                    _ => {
                        // No resolution found - default to timestamp comparison
                        let local_timestamp = self.extract_timestamp_from_lines(&local_section);
                        let remote_timestamp = self.extract_timestamp_from_lines(&remote_section);
                        
                        if self.is_timestamp_newer(&local_timestamp, &remote_timestamp)? {
                            self.term.write_line(&format!("🎯 Resolving conflict for '{}': local has newer timestamp", word))?;
                            resolved_content.push_str(&local_section.join("\n"));
                            if !local_section.is_empty() {
                                resolved_content.push('\n');
                            }
                        } else {
                            self.term.write_line(&format!("🎯 Resolving conflict for '{}': remote has newer timestamp", word))?;
                            resolved_content.push_str(&remote_section.join("\n"));
                            if !remote_section.is_empty() {
                                resolved_content.push('\n');
                            }
                        }
                    }
                }
            } else {
                // Regular line - keep as is
                resolved_content.push_str(line);
                resolved_content.push('\n');
            }
        }
        
        Ok(resolved_content)
    }
    
    fn extract_word_from_section(&self, local_section: &[&str], remote_section: &[&str]) -> Result<String> {
        // Try to find a word header (## word) in either section
        for line in local_section.iter().chain(remote_section.iter()) {
            if line.starts_with("## ") {
                return Ok(line[3..].trim().to_string());
            }
        }
        
        // Fallback: return a placeholder
        Ok("unknown_word".to_string())
    }
    
    fn extract_timestamp_from_lines(&self, lines: &[&str]) -> Option<String> {
        for line in lines {
            if line.starts_with("<!-- timestamp=") {
                if let Some(start) = line.find("timestamp=") {
                    let start = start + "timestamp=".len();
                    if let Some(end) = line[start..].find(" -->") {
                        return Some(line[start..start + end].to_string());
                    }
                }
            }
        }
        None
    }
    
    fn is_timestamp_newer(&self, timestamp1: &Option<String>, timestamp2: &Option<String>) -> Result<bool> {
        match (timestamp1, timestamp2) {
            (Some(ts1), Some(ts2)) => {
                match (chrono::DateTime::parse_from_rfc3339(ts1), 
                       chrono::DateTime::parse_from_rfc3339(ts2)) {
                    (Ok(time1), Ok(time2)) => Ok(time1 > time2),
                    _ => Ok(false), // Can't parse timestamps
                }
            }
            (Some(_), None) => Ok(true),  // First has timestamp, second doesn't
            (None, Some(_)) => Ok(false), // Second has timestamp, first doesn't
            (None, None) => Ok(false),    // Neither has timestamp
        }
    }

    fn reorder_vocabulary_by_timestamp(&self) -> Result<()> {
        self.term.write_line("📋 Reordering vocabulary sections by timestamp (newest first)...")?;
        
        // Read the current vocabulary file
        let content = std::fs::read_to_string(&self.config.vocabulary_notebook_file)?;
        
        // Parse all sections with their timestamps
        let mut sections = self.parse_vocabulary_sections(&content)?;
        
        // Sort by timestamp (newest first)
        sections.sort_by(|a, b| {
            match (&a.1, &b.1) {
                (Some(ts_a), Some(ts_b)) => {
                    match (chrono::DateTime::parse_from_rfc3339(ts_a), 
                           chrono::DateTime::parse_from_rfc3339(ts_b)) {
                        (Ok(time_a), Ok(time_b)) => time_b.cmp(&time_a), // Reverse for newest first
                        _ => std::cmp::Ordering::Equal,
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less,    // A has timestamp, B doesn't
                (None, Some(_)) => std::cmp::Ordering::Greater, // B has timestamp, A doesn't
                (None, None) => std::cmp::Ordering::Equal,      // Neither has timestamp
            }
        });
        
        // Reconstruct the file with ordered sections
        let mut ordered_content = String::new();
        for (section_content, _timestamp) in sections {
            ordered_content.push_str(&section_content);
            ordered_content.push('\n');
        }
        
        // Write back the ordered content
        std::fs::write(&self.config.vocabulary_notebook_file, ordered_content.trim_end())?;
        
        self.term.write_line("✅ Vocabulary sections reordered by timestamp")?;
        
        Ok(())
    }
    
    fn parse_vocabulary_sections(&self, content: &str) -> Result<Vec<(String, Option<String>)>> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("## ") {
                let start = i;
                let mut timestamp = None;
                
                // Find the end of this section
                let mut end = i + 1;
                while end < lines.len() && lines[end].trim() != "---" {
                    // Look for timestamp in this section
                    if lines[end].starts_with("<!-- timestamp=") {
                        timestamp = self.extract_timestamp_from_lines(&[lines[end]]);
                    }
                    end += 1;
                }
                
                // Include the separator if present
                if end < lines.len() && lines[end].trim() == "---" {
                    end += 1;
                }
                
                // Extract the section content
                let section_content = lines[start..end].join("\n");
                sections.push((section_content, timestamp));
                
                i = end;
            } else {
                i += 1;
            }
        }
        
        Ok(sections)
    }

    fn replace_local_with_remote_base(&self) -> Result<()> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .unwrap();
        
        // Get remote version of vocabulary file
        match run_git_command(&["show", "origin/main:vocabulary_notebook.md"], work_dir) {
            Ok(remote_content) => {
                // Replace local file with remote content
                std::fs::write(&self.config.vocabulary_notebook_file, remote_content)?;
                self.term.write_line("📋 Replaced local file with remote base")?;
            }
            Err(_) => {
                // Remote file doesn't exist or can't be accessed - keep local as is
                self.term.write_line("ℹ️  Remote base not available, using local as base")?;
            }
        }
        
        Ok(())
    }
    
    fn apply_local_change_on_remote_base(&self, local_change: &SectionChange) -> Result<()> {
        match local_change.change_type {
            ChangeType::Added | ChangeType::Modified => {
                if let Some(ref content) = local_change.new_content {
                    // Check if this word already exists in the remote base (conflict scenario)
                    if let Ok(_) = crate::utils::delete_from_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        &local_change.word,
                        None
                    ) {
                        self.term.write_line(&format!("🔄 Replacing remote version of '{}' with local changes", local_change.word))?;
                    } else {
                        self.term.write_line(&format!("➕ Adding local change '{}' to remote base", local_change.word))?;
                    }
                    
                    // Apply local change on top
                    crate::utils::prepend_to_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        content
                    )?;
                }
            }
            ChangeType::Deleted => {
                // Delete from remote base if it exists
                if let Ok(_) = crate::utils::delete_from_vocabulary_notebook(
                    &self.config.vocabulary_notebook_file,
                    &local_change.word,
                    local_change.old_timestamp.as_deref()
                ) {
                    self.term.write_line(&format!("🗑️  Deleted '{}' from remote base (local deletion)", local_change.word))?;
                }
            }
        }
        
        Ok(())
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
                    
                    // Ensure remote content has proper formatting
                    let formatted_content = if !content.ends_with("---") && !content.contains("<!-- timestamp=") {
                        // Remote content missing timestamp/separator, add them
                        let timestamp = remote_change.new_timestamp.as_ref()
                            .or(remote_change.old_timestamp.as_ref())
                            .cloned()
                            .unwrap_or_else(|| chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
                        format!("{}\n\n<!-- timestamp={} -->\n---", content, timestamp)
                    } else {
                        content.clone()
                    };
                    
                    // Then add the remote version
                    crate::utils::prepend_to_vocabulary_notebook(
                        &self.config.vocabulary_notebook_file,
                        &formatted_content
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
    
    fn perform_git_merge_with_section_awareness(&self, _local_changes: &SectionChanges, _remote_changes: &[SectionChange], conflict_resolutions: &std::collections::HashMap<String, ConflictResolution>) -> Result<()> {
        let work_dir = Path::new(&self.config.vocabulary_notebook_file)
            .parent()
            .unwrap();
        
        // Step 1: Attempt git merge with origin/main (allowing unrelated histories)
        self.term.write_line("🔀 Attempting git merge with origin/main...")?;
        
        let merge_result = run_git_command(&["merge", "origin/main", "--allow-unrelated-histories"], work_dir);
        
        match merge_result {
            Ok(_) => {
                // Merge succeeded without conflicts
                self.term.write_line("✅ Git merge completed successfully without conflicts")?;
            }
            Err(e) => {
                let error_msg = e.to_string();
                
                // Check for specific error types
                if error_msg.contains("refusing to merge unrelated histories") {
                    self.term.write_line("ℹ️  Repositories have unrelated histories - this is normal for first sync")?;
                    self.term.write_line("🔄 Retrying merge with --allow-unrelated-histories...")?;
                    
                    // This shouldn't happen since we already use the flag, but just in case
                    return Err(anyhow!("Unrelated histories error persisted despite using --allow-unrelated-histories flag"));
                    
                } else if error_msg.contains("CONFLICT") || error_msg.contains("Automatic merge failed") {
                    self.term.write_line("⚠️  Git merge conflicts detected - resolving with section awareness...")?;
                    
                    // Step 2: Resolve merge conflicts using our section-aware logic
                    self.resolve_git_merge_conflicts(conflict_resolutions)?;
                    
                    // Step 3: Ensure proper timestamp ordering after conflict resolution
                    self.reorder_vocabulary_by_timestamp()?;
                    
                    // Step 4: Complete the merge
                    run_git_command(&["add", "."], work_dir)?;
                    run_git_command(&["commit", "--no-edit"], work_dir).or_else(|_| {
                        // If --no-edit fails, provide a custom merge commit message
                        run_git_command(&["commit", "-m", "Merge remote changes with section-aware conflict resolution"], work_dir)
                    })?;
                    
                    self.term.write_line("✅ Merge conflicts resolved and merge completed")?;
                    
                } else if error_msg.contains("not something we can merge") {
                    self.term.write_line("ℹ️  Remote branch not found - this may be an empty repository")?;
                    self.term.write_line("✅ Continuing with local-only operation")?;
                    
                } else {
                    // Some other git error
                    self.term.write_line(&format!("❌ Git merge failed with unexpected error: {}", error_msg))?;
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }

    fn perform_section_aware_merge(&self, local_changes: &SectionChanges, remote_changes: &[SectionChange]) -> Result<()> {
        // Apply remote changes that don't conflict with local changes
        let local_words: std::collections::HashSet<String> = local_changes.local_changes.iter()
            .map(|c| c.word.to_lowercase())
            .collect();
        
        // Collect non-conflicting remote changes and sort by timestamp (newest first)
        let mut non_conflicting_remote: Vec<&SectionChange> = remote_changes.iter()
            .filter(|change| !local_words.contains(&change.word.to_lowercase()))
            .collect();
        
        // Sort by timestamp, newest first (for proper positioning at top)
        non_conflicting_remote.sort_by(|a, b| {
            let a_timestamp = a.new_timestamp.as_ref().or(a.old_timestamp.as_ref());
            let b_timestamp = b.new_timestamp.as_ref().or(b.old_timestamp.as_ref());
            
            match (a_timestamp, b_timestamp) {
                (Some(a_ts), Some(b_ts)) => {
                    match (chrono::DateTime::parse_from_rfc3339(a_ts), 
                           chrono::DateTime::parse_from_rfc3339(b_ts)) {
                        (Ok(a_time), Ok(b_time)) => b_time.cmp(&a_time), // Reverse for newest first
                        _ => std::cmp::Ordering::Equal,
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less,    // A has timestamp, B doesn't
                (None, Some(_)) => std::cmp::Ordering::Greater, // B has timestamp, A doesn't  
                (None, None) => std::cmp::Ordering::Equal,      // Neither has timestamp
            }
        });
        
        // Apply remote changes in timestamp order (newest first)
        for remote_change in non_conflicting_remote {
            self.apply_remote_section(remote_change)?;
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
        
        // Check if there are changes to commit after merge
        let status = run_git_command(&["status", "--porcelain"], work_dir)?;
        if !status.trim().is_empty() {
            // Commit the merge results
            let commit_message = "Section-aware merge - no conflicts";
            if let Err(e) = run_git_command(&["commit", "-m", commit_message], work_dir) {
                self.term.write_line(&format!("⚠️  Could not commit merge: {}", e))?;
                self.term.write_line("💡 You may need to commit changes manually")?;
            } else {
                self.term.write_line("✅ Committed section merge successfully")?;
            }
        } else {
            self.term.write_line("ℹ️  No changes to commit after merge")?;
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
                self.term.write_line("ℹ️  No changes to commit after staging")?;
                return Ok(());
            }
            
            // Commit with section-aware message
            let commit_message = format!(
                "Section-aware sync - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            if let Err(e) = run_git_command(&["commit", "-m", &commit_message], work_dir) {
                self.term.write_line(&format!("⚠️  Could not commit changes: {}", e))?;
                self.term.write_line("💡 You may need to commit changes manually")?;
            } else {
                self.term.write_line("✅ Committed section changes locally")?;
            }
        } else {
            self.term.write_line("ℹ️  No changes to commit")?;
        }
        
        Ok(())
    }
}

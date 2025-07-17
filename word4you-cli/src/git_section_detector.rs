use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use crate::git_utils::run_git_command;

#[derive(Debug, Clone)]
pub struct SectionChange {
    pub change_type: ChangeType,
    pub word: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub old_timestamp: Option<String>,
    pub new_timestamp: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,    // New section (only in new version)
    Deleted,  // Removed section (only in old version)
    Modified, // Section content changed
}

pub struct SectionChanges {
    pub local_changes: Vec<SectionChange>,
    pub has_common_parent: bool,
    pub common_parent_hash: Option<String>,
}

pub struct GitSectionDetector {
    work_dir: PathBuf,
    vocabulary_file: String,
}

impl GitSectionDetector {
    pub fn new(vocabulary_file: &str) -> Result<Self> {
        let work_dir = Path::new(vocabulary_file)
            .parent()
            .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?
            .to_path_buf();
        
        Ok(Self {
            work_dir,
            vocabulary_file: vocabulary_file.to_string(),
        })
    }
    
    pub fn detect_section_changes(&self) -> Result<SectionChanges> {
        // 1. Find common parent with remote
        let common_parent = self.find_common_parent()?;
        
        match common_parent {
            Some(commit_hash) => {
                // Compare with common parent
                let local_changes = self.get_changes_since_commit(&commit_hash)?;
                Ok(SectionChanges {
                    local_changes,
                    has_common_parent: true,
                    common_parent_hash: Some(commit_hash),
                })
            }
            None => {
                // No common parent - treat as first sync
                let local_changes = self.get_all_local_sections_as_new()?;
                Ok(SectionChanges {
                    local_changes,
                    has_common_parent: false,
                    common_parent_hash: None,
                })
            }
        }
    }
    
    pub fn detect_remote_changes(&self, common_parent: Option<&str>) -> Result<Vec<SectionChange>> {
        match common_parent {
            Some(commit_hash) => {
                // Get diff from common parent to remote
                let diff_output = run_git_command(&[
                    "diff", 
                    commit_hash, 
                    "origin/main", 
                    "--", 
                    &self.get_relative_vocabulary_path()?
                ], &self.work_dir)?;
                
                self.parse_diff_for_sections(&diff_output)
            }
            None => {
                // No common parent - get all remote sections as new
                self.get_all_remote_sections_as_new()
            }
        }
    }
    
    fn find_common_parent(&self) -> Result<Option<String>> {
        // Try to find merge base with remote
        match run_git_command(&["merge-base", "HEAD", "origin/main"], &self.work_dir) {
            Ok(output) => {
                let hash = output.trim().to_string();
                if hash.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(hash))
                }
            }
            Err(_) => {
                // No common parent (e.g., first sync, unrelated histories)
                println!("ℹ️  No common parent found - treating as first sync");
                Ok(None)
            }
        }
    }
    
    fn get_changes_since_commit(&self, commit_hash: &str) -> Result<Vec<SectionChange>> {
        // Get git diff from common parent to HEAD
        let diff_output = run_git_command(&[
            "diff", 
            commit_hash, 
            "HEAD", 
            "--", 
            &self.get_relative_vocabulary_path()?
        ], &self.work_dir)?;
        
        // Parse the diff to identify changed sections
        self.parse_diff_for_sections(&diff_output)
    }
    
    fn get_all_local_sections_as_new(&self) -> Result<Vec<SectionChange>> {
        // Parse current vocabulary file and treat all sections as new
        let sections = self.parse_current_vocabulary_file()?;
        
        Ok(sections.into_iter().map(|(word, content, timestamp)| {
            SectionChange {
                change_type: ChangeType::Added,
                word,
                old_content: None,
                new_content: Some(content),
                old_timestamp: None,
                new_timestamp: Some(timestamp),
            }
        }).collect())
    }
    
    fn get_all_remote_sections_as_new(&self) -> Result<Vec<SectionChange>> {
        // Get remote version of vocabulary file
        let remote_content = match run_git_command(&[
            "show", 
            "origin/main:vocabulary_notebook.md"
        ], &self.work_dir) {
            Ok(content) => content,
            Err(_) => {
                // Remote file doesn't exist or can't be accessed
                return Ok(Vec::new());
            }
        };
        
        // Parse remote sections
        let sections = self.parse_vocabulary_content(&remote_content)?;
        
        Ok(sections.into_iter().map(|(word, content, timestamp)| {
            SectionChange {
                change_type: ChangeType::Added,
                word,
                old_content: None,
                new_content: Some(content),
                old_timestamp: None,
                new_timestamp: Some(timestamp),
            }
        }).collect())
    }
    
    fn parse_diff_for_sections(&self, diff_output: &str) -> Result<Vec<SectionChange>> {
        let mut section_changes = Vec::new();
        let diff_lines: Vec<&str> = diff_output.lines().collect();
        
        if diff_lines.is_empty() {
            return Ok(section_changes);
        }
        
        let mut i = 0;
        while i < diff_lines.len() {
            let line = diff_lines[i];
            
            // Look for diff hunks (@@)
            if line.starts_with("@@") {
                let hunk_changes = self.parse_hunk_content(&diff_lines, &mut i)?;
                section_changes.extend(hunk_changes);
            } else {
                i += 1;
            }
        }
        
        Ok(section_changes)
    }
    
    fn parse_hunk_content(&self, lines: &[&str], i: &mut usize) -> Result<Vec<SectionChange>> {
        let mut changes = Vec::new();
        let mut current_section: Option<SectionBuilder> = None;
        
        *i += 1; // Skip hunk header
        
        while *i < lines.len() && !lines[*i].starts_with("@@") {
            let line = lines[*i];
            
            if line.is_empty() {
                *i += 1;
                continue;
            }
            
            match line.chars().next() {
                Some('+') => {
                    // Added line
                    let content = &line[1..]; // Remove '+'
                    if content.starts_with("## ") {
                        // New section starting
                        if let Some(builder) = current_section.take() {
                            if let Some(change) = builder.build()? {
                                changes.push(change);
                            }
                        }
                        current_section = Some(SectionBuilder::new_added(&content[3..]));
                    } else if let Some(ref mut builder) = current_section {
                        builder.add_new_content_line(content);
                    }
                }
                Some('-') => {
                    // Deleted line
                    let content = &line[1..]; // Remove '-'
                    if content.starts_with("## ") {
                        // Section being deleted
                        if let Some(builder) = current_section.take() {
                            if let Some(change) = builder.build()? {
                                changes.push(change);
                            }
                        }
                        current_section = Some(SectionBuilder::new_deleted(&content[3..]));
                    } else if let Some(ref mut builder) = current_section {
                        builder.add_old_content_line(content);
                    }
                }
                Some(' ') => {
                    // Unchanged line (context) - skip for now
                }
                _ => {}
            }
            
            *i += 1;
        }
        
        // Finish last section
        if let Some(builder) = current_section {
            if let Some(change) = builder.build()? {
                changes.push(change);
            }
        }
        
        Ok(changes)
    }
    
    fn parse_current_vocabulary_file(&self) -> Result<Vec<(String, String, String)>> {
        let content = std::fs::read_to_string(&self.vocabulary_file)?;
        self.parse_vocabulary_content(&content)
    }
    
    fn parse_vocabulary_content(&self, content: &str) -> Result<Vec<(String, String, String)>> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("## ") {
                let word = lines[i][3..].trim().to_string();
                let start = i;
                
                // Find section end
                let mut end = i + 1;
                let mut timestamp = String::new();
                
                while end < lines.len() && lines[end].trim() != "---" {
                    if lines[end].starts_with("<!-- timestamp=") {
                        timestamp = self.extract_timestamp_from_line(lines[end])?;
                    }
                    end += 1;
                }
                
                if end < lines.len() {
                    end += 1; // Include the "---" line
                }
                
                let section_content = lines[start..end].join("\n");
                sections.push((word, section_content, timestamp));
                
                i = end;
            } else {
                i += 1;
            }
        }
        
        Ok(sections)
    }
    
    fn extract_timestamp_from_line(&self, line: &str) -> Result<String> {
        // Extract timestamp from <!-- timestamp=2023-01-01T12:00:00.123+00:00 -->
        if let Some(start) = line.find("timestamp=") {
            let start = start + "timestamp=".len();
            if let Some(end) = line[start..].find(" -->") {
                return Ok(line[start..start + end].to_string());
            }
        }
        Ok(String::new())
    }
    
    fn get_relative_vocabulary_path(&self) -> Result<String> {
        let vocab_path = Path::new(&self.vocabulary_file);
        let relative_path = vocab_path.strip_prefix(&self.work_dir)
            .map_err(|_| anyhow!("Vocabulary file is not within work directory"))?;
        Ok(relative_path.to_string_lossy().to_string())
    }
}

// Helper struct for building section changes from diff parsing
struct SectionBuilder {
    word: String,
    change_type: ChangeType,
    old_content: Vec<String>,
    new_content: Vec<String>,
    old_timestamp: Option<String>,
    new_timestamp: Option<String>,
}

impl SectionBuilder {
    fn new_added(word: &str) -> Self {
        Self {
            word: word.to_string(),
            change_type: ChangeType::Added,
            old_content: Vec::new(),
            new_content: vec![format!("## {}", word)],
            old_timestamp: None,
            new_timestamp: None,
        }
    }
    
    fn new_deleted(word: &str) -> Self {
        Self {
            word: word.to_string(),
            change_type: ChangeType::Deleted,
            old_content: vec![format!("## {}", word)],
            new_content: Vec::new(),
            old_timestamp: None,
            new_timestamp: None,
        }
    }
    
    fn add_new_content_line(&mut self, line: &str) {
        self.new_content.push(line.to_string());
        
        // Extract timestamp if present
        if line.starts_with("<!-- timestamp=") {
            if let Some(start) = line.find("timestamp=") {
                let start = start + "timestamp=".len();
                if let Some(end) = line[start..].find(" -->") {
                    self.new_timestamp = Some(line[start..start + end].to_string());
                }
            }
        }
    }
    
    fn add_old_content_line(&mut self, line: &str) {
        self.old_content.push(line.to_string());
        
        // Extract timestamp if present
        if line.starts_with("<!-- timestamp=") {
            if let Some(start) = line.find("timestamp=") {
                let start = start + "timestamp=".len();
                if let Some(end) = line[start..].find(" -->") {
                    self.old_timestamp = Some(line[start..start + end].to_string());
                }
            }
        }
    }
    
    fn build(self) -> Result<Option<SectionChange>> {
        // Only return a change if we have meaningful content
        if self.old_content.is_empty() && self.new_content.is_empty() {
            return Ok(None);
        }
        
        let change_type = if !self.old_content.is_empty() && !self.new_content.is_empty() {
            ChangeType::Modified
        } else {
            self.change_type
        };
        
        Ok(Some(SectionChange {
            change_type,
            word: self.word,
            old_content: if self.old_content.is_empty() { None } else { Some(self.old_content.join("\n")) },
            new_content: if self.new_content.is_empty() { None } else { Some(self.new_content.join("\n")) },
            old_timestamp: self.old_timestamp,
            new_timestamp: self.new_timestamp,
        }))
    }
}

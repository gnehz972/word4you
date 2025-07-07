use anyhow::{anyhow, Result};
use chrono::Utc;
use git2::{Repository, Signature};
use std::fs;
use std::path::Path;

pub fn ensure_wordbook_exists(wordbook_file: &str) -> Result<()> {
    if !Path::new(wordbook_file).exists() {
        let content = r#"# My Vocabulary Notebook

This is my personal collection of English words with explanations.

---

"#;
        fs::write(wordbook_file, content)?;
    }
    Ok(())
}

pub fn prepend_to_wordbook(wordbook_file: &str, content: &str) -> Result<()> {
    ensure_wordbook_exists(wordbook_file)?;
    
    // Read existing content
    let existing_content = fs::read_to_string(wordbook_file)?;
    
    // Prepend new content
    let new_content = format!("{}\n\n---\n\n{}", content, existing_content);
    fs::write(wordbook_file, new_content)?;
    
    Ok(())
}

pub fn format_commit_message(word: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    format!("Add word: {} - {}", word, timestamp)
}

pub fn init_git_repo() -> Result<Repository> {
    match Repository::open(".") {
        Ok(repo) => Ok(repo),
        Err(_) => {
            // Initialize new repository
            let repo = Repository::init(".")?;
            Ok(repo)
        }
    }
}

pub fn commit_and_push_changes(commit_message: &str, git_remote_url: Option<&str>) -> Result<()> {
    let repo = init_git_repo()?;
    
    // Add all changes
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    
    // Create signature
    let signature = Signature::now("Word4You", "word4you@example.com")?;
    
    // Commit changes
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent_commit = repo.head().ok().and_then(|head| head.target()).and_then(|oid| repo.find_commit(oid).ok());
    
    let _commit_id = if let Some(parent) = parent_commit {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            commit_message,
            &tree,
            &[&parent],
        )?
    } else {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            commit_message,
            &tree,
            &[],
        )?
    };
    
    // Push to remote if configured
    if let Some(remote_url) = git_remote_url {
        let mut remote = match repo.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => {
                // Add remote if it doesn't exist
                repo.remote("origin", remote_url)?
            }
        };
        
        remote.push(&["refs/heads/main:refs/heads/main"], None)?;
        println!("✅ Changes committed and pushed to remote repository");
    } else {
        println!("✅ Changes committed to local repository");
    }
    
    Ok(())
}

pub fn validate_word(word: &str) -> Result<()> {
    if word.trim().is_empty() {
        return Err(anyhow!("Word cannot be empty"));
    }
    
    let word = word.trim();
    
    // Check if it contains only letters and hyphens
    if !word.chars().all(|c| c.is_ascii_alphabetic() || c == '-') {
        return Err(anyhow!("Word can only contain letters and hyphens"));
    }
    
    // Check length
    if word.len() < 1 || word.len() > 50 {
        return Err(anyhow!("Word length must be between 1 and 50 characters"));
    }
    
    Ok(())
} 
use anyhow::{anyhow, Result};
use chrono::Utc;
use git2::{Repository, Signature};
use std::fs;
use std::fs::File;
use std::path::Path;

pub fn ensure_vocabulary_notebook_exists(vocabulary_notebook_file: &str) -> Result<()> {
    let path = Path::new(vocabulary_notebook_file);
    
    // Create word4you directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            println!("📁 Created word4you directory: {}", parent.display());
        }
    }
    
    // Create empty file if it doesn't exist
    if !path.exists() {
        File::create(vocabulary_notebook_file)?;
        println!("📄 Created vocabulary notebook: {}", vocabulary_notebook_file);
    }
    Ok(())
}

pub fn prepend_to_vocabulary_notebook(vocabulary_notebook_file: &str, content: &str) -> Result<()> {
    ensure_vocabulary_notebook_exists(vocabulary_notebook_file)?;
    
    // Read existing content
    let existing_content = fs::read_to_string(vocabulary_notebook_file)?;
    
    // Prepend new content
    let new_content = format!("{}\n\n---\n\n{}", content, existing_content);
    fs::write(vocabulary_notebook_file, new_content)?;
    
    Ok(())
}

pub fn format_commit_message(word: &str) -> String {
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    format!("Add word: {} - {}", word, timestamp)
}

pub fn init_git_repo(vocabulary_notebook_file: &str) -> Result<Repository> {
    let notebook_path = Path::new(vocabulary_notebook_file);
    let word4you_dir = notebook_path.parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary notebook file path"))?;
    
    match Repository::open(word4you_dir) {
        Ok(repo) => Ok(repo),
        Err(_) => {
            // Initialize new repository in the word4you directory
            let repo = Repository::init(word4you_dir)?;
            println!("🔧 Initialized git repository in: {}", word4you_dir.display());
            Ok(repo)
        }
    }
}

pub fn commit_and_push_changes(commit_message: &str, vocabulary_notebook_file: &str, git_remote_url: Option<&str>) -> Result<()> {
    let repo = init_git_repo(vocabulary_notebook_file)?;
    
    // Add all files in the word4you directory (since it's a dedicated directory)
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
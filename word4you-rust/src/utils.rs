use anyhow::{anyhow, Result};
use chrono::Utc;
use git2::{Repository, Signature, RemoteCallbacks, PushOptions, Cred, Oid, DiffOptions};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::env;

fn perform_vocabulary_merge(repo: &Repository, vocabulary_file: &str, remote_oid: Oid, local_oid: Oid) -> Result<()> {
    // Get relative path for the vocabulary file within the repository
    let repo_path = repo.workdir().ok_or_else(|| anyhow!("Repository has no working directory"))?;
    let vocabulary_path = Path::new(vocabulary_file);
    let relative_path = vocabulary_path.strip_prefix(repo_path)
        .unwrap_or_else(|_| vocabulary_path.file_name().map(Path::new).unwrap_or(vocabulary_path));
    
    // Find the merge base (common ancestor)
    let merge_base_oid = repo.merge_base(local_oid, remote_oid)?;
    
    // Step 1: Get local additions since base using Git diff
    let local_additions = get_local_additions_from_diff(repo, merge_base_oid, local_oid, relative_path)?;
    
    // Step 2: Checkout remote content to working directory
    checkout_remote_file_to_working_directory(repo, remote_oid, vocabulary_file, relative_path)?;
    
    // Step 3: Apply local additions to the file
    apply_local_additions_to_file(vocabulary_file, &local_additions)?;
    
    // Stage the merged file
    let mut index = repo.index()?;
    index.add_path(relative_path)?;
    index.write()?;
    
    // Create merge commit with both parents
    let signature = Signature::now("Word4You", "word4you@example.com")?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let local_commit = repo.find_commit(local_oid)?;
    let remote_commit = repo.find_commit(remote_oid)?;
    
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Merge vocabulary entries from multiple devices",
        &tree,
        &[&local_commit, &remote_commit],
    )?;
    
    Ok(())
}

fn get_local_additions_from_diff(repo: &Repository, base_oid: Oid, local_oid: Oid, file_path: &Path) -> Result<String> {
    let base_commit = repo.find_commit(base_oid)?;
    let local_commit = repo.find_commit(local_oid)?;
    let base_tree = base_commit.tree()?;
    let local_tree = local_commit.tree()?;
    
    // Create diff between base and local
    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(file_path);
    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&local_tree), Some(&mut diff_options))?;
    
    let mut additions = String::new();
    
    // Process the diff to extract added lines
    diff.foreach(
        &mut |_delta, _progress| true,
        None,
        Some(&mut |_delta, _hunk| true),
        Some(&mut |_delta, _hunk, line| {
            // Only process added lines (lines that start with '+')
            if line.origin() == '+' {
                if let Ok(line_content) = std::str::from_utf8(line.content()) {
                    additions.push_str(line_content);
                }
            }
            true
        }),
    )?;
    
    Ok(additions)
}


fn checkout_remote_file_to_working_directory(repo: &Repository, remote_oid: Oid, vocabulary_file: &str, file_path: &Path) -> Result<()> {
    let remote_commit = repo.find_commit(remote_oid)?;
    let remote_tree = remote_commit.tree()?;
    
    match remote_tree.get_path(file_path) {
        Ok(entry) => {
            let blob = repo.find_blob(entry.id())?;
            fs::write(vocabulary_file, blob.content())?;
        }
        Err(_) => {
            // File doesn't exist in remote, create empty file
            fs::write(vocabulary_file, "")?;
        }
    }
    
    Ok(())
}

fn apply_local_additions_to_file(vocabulary_file: &str, local_additions: &str) -> Result<()> {
    if !local_additions.trim().is_empty() {
        // Read existing content
        let existing_content = fs::read_to_string(vocabulary_file)?;
        
        // Simply prepend local additions to the top of the file - no extra separators
        let new_content = format!("{}{}", local_additions, existing_content);
        
        // Write the new content back to the file
        fs::write(vocabulary_file, new_content)?;
    }
    
    Ok(())
}




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
            
            // Set the default branch to 'main'
            let mut config = repo.config()?;
            config.set_str("init.defaultBranch", "main")?;
            
            println!("🔧 Initialized git repository with main branch in: {}", word4you_dir.display());
            Ok(repo)
        }
    }
}

pub fn commit_and_push_changes(commit_message: &str, vocabulary_notebook_file: &str, git_remote_url: Option<&str>, ssh_private_key_path: Option<&str>, ssh_public_key_path: Option<&str>) -> Result<()> {
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
        // First commit - explicitly create main branch
        let commit_id = repo.commit(
            Some("refs/heads/main"),
            &signature,
            &signature,
            commit_message,
            &tree,
            &[],
        )?;
        
        // Set HEAD to point to main branch
        repo.set_head("refs/heads/main")?;
        
        commit_id
    };
    
    // Print success message after commit
    println!("✅ Successfully committed word locally");
    
    // Push to remote if configured
    if let Some(remote_url) = git_remote_url {
        // Get the current branch name
        let head_ref = match repo.head() {
            Ok(head) => head,
            Err(_) => {
                println!("✅ Changes committed to local repository (no HEAD reference yet)");
                return Ok(());
            }
        };
        
        let branch_name = match head_ref.shorthand() {
            Some(name) => name,
            None => {
                println!("✅ Changes committed to local repository (unable to determine branch name)");
                return Ok(());
            }
        };
        
        let mut remote = match repo.find_remote("origin") {
            Ok(remote) => remote,
            Err(_) => {
                // Add remote if it doesn't exist
                repo.remote("origin", remote_url)?
            }
        };
        
        // Helper function to create authentication callbacks
        let create_callbacks = || {
            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|_url, username_from_url, _allowed_types| {
                // Try SSH key authentication first
                if let Some(username) = username_from_url {
                    // Try provided SSH key paths (either from env vars or defaults)
                    if let (Some(private_key), Some(public_key)) = (ssh_private_key_path, ssh_public_key_path) {
                        if std::path::Path::new(private_key).exists() && std::path::Path::new(public_key).exists() {
                            return Cred::ssh_key(username, Some(std::path::Path::new(public_key)), std::path::Path::new(private_key), None);
                        }
                    }
                    
                    // Try additional common SSH key locations as fallback
                    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let fallback_keys = vec![
                        (format!("{}/.ssh/id_rsa", home_dir), format!("{}/.ssh/id_rsa.pub", home_dir)),
                    ];
                    
                    for (private_key, public_key) in fallback_keys {
                        if std::path::Path::new(&private_key).exists() && std::path::Path::new(&public_key).exists() {
                            return Cred::ssh_key(username, Some(std::path::Path::new(&public_key)), std::path::Path::new(&private_key), None);
                        }
                    }
                }
                
                // Fall back to SSH agent if available
                if let Some(username) = username_from_url {
                    return Cred::ssh_key_from_agent(username);
                }
                
                // Last resort: try default credential helper
                Cred::default()
            });
            callbacks
        };
        
        // Try to fetch first to check for remote changes
        let fetch_refspec = format!("+refs/heads/{}:refs/remotes/origin/{}", branch_name, branch_name);
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(create_callbacks());
        
        match remote.fetch(&[&fetch_refspec], Some(&mut fetch_options), None) {
            Ok(_) => {
                // Check if we need to merge
                let remote_ref_name = format!("refs/remotes/origin/{}", branch_name);
                if let Ok(remote_ref) = repo.find_reference(&remote_ref_name) {
                    if let Some(remote_oid) = remote_ref.target() {
                        let local_oid = repo.head()?.target().unwrap_or_else(|| panic!("No local HEAD"));
                        
                        if remote_oid != local_oid {
                            // Check if local is ahead of remote (fast-forward case)
                            let merge_base = repo.merge_base(local_oid, remote_oid)?;
                            
                            if merge_base == remote_oid {
                                // Local is ahead of remote, normal push should work
                                println!("📤 Local changes are ahead of remote, pushing...");
                            } else if merge_base == local_oid {
                                // Remote is ahead of local, need to merge
                                println!("🔄 Remote has new changes, performing intelligent merge...");
                                
                                if let Err(e) = perform_vocabulary_merge(&repo, vocabulary_notebook_file, remote_oid, local_oid) {
                                    println!("⚠️  Merge failed: {}. Changes saved locally only.", e);
                                    return Ok(());
                                }
                                
                                println!("✅ Successfully merged vocabulary entries from remote");
                            } else {
                                // Branches have diverged, need intelligent merge
                                println!("🔄 Branches have diverged, performing intelligent merge...");
                                
                                if let Err(e) = perform_vocabulary_merge(&repo, vocabulary_notebook_file, remote_oid, local_oid) {
                                    println!("⚠️  Merge failed: {}. Changes saved locally only.", e);
                                    return Ok(());
                                }
                                
                                println!("✅ Successfully merged vocabulary entries from multiple devices");
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // Fetch failed, might be a new remote branch, continue with push
            }
        }
        
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(create_callbacks());
        
        // Use the actual branch name instead of hardcoded "main"
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        match remote.push(&[&refspec], Some(&mut push_options)) {
            Ok(_) => println!("✅ Successfully pushed word to remote"),
            Err(e) if e.code() == git2::ErrorCode::NotFastForward => {
                println!("⚠️  Cannot push: remote has newer commits. Changes saved locally.");
                println!("   Run 'git pull' manually in the word4you directory to sync, then try again.");
            }
            Err(e) => return Err(e.into()),
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_local_additions_to_file() {
        // Create a temporary file
        let temp_file = "/tmp/test_vocab.md";
        fs::write(temp_file, "Remote content").unwrap();
        
        // Apply local additions (simulating what Git diff would provide - including separator)
        let local_additions = "Local addition line 1\nLocal addition line 2\n\n---\n\n";
        apply_local_additions_to_file(temp_file, local_additions).unwrap();
        
        // Read the result
        let result = fs::read_to_string(temp_file).unwrap();
        
        // Should contain both remote content and local additions
        assert!(result.contains("Remote content"));
        assert!(result.contains("Local addition line 1"));
        assert!(result.contains("Local addition line 2"));
        
        // Local additions should be at the top (before remote content)
        assert!(result.starts_with("Local addition line 1"));
        assert!(result.ends_with("Remote content"));
        
        // Should have exactly one separator (from the diff, not added extra)
        assert_eq!(result.matches("---").count(), 1);
        
        // Clean up
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_apply_local_additions_empty_file() {
        // Create an empty file
        let temp_file = "/tmp/test_vocab_empty.md";
        fs::write(temp_file, "").unwrap();
        
        // Apply local additions (exactly as they come from diff)
        let local_additions = "First addition\n\n---\n\n";
        apply_local_additions_to_file(temp_file, local_additions).unwrap();
        
        // Read the result
        let result = fs::read_to_string(temp_file).unwrap();
        
        // Should contain exactly what was in the diff
        assert_eq!(result, "First addition\n\n---\n\n");
        
        // Clean up
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_apply_local_additions_empty_additions() {
        // Create a file with content
        let temp_file = "/tmp/test_vocab_no_additions.md";
        let original_content = "Original content";
        fs::write(temp_file, original_content).unwrap();
        
        // Apply empty additions
        let local_additions = "";
        apply_local_additions_to_file(temp_file, local_additions).unwrap();
        
        // Read the result
        let result = fs::read_to_string(temp_file).unwrap();
        
        // Should remain unchanged
        assert_eq!(result, original_content);
        
        // Clean up
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_validate_word() {
        // Valid words
        assert!(validate_word("hello").is_ok());
        assert!(validate_word("test-word").is_ok());
        assert!(validate_word("a").is_ok());
        
        // Invalid words
        assert!(validate_word("").is_err());
        assert!(validate_word("   ").is_err());
        assert!(validate_word("hello123").is_err());
        assert!(validate_word("hello@world").is_err());
        
        // Word too long
        let long_word = "a".repeat(51);
        assert!(validate_word(&long_word).is_err());
    }

} 
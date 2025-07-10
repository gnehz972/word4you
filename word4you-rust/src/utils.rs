use anyhow::{anyhow, Result};
use chrono::Utc;
use git2::{Repository, Signature, RemoteCallbacks, PushOptions, Cred, Oid};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::env;
use std::collections::HashSet;

fn perform_vocabulary_merge(repo: &Repository, vocabulary_file: &str, remote_oid: Oid, local_oid: Oid) -> Result<()> {
    // Get relative path for the vocabulary file within the repository
    let repo_path = repo.workdir().ok_or_else(|| anyhow!("Repository has no working directory"))?;
    let vocabulary_path = Path::new(vocabulary_file);
    let relative_path = vocabulary_path.strip_prefix(repo_path)
        .unwrap_or_else(|_| vocabulary_path.file_name().map(Path::new).unwrap_or(vocabulary_path));
    
    // Find the merge base (common ancestor)
    let merge_base_oid = repo.merge_base(local_oid, remote_oid)?;
    
    // Get file content from all three states: base, local, remote
    let base_content = get_file_content_from_commit(repo, merge_base_oid, relative_path)?;
    let local_content = get_file_content_from_commit(repo, local_oid, relative_path)?;
    let remote_content = get_file_content_from_commit(repo, remote_oid, relative_path)?;
    
    // Perform 3-way merge
    let merged_content = three_way_merge_vocabulary(&base_content, &local_content, &remote_content)?;
    
    // Write merged content back to file
    fs::write(vocabulary_file, merged_content)?;
    
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

fn get_file_content_from_commit(repo: &Repository, commit_oid: Oid, file_path: &Path) -> Result<String> {
    let commit = repo.find_commit(commit_oid)?;
    let tree = commit.tree()?;
    
    match tree.get_path(file_path) {
        Ok(entry) => {
            let blob = repo.find_blob(entry.id())?;
            Ok(String::from_utf8_lossy(blob.content()).to_string())
        }
        Err(_) => Ok(String::new()), // File doesn't exist in this commit
    }
}

fn three_way_merge_vocabulary(base_content: &str, local_content: &str, remote_content: &str) -> Result<String> {
    // Parse all three versions
    let base_entries = parse_vocabulary_entries(base_content);
    let local_entries = parse_vocabulary_entries(local_content);
    let remote_entries = parse_vocabulary_entries(remote_content);
    
    // Create sets for easier comparison
    let base_words: HashSet<String> = base_entries.iter()
        .filter_map(|entry| extract_word_from_entry(entry))
        .map(|word| word.to_lowercase())
        .collect();
    
    let local_words: HashSet<String> = local_entries.iter()
        .filter_map(|entry| extract_word_from_entry(entry))
        .map(|word| word.to_lowercase())
        .collect();
    
    let remote_words: HashSet<String> = remote_entries.iter()
        .filter_map(|entry| extract_word_from_entry(entry))
        .map(|word| word.to_lowercase())
        .collect();
    
    let mut merged_entries = Vec::new();
    let mut seen_words = HashSet::new();
    
    // Add entries that are new in local (not in base)
    for entry in &local_entries {
        if let Some(word) = extract_word_from_entry(entry) {
            let word_lower = word.to_lowercase();
            if !base_words.contains(&word_lower) && seen_words.insert(word_lower) {
                merged_entries.push(entry.clone());
            }
        }
    }
    
    // Add entries that are new in remote (not in base)
    for entry in &remote_entries {
        if let Some(word) = extract_word_from_entry(entry) {
            let word_lower = word.to_lowercase();
            if !base_words.contains(&word_lower) && seen_words.insert(word_lower) {
                merged_entries.push(entry.clone());
            }
        }
    }
    
    // Add entries that existed in base and still exist in either local or remote
    for entry in base_entries {
        if let Some(word) = extract_word_from_entry(&entry) {
            let word_lower = word.to_lowercase();
            if (local_words.contains(&word_lower) || remote_words.contains(&word_lower)) 
                && seen_words.insert(word_lower.clone()) {
                // Use the version from local if available, otherwise remote
                if local_words.contains(&word_lower) {
                    if let Some(local_entry) = local_entries.iter().find(|e| {
                        extract_word_from_entry(e).map(|w| w.to_lowercase()) == Some(word_lower.clone())
                    }) {
                        merged_entries.push(local_entry.clone());
                    }
                } else if let Some(remote_entry) = remote_entries.iter().find(|e| {
                    extract_word_from_entry(e).map(|w| w.to_lowercase()) == Some(word_lower.clone())
                }) {
                    merged_entries.push(remote_entry.clone());
                }
            }
        }
    }
    
    // Join entries back with separator
    Ok(merged_entries.join("\n\n---\n\n"))
}


fn parse_vocabulary_entries(content: &str) -> Vec<String> {
    content
        .split("\n---\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn extract_word_from_entry(entry: &str) -> Option<&str> {
    // Extract word from markdown header "## word"
    entry
        .lines()
        .find(|line| line.starts_with("## "))
        .map(|line| line.trim_start_matches("## ").trim())
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
                    // Try default SSH key locations
                    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let ssh_key_path = format!("{}/.ssh/id_ed25519_personal", home_dir);
                    let ssh_pub_key_path = format!("{}/.ssh/id_ed25519_personal.pub", home_dir);
                    
                    if std::path::Path::new(&ssh_key_path).exists() {
                        return Cred::ssh_key(username, Some(std::path::Path::new(&ssh_pub_key_path)), std::path::Path::new(&ssh_key_path), None);
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
            Ok(_) => println!("✅ Changes committed and pushed to remote repository (branch: {})", branch_name),
            Err(e) if e.code() == git2::ErrorCode::NotFastForward => {
                println!("⚠️  Cannot push: remote has newer commits. Changes saved locally.");
                println!("   Run 'git pull' manually in the word4you directory to sync, then try again.");
            }
            Err(e) => return Err(e.into()),
        };
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
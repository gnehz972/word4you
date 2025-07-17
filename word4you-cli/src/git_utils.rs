use anyhow::{anyhow, Result};
use std::process::Command;
use std::path::Path;
use std::fs;
use crate::config::Config;
use crate::git_section_sync::{GitSectionSynchronizer, SyncResult};

pub fn run_git_command(args: &[&str], work_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(work_dir)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow!(
            "Git command failed: {:?}\nStderr: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn init_git_repo(vocabulary_notebook_file: &str) -> Result<()> {
    let notebook_path = Path::new(vocabulary_notebook_file);
    let work_dir = notebook_path.parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary notebook file path"))?;

    if !work_dir.join(".git").exists() {
        run_git_command(&["init"], work_dir)?;
        run_git_command(&["config", "init.defaultBranch", "main"], work_dir)?;
        run_git_command(&["config", "user.name", "word4you"], work_dir)?;
        run_git_command(&["config", "user.email", "word4you@example.com"], work_dir)?;
        run_git_command(&["config", "merge.union.name", "Union merge driver for text files"], work_dir)?;
        run_git_command(&["config", "merge.union.driver", "git merge-file --union %A %O %B"], work_dir)?;
        println!("🔧 Initialized git repository with main branch in: {}", work_dir.display());
    }
    Ok(())
}

pub fn commit_and_push_changes(commit_message: &str, vocabulary_notebook_file: &str, git_remote_url: Option<&str>, _ssh_private_key_path: Option<&str>, _ssh_public_key_path: Option<&str>) -> Result<()> {
    let notebook_path = Path::new(vocabulary_notebook_file);
    let work_dir = notebook_path.parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary notebook file path"))?;

    init_git_repo(vocabulary_notebook_file)?;

    // Set up remote and merge driver configuration before the first commit if possible.
    if let Some(remote_url) = git_remote_url {
        let remotes = run_git_command(&["remote"], work_dir)?;
        if !remotes.lines().any(|line| line == "origin") {
            run_git_command(&["remote", "add", "origin", remote_url], work_dir)?;
        }

        // Create .gitattributes if it doesn't exist
        let gitattributes_path = work_dir.join(".gitattributes");
        if !gitattributes_path.exists() {
            fs::write(&gitattributes_path, "*.md merge=union\n")?;
        }
    }

    // Add all files, including the new .gitattributes if it was created.
    run_git_command(&["add", "."], work_dir)?;

    // Check for changes to commit
    let status = run_git_command(&["status", "--porcelain"], work_dir)?;
    if status.is_empty() {
        println!("No changes to commit.");
        return Ok(());
    }

    run_git_command(&["commit", "-m", commit_message], work_dir)?;
    println!("✅ Successfully committed word locally");

    if git_remote_url.is_some() {
        // Fetch the latest from the remote
        if let Err(e) = run_git_command(&["fetch", "origin"], work_dir) {
            println!("Warning: Could not fetch from remote. This might be due to network issues or an empty remote repository. {}", e);
        }

        let branch_name = run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"], work_dir)?;
        let branch_name = branch_name.trim();

        // Set upstream branch to track origin/main.
        if branch_name == "main" {
            if run_git_command(&["branch", "--set-upstream-to=origin/main", "main"], work_dir).is_err() {
                println!("Info: Could not set upstream to origin/main. This is expected on the first run against an empty remote.");
            }
        }

        // Pull changes using the union merge strategy, allowing unrelated histories
        if let Err(e) = run_git_command(&["pull", "--rebase=false", "origin", branch_name, "--allow-unrelated-histories"], work_dir) {
             println!("Pull failed, but continuing to push. This might fail if there are conflicts not handled by the union merge driver. Error: {}", e);
        }

        match run_git_command(&["push", "origin", branch_name], work_dir) {
            Ok(_) => println!("✅ Successfully pushed word to remote"),
            Err(e) => {
                println!("⚠️  Cannot push. Please resolve conflicts manually and push.");
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Section-aware synchronization that uses git's change detection
pub fn sync_with_section_awareness(
    vocabulary_file: &str,
    git_remote_url: Option<&str>,
    ssh_private_key_path: Option<&str>,
    ssh_public_key_path: Option<&str>
) -> Result<()> {
    let _work_dir = Path::new(vocabulary_file)
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;
    
    // Initialize git repo if needed
    init_git_repo(vocabulary_file)?;
    
    if git_remote_url.is_none() {
        // Local-only mode - just commit any pending changes
        return commit_local_changes("Update vocabulary", vocabulary_file);
    }
    
    // Create config for section synchronizer
    let config = Config {
        vocabulary_notebook_file: vocabulary_file.to_string(),
        git_remote_url: git_remote_url.map(String::from),
        ssh_private_key_path: ssh_private_key_path.map(String::from),
        ssh_public_key_path: ssh_public_key_path.map(String::from),
        git_enabled: true,
        // These fields would normally come from actual config, but we'll use defaults
        gemini_api_key: String::new(),
        gemini_model_name: "gemini-pro".to_string(),
        gemini_prompt_template: String::new(),
    };
    
    // Create section synchronizer
    let synchronizer = GitSectionSynchronizer::new(config)?;
    
    // Perform section-aware sync
    match synchronizer.sync_with_remote() {
        Ok(SyncResult::Success) => {
            println!("✅ Successfully synchronized vocabulary with section awareness");
            Ok(())
        }
        Ok(SyncResult::NoChanges) => {
            println!("ℹ️  No changes to synchronize");
            Ok(())
        }
        Ok(SyncResult::Conflicts(conflicts)) => {
            println!("⚠️  Section conflicts detected:");
            for conflict in conflicts {
                println!("  - Word '{}' modified in both local and remote", conflict.word);
            }
            Err(anyhow!("Please resolve conflicts manually and run sync again"))
        }
        Err(e) => Err(e),
    }
}

/// Helper function to commit local changes without sync
fn commit_local_changes(message: &str, vocabulary_file: &str) -> Result<()> {
    let work_dir = Path::new(vocabulary_file)
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;
    
    run_git_command(&["add", "."], work_dir)?;
    
    let status = run_git_command(&["status", "--porcelain"], work_dir)?;
    if !status.trim().is_empty() {
        run_git_command(&["commit", "-m", message], work_dir)?;
        println!("✅ Successfully committed changes locally");
    }
    
    Ok(())
}

use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Provide more context for debugging
        let error_msg = if stderr.trim().is_empty() && stdout.trim().is_empty() {
            format!(
                "Git command failed: {:?}\nExit code: {}\nWorking directory: {}\nLikely cause: Nothing to commit or repository state issue",
                args,
                output.status.code().unwrap_or(-1),
                work_dir.display()
            )
        } else {
            format!(
                "Git command failed: {:?}\nExit code: {}\nWorking directory: {}\nStderr: {}\nStdout: {}",
                args,
                output.status.code().unwrap_or(-1),
                work_dir.display(),
                stderr,
                stdout
            )
        };

        Err(anyhow!(error_msg))
    }
}

pub fn init_git_repo(
    vocabulary_notebook_file: &str,
    remote_url: Option<&str>,
) -> Result<()> {
    let notebook_path = Path::new(vocabulary_notebook_file);
    let work_dir = notebook_path
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary notebook file path"))?;

    if !work_dir.join(".git").exists() {
        run_git_command(&["init"], work_dir)?;
        run_git_command(&["config", "init.defaultBranch", "main"], work_dir)?;
        run_git_command(&["config", "user.name", "word4you"], work_dir)?;
        run_git_command(&["config", "user.email", "word4you@example.com"], work_dir)?;

        println!(
            "🔧 Initialized git repository with main branch in: {}",
            work_dir.display()
        );
    }

    if let Some(url) = remote_url {
        // Repository exists, but we might need to add the remote
        let existing_remote = run_git_command(&["remote", "get-url", "origin"], work_dir);
        if existing_remote.is_err() {
            run_git_command(&["remote", "add", "origin", url], work_dir)?;
            println!("🔧 Added remote origin: {}", url);
        }
        run_git_command(&["branch", "--set-upstream-to=origin/main", "main"], work_dir)?;
    }

    Ok(())
}

/// Section-aware synchronization that uses git's change detection
pub fn sync_with_remote(
    vocabulary_file: &str,
    git_remote_url: Option<&str>,
) -> Result<()> {
    let _work_dir = Path::new(vocabulary_file)
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;

    // Initialize git repo if needed with remote URL
    init_git_repo(vocabulary_file, git_remote_url)?;

    // Always commit local changes first, before any sync operations
    commit("Update vocabulary", vocabulary_file)?;

    if git_remote_url.is_none() {
        // Local-only mode - we're done after committing
        return Ok(());
    }

    // Create config for section synchronizer
    let config = Config {
        vocabulary_notebook_file: vocabulary_file.to_string(),
        git_remote_url: git_remote_url.map(String::from),
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
        Err(e) => Err(e),
    }
}

/// Helper function to commit local changes
pub fn commit(message: &str, vocabulary_file: &str) -> Result<()> {
    let work_dir = Path::new(vocabulary_file)
        .parent()
        .ok_or_else(|| anyhow!("Invalid vocabulary file path"))?;

    run_git_command(&["add", "."], work_dir)?;

    let status = run_git_command(&["status", "--porcelain"], work_dir)?;
    if !status.trim().is_empty() {
        run_git_command(&["commit", "-m", message], work_dir)?;
        println!("✅ Successfully committed changes locally");
    } else {
        println!("ℹ️  No local changes to commit");
    }

    Ok(())
}



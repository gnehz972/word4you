import os
import git
from datetime import datetime
from config import Config

def init_git_repo():
    """Initialize git repository if it doesn't exist."""
    try:
        repo = git.Repo('.')
        return repo
    except git.InvalidGitRepositoryError:
        # Initialize new repository
        repo = git.Repo.init('.')
        return repo

def commit_and_push_changes(commit_message: str):
    """
    Commit changes and push to remote repository.
    
    Args:
        commit_message: The commit message
    """
    try:
        repo = init_git_repo()
        
        # Add all changes
        repo.index.add('*')
        
        # Commit changes
        repo.index.commit(commit_message)
        
        # Push to remote if configured
        if Config.GIT_REMOTE_URL:
            try:
                origin = repo.remote('origin')
            except ValueError:
                # Add remote if it doesn't exist
                origin = repo.create_remote('origin', Config.GIT_REMOTE_URL)
            
            origin.push()
            print(f"✅ Changes committed and pushed to remote repository")
        else:
            print(f"✅ Changes committed to local repository")
            
    except Exception as e:
        print(f"⚠️  Warning: Could not commit/push changes: {str(e)}")

def ensure_wordbook_exists():
    """Ensure the wordbook file exists with proper header."""
    if not os.path.exists(Config.WORDBOOK_FILE):
        with open(Config.WORDBOOK_FILE, 'w', encoding='utf-8') as f:
            f.write("# My English Word Book\n\n")
            f.write("This is my personal collection of English words with explanations.\n\n")
            f.write("---\n\n")

def prepend_to_wordbook(content: str):
    """
    Prepend new content to the wordbook file.
    
    Args:
        content: The content to prepend
    """
    ensure_wordbook_exists()
    
    # Read existing content
    with open(Config.WORDBOOK_FILE, 'r', encoding='utf-8') as f:
        existing_content = f.read()
    
    # Prepend new content
    with open(Config.WORDBOOK_FILE, 'w', encoding='utf-8') as f:
        f.write(content + "\n\n---\n\n" + existing_content)

def format_commit_message(word: str) -> str:
    """Format commit message for word addition."""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    return f"Add word: {word} - {timestamp}" 
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

def ensure_vocabulary_notebook_exists():
    """Ensure the vocabulary notebook file exists with proper header."""
    if not os.path.exists(Config.VOCABULARY_NOTEBOOK_FILE):
        with open(Config.VOCABULARY_NOTEBOOK_FILE, 'w', encoding='utf-8') as f:
            f.write("# My Vocabulary Notebook\n\n")
            f.write("This is my personal collection of English words with explanations.\n\n")
            f.write("---\n\n")

def prepend_to_vocabulary_notebook(content: str):
    """
    Prepend new content to the vocabulary notebook file.
    
    Args:
        content: The content to prepend
    """
    ensure_vocabulary_notebook_exists()
    
    # Read existing content
    with open(Config.VOCABULARY_NOTEBOOK_FILE, 'r', encoding='utf-8') as f:
        existing_content = f.read()
    
    # Prepend new content
    with open(Config.VOCABULARY_NOTEBOOK_FILE, 'w', encoding='utf-8') as f:
        f.write(content + "\n\n---\n\n" + existing_content)

def format_commit_message(word: str) -> str:
    """Format commit message for word addition."""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    return f"Add word: {word} - {timestamp}" 
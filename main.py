#!/usr/bin/env python3
"""
Word4You - English Word Learning CLI Application

A command-line tool for learning English words with AI-powered explanations
and automatic Git integration for version control.
"""

import os
import click
from rich.console import Console
from rich.panel import Panel
from rich.text import Text
from word_processor import WordProcessor
from config import Config

console = Console()

@click.group()
@click.version_option(version="1.0.0")
def cli():
    """Word4You - Learn English words with AI assistance."""
    pass

@cli.command()
@click.argument('word')
def learn(word):
    """Learn a new English word."""
    try:
        # Validate configuration
        Config.validate_config()
        
        # Initialize word processor
        processor = WordProcessor()
        
        # Process the word
        success = processor.process_word(word)
        
        if not success:
            click.echo("Failed to process word. Please check your configuration and try again.")
            exit(1)
            
    except ValueError as e:
        console.print(f"❌ Configuration Error: {str(e)}", style="red")
        
        # Auto-configure if API key is missing
        if "GEMINI_API_KEY" in str(e):
            console.print("\n" + "="*50, style="cyan")
            if _auto_configure():
                # Retry the operation after configuration
                try:
                    Config.validate_config()
                    processor = WordProcessor()
                    success = processor.process_word(word)
                    
                    if not success:
                        click.echo("Failed to process word. Please check your configuration and try again.")
                        exit(1)
                except Exception as retry_error:
                    console.print(f"❌ Error after configuration: {str(retry_error)}", style="red")
                    exit(1)
            else:
                console.print("❌ Configuration failed. Please try again.", style="red")
                exit(1)
        else:
            console.print("\nPlease set up your .env file with your Gemini API key.", style="yellow")
            console.print("See env.example for reference.", style="yellow")
            exit(1)
    except Exception as e:
        console.print(f"❌ Unexpected error: {str(e)}", style="red")
        exit(1)

@cli.command()
def test():
    """Test the API connection."""
    try:
        Config.validate_config()
        processor = WordProcessor()
        success = processor.test_api_connection()
        
        if not success:
            exit(1)
            
    except ValueError as e:
        console.print(f"❌ Configuration Error: {str(e)}", style="red")
        
        # Auto-configure if API key is missing
        if "GEMINI_API_KEY" in str(e):
            console.print("\n" + "="*50, style="cyan")
            if _auto_configure():
                # Retry the test after configuration
                try:
                    Config.validate_config()
                    processor = WordProcessor()
                    success = processor.test_api_connection()
                    
                    if not success:
                        exit(1)
                except Exception as retry_error:
                    console.print(f"❌ Error after configuration: {str(retry_error)}", style="red")
                    exit(1)
            else:
                console.print("❌ Configuration failed. Please try again.", style="red")
                exit(1)
        else:
            exit(1)

@cli.command()
def setup():
    """Display setup instructions."""
    setup_text = Text()
    setup_text.append("Word4You Setup Instructions\n\n", style="bold blue")
    
    setup_text.append("1. ", style="bold")
    setup_text.append("Install dependencies:\n")
    setup_text.append("   # Using uv (recommended):\n", style="cyan")
    setup_text.append("   uv sync\n\n", style="cyan")
    setup_text.append("   # Or using pip:\n", style="cyan")
    setup_text.append("   pip install -r requirements.txt\n\n", style="cyan")
    
    setup_text.append("2. ", style="bold")
    setup_text.append("Get your Google Gemini API key:\n")
    setup_text.append("   Visit: https://makersuite.google.com/app/apikey\n\n", style="cyan")
    
    setup_text.append("3. ", style="bold")
    setup_text.append("Create a .env file with your API key:\n")
    setup_text.append("   GEMINI_API_KEY=your_api_key_here\n\n", style="cyan")
    
    setup_text.append("4. ", style="bold")
    setup_text.append("Configure the application:\n")
    setup_text.append("   # Interactive setup:\n", style="cyan")
    setup_text.append("   python main.py init\n\n", style="cyan")
    setup_text.append("   # Or just start using it - configuration will be prompted automatically!\n", style="cyan")
    setup_text.append("   python main.py learn beautiful\n\n", style="cyan")
    
    setup_text.append("5. ", style="bold")
    setup_text.append("Test the connection:\n")
    setup_text.append("   python main.py test\n\n", style="cyan")
    
    setup_text.append("6. ", style="bold")
    setup_text.append("Start learning words:\n")
    setup_text.append("   python main.py learn <word>\n", style="cyan")
    setup_text.append("   # or with uv:\n", style="cyan")
    setup_text.append("   uv run main.py learn <word>\n\n", style="cyan")
    
    setup_text.append("Optional: Set GIT_REMOTE_URL in .env for automatic pushing to remote repository.", style="yellow")
    
    panel = Panel(setup_text, title="Setup Guide", border_style="blue")
    console.print(panel)

@cli.command()
def info():
    """Display application information."""
    info_text = Text()
    info_text.append("Word4You - English Word Learning Tool\n\n", style="bold green")
    
    info_text.append("Features:\n", style="bold")
    info_text.append("• AI-powered word explanations using Google Gemini\n")
    info_text.append("• Chinese translations and phonetic symbols\n")
    info_text.append("• Example sentences in both English and Chinese\n")
    info_text.append("• Automatic Git integration for version control\n")
    info_text.append("• Markdown-formatted word book\n\n")
    
    info_text.append("Usage:\n", style="bold")
    info_text.append("  python main.py init             # Interactive first-time setup\n")
    info_text.append("  python main.py learn <word>     # Learn a new word (auto-configures if needed)\n")
    info_text.append("  python main.py test             # Test API connection (auto-configures if needed)\n")
    info_text.append("  python main.py setup            # Show setup instructions\n")
    info_text.append("  python main.py info             # Show this information\n")
    
    panel = Panel(info_text, title="About Word4You", border_style="green")
    console.print(panel)

def _save_config(api_key, wordbook_path, git_remote):
    """Internal function to save configuration."""
    env_content = f"""# Word4You Configuration
GEMINI_API_KEY={api_key}
WORDBOOK_FILE={wordbook_path}
"""
    
    if git_remote:
        env_content += f"GIT_REMOTE_URL={git_remote}\n"
    
    with open('.env', 'w') as f:
        f.write(env_content)
    
    # Reload environment variables
    from dotenv import load_dotenv
    load_dotenv(override=True)
    
    # Update Config class variables
    Config.GEMINI_API_KEY = os.getenv('GEMINI_API_KEY')
    Config.WORDBOOK_FILE = os.getenv('WORDBOOK_FILE', 'wordbook.md')
    Config.GIT_REMOTE_URL = os.getenv('GIT_REMOTE_URL')
    
    console.print("✅ Configuration saved successfully!", style="green")
    console.print(f"📁 Wordbook will be saved to: {wordbook_path}", style="cyan")
    
    if git_remote:
        console.print(f"🔗 Git remote set to: {git_remote}", style="cyan")

def _test_config():
    """Internal function to test configuration."""
    console.print("\n🧪 Testing configuration...", style="yellow")
    try:
        Config.validate_config()
        processor = WordProcessor()
        success = processor.test_api_connection()
        
        if success:
            console.print("✅ Configuration is working correctly!", style="green")
            return True
        else:
            console.print("⚠️  Configuration saved but API test failed. Please check your API key.", style="yellow")
            return False
    except Exception as e:
        console.print(f"❌ Configuration error: {str(e)}", style="red")
        return False

def _auto_configure():
    """Automatically configure when API key is missing."""
    console.print("🔧 Configuration Required", style="bold yellow")
    console.print("No API key found. Let's set up Word4You!\n", style="cyan")
    
    # Get API key
    api_key = input("Enter your Gemini API key: ").strip()
    if not api_key:
        console.print("❌ API key is required!", style="red")
        return False
    
    # Get wordbook path
    wordbook_path = input("Enter path for wordbook file (default: wordbook.md): ").strip()
    if not wordbook_path:
        wordbook_path = 'wordbook.md'
    
    # Get Git remote (optional)
    git_remote = input("Enter Git remote URL (optional, press Enter to skip): ").strip()
    
    try:
        _save_config(api_key, wordbook_path, git_remote)
        success = _test_config()
        
        if success:
            console.print("\n🎉 Configuration complete! You can now use Word4You.", style="bold green")
            return True
        else:
            console.print("\n⚠️  Configuration saved but API test failed. Please check your API key.", style="yellow")
            return False
    except Exception as e:
        console.print(f"❌ Configuration error: {str(e)}", style="red")
        return False

@cli.command()
def init():
    """Initialize Word4You with interactive configuration."""
    console.print("🚀 Welcome to Word4You Setup!", style="bold blue")
    console.print("\nThis will help you configure Word4You for first use.\n", style="cyan")
    
    # Check if .env already exists
    if os.path.exists('.env'):
        console.print("⚠️  .env file already exists. Do you want to overwrite it?", style="yellow")
        response = input("Type 'yes' to continue: ").lower().strip()
        if response != 'yes':
            console.print("Setup cancelled.", style="red")
            return
    
    # Interactive configuration
    console.print("\n📝 Let's configure your Word4You installation:\n", style="bold")
    
    # Get API key
    api_key = input("Enter your Gemini API key: ").strip()
    if not api_key:
        console.print("❌ API key is required!", style="red")
        return
    
    # Get wordbook path
    wordbook_path = input("Enter path for wordbook file (default: wordbook.md): ").strip()
    if not wordbook_path:
        wordbook_path = 'wordbook.md'
    
    # Get Git remote (optional)
    git_remote = input("Enter Git remote URL (optional, press Enter to skip): ").strip()
    
    try:
        # Use shared functions
        _save_config(api_key, wordbook_path, git_remote)
        success = _test_config()
        
        if success:
            console.print("\n🎉 You're ready to start learning words!", style="bold green")
            console.print("Try: python main.py learn beautiful", style="cyan")
    except Exception as e:
        console.print(f"❌ Configuration error: {str(e)}", style="red")

if __name__ == '__main__':
    cli() 
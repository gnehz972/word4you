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

@click.group(invoke_without_command=True)
@click.argument('word', required=False)
@click.version_option(version="1.0.0")
@click.pass_context
def cli(ctx, word):
    """Word4You - Learn English words with AI assistance."""
    if ctx.invoked_subcommand is None and word:
        # Check if the word is actually a command
        if word in ['info', 'test']:
            # If it's a command, show help
            click.echo(ctx.get_help())
        else:
            # If no subcommand is used but a word is provided, learn the word
            ctx.invoke(learn_word, word=word)
    elif ctx.invoked_subcommand is None:
        # If no subcommand and no word, show help
        click.echo(ctx.get_help())

def learn_word(word):
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
    info_text.append("  python main.py <word>           # Learn a new word (auto-configures if needed)\n")
    info_text.append("  python main.py test             # Test API connection (auto-configures if needed)\n")
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



if __name__ == '__main__':
    cli() 
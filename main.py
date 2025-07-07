#!/usr/bin/env python3
"""
Word4You - English Word Learning CLI Application

A command-line tool for learning English words with AI-powered explanations
and automatic Git integration for version control.
"""

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
    setup_text.append("Test the connection:\n")
    setup_text.append("   python main.py test\n\n", style="cyan")
    
    setup_text.append("5. ", style="bold")
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
    info_text.append("  python main.py learn <word>     # Learn a new word\n")
    info_text.append("  python main.py test             # Test API connection\n")
    info_text.append("  python main.py setup            # Show setup instructions\n")
    info_text.append("  python main.py info             # Show this information\n")
    
    panel = Panel(info_text, title="About Word4You", border_style="green")
    console.print(panel)

if __name__ == '__main__':
    cli() 
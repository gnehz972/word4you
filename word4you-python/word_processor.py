from gemini_client import GeminiClient
from utils import prepend_to_vocabulary_notebook, commit_and_push_changes, format_commit_message
from rich.console import Console
from rich.markdown import Markdown
from rich.prompt import Confirm
import re

console = Console()

class WordProcessor:
    """Handles word processing and integration."""
    
    def __init__(self):
        """Initialize the word processor."""
        self.gemini_client = GeminiClient()
    
    def process_word(self, word: str) -> bool:
        """
        Process a word: get explanation, save to wordbook, and commit changes.
        
        Args:
            word: The English word to process
            
        Returns:
            bool: True if successful, False otherwise
        """
        try:
            # Validate word
            if not self._validate_word(word):
                console.print(f"❌ Invalid word: {word}", style="red")
                return False
            
            console.print(f"🔍 Processing word: {word}", style="blue")
            
            # Get explanation from Gemini
            console.print("🤖 Querying Gemini API...", style="yellow")
            explanation = self.gemini_client.get_word_explanation(word)
            
            # Display the explanation
            console.print("\n📖 Word Explanation:", style="green")
            md = Markdown(explanation)
            console.print(md)
            
            # Ask for user confirmation with options
            console.print("\n" + "="*50, style="blue")
            while True:
                console.print("\n[bold]Choose an action:[/bold]")
                console.print("[green]s[/green] - Save to vocabulary notebook")
                console.print("[yellow]r[/yellow] - Regenerate explanation") 
                console.print("[blue]p[/blue] - Preview what will be saved")
                console.print("[red]k[/red] - Skip this word")
                console.print()
                
                action = console.input("Enter your choice (s/r/p/k): ").lower().strip()
                
                if action in ['s', 'save']:
                    # Save to vocabulary notebook
                    console.print("\n💾 Saving to vocabulary notebook...", style="yellow")
                    prepend_to_vocabulary_notebook(explanation)
                    
                    # Commit and push changes
                    console.print("📝 Committing changes...", style="yellow")
                    commit_message = format_commit_message(word)
                    commit_and_push_changes(commit_message)
                    
                    console.print(f"✅ Successfully processed and saved word: {word}", style="green")
                    return True
                    
                elif action in ['r', 'regenerate']:
                    console.print("🔄 Regenerating explanation...", style="yellow")
                    explanation = self.gemini_client.get_word_explanation(word)
                    
                    console.print("\n📖 New Word Explanation:", style="green")
                    md = Markdown(explanation)
                    console.print(md)
                    console.print("\n" + "="*50, style="blue")
                    continue  # Ask again
                    
                elif action in ['p', 'preview']:
                    console.print("\n📋 [bold]Preview of what will be saved:[/bold]", style="blue")
                    console.print("="*50, style="blue")
                    # Show first few lines as preview
                    lines = explanation.split('\n')
                    preview_lines = lines[:10]  # Show first 10 lines
                    console.print('\n'.join(preview_lines))
                    if len(lines) > 10:
                        console.print(f"\n... and {len(lines) - 10} more lines")
                    console.print("="*50, style="blue")
                    continue
                    
                elif action in ['k', 'skip']:
                    console.print("❌ Word explanation skipped.", style="yellow")
                    return True
                    
                else:
                    console.print("❓ Invalid choice. Please enter 's', 'r', 'p', or 'k'.", style="red")
                    continue
            
        except Exception as e:
            console.print(f"❌ Error processing word: {str(e)}", style="red")
            return False
    
    def _validate_word(self, word: str) -> bool:
        """
        Validate that the input is a valid English word.
        
        Args:
            word: The word to validate
            
        Returns:
            bool: True if valid, False otherwise
        """
        if not word or not word.strip():
            return False
        
        # Remove extra whitespace
        word = word.strip()
        
        # Check if it contains only letters and hyphens
        if not re.match(r'^[a-zA-Z\-]+$', word):
            return False
        
        # Check length
        if len(word) < 1 or len(word) > 50:
            return False
        
        return True
    
    def test_api_connection(self) -> bool:
        """Test the API connection."""
        try:
            console.print("🔍 Testing Gemini API connection...", style="yellow")
            success = self.gemini_client.test_connection()
            if success:
                console.print("✅ Gemini API connection successful", style="green")
            else:
                console.print("❌ Gemini API connection failed", style="red")
            return success
        except Exception as e:
            console.print(f"❌ API connection error: {str(e)}", style="red")
            return False 
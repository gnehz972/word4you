from gemini_client import GeminiClient
from utils import prepend_to_wordbook, commit_and_push_changes, format_commit_message
from rich.console import Console
from rich.markdown import Markdown
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
            
            # Save to wordbook
            console.print("\n💾 Saving to wordbook...", style="yellow")
            prepend_to_wordbook(explanation)
            
            # Commit and push changes
            console.print("📝 Committing changes...", style="yellow")
            commit_message = format_commit_message(word)
            commit_and_push_changes(commit_message)
            
            console.print(f"✅ Successfully processed word: {word}", style="green")
            return True
            
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
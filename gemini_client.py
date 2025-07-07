import google.generativeai as genai
from config import Config
import time

class GeminiClient:
    """Client for interacting with Google Gemini API."""
    
    def __init__(self):
        """Initialize the Gemini client."""
        print(f"Initializing Gemini client with API key: {Config.GEMINI_API_KEY}")
        genai.configure(api_key=Config.GEMINI_API_KEY)
        self.model = genai.GenerativeModel('gemini-2.0-flash-lite')
    
    def get_word_explanation(self, word: str) -> str:
        """
        Get word explanation from Gemini API.
        
        Args:
            word: The English word to explain
            
        Returns:
            str: Markdown formatted explanation
        """
        try:
            # Format the prompt with the word
            prompt = Config.GEMINI_PROMPT_TEMPLATE.format(word=word.lower())
            
            # Generate response
            response = self.model.generate_content(prompt)
            
            if response.text:
                return response.text
            else:
                raise Exception("No response received from Gemini API")
                
        except Exception as e:
            raise Exception(f"Error getting word explanation: {str(e)}")
    
    def test_connection(self) -> bool:
        """Test the connection to Gemini API."""
        try:
            response = self.model.generate_content("Hello")
            return response.text is not None
        except Exception:
            return False 
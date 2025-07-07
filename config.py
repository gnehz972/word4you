import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class Config:
    """Configuration settings for the application."""
    
    # Gemini API settings
    GEMINI_API_KEY = os.getenv('GEMINI_API_KEY')
    
    # Git settings
    GIT_REMOTE_URL = os.getenv('GIT_REMOTE_URL')
    
    # File paths
    WORDBOOK_FILE = 'wordbook.md'
    
    # Gemini prompt template
    GEMINI_PROMPT_TEMPLATE = """
    Please provide a comprehensive explanation for the English word "{word}" in the following format:
    
    ## {word}
    
    **Phonetic:** [IPA phonetic symbols]
    
    **Chinese Translation:** [Simplified Chinese explanation]
    
    **Example Sentence:**
    - English: [One example sentence using the word]
    - Chinese: [Chinese translation of the example sentence]
    
    **Definition:** [Brief English definition]
    
    **Usage Notes:** [Any important usage notes or tips]
    
    Please ensure the response is in proper markdown format and the Chinese translation is clear and accurate.
    """
    
    @classmethod
    def validate_config(cls):
        """Validate that required configuration is present."""
        if not cls.GEMINI_API_KEY:
            raise ValueError(
                "GEMINI_API_KEY not found in environment variables. "
                "Please set it in your .env file or environment."
            ) 
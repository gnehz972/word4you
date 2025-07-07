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
    VOCABULARY_NOTEBOOK_FILE = os.getenv('VOCABULARY_NOTEBOOK_FILE', 'vocabulary_notebook.md')
    
    # Gemini prompt template
    GEMINI_PROMPT_TEMPLATE = """
    Please provide a comprehensive explanation for the English word "{word}" in the following format:
    
    ## {word}
    
    *[IPA phonetic symbols]*
    
    > [Brief English definition]
    
    **[Simplified Chinese meaning, NO pinyin]**
    
    - [One example sentence using the word]
    - [Chinese translation using only Simplified Chinese characters, NO pinyin]
    
    *[one usage note or tip]*
    
    Important formatting rules:
    - Use only Simplified Chinese characters for Chinese translations
    - Do NOT include pinyin (romanized Chinese) in any Chinese text
    - Ensure the response is in proper markdown format
    """
    
    @classmethod
    def validate_config(cls):
        """Validate that required configuration is present."""
        if not cls.GEMINI_API_KEY:
            raise ValueError(
                "GEMINI_API_KEY not found in environment variables. "
                "Please set it in your .env file or environment."
            ) 
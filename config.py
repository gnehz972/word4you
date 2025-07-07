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
    
    **Chinese Translation:** [Simplified Chinese characters only, NO pinyin]
    
    **Example Sentence:**
    - English: [One example sentence using the word]
    - Chinese: [Chinese translation using only Simplified Chinese characters, NO pinyin]
    
    **Definition:** [Brief English definition]
    
    **Usage Notes:** [Any important usage notes or tips]
    
    Important formatting rules:
    - Use only Simplified Chinese characters for Chinese translations
    - Do NOT include pinyin (romanized Chinese) in any Chinese text
    - Keep Chinese translations concise and clear
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
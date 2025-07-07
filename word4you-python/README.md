# Word4You - English Word Learning CLI Tool

A command-line tool for learning English words with AI-powered explanations using Google Gemini.

## Features

- 🤖 AI-powered word explanations
- 🇨🇳 Chinese translations
- 📝 Phonetic symbols (IPA)
- 🌍 Bilingual examples
- 📚 Markdown word book
- 🔄 Git integration
- 🎨 Rich terminal UI

## Quick Start

1. **Install uv** (if not installed):
   ```bash
   curl -LsSf https://astral.sh/uv/install.sh | sh
   ```

2. **Setup project**:
   ```bash
   git clone <your-repo-url>
   cd word4you
   uv sync
   ```

3. **Get API key**:
   - Visit [Google AI Studio](https://makersuite.google.com/app/apikey)
   - Create API key and copy it

4. **Configure**:
   ```bash
   cp env.example .env
   # Edit .env and add: GEMINI_API_KEY=your_api_key_here
   ```

5. **Learn a word**:
   ```bash
   uv run main.py beautiful
   ```

## Build Standalone Executable

1. **Install all dependencies** (including build tools):
   ```bash
   uv sync
   pip install pyinstaller
   ```

2. **Build the executable**:
   ```bash
   pyinstaller word4you.spec
   ```

3. **Find the executable**:
   - The standalone binary will be in the `dist/` directory as `word4you` (or `word4you.exe` on Windows).

4. **Run the executable**:
   ```bash
   ./dist/word4you beautiful
   ```

**Troubleshooting:**
- If you see errors about missing modules (e.g., `ModuleNotFoundError: No module named 'google'` or `No module named 'git'`), make sure all dependencies are installed in your environment:
  ```bash
  uv sync
  pip install google-genai gitpython
  ```
- If you still have issues, check your Python version (Python 3.9+ recommended) and ensure you are building in a clean environment.

## Commands

```bash
uv run main.py <word>    # Learn a new word
uv run main.py test           # Test API connection
uv run main.py info           # Show app info
```

## Example Output

```
🔍 Processing word: beautiful
🤖 Querying Gemini API...

📖 Word Explanation:

                                                 beautiful


/ˈbjuːtɪfl/


▌ Pleasing the senses or mind aesthetically.                                                              

美丽                                                                                                        

 • She has a beautiful smile.

 • 她有一个美丽的笑容。                                                                                     

The word "beautiful" is often used to describe things that are visually appealing, but can also be used to

describe things that are admirable or morally good.


==================================================

Choose an action:
s - Save to wordbook
r - Regenerate explanation
p - Preview what will be saved
k - Skip this word

Enter your choice (s/r/p/k):
```

## Word Book

Words are saved in `wordbook.md` with phonetic symbols, translations, examples, and definitions.

## Git Integration

Automatically commits and pushes each word addition to your repository.

## Requirements

- Python 3.7+
- Google Gemini API key
- Git (optional)
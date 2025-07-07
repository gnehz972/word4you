# Word4You - English Word Learning CLI Tool

A powerful command-line tool for learning English words with AI-powered explanations using Google Gemini. Features automatic Git integration for version control and beautiful markdown formatting.

## Features

- 🤖 **AI-Powered Explanations**: Uses Google Gemini API for comprehensive word explanations
- 🇨🇳 **Chinese Translations**: Simplified Chinese explanations for better understanding
- 📝 **Phonetic Symbols**: IPA phonetic symbols for correct pronunciation
- 🌍 **Bilingual Examples**: Example sentences in both English and Chinese
- 📚 **Markdown Word Book**: All words saved in a beautiful markdown format
- 🔄 **Git Integration**: Automatic commits and pushes to remote repository
- 🎨 **Rich Terminal UI**: Beautiful colored output with progress indicators

## Installation

### Option 1: Using uv (Recommended)

1. **Install uv** (if not already installed):
   ```bash
   curl -LsSf https://astral.sh/uv/install.sh | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone <your-repo-url>
   cd word4you
   ```

3. **Create virtual environment and install dependencies**:
   ```bash
   uv sync
   ```

4. **Activate the virtual environment**:
   ```bash
   source .venv/bin/activate  # On macOS/Linux
   # or
   .venv\Scripts\activate     # On Windows
   ```

### Option 2: Using pip

1. **Clone the repository**:
   ```bash
   git clone <your-repo-url>
   cd word4you
   ```

2. **Create virtual environment**:
   ```bash
   python -m venv venv
   source venv/bin/activate  # On macOS/Linux
   # or
   venv\Scripts\activate     # On Windows
   ```

3. **Install dependencies**:
   ```bash
   pip install -r requirements.txt
   ```

3. **Get your Google Gemini API key**:
   - Visit [Google AI Studio](https://makersuite.google.com/app/apikey)
   - Create a new API key
   - Copy the API key

4. **Set up environment variables**:
   ```bash
   cp env.example .env
   ```
   
   Edit `.env` file and add your API key:
   ```
   GEMINI_API_KEY=your_api_key_here
   GIT_REMOTE_URL=your_git_repo_url_here  # Optional
   ```

## Usage

### Basic Commands

**Learn a new word**:
```bash
python main.py learn beautiful
# or with uv
uv run main.py learn beautiful
```

**Test API connection**:
```bash
python main.py test
# or with uv
uv run main.py test
```

**Show setup instructions**:
```bash
python main.py setup
# or with uv
uv run main.py setup
```

**Show application info**:
```bash
python main.py info
# or with uv
uv run main.py info
```

### Development Commands

**Using the development script:**
```bash
./scripts/dev.sh install    # Install dependencies
./scripts/dev.sh run learn beautiful  # Run the app
./scripts/dev.sh test       # Run tests
./scripts/dev.sh format     # Format code
./scripts/dev.sh lint       # Lint code
./scripts/dev.sh check      # Run all checks
```

**Using Make:**
```bash
make install              # Install dependencies
make run ARGS="learn beautiful"  # Run the app
make test                 # Run tests
make format               # Format code
make lint                 # Lint code
make check                # Run all checks
make help                 # Show all commands
```

### Example Output

When you run `python main.py learn beautiful`, you'll see:

```
🔍 Processing word: beautiful
🤖 Querying Gemini API...

📖 Word Explanation:

## beautiful

**Phonetic:** /ˈbjuːtɪfʊl/

**Chinese Translation:** 美丽的，漂亮的

**Example Sentence:**
- English: She wore a beautiful dress to the party.
- Chinese: 她穿着一件漂亮的裙子去参加聚会。

**Definition:** Pleasing to the senses or mind aesthetically.

**Usage Notes:** Used to describe something that is attractive or pleasing to look at.

💾 Saving to wordbook...
📝 Committing changes...
✅ Successfully processed word: beautiful
```

## Word Book Format

All words are saved in `wordbook.md` with the following structure:

```markdown
# My English Word Book

This is my personal collection of English words with explanations.

---

## beautiful

**Phonetic:** /ˈbjuːtɪfʊl/

**Chinese Translation:** 美丽的，漂亮的

**Example Sentence:**
- English: She wore a beautiful dress to the party.
- Chinese: 她穿着一件漂亮的裙子去参加聚会。

**Definition:** Pleasing to the senses or mind aesthetically.

**Usage Notes:** Used to describe something that is attractive or pleasing to look at.

---
```

## Git Integration

The application automatically:
- Initializes a Git repository if one doesn't exist
- Commits each word addition with a timestamp
- Pushes changes to remote repository (if configured)

## Configuration

### Environment Variables

- `GEMINI_API_KEY`: Your Google Gemini API key (required)
- `GIT_REMOTE_URL`: Remote Git repository URL (optional)

### File Structure

```
word4you/
├── main.py              # Main CLI application
├── config.py            # Configuration settings
├── gemini_client.py     # Google Gemini API client
├── word_processor.py    # Word processing logic
├── utils.py             # Utility functions
├── wordbook.md          # Word book file
├── pyproject.toml       # Project configuration (uv)
├── uv.lock              # Dependency lock file (uv)
├── requirements.txt     # Python dependencies (pip fallback)
├── Makefile             # Development commands
├── scripts/dev.sh       # Development script
├── .env                 # Environment variables
└── README.md           # This file
```

## Requirements

- Python 3.7+
- Google Gemini API key
- Git (for version control)

## Dependencies

### Core Dependencies
- `google-genai`: Google Gemini API client (latest SDK)
- `python-dotenv`: Environment variable management
- `click`: CLI framework
- `rich`: Terminal formatting
- `gitpython`: Git operations

### Development Dependencies (optional)
- `pytest`: Testing framework
- `black`: Code formatter
- `flake8`: Linter

## Error Handling

The application includes comprehensive error handling for:
- Invalid API keys
- Network connectivity issues
- Invalid word inputs
- Git repository issues
- File system errors

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Support

If you encounter any issues:
1. Check that your API key is correctly set in `.env`
2. Test the API connection with `python main.py test`
3. Ensure you have all dependencies installed
4. Check the error messages for specific guidance 
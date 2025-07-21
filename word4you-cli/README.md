# Word4You - English Word Learning CLI Tool (Rust)

A fast, efficient command-line tool for learning English words with AI-powered explanations using Google Gemini, written in Rust.

## Features

- 🤖 AI-powered word explanations
- 🇨🇳 Chinese translations
- 📝 Phonetic symbols (IPA)
- 🌍 Bilingual examples
- 📚 Markdown vocabulary notebook
- 🔄 Git integration
- 🎨 Rich terminal UI
- ⚡ Fast execution (Rust)

## Quick Start

1. **Install Rust** (if not installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Setup project**:
   ```bash
   git clone <your-repo-url>
   cd word4you-rust
   ```

3. **Get API key**:
   - Visit [Google AI Studio](https://makersuite.google.com/app/apikey)
   - Create API key and copy it

4. **Configure**:
   ```bash
   cp .env.example .env
   # Edit .env and add: GEMINI_API_KEY=your_api_key_here
   ```

5. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/word4you beautiful
   ```

## Installation

### From Source

```bash
# Clone the repository
git clone <your-repo-url>
cd word4you-rust

# Build the application
cargo build --release

# The binary will be in target/release/word4you
```

### Using Cargo Install

```bash
# Install directly from crates.io (when published)
cargo install word4you
```

## Configuration

Create a `.env` file in the project directory:

```bash
cp .env.example .env
```

Edit the `.env` file with your settings:

```env
# Required: Your Gemini API key
GEMINI_API_KEY=your_api_key_here

# Optional: Gemini model name to use
# Default: gemini-2.0-flash-001
# Other options: gemini-1.5-flash, gemini-1.5-pro, etc.
GEMINI_MODEL_NAME=gemini-2.0-flash-001

# Optional: Base directory where 'word4you' subdirectory will be created
# Default: home directory (~)
# The program will create: {VOCABULARY_BASE_DIR}/word4you/vocabulary_notebook.md
VOCABULARY_BASE_DIR=~

# Optional: Git remote URL for automatic pushing
GIT_REMOTE_URL=git@github.com:yourusername/your-repo.git

# Optional: Control whether git operations should be performed
# Default: false (git operations disabled)
# Set to true, 1, or yes to enable git operations
GIT_ENABLED=false
```

### Directory Structure

The program automatically creates a dedicated `word4you` subdirectory:

```
{VOCABULARY_BASE_DIR}/
└── word4you/                    # Dedicated directory for Word4You
    ├── .git/                    # Git repository (auto-initialized)
    └── vocabulary_notebook.md   # Your vocabulary notebook
```

**Benefits of this approach:**
- ✅ Clean separation from other files
- ✅ Safe git operations (only tracks Word4You files)
- ✅ Easy backup/sync of just the `word4you` directory
- ✅ No risk of accidentally tracking unrelated files

## Commands

```bash
# Interactive mode (enter words one by one)
./target/release/word4you

# Learn a new word
./target/release/word4you beautiful

# Test API connection
./target/release/word4you --test

# Show app info
./target/release/word4you --info

# Show help
./target/release/word4you --help
```

## Interactive Mode

When you run `word4you` without any arguments, it enters interactive mode:

```
🎯 Welcome to Word4You Interactive Mode!
Enter words to learn, or type 'exit' to quit.

Enter a word to learn: beautiful
🔍 Processing word: beautiful
🤖 Querying Gemini API...

📖 Word Explanation:
==================================================

## beautiful

*/ˈbjuːtɪfl/*

> Pleasing the senses or mind aesthetically.

**美丽**

- She has a beautiful smile.
- 她有一个美丽的笑容。

*The word "beautiful" is often used to describe things that are visually appealing, but can also be used to describe things that are admirable or morally good.*

==================================================

Choose an action:
s - Save to vocabulary notebook
k - Skip this word
r - Regenerate explanation
p - Preview what will be saved

Enter your choice: k
⏭ Word explanation skipped.

==================================================

Enter a word to learn: exit
👋 Goodbye!
```

**Interactive Mode Features:**
- 🔄 **Continuous learning**: Enter words one by one without restarting the program
- 💾 **Save or skip**: Choose to save words to your vocabulary notebook or skip them
- 🔄 **Regenerate**: Get a new explanation if you're not satisfied with the current one
- 👀 **Preview**: See what will be saved before committing
- 🚪 **Easy exit**: Type 'exit', 'quit', or 'q' to leave

## Example Output

## Vocabulary Notebook

Words are saved in `{VOCABULARY_BASE_DIR}/word4you/vocabulary_notebook.md` with:
- 📝 Phonetic symbols (IPA)
- 🇨🇳 Chinese translations
- 🌍 Bilingual examples
- 📖 Detailed definitions
- 🏷️ Usage notes and tips

## Git Integration

- **Configurable**: Git operations can be enabled/disabled via `GIT_ENABLED` environment variable (default: disabled)
- **Automatic initialization**: Git repository is created in the `word4you` directory when enabled
- **Safe operations**: Only tracks files within the dedicated `word4you` directory
- **Auto-commit**: Each word addition is automatically committed when git is enabled
- **Optional push**: Configure `GIT_REMOTE_URL` for automatic pushing to remote repository

## Performance Benefits

Compared to the Python version, the Rust implementation offers:

- ⚡ **Faster startup time** - No Python interpreter overhead
- 🚀 **Better performance** - Compiled native code
- 📦 **Single binary** - No dependency management needed
- 🔒 **Memory safety** - Rust's ownership system prevents common bugs
- 🛡️ **Thread safety** - Safe concurrent operations

## Requirements

- Rust 1.70+
- Google Gemini API key
- Git (optional)

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test
```

### Code Structure

```
src/
├── main.rs          # CLI entry point
├── config.rs        # Configuration management
├── gemini_client.rs # Gemini API client
├── word_processor.rs # Main word processing logic
└── utils.rs         # File and Git operations
```

## Troubleshooting

### Common Issues

1. **API Key Error**:
   ```
   Error: GEMINI_API_KEY not found in environment variables
   ```
   Solution: Make sure your `.env` file exists and contains the correct API key.

2. **Network Error**:
   ```
   Error: Gemini API error: ...
   ```
   Solution: Check your internet connection and API key validity.

3. **Git Error**:
   ```
   Warning: Could not commit/push changes
   ```
   Solution: Ensure Git is configured properly and remote URL is correct.

### Debug Mode

Run with debug logging:

```bash
RUST_LOG=debug ./target/release/word4you beautiful
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
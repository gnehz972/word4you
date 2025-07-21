# Word4You - English Word Query & Collection CLI Tool

A fast, efficient command-line tool for querying and collecting English words with AI-powered explanations using Google Gemini, written in Rust.

## Features

- 🤖 AI-powered word explanations
- 🔄 Git backup/sync with smart conflict resolution
- 📚 Markdown vocabulary notebook
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
   cd word4you-cli
   ```

3. **Build and run**:
   ```bash
   cargo build --release
   ./target/release/word4you
   ```

4. **Follow the onboarding process**:
   - The first time you run Word4You, it will guide you through the setup process
   - You'll need a Google Gemini API key (get one at [Google AI Studio](https://aistudio.google.com/app/apikey))
   - The configuration will be saved to `$HOME/.config/word4you/config.toml`


### Vocabulary Notebook Store Directory Structure

The program automatically creates a dedicated `word4you` subdirectory for easy backup/sync the vocabulary notebook:

```
{VOCABULARY_BASE_DIR}/
└── word4you/                    # Dedicated directory for Word4You
    ├── .git/                    # Git repository (auto-initialized)
    └── vocabulary_notebook.md   # Your vocabulary notebook
```

## Commands

```bash
# Interactive mode (enter words one by one)
./target/release/word4you

# Query a new word
./target/release/word4you query beautiful

# Test API connection
./target/release/word4you test

# Configure the application
./target/release/word4you config

# Save a word with content
./target/release/word4you save <word> --content <content>

# Delete a word
./target/release/word4you delete <word> [--timestamp <timestamp>]

# Update a word
./target/release/word4you update <word> --content <content> [--timestamp <timestamp>]

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

Enter your choice: k
✔️ Word explanation skipped.

==================================================

Enter a word to learn: exit
👋 Goodbye!
```

**Interactive Mode Features:**
- 🔄 **Continuous learning**: Enter words one by one without restarting the program
- 💾 **Save or skip**: Choose to save words to your vocabulary notebook or skip them
- 🔄 **Regenerate**: Get a new explanation if you're not satisfied with the current one
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

- Backup/Sync your vocabulary notebook with Git
- Smart git conflict resolution with multiple devices syncing
- Word collection with clear history

## Requirements

- Rust 1.70+
- Google Gemini API key
- Git (optional)
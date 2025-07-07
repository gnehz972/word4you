# Word4You - AI-Powered English Word Learning Tool

A command-line tool for learning English words with AI-powered explanations using Google Gemini API. Available in both Python and Rust implementations.

## 🌟 Features

- **AI-Powered Explanations**: Get comprehensive word explanations using Google Gemini API
- **Chinese Translations**: Simplified Chinese translations with phonetic symbols
- **Bilingual Examples**: Example sentences in both English and Chinese
- **Markdown Vocabulary Notebook**: Beautifully formatted vocabulary notebook in markdown
- **Git Integration**: Automatic version control with Git commit and push
- **Interactive Interface**: Choose to save, regenerate, preview, or skip word explanations
- **Cross-Platform**: Available in both Python and Rust implementations

## 📁 Project Structure

```
word4you/
├── word4you-python/     # Python implementation
│   ├── main.py
│   ├── word_processor.py
│   ├── gemini_client.py
│   ├── config.py
│   ├── utils.py
│   ├── requirements.txt
│   ├── pyproject.toml
│   └── env.example
├── word4you-rust/       # Rust implementation
│   ├── src/
│   │   ├── main.rs
│   │   ├── word_processor.rs
│   │   ├── gemini_client.rs
│   │   ├── config.rs
│   │   └── utils.rs
│   ├── Cargo.toml
│   └── env.example
└── README.md
```

## 🚀 Quick Start

### Prerequisites

1. **Google Gemini API Key**: Get your API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
2. **Git** (optional, for version control)

### Python Implementation

```bash
# Navigate to Python project
cd word4you-python

# Install dependencies
pip install -r requirements.txt

# Set up environment
cp env.example .env
# Edit .env and add your GEMINI_API_KEY

# Learn a word
python main.py hello

# Test API connection
python main.py test

# Show help
python main.py info
```

### Rust Implementation

```bash
# Navigate to Rust project
cd word4you-rust

# Set up environment
cp env.example .env
# Edit .env and add your GEMINI_API_KEY

# Build the project
cargo build --release

# Learn a word
cargo run -- hello

# Test API connection
cargo run -- test

# Show help
cargo run -- info
```

## ⚙️ Configuration

Both implementations use environment variables for configuration. Create a `.env` file in the respective project directory:

```bash
# Required: Google Gemini API Key
GEMINI_API_KEY=your_gemini_api_key_here

# Optional: Vocabulary notebook file path (defaults to vocabulary_notebook.md)
VOCABULARY_NOTEBOOK_FILE=vocabulary_notebook.md

# Optional: Git remote URL for automatic push
GIT_REMOTE_URL=your_git_repo_url_here
```

## 📖 Usage

### Basic Usage

```bash
# Python
python main.py <word>

# Rust
cargo run -- <word>
```

### Interactive Options

When you learn a word, you'll get an interactive prompt with these options:

- **Save (s)**: Save the word explanation to your vocabulary notebook
- **Regenerate (r)**: Get a new explanation from the AI
- **Preview (p)**: See what will be saved before committing
- **Skip (k)**: Skip this word

### Example Output

```
## hello

*[həˈloʊ]*

> Used as a greeting or to attract attention

**[你好]**

- Hello, how are you today?
- 你好，你今天怎么样？

*Common greeting in English, equivalent to "hi" in casual situations*
```

## 📝 Vocabulary Notebook

The tool creates a markdown file (`vocabulary_notebook.md` by default) with your learned words. Each word includes:

- **Phonetic symbols** (IPA)
- **English definition**
- **Chinese translation**
- **Example sentences** in both languages
- **Usage notes**

## 🔧 Development

### Python Implementation

- **Framework**: Click for CLI, Rich for UI
- **Dependencies**: See `requirements.txt`
- **Structure**: Modular design with separate modules for different concerns

### Rust Implementation

- **Framework**: Clap for CLI, Termimad for markdown rendering
- **Dependencies**: See `Cargo.toml`
- **Features**: Async/await, beautiful markdown rendering, Git integration

## 🛠️ Building

### Python

```bash
cd word4you-python
pip install -r requirements.txt
```

### Rust

```bash
cd word4you-rust
cargo build --release
```

## 📦 Standalone Executable (Rust)

The Rust implementation can be built into a standalone executable:

```bash
cd word4you-rust
cargo build --release
# Executable will be in target/release/word4you
```

## 🔄 Git Integration

Both implementations support automatic Git integration:

1. **Automatic commits** when words are saved
2. **Push to remote** if `GIT_REMOTE_URL` is configured
3. **Version control** for your vocabulary notebook

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Google Gemini API** for AI-powered word explanations
- **Rich** (Python) and **Termimad** (Rust) for beautiful terminal output
- **Click** (Python) and **Clap** (Rust) for CLI framework

## 🔗 Links

- [Google AI Studio](https://makersuite.google.com/app/apikey) - Get your Gemini API key
- [Python Implementation](./word4you-python/) - Python version details
- [Rust Implementation](./word4you-rust/) - Rust version details 
# Word4You - AI-Powered English Word Learning Tool

A command-line tool for learning English words with AI-powered explanations using Google Gemini API.

## 🎯 What is Word4You?

Word4You is an intelligent command-line tool designed to help you learn English vocabulary effectively. It combines the power of Google's Gemini AI with a structured learning approach to create comprehensive word explanations that include:

- **Phonetic pronunciations** using International Phonetic Alphabet (IPA)
- **Clear English definitions** with usage context
- **Chinese translations** for bilingual learners
- **Practical example sentences** in both languages
- **Usage tips and notes** to enhance understanding

### Why Word4You?

Learning vocabulary can be challenging, especially when you need:
- **Contextual understanding** of how words are used
- **Pronunciation guidance** for proper speaking
- **Bilingual support** for non-native English speakers
- **Structured learning** with organized vocabulary notes
- **Version control** to track your learning progress

Word4You addresses these needs by providing an AI-powered, interactive learning experience that creates beautiful, markdown-formatted vocabulary notebooks that you can review, share, and version control with Git.

### How It Works

1. **Input a word** you want to learn
2. **AI generates** a comprehensive explanation using Google Gemini
3. **Review and interact** with the explanation (save, regenerate, preview, or skip)
4. **Save to notebook** for future reference
5. **Version control** automatically commits your learning progress

## 🌟 Features

- **AI-Powered Explanations**: Get comprehensive word explanations using Google Gemini API
- **Chinese Translations**: Simplified Chinese translations with phonetic symbols
- **Bilingual Examples**: Example sentences in both English and Chinese
- **Markdown Vocabulary Notebook**: Beautifully formatted vocabulary notebook in markdown
- **Git Integration**: Automatic version control with Git commit and push
- **Interactive Interface**: Choose to save, regenerate, preview, or skip word explanations

## 📖 Usage

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

## ⚙️ Configuration

The tool uses environment variables for configuration. Create a `.env` file in your project directory:

```bash
# Required: Google Gemini API Key
GEMINI_API_KEY=your_gemini_api_key_here

# Optional: Vocabulary notebook file path (defaults to vocabulary_notebook.md)
VOCABULARY_NOTEBOOK_FILE=vocabulary_notebook.md

# Optional: Git remote URL for automatic push
GIT_REMOTE_URL=your_git_repo_url_here
```

## 🔄 Git Integration

The tool supports automatic Git integration:

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
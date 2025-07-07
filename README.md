# Word4You - AI-Powered English Word Learning CLI Tool

A command-line tool for learning English words with AI-powered explanations using Google Gemini API.

## 🎯 What is Word4You?

Word4You is an intelligent command-line tool designed to help you learn English vocabulary effectively. It combines the power of Google's Gemini AI with a structured learning approach to create comprehensive word explanations that include:

- **Phonetic pronunciations** using International Phonetic Alphabet (IPA)
- **Clear English definitions** with usage context
- **Chinese translations** for bilingual learners
- **Practical example sentences** in both languages
- **Usage tips and notes** to enhance understanding

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

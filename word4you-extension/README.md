# Word4You Raycast Extension

Learn English words with AI-powered explanations using Google Gemini.

## Features

- 🔍 Get detailed word explanations with AI
- 🔊 Pronunciation guides
- 🇨🇳 Chinese translations
- 📝 Example sentences in both languages
- 💾 Save words to vocabulary notebook
- 📚 Git integration for version control

## Setup

1. Install the `word4you` executable (see Installation section below)
2. Install the extension in Raycast
3. Configure your settings in Raycast extension preferences:
   - **Gemini API Key** (required): Your Google Gemini API key
   - **Vocabulary File** (optional): Path to your vocabulary notebook markdown file
   - **Git Remote URL** (optional): Git repository for vocabulary backup

## Usage

1. Open Raycast
2. Search for "Learn Word" 
3. Enter an English word
4. View the detailed explanation
5. Choose to save to vocabulary or just view

## Commands

- **Learn Word**: Get explanation for any English word with option to save

## Actions

- **Save to Vocabulary**: Add the word to your vocabulary notebook and commit to git
- **Back**: Return to word input form

## Requirements

- Raycast
- Node.js
- The `word4you` executable
- Gemini API key
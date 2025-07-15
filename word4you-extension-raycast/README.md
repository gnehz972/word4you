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

1. Install the dev extension in Raycast
   - Run `npm run dev` in the `word4you-extension` directory
   - This will start the Raycast development server and open the extension in Raycast
2. Configure your settings in Raycast extension preferences:
   - **Gemini API Key** (required): Your Google Gemini API key
   - **Gemini Model Name** (optional): Gemini model name to use for AI explanations (defaults to gemini-2.0-flash-001)
   - **Vocabulary Base Directory** (optional): Base directory where 'word4you' subdirectory will be created (defaults to home directory)
   - **Enable Git Operations** (optional): Enable automatic Git commit and push operations when saving words (defaults to false)
   - **Git Remote URL** (optional): Git repository for vocabulary backup (SSH URLs only, e.g., git@github.com:username/repo.git)
   - **SSH Private Key Path** (optional): Path to SSH private key file for Git authentication (defaults to ~/.ssh/id_ed25519)
   - **SSH Public Key Path** (optional): Path to SSH public key file for Git authentication (defaults to ~/.ssh/id_ed25519.pub)

## Directory Structure

The extension automatically creates a dedicated `word4you` subdirectory in your configured base directory:

```
{Base Directory}/
└── word4you/                    # Dedicated directory for Word4You
    ├── .git/                    # Git repository (auto-initialized)
    └── vocabulary_notebook.md   # Your vocabulary notebook
```

**Benefits:**
- ✅ Clean separation from other files
- ✅ Safe git operations (only tracks Word4You files)
- ✅ Easy backup/sync of just the `word4you` directory
- ✅ No risk of accidentally tracking unrelated files

## Usage

1. Open Raycast
2. Search for "Word4You" 
3. Enter an English word
4. View the detailed explanation
5. Choose to save to vocabulary or just view

## Commands

- **Query Word**: Get explanation for any English word with option to save

## Actions

- **Save to Vocabulary**: Add the word to your vocabulary notebook and commit to git
- **Close**: Return to word input form

## Requirements

- Raycast
- Node.js
- The `word4you` executable
- Gemini API key
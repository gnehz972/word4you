# Word4You Raycast Extension

Raycast extension for quick UI interaction with the word4you CLI.

## Features

- 🔍 More comprehensive UI interaction comparing with the CLI 
- 🔊 CURD supprt on the markdown vocabulary notebook

## Prerequisites

### Install Word4You CLI

This extension requires the Word4You CLI to be installed on your system. You can download the latest version from:

**[https://github.com/gnehz972/word4you/releases](https://github.com/gnehz972/word4you/releases)**

1. Download the appropriate binary for your system
2. Make it executable: `chmod +x word4you`
3. Move it to a directory in your PATH:
   ```
   sudo mv word4you /usr/local/bin/
   ```
4. Verify installation:
   ```
   word4you --version
   ```

### Configure Word4You CLI

After installing the CLI, you need to configure it:

1. Run the configuration setup:
   ```
   word4you config
   ```
2. Follow the prompts to set up your:
   - Gemini API Key
   - Vocabulary directory
   - Git integration (optional)
   - Git remote URL (optional)
   - Gemini model name (optional)

The Raycast extension will automatically use the vocabulary path configured in the CLI. You can check your current vocabulary path with:
```
word4you config --show-vob-path
```

## Setup

1. Install the extension in Raycast
   - Run `npm run dev` in the `word4you-extension-raycast` directory
   - This will start the Raycast development server and open the extension in Raycast
2. If needed, configure the CLI path in Raycast extension preferences:
   - **CLI Path** (optional): Path to the Word4You CLI executable (leave empty if installed in PATH)

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
- The `word4you` CLI (installed separately)
- Gemini API key

## Troubleshooting

### CLI Not Found

If the extension shows "Word4You CLI Not Installed" error:

1. Make sure you've downloaded and installed the CLI from [GitHub Releases](https://github.com/gnehz972/word4you/releases)
2. Verify the CLI is working by running `word4you --version` in your terminal
3. If you installed the CLI in a custom location, specify the full path in the extension preferences

### Permission Issues

If you encounter permission issues:

1. Make sure the CLI is executable: `chmod +x /path/to/word4you`
2. Check if the CLI is accessible from your PATH or specify the full path in preferences

### API Key Issues

If you encounter API key issues:

1. Verify your Gemini API key is correct
2. Make sure you have sufficient quota for the Gemini API
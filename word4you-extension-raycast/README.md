# Word4You Raycast Extension

Raycast extension for quick/enriched UI interaction with the word4you CLI.

## Features

- 🚀 Quick launch，get explained with shortcuts 
- 😻 Rich UI interaction comparing with the CLI 
- 🔊 CURD management for the markdown vocabulary notebook

## Prerequisites

### Install Word4You CLI

This extension requires the [Word4You CLI](https://github.com/gnehz972/word4you/releases) to be installed on your system.
Follow the [instructions](../word4you-cli/README.md) to download and config the CLI. 
The `Gemini API key` is required for the CLI to function properly.

Check the CLI installation by running:
```bash
word4you --version
```

## Setup

1. Install the extension in Raycast
   - Run `npm run dev` in the `word4you-extension-raycast` directory
   - This will start the Raycast development server and open the extension in Raycast
2. If needed, configure the CLI path in Raycast extension preferences:
   - **CLI Path** (optional): Path to the Word4You CLI executable (leave empty if installed in PATH)

## Requirements

- Raycast
- Node.js
- The `word4you` CLI (installed separately)
- Gemini API key

## Functionality

1. Query word explanations
2. Save word and structured explanation to vocabulary
3. Update structured explanation
4. Delete words from vocabulary

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
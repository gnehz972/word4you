# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Word4You is a multi-platform AI-powered English word learning tool that provides comprehensive word explanations using Google Gemini API. The project has three main implementations:

1. **Python CLI** (`word4you-python/`) - Full-featured command-line tool
2. **Rust CLI** (`word4you-rust/`) - Fast, optimized native binary
3. **Raycast Extension** (`word4you-extension/`) - macOS Raycast integration

All implementations create a dedicated `word4you` subdirectory in the user's chosen base directory (defaults to home directory) for vocabulary storage and Git operations.

## Key Architecture Components

### Core Functionality
- **Gemini API Integration**: AI-powered word explanations with phonetic symbols, definitions, Chinese translations, and bilingual examples
- **Vocabulary Management**: Structured markdown notebook with automatic Git version control
- **Interactive CLI**: Options to save, regenerate, preview, or skip word explanations
- **Cross-platform File Handling**: Safe directory operations with `VOCABULARY_BASE_DIR` configuration

### Directory Structure Pattern
All implementations follow this consistent pattern:
```
{VOCABULARY_BASE_DIR}/
└── word4you/                    # Dedicated directory
    ├── .git/                    # Git repository (auto-initialized)
    └── vocabulary_notebook.md   # Vocabulary storage
```

## Development Commands

### Python Implementation (`word4you-python/`)
- **Environment**: Use `uv` for virtual environment and Python command execution
- **Setup**: `uv sync`
- **Run**: `uv run main.py <word>`
- **Build standalone**: `pyinstaller word4you.spec`
- **Test**: `uv run pytest` (if tests exist)
- **Format**: `uv run black .`
- **Lint**: `uv run flake8`

### Rust Implementation (`word4you-rust/`)
- **Build debug**: `cargo build`
- **Build release**: `cargo build --release`
- **Run**: `./target/release/word4you <word>`
- **Test**: `cargo test`
- **Format**: `cargo fmt`
- **Lint**: `cargo clippy`

### Raycast Extension (`word4you-extension/`)
- **Development**: `npm run dev`
- **Build**: `npm run build`
- **Lint**: `npm run lint`
- **Fix lint**: `npm run fix-lint`
- **Publish**: `npm run publish`

## Configuration Requirements

### Environment Variables
- `GEMINI_API_KEY`: Required Google Gemini API key
- `VOCABULARY_BASE_DIR`: Optional base directory (defaults to home directory)
- `GIT_REMOTE_URL`: Optional Git remote for automatic pushing

### API Integration
- Uses Google Gemini API for word explanations
- Structured prompt format for consistent output
- Error handling for API failures and network issues

## Git Integration Behavior

All implementations include automatic Git operations:
- Auto-initialize Git repository in `word4you` directory
- Commit each word addition with descriptive messages
- Optional automatic pushing to remote repository
- Safe operations that only track Word4You files

## Important Development Notes

- **Use `uv` for Python**: The project uses `uv` for Python virtual environment management
- **Commit Message Format**: When committing, attach raw user prompts to commit messages (excluding the "commit" command itself)
- **Binary Optimization**: The Rust implementation is optimized for size and performance with release profile settings
- **Cross-platform Compatibility**: All implementations handle path resolution across different operating systems
- **Error Handling**: Graceful degradation when Git operations fail, API is unavailable, or configuration is missing

## Testing Strategy

- Python: Uses pytest framework
- Rust: Uses built-in `cargo test`
- Raycast Extension: Uses standard JavaScript testing with Node.js

## Performance Considerations

- The Rust implementation provides significant performance benefits over Python
- Standalone executables are optimized for distribution
- Binary size optimization is implemented in the Rust version (reduced from 5.7MB to 2.8MB)

## Git Commit Rules
- Attach the raw prompt from user to the end of the commit message
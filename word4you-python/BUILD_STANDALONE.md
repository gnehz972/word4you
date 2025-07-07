# Building Word4You Standalone Executable

This guide explains how to compile Word4You into a standalone executable that can run independently without requiring Python installation.

## Prerequisites

- Python 3.9 or higher
- pip or uv package manager

## Quick Build

### Option 1: Using the build script (Recommended)

```bash
# Install build dependencies
pip install pyinstaller

# Run the build script
python build_standalone.py
```

### Option 2: Using Makefile

```bash
# Install dependencies and build
make build-standalone
```

### Option 3: Manual PyInstaller

```bash
# Install PyInstaller
pip install pyinstaller

# Build the executable
pyinstaller --onefile --name=word4you --add-data=env.example;. --add-data=vocabulary_notebook.md;. --hidden-import=click --hidden-import=rich --hidden-import=google.genai --hidden-import=dotenv --hidden-import=git main.py
```

## Build Output

After successful build, you'll find:

- `dist/word4you` (Linux/macOS) or `dist/word4you.exe` (Windows) - The standalone executable
- `run_word4you.sh` - A launcher script for easier usage

## Using the Standalone Executable

### 1. Copy the executable to your desired location

```bash
# Copy to a directory in your PATH
sudo cp dist/word4you /usr/local/bin/
# Or copy to your home directory
cp dist/word4you ~/bin/
```

### 2. Set up configuration

The application will automatically prompt for configuration when needed! You have several options:

#### Option A: Automatic Configuration (Recommended)
Just start using the application - it will prompt for setup when needed:
```bash
./word4you beautiful
# or
./word4you test
```

#### Option B: Manual Configuration
Create a `.env` file in the same directory as the executable:

```bash
# Copy the example
cp env.example .env

# Edit the file and add your configuration
nano .env
```

Add your configuration:
```
GEMINI_API_KEY=your_actual_api_key_here
VOCABULARY_NOTEBOOK_FILE=my_words.md
GIT_REMOTE_URL=https://github.com/your/repo.git  # Optional
```

### 3. Run the application

```bash
# Just start using it - configuration will be prompted automatically!
./word4you beautiful

# Test the API connection
./word4you test

# Using the launcher script
./run_word4you.sh beautiful
```

## Features of the Standalone Build

- **Single File**: All dependencies are bundled into one executable
- **No Python Required**: Runs on systems without Python installation
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Self-Contained**: Includes all necessary libraries and resources

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   chmod +x word4you
   ```

2. **Missing .env file**
   - The launcher script will create one automatically
   - Or create manually: `echo "GEMINI_API_KEY=your_key" > .env`

3. **Large file size**
   - This is normal for PyInstaller builds
   - The executable includes Python runtime and all dependencies

4. **Antivirus warnings**
   - Some antivirus software may flag PyInstaller executables
   - This is a false positive - add the executable to your antivirus whitelist

### Build Issues

1. **PyInstaller not found**
   ```bash
   pip install pyinstaller
   ```

2. **Missing dependencies**
   ```bash
   pip install -r requirements.txt
   pip install pyinstaller
   ```

3. **Build fails on macOS**
   - Ensure you have Xcode Command Line Tools installed
   - Run: `xcode-select --install`

## Distribution

To distribute the standalone executable:

1. Build the executable on the target platform
2. Include the `env.example` file
3. Provide setup instructions for the `.env` file
4. Optionally include the `run_word4you.sh` launcher script

## Platform-Specific Notes

### Windows
- Executable will be named `word4you.exe`
- May require running as administrator for certain operations
- Consider using a Windows installer for distribution

### macOS
- Executable will be named `word4you`
- May need to be signed for Gatekeeper compatibility
- Consider creating a `.app` bundle for better integration

### Linux
- Executable will be named `word4you`
- May need to be placed in `/usr/local/bin/` for system-wide access
- Consider creating a `.deb` or `.rpm` package for distribution

## Advanced Configuration

### Custom PyInstaller Options

You can modify the build script to add additional options:

```python
build_cmd = [
    "pyinstaller",
    "--onefile",
    "--name=word4you",
    "--add-data=env.example;.",
    "--add-data=vocabulary_notebook.md;.",
    "--hidden-import=click",
    "--hidden-import=rich",
    "--hidden-import=google.genai",
    "--hidden-import=dotenv",
    "--hidden-import=git",
    "--icon=icon.ico",  # Add custom icon
    "--windowed",       # Hide console window (Windows/macOS)
    "main.py"
]
```

### Reducing File Size

To create a smaller executable:

```bash
# Use UPX compression (install UPX first)
pyinstaller --onefile --upx-dir=/path/to/upx --name=word4you main.py

# Exclude unnecessary modules
pyinstaller --onefile --exclude-module=matplotlib --exclude-module=numpy --name=word4you main.py
```

## Security Considerations

- The executable contains all source code and dependencies
- API keys in `.env` files should be kept secure
- Consider using environment variables for sensitive data
- The executable can be reverse-engineered (like any compiled application) 
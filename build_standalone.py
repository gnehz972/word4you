#!/usr/bin/env python3
"""
Build script for creating standalone executable of Word4You using uv
"""

import os
import subprocess
import sys
import shutil
from pathlib import Path

def run_command(cmd, description):
    """Run a command and handle errors"""
    print(f"🔄 {description}...")
    try:
        result = subprocess.run(cmd, shell=True, check=True, capture_output=True, text=True)
        print(f"✅ {description} completed successfully")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ {description} failed:")
        print(f"Error: {e.stderr}")
        return False

def main():
    print("🚀 Building Word4You standalone executable using uv...")
    
    # Check if PyInstaller is available in uv environment
    try:
        result = subprocess.run("uv run python -c 'import PyInstaller'", shell=True, capture_output=True, text=True)
        if result.returncode == 0:
            print("✅ PyInstaller is available in uv environment")
        else:
            print("📦 Installing PyInstaller in uv environment...")
            if not run_command("uv add pyinstaller", "Installing PyInstaller with uv"):
                print("❌ Failed to install PyInstaller. Please install it manually:")
                print("   uv add pyinstaller")
                return False
    except Exception as e:
        print(f"❌ Error checking PyInstaller: {e}")
        return False
    
    # Clean previous builds
    print("🧹 Cleaning previous builds...")
    for path in ["build", "dist", "word4you.spec"]:
        if os.path.exists(path):
            shutil.rmtree(path) if os.path.isdir(path) else os.remove(path)
    
    # Build the executable using uv
    print("🔨 Building executable with uv...")
    
    # Platform-specific data separator
    data_sep = ";" if sys.platform == "win32" else ":"
    
    build_cmd = [
        "uv run pyinstaller",
        "--onefile",  # Create a single executable file
        "--name=word4you",  # Name of the executable
        f"--add-data=env.example{data_sep}.",  # Include example env file
        f"--add-data=wordbook.md{data_sep}.",  # Include wordbook if it exists
        "--hidden-import=click",
        "--hidden-import=rich",
        "--hidden-import=google",
        "--hidden-import=google.genai",
        "--hidden-import=dotenv",
        "--hidden-import=git",
        "--collect-all=google",
        "--collect-all=google.genai",
        "--additional-hooks-dir=.",
        "main.py"
    ]
    
    if not run_command(" ".join(build_cmd), "Building executable with uv"):
        return False
    
    # Check if build was successful
    executable_path = Path("dist/word4you")
    if sys.platform == "win32":
        executable_path = Path("dist/word4you.exe")
    
    if not executable_path.exists():
        print("❌ Executable was not created successfully")
        return False
    
    print(f"✅ Executable created successfully: {executable_path}")
    print(f"📁 Location: {executable_path.absolute()}")
    
    # Create a simple launcher script
    create_launcher_script()
    
    print("\n🎉 Build completed successfully!")
    print("\n📋 Next steps:")
    print("1. Copy the executable to your desired location")
    print("2. Create a .env file with your GEMINI_API_KEY")
    print("3. Run: ./word4you <word>")
    print("\n💡 For development, use: uv run main.py <command>")
    
    return True

def create_launcher_script():
    """Create a simple launcher script for easier usage"""
    launcher_content = """#!/bin/bash
# Word4You Launcher Script
# This script makes it easier to run Word4You

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXECUTABLE="$SCRIPT_DIR/word4you"

if [ ! -f "$EXECUTABLE" ]; then
    echo "❌ word4you executable not found in $SCRIPT_DIR"
    exit 1
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "⚠️  No .env file found. Creating one from example..."
    if [ -f "env.example" ]; then
        cp env.example .env
        echo "📝 Please edit .env file and add your GEMINI_API_KEY"
    else
        echo "GEMINI_API_KEY=your_api_key_here" > .env
        echo "📝 Please edit .env file and add your GEMINI_API_KEY"
    fi
fi

# Run the executable with all arguments
"$EXECUTABLE" "$@"
"""
    
    with open("run_word4you.sh", "w") as f:
        f.write(launcher_content)
    
    # Make it executable
    os.chmod("run_word4you.sh", 0o755)
    print("✅ Created launcher script: run_word4you.sh")

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1) 
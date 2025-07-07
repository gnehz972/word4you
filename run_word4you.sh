#!/bin/bash
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

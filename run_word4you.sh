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
    echo "⚠️  No .env file found."
    echo "💡 Configuration will be prompted automatically when needed!"
    echo "   Just run: ./word4you learn <word> or ./word4you test"
    echo ""
fi

# Run the executable with all arguments
"$EXECUTABLE" "$@"

#!/bin/bash

# Development script for Word4You project

set -e

case "$1" in
    "install")
        echo "Installing dependencies with uv..."
        uv sync
        ;;
    "run")
        shift
        echo "Running with uv..."
        uv run main.py "$@"
        ;;
    "test")
        echo "Running tests..."
        uv run pytest
        ;;
    "format")
        echo "Formatting code with black..."
        uv run black .
        ;;
    "lint")
        echo "Linting code with flake8..."
        uv run flake8 .
        ;;
    "check")
        echo "Running all checks..."
        uv run black --check .
        uv run flake8 .
        uv run pytest
        ;;
    "dev-install")
        echo "Installing development dependencies..."
        uv sync --extra dev
        ;;
    *)
        echo "Usage: $0 {install|run|test|format|lint|check|dev-install}"
        echo ""
        echo "Commands:"
        echo "  install      - Install dependencies"
        echo "  run          - Run the application with uv"
        echo "  test         - Run tests"
        echo "  format       - Format code with black"
        echo "  lint         - Lint code with flake8"
        echo "  check        - Run all checks (format, lint, test)"
        echo "  dev-install  - Install development dependencies"
        exit 1
        ;;
esac 